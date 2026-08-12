use std::path::{Path, PathBuf};

use super::{
    CompleteThreadExportOutcome, SemanticPartExportResult, ThreadExportFailure,
    ValidatedTweetArchive,
};

/// User actions enabled by exactly one typed application state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadExportAction {
    /// Open the archive folder picker.
    SelectArchive,
    /// Create the complete TXT.
    CreateFull,
    /// Retry a failed complete TXT export.
    RetryFull,
    /// Create upload-friendly semantic TXT parts.
    CreateParts,
    /// Complete the run without numbered parts.
    KeepFullOnly,
    /// Retry a failed numbered-parts attempt.
    RetryParts,
    /// Reveal a completed run in Finder.
    RevealOutput,
    /// Reset the view for another archive.
    ConvertAnother,
}

/// External events and async results accepted by the pure view state machine.
#[derive(Debug)]
pub enum ThreadExportEvent {
    /// Folder selection validated successfully.
    ArchiveValidated(ValidatedTweetArchive),
    /// Folder selection failed validation.
    ArchiveRejected(ThreadExportFailure),
    /// User requested the complete TXT.
    StartFullExport,
    /// Complete TXT was committed successfully.
    FullExportSucceeded(CompleteThreadExportOutcome),
    /// Complete TXT creation failed.
    FullExportFailed(ThreadExportFailure),
    /// User requested semantic numbered parts.
    StartPartsExport,
    /// User declined or dismissed the optional parts prompt.
    KeepFullOnly,
    /// Every numbered part was committed successfully.
    PartsExportSucceeded(SemanticPartExportResult),
    /// Numbered-parts planning, writing, or commit failed.
    PartsExportFailed(ThreadExportFailure),
    /// User requested another complete TXT attempt.
    RetryFullExport,
    /// User requested another numbered-parts attempt.
    RetryPartsExport,
    /// Finder reveal failed and must not invalidate completed output.
    FinderRevealFailed(String),
    /// Reset the terminal state for another archive.
    ConvertAnotherArchive,
}

/// Durable completed metadata that deliberately excludes prepared tweet models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedThreadExport {
    output_folder: PathBuf,
    full_output_path: PathBuf,
    full_byte_count: u64,
    thread_count: usize,
    part_paths: Vec<PathBuf>,
}

impl CompletedThreadExport {
    /// Returns the run folder suitable for Finder reveal.
    pub fn output_folder(&self) -> &Path {
        &self.output_folder
    }

    /// Returns the committed complete TXT path.
    pub fn full_output_path(&self) -> &Path {
        &self.full_output_path
    }

    /// Returns the exact complete TXT byte count.
    pub fn full_byte_count(&self) -> u64 {
        self.full_byte_count
    }

    /// Returns the number of exported semantic threads.
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }

    /// Returns the optional committed numbered part paths.
    pub fn part_paths(&self) -> &[PathBuf] {
        &self.part_paths
    }
}

/// Single authority for Mac view status, enabled actions, data, and recovery.
#[derive(Debug)]
pub enum ThreadExportViewState {
    /// No archive has been selected.
    NeedsArchive,
    /// The most recent archive selection is invalid.
    InvalidArchive {
        /// Typed validation failure.
        failure: ThreadExportFailure,
    },
    /// A validated archive is ready for full export.
    Ready {
        /// Validated archive selection.
        archive: ValidatedTweetArchive,
    },
    /// One complete TXT export is in flight.
    ExportingFull {
        /// Archive retained for recovery.
        archive: ValidatedTweetArchive,
    },
    /// The complete TXT is committed and optional parts may be requested.
    FullExportReady {
        /// Complete outcome retaining prepared thread data.
        outcome: CompleteThreadExportOutcome,
    },
    /// Complete TXT export failed and can be retried.
    FullExportFailed {
        /// Archive retained for retry.
        archive: ValidatedTweetArchive,
        /// Typed export failure.
        failure: ThreadExportFailure,
    },
    /// One optional-parts export is in flight.
    ExportingParts {
        /// Complete outcome retained for semantic planning.
        outcome: CompleteThreadExportOutcome,
    },
    /// Optional-parts export failed without invalidating the full TXT.
    PartsExportFailed {
        /// Complete outcome retained for retry or decline.
        outcome: CompleteThreadExportOutcome,
        /// Typed parts failure.
        failure: ThreadExportFailure,
    },
    /// Export completed; prepared tweet models have been dropped.
    Completed {
        /// Durable output metadata.
        result: CompletedThreadExport,
    },
}

impl ThreadExportViewState {
    /// Reports whether a user action is legal in the current state.
    pub fn allows_action(&self, action: ThreadExportAction) -> bool {
        matches!(
            (self, action),
            (
                Self::NeedsArchive | Self::InvalidArchive { .. },
                ThreadExportAction::SelectArchive
            ) | (Self::Ready { .. }, ThreadExportAction::SelectArchive)
                | (Self::Ready { .. }, ThreadExportAction::CreateFull)
                | (Self::FullExportFailed { .. }, ThreadExportAction::RetryFull)
                | (
                    Self::FullExportFailed { .. },
                    ThreadExportAction::SelectArchive
                )
                | (
                    Self::FullExportReady { .. },
                    ThreadExportAction::CreateParts
                )
                | (
                    Self::FullExportReady { .. },
                    ThreadExportAction::KeepFullOnly
                )
                | (
                    Self::PartsExportFailed { .. },
                    ThreadExportAction::RetryParts
                )
                | (
                    Self::PartsExportFailed { .. },
                    ThreadExportAction::KeepFullOnly
                )
                | (Self::Completed { .. }, ThreadExportAction::RevealOutput)
                | (Self::Completed { .. }, ThreadExportAction::ConvertAnother)
        )
    }

    /// Returns a validated archive retained for full-export dispatch.
    pub fn selected_archive(&self) -> Option<&ValidatedTweetArchive> {
        match self {
            Self::Ready { archive }
            | Self::ExportingFull { archive }
            | Self::FullExportFailed { archive, .. } => Some(archive),
            _ => None,
        }
    }

    /// Returns a committed full outcome retained for optional-parts dispatch.
    pub fn complete_outcome(&self) -> Option<&CompleteThreadExportOutcome> {
        match self {
            Self::FullExportReady { outcome }
            | Self::ExportingParts { outcome }
            | Self::PartsExportFailed { outcome, .. } => Some(outcome),
            _ => None,
        }
    }

    /// Returns durable completed output metadata when the run is terminal.
    pub fn completed_result(&self) -> Option<&CompletedThreadExport> {
        match self {
            Self::Completed { result } => Some(result),
            _ => None,
        }
    }

    /// Returns concise status text for the native adapter.
    pub fn status_text(&self) -> String {
        match self {
            Self::NeedsArchive => "Choose an extracted Twitter archive folder.".to_owned(),
            Self::InvalidArchive { failure }
            | Self::FullExportFailed { failure, .. }
            | Self::PartsExportFailed { failure, .. } => failure.to_string(),
            Self::Ready { archive } => format!("Ready: {}", archive.archive_folder().display()),
            Self::ExportingFull { .. } => "Creating the complete TXT…".to_owned(),
            Self::FullExportReady { outcome } => format!(
                "Complete TXT created: {} threads, {} bytes.",
                outcome.thread_count(),
                outcome.byte_count()
            ),
            Self::ExportingParts { .. } => "Creating upload-friendly TXT parts…".to_owned(),
            Self::Completed { result } => format!(
                "Done: {} threads in {}.",
                result.thread_count,
                result.output_folder.display()
            ),
        }
    }
}

/// Applies only architecture-approved events; invalid events leave state unchanged.
pub fn transition_thread_export_state(
    state: ThreadExportViewState,
    event: ThreadExportEvent,
) -> ThreadExportViewState {
    match (state, event) {
        (
            SelfState::NeedsArchive | SelfState::InvalidArchive { .. },
            ThreadExportEvent::ArchiveValidated(archive),
        ) => SelfState::Ready { archive },
        (
            SelfState::NeedsArchive | SelfState::InvalidArchive { .. },
            ThreadExportEvent::ArchiveRejected(failure),
        ) => SelfState::InvalidArchive { failure },
        (SelfState::Ready { .. }, ThreadExportEvent::ArchiveValidated(archive)) => {
            SelfState::Ready { archive }
        }
        (SelfState::Ready { archive }, ThreadExportEvent::StartFullExport) => {
            SelfState::ExportingFull { archive }
        }
        (SelfState::ExportingFull { .. }, ThreadExportEvent::FullExportSucceeded(outcome)) => {
            if outcome.has_thread_content() {
                SelfState::FullExportReady { outcome }
            } else {
                SelfState::Completed {
                    result: complete_without_part_paths(outcome),
                }
            }
        }
        (SelfState::ExportingFull { archive }, ThreadExportEvent::FullExportFailed(failure)) => {
            SelfState::FullExportFailed { archive, failure }
        }
        (SelfState::FullExportFailed { archive, .. }, ThreadExportEvent::RetryFullExport) => {
            SelfState::ExportingFull { archive }
        }
        (SelfState::FullExportReady { outcome }, ThreadExportEvent::StartPartsExport) => {
            SelfState::ExportingParts { outcome }
        }
        (SelfState::FullExportReady { outcome }, ThreadExportEvent::KeepFullOnly)
        | (SelfState::PartsExportFailed { outcome, .. }, ThreadExportEvent::KeepFullOnly) => {
            SelfState::Completed {
                result: complete_without_part_paths(outcome),
            }
        }
        (SelfState::ExportingParts { outcome }, ThreadExportEvent::PartsExportSucceeded(parts)) => {
            SelfState::Completed {
                result: complete_with_part_paths(outcome, parts),
            }
        }
        (SelfState::ExportingParts { outcome }, ThreadExportEvent::PartsExportFailed(failure)) => {
            SelfState::PartsExportFailed { outcome, failure }
        }
        (SelfState::PartsExportFailed { outcome, .. }, ThreadExportEvent::RetryPartsExport) => {
            SelfState::ExportingParts { outcome }
        }
        (SelfState::Completed { result }, ThreadExportEvent::FinderRevealFailed(message)) => {
            let _ = message;
            SelfState::Completed { result }
        }
        (SelfState::Completed { .. }, ThreadExportEvent::ConvertAnotherArchive) => {
            SelfState::NeedsArchive
        }
        (state, _) => state,
    }
}

type SelfState = ThreadExportViewState;

fn complete_without_part_paths(outcome: CompleteThreadExportOutcome) -> CompletedThreadExport {
    build_completed_export_result(outcome, Vec::new())
}

fn complete_with_part_paths(
    outcome: CompleteThreadExportOutcome,
    parts: SemanticPartExportResult,
) -> CompletedThreadExport {
    build_completed_export_result(outcome, parts.part_paths().to_vec())
}

fn build_completed_export_result(
    outcome: CompleteThreadExportOutcome,
    part_paths: Vec<PathBuf>,
) -> CompletedThreadExport {
    CompletedThreadExport {
        output_folder: outcome.output_folder().to_path_buf(),
        full_output_path: outcome.full_output_path().to_path_buf(),
        full_byte_count: outcome.byte_count(),
        thread_count: outcome.thread_count(),
        part_paths,
    }
}
