use crate::processing::data_structures::{Tweet, Thread};
use crate::utils::tweet_classifier::{classify_tweet_type, generate_twitter_url, create_reply_context};

use anyhow::Result;
use csv::WriterBuilder;
use serde::Serialize;
use std::fs::File;


/// CSV record structure for enhanced tweet data
#[derive(Debug, Serialize)]
pub struct CsvRecord {
    /// Unique identifier of the tweet
    pub tweet_id: String,
    /// Full text content of the tweet
    pub tweet_text: String,
    /// Type of tweet (Original, ReplyToUser, ReplyToOthers)
    pub tweet_type: String,
    /// Creation timestamp of the tweet
    pub created_at: String,
    /// Number of favorites/likes on the tweet
    pub favorite_count: String,
    /// Number of retweets
    pub retweet_count: String,
    /// Unique identifier of the thread this tweet belongs to
    pub thread_id: String,
    /// Position of this tweet in its thread (1-based)
    pub thread_position: usize,
    /// Total number of tweets in this thread
    pub thread_tweet_count: usize,
    /// Total number of favorites/likes across all tweets in the thread
    pub thread_favorite_count: u32,
    /// Total number of retweets across all tweets in the thread
    pub thread_retweet_count: u32,
    /// URL to view this tweet on Twitter
    pub twitter_url: String,
    /// Context about what this tweet is replying to
    pub reply_context: String,
    /// Language code of the tweet
    pub lang: String,
    /// Source application used to post the tweet
    pub source: String,
}

impl CsvRecord {
    /// Create a CSV record from a tweet and its thread context
    pub fn from_tweet_and_thread(
        tweet: &Tweet, 
        thread: &Thread, 
        screen_name: &str, 
        position: usize
    ) -> Self {
        let tweet_type = classify_tweet_type(tweet, screen_name);
        let twitter_url = generate_twitter_url(tweet, screen_name);
        let reply_context = create_reply_context(tweet).unwrap_or_default();

        CsvRecord {
            tweet_id: tweet.id_str.clone(),
            tweet_text: tweet.full_text.clone(),
            tweet_type: format!("{:?}", tweet_type),
            created_at: tweet.created_at.clone(),
            favorite_count: tweet.favorite_count.clone(),
            retweet_count: tweet.retweet_count.clone(),
            thread_id: thread.id.clone(),
            thread_position: position,
            thread_tweet_count: thread.tweet_count,
            thread_favorite_count: thread.favorite_count,
            thread_retweet_count: thread.retweet_count,
            twitter_url,
            reply_context,
            lang: tweet.lang.clone().unwrap_or_default(),
            source: tweet.source.clone(),
        }
    }
}

/// Enhanced CSV writer for tweet threads
pub struct EnhancedCsvWriter {
    output_path: String,
    records: Vec<CsvRecord>,
}

impl EnhancedCsvWriter {
    /// Create a new enhanced CSV writer
    pub async fn new(output_path: &str) -> Result<Self> {
        Ok(EnhancedCsvWriter {
            output_path: output_path.to_string(),
            records: Vec::new(),
        })
    }

    /// Write a thread to the CSV buffer
    pub async fn write_thread(&mut self, thread: &Thread, screen_name: &str) -> Result<()> {
        for (position, tweet) in thread.tweets.iter().enumerate() {
            let record = CsvRecord::from_tweet_and_thread(tweet, thread, screen_name, position + 1);
            self.records.push(record);
        }
        Ok(())
    }

    /// Finalize and write all records to the CSV file
    pub async fn finalize(self) -> Result<()> {
        let file = File::create(&self.output_path)?;
        let mut writer = WriterBuilder::new().has_headers(true).from_writer(file);

        for record in self.records {
            writer.serialize(record)?;
        }

        writer.flush()?;
        Ok(())
    }
}
