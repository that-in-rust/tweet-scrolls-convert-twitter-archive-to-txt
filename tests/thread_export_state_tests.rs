use std::path::Path;

use tempfile::tempdir;
use tweet_scrolls::{
    processing::data_structures::{Tweet, TweetEntities},
    thread_export::{
        export_archive_threads_text, transition_thread_export_state, validate_archive_tweets_input,
        CompleteThreadExportRequest, SemanticPartExportResult, ThreadExportAction,
        ThreadExportEvent, ThreadExportFailure, ThreadExportViewState,
    },
};

fn create_state_fixture_tweet() -> Tweet {
    Tweet {
        id_str: "1".to_owned(),
        id: "1".to_owned(),
        full_text: "state fixture".to_owned(),
        created_at: "Mon Jan 01 12:01:00 +0000 2024".to_owned(),
        favorite_count: "0".to_owned(),
        retweet_count: "0".to_owned(),
        retweeted: false,
        favorited: false,
        truncated: false,
        lang: "en".to_owned(),
        source: "test".to_owned(),
        display_text_range: Vec::new(),
        in_reply_to_status_id: None,
        in_reply_to_status_id_str: None,
        in_reply_to_user_id: None,
        in_reply_to_user_id_str: None,
        in_reply_to_screen_name: None,
        edit_info: None,
        entities: TweetEntities::default(),
        possibly_sensitive: None,
    }
}

async fn create_state_full_outcome(
    archive_folder: &Path,
    timestamp: i64,
) -> anyhow::Result<tweet_scrolls::thread_export::CompleteThreadExportOutcome> {
    let wrapped = vec![serde_json::json!({ "tweet": create_state_fixture_tweet() })];
    let script = format!(
        "window.YTD.tweets.part0 = {}",
        serde_json::to_string(&wrapped)?
    );
    tokio::fs::write(archive_folder.join("tweets.js"), script).await?;
    let archive = validate_archive_tweets_input(archive_folder)?;
    Ok(export_archive_threads_text(CompleteThreadExportRequest::new(archive, timestamp)).await?)
}

fn create_state_test_failure() -> ThreadExportFailure {
    ThreadExportFailure::InvalidArchive {
        path: "/missing/archive".into(),
        message: "fixture failure".to_owned(),
    }
}

// TEST-STATE-PART-001 / REQ-PART-001.0
#[tokio::test]
async fn parts_prompt_follows_commit() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let outcome = create_state_full_outcome(temporary.path(), 21).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;
    let ready = transition_thread_export_state(
        ThreadExportViewState::NeedsArchive,
        ThreadExportEvent::ArchiveValidated(archive),
    );
    let exporting = transition_thread_export_state(ready, ThreadExportEvent::StartFullExport);
    assert!(!exporting.allows_action(ThreadExportAction::CreateParts));

    let full_ready =
        transition_thread_export_state(exporting, ThreadExportEvent::FullExportSucceeded(outcome));

    assert!(full_ready.allows_action(ThreadExportAction::CreateParts));
    assert!(full_ready.allows_action(ThreadExportAction::KeepFullOnly));
    Ok(())
}

// TEST-INTEG-PART-002 / REQ-PART-002.0
#[tokio::test]
async fn parts_decline_preserves_full() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let outcome = create_state_full_outcome(temporary.path(), 22).await?;
    let full_path = outcome.full_output_path().to_path_buf();
    let full_ready = ThreadExportViewState::FullExportReady { outcome };

    let completed = transition_thread_export_state(full_ready, ThreadExportEvent::KeepFullOnly);

    assert!(matches!(completed, ThreadExportViewState::Completed { .. }));
    assert!(full_path.is_file());
    assert_eq!(
        std::fs::read_dir(full_path.parent().expect("full output has a parent"))?.count(),
        1
    );
    Ok(())
}

// TEST-UNIT-APP-003 / REQ-APP-002.0
#[tokio::test]
async fn view_state_transitions_match() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let outcome = create_state_full_outcome(temporary.path(), 23).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;
    let state = transition_thread_export_state(
        ThreadExportViewState::NeedsArchive,
        ThreadExportEvent::ArchiveValidated(archive),
    );
    assert!(matches!(state, ThreadExportViewState::Ready { .. }));
    let state = transition_thread_export_state(state, ThreadExportEvent::StartFullExport);
    assert!(matches!(state, ThreadExportViewState::ExportingFull { .. }));
    let state =
        transition_thread_export_state(state, ThreadExportEvent::FullExportSucceeded(outcome));
    assert!(matches!(
        state,
        ThreadExportViewState::FullExportReady { .. }
    ));
    let state = transition_thread_export_state(state, ThreadExportEvent::StartPartsExport);
    assert!(matches!(
        state,
        ThreadExportViewState::ExportingParts { .. }
    ));
    let state = transition_thread_export_state(
        state,
        ThreadExportEvent::PartsExportFailed(create_state_test_failure()),
    );
    assert!(matches!(
        state,
        ThreadExportViewState::PartsExportFailed { .. }
    ));
    assert!(state.allows_action(ThreadExportAction::RetryParts));
    assert!(state.allows_action(ThreadExportAction::KeepFullOnly));
    let state = transition_thread_export_state(state, ThreadExportEvent::RetryPartsExport);
    assert!(matches!(
        state,
        ThreadExportViewState::ExportingParts { .. }
    ));
    let state = transition_thread_export_state(
        state,
        ThreadExportEvent::PartsExportSucceeded(SemanticPartExportResult::from_paths(Vec::new())),
    );
    assert!(matches!(state, ThreadExportViewState::Completed { .. }));
    Ok(())
}

// TEST-GPUI-APP-004 / REQ-APP-002.0
#[tokio::test]
async fn duplicate_export_action_ignored() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let outcome = create_state_full_outcome(temporary.path(), 24).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;
    let ready = ThreadExportViewState::Ready { archive };
    let exporting = transition_thread_export_state(ready, ThreadExportEvent::StartFullExport);

    let still_exporting =
        transition_thread_export_state(exporting, ThreadExportEvent::StartFullExport);

    assert!(matches!(
        still_exporting,
        ThreadExportViewState::ExportingFull { .. }
    ));
    let completed = transition_thread_export_state(
        still_exporting,
        ThreadExportEvent::FullExportSucceeded(outcome),
    );
    assert!(matches!(
        completed,
        ThreadExportViewState::FullExportReady { .. }
    ));
    Ok(())
}

// TEST-GPUI-APP-006 / REQ-APP-004.0
#[tokio::test]
async fn export_failure_retry_succeeds() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let outcome = create_state_full_outcome(temporary.path(), 25).await?;
    let archive = validate_archive_tweets_input(temporary.path())?;
    let exporting = ThreadExportViewState::ExportingFull { archive };
    let failed = transition_thread_export_state(
        exporting,
        ThreadExportEvent::FullExportFailed(create_state_test_failure()),
    );
    assert!(failed.allows_action(ThreadExportAction::RetryFull));
    let retrying = transition_thread_export_state(failed, ThreadExportEvent::RetryFullExport);
    assert!(matches!(
        retrying,
        ThreadExportViewState::ExportingFull { .. }
    ));

    let succeeded =
        transition_thread_export_state(retrying, ThreadExportEvent::FullExportSucceeded(outcome));

    assert!(matches!(
        succeeded,
        ThreadExportViewState::FullExportReady { .. }
    ));
    Ok(())
}

// TEST-GPUI-APP-008 / REQ-APP-005.0
#[tokio::test]
async fn finder_reveal_failure_nonfatal() -> anyhow::Result<()> {
    let temporary = tempdir()?;
    let outcome = create_state_full_outcome(temporary.path(), 26).await?;
    let full_path = outcome.full_output_path().to_path_buf();
    let completed = transition_thread_export_state(
        ThreadExportViewState::FullExportReady { outcome },
        ThreadExportEvent::KeepFullOnly,
    );

    let unchanged = transition_thread_export_state(
        completed,
        ThreadExportEvent::FinderRevealFailed("fixture reveal error".to_owned()),
    );

    assert!(matches!(unchanged, ThreadExportViewState::Completed { .. }));
    assert!(full_path.is_file());
    Ok(())
}
