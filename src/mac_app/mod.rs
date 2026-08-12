//! Feature-gated native GPUI adapter for the local tweet-thread export service.

use std::{mem, path::PathBuf};

use chrono::Utc;
use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, PathPromptOptions, PromptLevel, Render,
    Role, Task, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;
use gpui_tokio::Tokio;

use crate::thread_export::{
    export_archive_threads_text, transition_thread_export_state, validate_archive_tweets_input,
    write_optional_thread_parts, CompleteThreadExportOutcome, CompleteThreadExportRequest,
    SemanticPartExportRequest, SemanticPartExportResult, ThreadExportAction, ThreadExportEvent,
    ThreadExportFailure, ThreadExportViewState,
};

enum CompleteExportTaskResult {
    Succeeded(CompleteThreadExportOutcome),
    Failed(ThreadExportFailure),
}

enum SemanticPartsTaskResult {
    Succeeded(SemanticPartExportResult),
    Failed(ThreadExportFailure),
}

struct TweetScrollsMacView {
    state: ThreadExportViewState,
    folder_picker_task: Option<Task<()>>,
    export_task: Option<Task<()>>,
    parts_prompt_task: Option<Task<()>>,
}

impl TweetScrollsMacView {
    fn new() -> Self {
        Self {
            state: ThreadExportViewState::NeedsArchive,
            folder_picker_task: None,
            export_task: None,
            parts_prompt_task: None,
        }
    }

    fn apply_thread_export_event(&mut self, event: ThreadExportEvent, cx: &mut Context<Self>) {
        let current = mem::replace(&mut self.state, ThreadExportViewState::NeedsArchive);
        self.state = transition_thread_export_state(current, event);
        cx.notify();
    }

    fn request_archive_folder_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.state.allows_action(ThreadExportAction::SelectArchive)
            || self.folder_picker_task.is_some()
        {
            return;
        }

        let receiver = cx.prompt_for_paths(archive_folder_prompt_options());
        let task = cx.spawn_in(window, async move |this, cx| {
            let selection = match receiver.await {
                Ok(Ok(paths)) => Ok(paths.and_then(|paths| paths.into_iter().next())),
                Ok(Err(error)) => Err(error.to_string()),
                Err(error) => Err(error.to_string()),
            };
            let _ = this.update_in(cx, |this, _window, cx| {
                this.folder_picker_task = None;
                match selection {
                    Ok(selected_path) => {
                        let current =
                            mem::replace(&mut this.state, ThreadExportViewState::NeedsArchive);
                        this.state = apply_archive_selection_result(current, selected_path);
                        cx.notify();
                    }
                    Err(message) => this.apply_thread_export_event(
                        ThreadExportEvent::ArchiveRejected(ThreadExportFailure::BackgroundTask {
                            message,
                        }),
                        cx,
                    ),
                }
            });
        });
        self.folder_picker_task = Some(task);
    }

    fn start_background_export_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.export_task.is_some() {
            return;
        }
        let Some(archive) = self.state.selected_archive().cloned() else {
            return;
        };
        let start_event = if self.state.allows_action(ThreadExportAction::CreateFull) {
            ThreadExportEvent::StartFullExport
        } else if self.state.allows_action(ThreadExportAction::RetryFull) {
            ThreadExportEvent::RetryFullExport
        } else {
            return;
        };
        self.apply_thread_export_event(start_event, cx);

        let timestamp_millis = Utc::now().timestamp_millis();
        let worker = Tokio::spawn_result(cx, async move {
            let request = CompleteThreadExportRequest::new(archive, timestamp_millis);
            let result = match export_archive_threads_text(request).await {
                Ok(outcome) => CompleteExportTaskResult::Succeeded(outcome),
                Err(failure) => CompleteExportTaskResult::Failed(failure),
            };
            Ok::<_, anyhow::Error>(result)
        });
        let completion = cx.spawn_in(window, async move |this, cx| {
            let event = match worker.await {
                Ok(CompleteExportTaskResult::Succeeded(outcome)) => {
                    ThreadExportEvent::FullExportSucceeded(outcome)
                }
                Ok(CompleteExportTaskResult::Failed(failure)) => {
                    ThreadExportEvent::FullExportFailed(failure)
                }
                Err(error) => {
                    ThreadExportEvent::FullExportFailed(ThreadExportFailure::BackgroundTask {
                        message: error.to_string(),
                    })
                }
            };
            let _ = this.update_in(cx, |this, window, cx| {
                this.export_task = None;
                this.apply_thread_export_event(event, cx);
                if matches!(this.state, ThreadExportViewState::FullExportReady { .. }) {
                    this.request_optional_parts_choice(window, cx);
                }
            });
        });
        self.export_task = Some(completion);
    }

    fn request_optional_parts_choice(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.state.allows_action(ThreadExportAction::CreateParts)
            || self.parts_prompt_task.is_some()
        {
            return;
        }
        let answer = window.prompt(
            PromptLevel::Info,
            "Create smaller TXT parts too?",
            Some(
                "The complete TXT is ready. Smaller semantic parts stay below 1 MB for easier LLM uploads.",
            ),
            &["Create Parts", "Keep Full Only"],
            cx,
        );
        let task = cx.spawn_in(window, async move |this, cx| {
            let choice = answer.await.unwrap_or(1);
            let _ = this.update_in(cx, |this, window, cx| {
                this.parts_prompt_task = None;
                if choice == 0 {
                    this.start_background_parts_task(window, cx);
                } else {
                    this.apply_thread_export_event(ThreadExportEvent::KeepFullOnly, cx);
                }
            });
        });
        self.parts_prompt_task = Some(task);
    }

    fn start_background_parts_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.export_task.is_some() {
            return;
        }
        let Some(outcome) = self.state.complete_outcome().cloned() else {
            return;
        };
        let start_event = if self.state.allows_action(ThreadExportAction::CreateParts) {
            ThreadExportEvent::StartPartsExport
        } else if self.state.allows_action(ThreadExportAction::RetryParts) {
            ThreadExportEvent::RetryPartsExport
        } else {
            return;
        };
        self.apply_thread_export_event(start_event, cx);

        let worker = Tokio::spawn_result(cx, async move {
            let request = SemanticPartExportRequest::new(&outcome);
            let result = match write_optional_thread_parts(&request).await {
                Ok(parts) => SemanticPartsTaskResult::Succeeded(parts),
                Err(failure) => SemanticPartsTaskResult::Failed(failure),
            };
            Ok::<_, anyhow::Error>(result)
        });
        let completion = cx.spawn_in(window, async move |this, cx| {
            let event = match worker.await {
                Ok(SemanticPartsTaskResult::Succeeded(parts)) => {
                    ThreadExportEvent::PartsExportSucceeded(parts)
                }
                Ok(SemanticPartsTaskResult::Failed(failure)) => {
                    ThreadExportEvent::PartsExportFailed(failure)
                }
                Err(error) => {
                    ThreadExportEvent::PartsExportFailed(ThreadExportFailure::BackgroundTask {
                        message: error.to_string(),
                    })
                }
            };
            let _ = this.update_in(cx, |this, _window, cx| {
                this.export_task = None;
                this.apply_thread_export_event(event, cx);
            });
        });
        self.export_task = Some(completion);
    }

    fn keep_complete_output_only(&mut self, cx: &mut Context<Self>) {
        if self.state.allows_action(ThreadExportAction::KeepFullOnly) {
            self.apply_thread_export_event(ThreadExportEvent::KeepFullOnly, cx);
        }
    }

    fn reveal_completed_output_folder(&mut self, cx: &mut Context<Self>) {
        if !self.state.allows_action(ThreadExportAction::RevealOutput) {
            return;
        }
        if let Some(result) = self.state.completed_result() {
            cx.reveal_path(result.output_folder());
        }
    }

    fn reset_another_archive_choice(&mut self, cx: &mut Context<Self>) {
        if self.state.allows_action(ThreadExportAction::ConvertAnother) {
            self.apply_thread_export_event(ThreadExportEvent::ConvertAnotherArchive, cx);
        }
    }
}

impl Render for TweetScrollsMacView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let can_select = self.state.allows_action(ThreadExportAction::SelectArchive);
        let can_export = self.state.allows_action(ThreadExportAction::CreateFull)
            || self.state.allows_action(ThreadExportAction::RetryFull);
        let can_parts = self.state.allows_action(ThreadExportAction::CreateParts)
            || self.state.allows_action(ThreadExportAction::RetryParts);
        let can_keep = self.state.allows_action(ThreadExportAction::KeepFullOnly);
        let can_reveal = self.state.allows_action(ThreadExportAction::RevealOutput);
        let can_reset = self.state.allows_action(ThreadExportAction::ConvertAnother);
        let status = self.state.status_text();

        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .gap_4()
            .p_8()
            .bg(rgb(0x111318))
            .text_color(rgb(0xf4f4f5))
            .child(div().text_2xl().child("Tweet Scrolls"))
            .child(
                div()
                    .max_w(px(560.0))
                    .text_center()
                    .text_color(rgb(0xaeb4bf))
                    .child(
                        "Turn your public tweet threads into local TXT input for NotebookLM or another LLM.",
                    ),
            )
            .child(
                div()
                    .max_w(px(560.0))
                    .text_center()
                    .child(status),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap_3()
                    .when(can_select, |buttons| {
                        buttons.child(
                            build_primary_action_button("select-archive", "Choose Archive Folder")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.request_archive_folder_selection(window, cx);
                                })),
                        )
                    })
                    .when(can_export, |buttons| {
                        buttons.child(
                            build_primary_action_button("create-full", "Create Full TXT").on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.start_background_export_task(window, cx);
                                }),
                            ),
                        )
                    })
                    .when(can_parts, |buttons| {
                        buttons.child(
                            build_primary_action_button("create-parts", "Create <1 MB Parts").on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.start_background_parts_task(window, cx);
                                }),
                            ),
                        )
                    })
                    .when(can_keep, |buttons| {
                        buttons.child(
                            build_primary_action_button("keep-full", "Keep Full Only").on_click(
                                cx.listener(|this, _, _window, cx| {
                                    this.keep_complete_output_only(cx);
                                }),
                            ),
                        )
                    })
                    .when(can_reveal, |buttons| {
                        buttons.child(
                            build_primary_action_button("reveal-output", "Show in Finder").on_click(
                                cx.listener(|this, _, _window, cx| {
                                    this.reveal_completed_output_folder(cx);
                                }),
                            ),
                        )
                    })
                    .when(can_reset, |buttons| {
                        buttons.child(
                            build_primary_action_button("convert-another", "Convert Another").on_click(
                                cx.listener(|this, _, _window, cx| {
                                    this.reset_another_archive_choice(cx);
                                }),
                            ),
                        )
                    }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x7f8794))
                    .child("Local only · public tweet threads · TXT only"),
            )
    }
}

fn build_primary_action_button(
    identifier: &'static str,
    label: &'static str,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(identifier)
        .role(Role::Button)
        .aria_label(label)
        .tab_index(0)
        .cursor_pointer()
        .rounded_md()
        .px_4()
        .py_2()
        .bg(rgb(0x2f6feb))
        .hover(|style| style.bg(rgb(0x3d7df2)))
        .child(label)
}

/// Returns the single-directory picker options verified against Zed GPUI patterns.
pub fn archive_folder_prompt_options() -> PathPromptOptions {
    PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some("Choose Archive".into()),
    }
}

/// Projects an optional picker result into state without changing state on cancellation.
pub fn apply_archive_selection_result(
    state: ThreadExportViewState,
    selected_path: Option<PathBuf>,
) -> ThreadExportViewState {
    let Some(selected_path) = selected_path else {
        return state;
    };
    let event = match validate_archive_tweets_input(selected_path) {
        Ok(archive) => ThreadExportEvent::ArchiveValidated(archive),
        Err(failure) => ThreadExportEvent::ArchiveRejected(failure),
    };
    transition_thread_export_state(state, event)
}

/// Starts the native macOS GPUI application.
pub fn run_tweet_scrolls_mac() {
    application().run(|cx: &mut App| {
        gpui_tokio::init(cx);
        let bounds = Bounds::centered(None, size(px(680.0), px(460.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| TweetScrollsMacView::new()),
        )
        .unwrap_or_else(|error| panic!("failed to open Tweet Scrolls window: {error}"));
        cx.activate(true);
    });
}
