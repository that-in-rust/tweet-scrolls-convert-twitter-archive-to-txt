use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
};

use super::{
    render_single_thread_block, CompleteThreadExportOutcome, PreparedThreadExportData,
    ThreadExportFailure,
};

/// Production parts are strictly smaller than one million encoded bytes.
pub const PRODUCTION_PART_LIMIT_EXCLUSIVE: usize = 1_000_000;

#[derive(Debug, Clone)]
enum SemanticUnitDescriptor {
    CompleteThread {
        thread_index: usize,
    },
    ThreadContinuation {
        thread_index: usize,
        tweet_range: Range<usize>,
        continuation_ordinal: usize,
    },
    TweetContinuation {
        thread_index: usize,
        tweet_index: usize,
        text_range: Range<usize>,
        thread_continuation_ordinal: usize,
        tweet_continuation_ordinal: usize,
    },
}

#[derive(Debug, Clone)]
struct PlannedSemanticPart {
    units: Vec<SemanticUnitDescriptor>,
    encoded_bytes: usize,
}

/// Pure semantic part plan containing source descriptors rather than archive payload copies.
#[derive(Debug, Clone)]
pub struct SemanticThreadPartPlan {
    parts: Vec<PlannedSemanticPart>,
    exclusive_limit: usize,
}

impl SemanticThreadPartPlan {
    /// Returns the number of consecutive part files in this plan.
    pub fn part_count(&self) -> usize {
        self.parts.len()
    }

    /// Returns each exact encoded part size without rendering the archive as one string.
    pub fn encoded_part_sizes(&self) -> Vec<usize> {
        self.parts.iter().map(|part| part.encoded_bytes).collect()
    }
}

/// Optional-parts request retaining the already prepared full-export outcome.
#[derive(Debug, Clone, Copy)]
pub struct SemanticPartExportRequest<'a> {
    outcome: &'a CompleteThreadExportOutcome,
    exclusive_limit: usize,
}

impl<'a> SemanticPartExportRequest<'a> {
    /// Creates a production request using the exclusive one-million-byte boundary.
    pub fn new(outcome: &'a CompleteThreadExportOutcome) -> Self {
        Self {
            outcome,
            exclusive_limit: PRODUCTION_PART_LIMIT_EXCLUSIVE,
        }
    }

    /// Creates a request with a smaller boundary for deterministic verification.
    pub fn with_limit(outcome: &'a CompleteThreadExportOutcome, exclusive_limit: usize) -> Self {
        Self {
            outcome,
            exclusive_limit,
        }
    }
}

/// Result returned only after every numbered part has been committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticPartExportResult {
    part_paths: Vec<PathBuf>,
}

impl SemanticPartExportResult {
    /// Creates a result from paths already committed by an adapter or controlled test.
    pub fn from_paths(part_paths: Vec<PathBuf>) -> Self {
        Self { part_paths }
    }

    /// Returns the number of atomically published numbered part files.
    pub fn part_count(&self) -> usize {
        self.part_paths.len()
    }

    /// Returns the consecutive completed part paths.
    pub fn part_paths(&self) -> &[PathBuf] {
        &self.part_paths
    }
}

/// Plans ordered, UTF-8-safe semantic part descriptors under an exclusive byte limit.
pub fn plan_semantic_thread_parts(
    prepared: &PreparedThreadExportData,
    exclusive_limit: usize,
) -> Result<SemanticThreadPartPlan, ThreadExportFailure> {
    let mut descriptors = Vec::new();
    for thread_index in 0..prepared.threads().len() {
        let complete = SemanticUnitDescriptor::CompleteThread { thread_index };
        let complete_size = render_semantic_unit_descriptor(prepared, &complete)?.len();
        if complete_size < exclusive_limit {
            descriptors.push(complete);
        } else {
            descriptors.extend(split_oversized_thread_units(
                prepared,
                thread_index,
                exclusive_limit,
            )?);
        }
    }

    let mut parts: Vec<PlannedSemanticPart> = Vec::new();
    for descriptor in descriptors {
        let encoded_bytes = render_semantic_unit_descriptor(prepared, &descriptor)?.len();
        if encoded_bytes >= exclusive_limit {
            return Err(part_limit_too_small(exclusive_limit, encoded_bytes));
        }
        if let Some(part) = parts.last_mut() {
            if part.encoded_bytes.saturating_add(encoded_bytes) < exclusive_limit {
                part.units.push(descriptor);
                part.encoded_bytes += encoded_bytes;
                continue;
            }
        }
        parts.push(PlannedSemanticPart {
            units: vec![descriptor],
            encoded_bytes,
        });
    }

    Ok(SemanticThreadPartPlan {
        parts,
        exclusive_limit,
    })
}

/// Writes every planned part to temporary files and publishes the complete set atomically.
pub async fn write_optional_thread_parts(
    request: &SemanticPartExportRequest<'_>,
) -> Result<SemanticPartExportResult, ThreadExportFailure> {
    let plan =
        plan_semantic_thread_parts(request.outcome.prepared_data(), request.exclusive_limit)?;
    cleanup_failed_part_attempt(request.outcome.output_folder()).await;

    let mut path_pairs = Vec::with_capacity(plan.part_count());
    for (index, part) in plan.parts.iter().enumerate() {
        let ordinal = index + 1;
        let completed_path = request
            .outcome
            .output_folder()
            .join(format!("tweet-scrolls-part-{ordinal:03}.txt"));
        let temporary_path = request
            .outcome
            .output_folder()
            .join(format!("tweet-scrolls-part-{ordinal:03}.txt.partial"));
        if let Err(failure) = write_planned_part_temporary(
            request.outcome.prepared_data(),
            &plan,
            part,
            &temporary_path,
        )
        .await
        {
            cleanup_failed_part_attempt(request.outcome.output_folder()).await;
            return Err(failure);
        }
        path_pairs.push((temporary_path, completed_path));
    }

    let mut completed_paths = Vec::with_capacity(path_pairs.len());
    for (temporary_path, completed_path) in path_pairs {
        if let Err(source) = fs::rename(&temporary_path, &completed_path).await {
            cleanup_failed_part_attempt(request.outcome.output_folder()).await;
            return Err(ThreadExportFailure::PartCommit {
                temporary_path,
                completed_path,
                source,
            });
        }
        completed_paths.push(completed_path);
    }

    Ok(SemanticPartExportResult {
        part_paths: completed_paths,
    })
}

/// Best-effort removes numbered part files owned by a failed or stale attempt.
pub async fn cleanup_failed_part_attempt(output_folder: &Path) {
    let mut entries = match fs::read_dir(output_folder).await {
        Ok(entries) => entries,
        Err(_) => return,
    };
    loop {
        let entry = match entries.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) | Err(_) => break,
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with("tweet-scrolls-part-")
            && (name.ends_with(".txt") || name.ends_with(".txt.partial"))
        {
            let _ = fs::remove_file(entry.path()).await;
        }
    }
}

fn split_oversized_thread_units(
    prepared: &PreparedThreadExportData,
    thread_index: usize,
    exclusive_limit: usize,
) -> Result<Vec<SemanticUnitDescriptor>, ThreadExportFailure> {
    let thread = &prepared.threads()[thread_index];
    let mut descriptors = Vec::new();
    let mut range_start = 0;
    let mut continuation_ordinal = 1;

    while range_start < thread.tweets.len() {
        let mut accepted_end = None;
        for range_end in (range_start + 1)..=thread.tweets.len() {
            let candidate = SemanticUnitDescriptor::ThreadContinuation {
                thread_index,
                tweet_range: range_start..range_end,
                continuation_ordinal,
            };
            if render_semantic_unit_descriptor(prepared, &candidate)?.len() < exclusive_limit {
                accepted_end = Some(range_end);
            } else {
                break;
            }
        }

        if let Some(range_end) = accepted_end {
            descriptors.push(SemanticUnitDescriptor::ThreadContinuation {
                thread_index,
                tweet_range: range_start..range_end,
                continuation_ordinal,
            });
            range_start = range_end;
            continuation_ordinal += 1;
            continue;
        }

        let tweet_descriptors = split_oversized_tweet_units(
            prepared,
            thread_index,
            range_start,
            continuation_ordinal,
            exclusive_limit,
        )?;
        continuation_ordinal += tweet_descriptors.len();
        descriptors.extend(tweet_descriptors);
        range_start += 1;
    }

    Ok(descriptors)
}

fn split_oversized_tweet_units(
    prepared: &PreparedThreadExportData,
    thread_index: usize,
    tweet_index: usize,
    starting_thread_ordinal: usize,
    exclusive_limit: usize,
) -> Result<Vec<SemanticUnitDescriptor>, ThreadExportFailure> {
    let text = &prepared.threads()[thread_index].tweets[tweet_index].full_text;
    let mut descriptors = Vec::new();
    let mut range_start = 0;
    let mut tweet_ordinal = 1;

    while range_start < text.len() {
        let descriptor = SemanticUnitDescriptor::TweetContinuation {
            thread_index,
            tweet_index,
            text_range: range_start..range_start,
            thread_continuation_ordinal: starting_thread_ordinal + descriptors.len(),
            tweet_continuation_ordinal: tweet_ordinal,
        };
        let metadata_size = render_semantic_unit_descriptor(prepared, &descriptor)?.len();
        if metadata_size.saturating_add(1) >= exclusive_limit {
            return Err(part_limit_too_small(exclusive_limit, metadata_size + 2));
        }
        let available_bytes = exclusive_limit - metadata_size - 1;
        let mut range_end = range_start.saturating_add(available_bytes).min(text.len());
        while range_end > range_start && !text.is_char_boundary(range_end) {
            range_end -= 1;
        }
        if range_end == range_start {
            return Err(part_limit_too_small(exclusive_limit, metadata_size + 2));
        }
        descriptors.push(SemanticUnitDescriptor::TweetContinuation {
            thread_index,
            tweet_index,
            text_range: range_start..range_end,
            thread_continuation_ordinal: starting_thread_ordinal + descriptors.len(),
            tweet_continuation_ordinal: tweet_ordinal,
        });
        range_start = range_end;
        tweet_ordinal += 1;
    }

    if descriptors.is_empty() {
        return Err(part_limit_too_small(exclusive_limit, exclusive_limit));
    }
    Ok(descriptors)
}

fn render_semantic_unit_descriptor(
    prepared: &PreparedThreadExportData,
    descriptor: &SemanticUnitDescriptor,
) -> Result<String, ThreadExportFailure> {
    match descriptor {
        SemanticUnitDescriptor::CompleteThread { thread_index } => {
            render_single_thread_block(&prepared.threads()[*thread_index])
        }
        SemanticUnitDescriptor::ThreadContinuation {
            thread_index,
            tweet_range,
            continuation_ordinal,
        } => render_thread_continuation_block(
            prepared,
            *thread_index,
            tweet_range.clone(),
            *continuation_ordinal,
            None,
        ),
        SemanticUnitDescriptor::TweetContinuation {
            thread_index,
            tweet_index,
            text_range,
            thread_continuation_ordinal,
            tweet_continuation_ordinal,
        } => render_thread_continuation_block(
            prepared,
            *thread_index,
            *tweet_index..(*tweet_index + 1),
            *thread_continuation_ordinal,
            Some((
                *tweet_index,
                text_range.clone(),
                *tweet_continuation_ordinal,
            )),
        ),
    }
}

fn render_thread_continuation_block(
    prepared: &PreparedThreadExportData,
    thread_index: usize,
    tweet_range: Range<usize>,
    continuation_ordinal: usize,
    text_piece: Option<(usize, Range<usize>, usize)>,
) -> Result<String, ThreadExportFailure> {
    let thread = &prepared.threads()[thread_index];
    let first_tweet = thread
        .tweets
        .first()
        .ok_or_else(|| ThreadExportFailure::InvalidThread {
            thread_id: thread.id.clone(),
        })?;
    let mut rendered = String::new();
    rendered.push_str("--- Start of Thread ---\n");
    rendered.push_str(&format!("Thread ID: {}\n", thread.id));
    rendered.push_str(&format!("Thread continuation: {continuation_ordinal:06}\n"));
    rendered.push_str(&format!("Timestamp: {}\n", first_tweet.created_at));
    rendered.push_str(&format!(
        "Public Support: {} retweets, {} likes\n",
        first_tweet.retweet_count, first_tweet.favorite_count
    ));
    rendered.push_str("Thread text:\n");
    for tweet_index in tweet_range {
        let tweet = &thread.tweets[tweet_index];
        rendered.push_str(&format!("- Tweet {}:\n", tweet_index + 1));
        if let Some((piece_tweet_index, range, piece_ordinal)) = &text_piece {
            if *piece_tweet_index == tweet_index {
                rendered.push_str(&format!("Tweet text continuation: {piece_ordinal:06}\n"));
                rendered.push_str(&tweet.full_text[range.clone()]);
            } else {
                rendered.push_str(&tweet.full_text);
            }
        } else {
            rendered.push_str(&tweet.full_text);
        }
        rendered.push_str("\n\n");
    }
    rendered.push_str("--- End of Thread ---\n\n");
    Ok(rendered)
}

fn render_planned_part_payload(
    prepared: &PreparedThreadExportData,
    part: &PlannedSemanticPart,
) -> Result<String, ThreadExportFailure> {
    let mut payload = String::with_capacity(part.encoded_bytes);
    for descriptor in &part.units {
        payload.push_str(&render_semantic_unit_descriptor(prepared, descriptor)?);
    }
    Ok(payload)
}

async fn write_planned_part_temporary(
    prepared: &PreparedThreadExportData,
    plan: &SemanticThreadPartPlan,
    part: &PlannedSemanticPart,
    temporary_path: &Path,
) -> Result<(), ThreadExportFailure> {
    let payload = render_planned_part_payload(prepared, part)?;
    if payload.is_empty() || payload.len() >= plan.exclusive_limit {
        return Err(part_limit_too_small(
            plan.exclusive_limit,
            payload.len().saturating_add(1),
        ));
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temporary_path)
        .await
        .map_err(|source| ThreadExportFailure::PartWrite {
            path: temporary_path.to_path_buf(),
            source,
        })?;
    let mut writer = BufWriter::new(file);
    writer
        .write_all(payload.as_bytes())
        .await
        .map_err(|source| ThreadExportFailure::PartWrite {
            path: temporary_path.to_path_buf(),
            source,
        })?;
    writer
        .flush()
        .await
        .map_err(|source| ThreadExportFailure::PartWrite {
            path: temporary_path.to_path_buf(),
            source,
        })?;
    writer
        .get_ref()
        .sync_all()
        .await
        .map_err(|source| ThreadExportFailure::PartWrite {
            path: temporary_path.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn part_limit_too_small(exclusive_limit: usize, minimum_required: usize) -> ThreadExportFailure {
    ThreadExportFailure::PartLimitTooSmall {
        exclusive_limit,
        minimum_required,
    }
}
