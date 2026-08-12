//! Shared, local-only tweet-thread export behavior used by the CLI and Mac app.

mod app_state;
mod archive;
mod error;
mod full_export;
mod semantic_parts;

pub use app_state::{
    transition_thread_export_state, CompletedThreadExport, ThreadExportAction, ThreadExportEvent,
    ThreadExportViewState,
};
pub(crate) use archive::load_tweet_file_threads;
pub use archive::{
    load_archive_tweet_threads, validate_archive_tweets_input, PreparedThreadExportData,
    ValidatedTweetArchive,
};
pub use error::{ThreadExportFailure, ThreadExportFailureKind};
pub use full_export::{
    derive_timestamp_output_folder, export_archive_threads_text, render_single_thread_block,
    write_complete_thread_output, CompleteThreadExportOutcome, CompleteThreadExportRequest,
    CompleteThreadOutput,
};
pub use semantic_parts::{
    cleanup_failed_part_attempt, plan_semantic_thread_parts, write_optional_thread_parts,
    SemanticPartExportRequest, SemanticPartExportResult, SemanticThreadPartPlan,
    PRODUCTION_PART_LIMIT_EXCLUSIVE,
};
