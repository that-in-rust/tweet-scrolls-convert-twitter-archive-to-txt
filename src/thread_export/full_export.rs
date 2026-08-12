use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};

use crate::processing::data_structures::Thread;

use super::{
    load_archive_tweet_threads, PreparedThreadExportData, ThreadExportFailure,
    ValidatedTweetArchive,
};

const FULL_OUTPUT_FILE_NAME: &str = "tweet-scrolls-full.txt";
const PARTIAL_OUTPUT_FILE_NAME: &str = "tweet-scrolls-full.txt.partial";

/// Immutable input for one complete app export attempt.
#[derive(Debug, Clone)]
pub struct CompleteThreadExportRequest {
    archive: ValidatedTweetArchive,
    timestamp_millis: i64,
}

impl CompleteThreadExportRequest {
    /// Creates an export request with one adapter-injected run timestamp.
    pub fn new(archive: ValidatedTweetArchive, timestamp_millis: i64) -> Self {
        Self {
            archive,
            timestamp_millis,
        }
    }
}

/// Metadata returned only after the complete output filename is committed.
#[derive(Debug, Clone)]
pub struct CompleteThreadOutput {
    full_output_path: PathBuf,
    byte_count: u64,
    thread_count: usize,
}

impl CompleteThreadOutput {
    /// Returns the committed complete TXT path.
    pub fn full_output_path(&self) -> &Path {
        &self.full_output_path
    }

    /// Returns the exact encoded size of the complete TXT.
    pub fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Returns the number of semantic thread blocks written.
    pub fn thread_count(&self) -> usize {
        self.thread_count
    }
}

/// Complete full-export result retained while the user decides about parts.
#[derive(Debug, Clone)]
pub struct CompleteThreadExportOutcome {
    output_folder: PathBuf,
    output: CompleteThreadOutput,
    prepared: Arc<PreparedThreadExportData>,
}

impl CompleteThreadExportOutcome {
    /// Returns the timestamp-derived run folder.
    pub fn output_folder(&self) -> &Path {
        &self.output_folder
    }

    /// Returns the atomically committed complete TXT path.
    pub fn full_output_path(&self) -> &Path {
        self.output.full_output_path()
    }

    /// Returns the complete file's exact byte count.
    pub fn byte_count(&self) -> u64 {
        self.output.byte_count()
    }

    /// Returns the number of exported thread blocks.
    pub fn thread_count(&self) -> usize {
        self.output.thread_count()
    }

    /// Reports whether optional semantic parts can contain tweet content.
    pub fn has_thread_content(&self) -> bool {
        self.thread_count() > 0
    }

    /// Returns the prepared thread model without reloading the archive.
    pub fn prepared_data(&self) -> &Arc<PreparedThreadExportData> {
        &self.prepared
    }
}

/// Renders one non-empty semantic thread using the established CLI TXT format.
pub fn render_single_thread_block(thread: &Thread) -> Result<String, ThreadExportFailure> {
    let first_tweet = thread
        .tweets
        .first()
        .ok_or_else(|| ThreadExportFailure::InvalidThread {
            thread_id: thread.id.clone(),
        })?;
    let mut rendered = String::new();
    rendered.push_str("--- Start of Thread ---\n");
    rendered.push_str(&format!("Thread ID: {}\n", thread.id));
    rendered.push_str(&format!("Timestamp: {}\n", first_tweet.created_at));
    rendered.push_str(&format!(
        "Public Support: {} retweets, {} likes\n",
        first_tweet.retweet_count, first_tweet.favorite_count
    ));
    rendered.push_str("Thread text:\n");
    for (index, tweet) in thread.tweets.iter().enumerate() {
        rendered.push_str(&format!("- Tweet {}:\n", index + 1));
        rendered.push_str(&tweet.full_text);
        rendered.push_str("\n\n");
    }
    rendered.push_str("--- End of Thread ---\n\n");
    Ok(rendered)
}

/// Derives the one timestamp-named child folder owned by an app export run.
pub fn derive_timestamp_output_folder(archive_folder: &Path, timestamp_millis: i64) -> PathBuf {
    archive_folder.join(format!("tweet-scrolls-{timestamp_millis}"))
}

/// Streams and atomically publishes the complete TXT inside an existing run folder.
pub async fn write_complete_thread_output(
    prepared: &PreparedThreadExportData,
    output_folder: &Path,
) -> Result<CompleteThreadOutput, ThreadExportFailure> {
    let partial_path = output_folder.join(PARTIAL_OUTPUT_FILE_NAME);
    let completed_path = output_folder.join(FULL_OUTPUT_FILE_NAME);
    let stream_result = stream_complete_thread_blocks(prepared, &partial_path).await;
    let byte_count = match stream_result {
        Ok(byte_count) => byte_count,
        Err(failure) => {
            remove_partial_output_file(&partial_path).await;
            return Err(failure);
        }
    };

    if let Err(source) = fs::rename(&partial_path, &completed_path).await {
        remove_partial_output_file(&partial_path).await;
        return Err(ThreadExportFailure::OutputCommit {
            temporary_path: partial_path,
            completed_path,
            source,
        });
    }

    Ok(CompleteThreadOutput {
        full_output_path: completed_path,
        byte_count,
        thread_count: prepared.threads().len(),
    })
}

/// Validates collision ownership, loads threads, and publishes the complete TXT.
pub async fn export_archive_threads_text(
    request: CompleteThreadExportRequest,
) -> Result<CompleteThreadExportOutcome, ThreadExportFailure> {
    let output_folder =
        derive_timestamp_output_folder(request.archive.archive_folder(), request.timestamp_millis);
    if output_folder.exists() {
        return Err(ThreadExportFailure::OutputCollision {
            path: output_folder,
        });
    }

    let prepared = Arc::new(load_archive_tweet_threads(&request.archive).await?);
    match fs::create_dir(&output_folder).await {
        Ok(()) => {}
        Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(ThreadExportFailure::OutputCollision {
                path: output_folder,
            });
        }
        Err(source) => {
            return Err(ThreadExportFailure::OutputWrite {
                path: output_folder,
                source,
            });
        }
    }

    let output = write_complete_thread_output(&prepared, &output_folder).await?;
    Ok(CompleteThreadExportOutcome {
        output_folder,
        output,
        prepared,
    })
}

async fn stream_complete_thread_blocks(
    prepared: &PreparedThreadExportData,
    partial_path: &Path,
) -> Result<u64, ThreadExportFailure> {
    let file = create_partial_output_file(partial_path).await?;
    let mut writer = BufWriter::new(file);
    let mut byte_count = 0_u64;
    for thread in prepared.threads() {
        let block = render_single_thread_block(thread)?;
        writer.write_all(block.as_bytes()).await.map_err(|source| {
            ThreadExportFailure::OutputWrite {
                path: partial_path.to_path_buf(),
                source,
            }
        })?;
        byte_count = byte_count.saturating_add(block.len() as u64);
    }
    writer
        .flush()
        .await
        .map_err(|source| ThreadExportFailure::OutputWrite {
            path: partial_path.to_path_buf(),
            source,
        })?;
    writer
        .get_ref()
        .sync_all()
        .await
        .map_err(|source| ThreadExportFailure::OutputWrite {
            path: partial_path.to_path_buf(),
            source,
        })?;
    Ok(byte_count)
}

async fn create_partial_output_file(partial_path: &Path) -> Result<File, ThreadExportFailure> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(partial_path)
        .await
        .map_err(|source| ThreadExportFailure::OutputWrite {
            path: partial_path.to_path_buf(),
            source,
        })
}

async fn remove_partial_output_file(partial_path: &Path) {
    if let Err(error) = fs::remove_file(partial_path).await {
        if error.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "warning: failed to clean temporary export {}: {error}",
                partial_path.display()
            );
        }
    }
}
