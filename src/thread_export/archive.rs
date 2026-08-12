use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use chrono::{DateTime, FixedOffset};
use tokio::{fs as async_fs, task};

use crate::processing::{
    data_structures::{Thread, Tweet, TweetWrapper},
    reply_threads::process_reply_threads,
};

use super::ThreadExportFailure;

const ARCHIVE_TWEETS_FILE_NAME: &str = "tweets.js";
const TWITTER_TIMESTAMP_FORMAT: &str = "%a %b %d %H:%M:%S %z %Y";

/// A selected archive folder whose `tweets.js` is a readable regular file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTweetArchive {
    archive_folder: PathBuf,
    tweets_file: PathBuf,
}

impl ValidatedTweetArchive {
    /// Returns the selected extracted archive folder.
    pub fn archive_folder(&self) -> &Path {
        &self.archive_folder
    }

    /// Returns the validated `tweets.js` path.
    pub fn tweets_file(&self) -> &Path {
        &self.tweets_file
    }
}

/// Prepared semantic thread data retained for the optional parts decision.
#[derive(Debug, Clone)]
pub struct PreparedThreadExportData {
    threads: Vec<Thread>,
    source_tweet_count: usize,
    excluded_retweet_count: usize,
}

impl PreparedThreadExportData {
    /// Returns reconstructed threads ordered newest root first.
    pub fn threads(&self) -> &[Thread] {
        &self.threads
    }

    /// Returns the number of wrappers decoded from `tweets.js`.
    pub fn source_tweet_count(&self) -> usize {
        self.source_tweet_count
    }

    /// Returns the number of archive entries excluded by the retweet flag.
    pub fn excluded_retweet_count(&self) -> usize {
        self.excluded_retweet_count
    }
}

/// Validates that a selected folder contains one readable regular `tweets.js`.
pub fn validate_archive_tweets_input(
    selected_path: impl AsRef<Path>,
) -> Result<ValidatedTweetArchive, ThreadExportFailure> {
    let archive_folder = selected_path.as_ref().to_path_buf();
    if !archive_folder.is_dir() {
        return Err(ThreadExportFailure::InvalidArchive {
            path: archive_folder,
            message: "select a directory containing tweets.js".to_owned(),
        });
    }

    let tweets_file = archive_folder.join(ARCHIVE_TWEETS_FILE_NAME);
    let metadata = tweets_file
        .metadata()
        .map_err(|error| ThreadExportFailure::InvalidArchive {
            path: archive_folder.clone(),
            message: format!("tweets.js is unavailable: {error}"),
        })?;
    if !metadata.is_file() {
        return Err(ThreadExportFailure::InvalidArchive {
            path: archive_folder,
            message: "tweets.js is not a regular file".to_owned(),
        });
    }
    File::open(&tweets_file).map_err(|error| ThreadExportFailure::InvalidArchive {
        path: archive_folder.clone(),
        message: format!("tweets.js is not readable: {error}"),
    })?;

    Ok(ValidatedTweetArchive {
        archive_folder,
        tweets_file,
    })
}

/// Loads, validates, filters, and reconstructs an archive into semantic threads.
pub async fn load_archive_tweet_threads(
    archive: &ValidatedTweetArchive,
) -> Result<PreparedThreadExportData, ThreadExportFailure> {
    load_tweet_file_threads(&archive.tweets_file).await
}

pub(crate) async fn load_tweet_file_threads(
    tweets_file: &Path,
) -> Result<PreparedThreadExportData, ThreadExportFailure> {
    let tweets_file = tweets_file.to_path_buf();
    let script_content = async_fs::read_to_string(&tweets_file)
        .await
        .map_err(|source| ThreadExportFailure::ArchiveRead {
            path: tweets_file.clone(),
            source,
        })?;
    let worker_path = tweets_file.clone();

    task::spawn_blocking(move || prepare_archive_thread_data(&script_content, &worker_path))
        .await
        .map_err(|error| ThreadExportFailure::BackgroundTask {
            message: error.to_string(),
        })?
}

fn prepare_archive_thread_data(
    script_content: &str,
    tweets_file: &Path,
) -> Result<PreparedThreadExportData, ThreadExportFailure> {
    let wrappers = parse_wrapped_archive_tweets(script_content, tweets_file)?;
    let source_tweet_count = wrappers.len();
    let mut tweets = wrappers
        .into_iter()
        .map(|wrapper| wrapper.tweet)
        .collect::<Vec<_>>();
    tweets.retain(|tweet| !tweet.retweeted);
    let excluded_retweet_count = source_tweet_count.saturating_sub(tweets.len());
    let parsed_timestamps = validate_all_tweet_timestamps(&tweets)?;
    let mut grouped = process_reply_threads(&tweets, "");
    sort_grouped_tweet_timestamps(&mut grouped, &parsed_timestamps);
    let mut threads = convert_grouped_thread_models(grouped);
    threads.sort_by(|left, right| {
        let left_timestamp = parsed_timestamps.get(&left.id);
        let right_timestamp = parsed_timestamps.get(&right.id);
        right_timestamp.cmp(&left_timestamp)
    });

    Ok(PreparedThreadExportData {
        threads,
        source_tweet_count,
        excluded_retweet_count,
    })
}

fn sort_grouped_tweet_timestamps(
    grouped: &mut [Vec<Tweet>],
    parsed_timestamps: &HashMap<String, DateTime<FixedOffset>>,
) {
    for tweets in grouped {
        tweets.sort_by(|left, right| {
            parsed_timestamps
                .get(&left.id_str)
                .cmp(&parsed_timestamps.get(&right.id_str))
        });
    }
}

fn parse_wrapped_archive_tweets(
    script_content: &str,
    tweets_file: &Path,
) -> Result<Vec<TweetWrapper>, ThreadExportFailure> {
    let json_start = script_content
        .find('[')
        .ok_or_else(|| ThreadExportFailure::ArchiveParse {
            path: tweets_file.to_path_buf(),
            message: "missing opening bracket".to_owned(),
        })?;
    let json_end = script_content
        .rfind(']')
        .ok_or_else(|| ThreadExportFailure::ArchiveParse {
            path: tweets_file.to_path_buf(),
            message: "missing closing bracket".to_owned(),
        })?;
    if json_end < json_start {
        return Err(ThreadExportFailure::ArchiveParse {
            path: tweets_file.to_path_buf(),
            message: "closing bracket precedes opening bracket".to_owned(),
        });
    }

    serde_json::from_str(&script_content[json_start..=json_end]).map_err(|error| {
        ThreadExportFailure::ArchiveParse {
            path: tweets_file.to_path_buf(),
            message: error.to_string(),
        }
    })
}

fn validate_all_tweet_timestamps(
    tweets: &[Tweet],
) -> Result<HashMap<String, DateTime<FixedOffset>>, ThreadExportFailure> {
    tweets
        .iter()
        .map(|tweet| {
            DateTime::parse_from_str(&tweet.created_at, TWITTER_TIMESTAMP_FORMAT)
                .map(|timestamp| (tweet.id_str.clone(), timestamp))
                .map_err(|_| ThreadExportFailure::InvalidTimestamp {
                    tweet_id: tweet.id_str.clone(),
                    value: tweet.created_at.clone(),
                })
        })
        .collect()
}

fn convert_grouped_thread_models(grouped: Vec<Vec<Tweet>>) -> Vec<Thread> {
    grouped
        .into_iter()
        .filter_map(|tweets| {
            let id = tweets.first()?.id_str.clone();
            let tweet_count = tweets.len();
            let favorite_count = tweets
                .iter()
                .filter_map(|tweet| tweet.favorite_count.parse::<u32>().ok())
                .sum();
            let retweet_count = tweets
                .iter()
                .filter_map(|tweet| tweet.retweet_count.parse::<u32>().ok())
                .sum();
            Some(Thread {
                id,
                tweets,
                tweet_count,
                favorite_count,
                retweet_count,
            })
        })
        .collect()
}
