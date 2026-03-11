//! File I/O operations for CSV writing and file management

use anyhow::{Context, Result};
use csv::Writer as CsvWriterLib;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use tokio::sync::mpsc as async_mpsc;

use super::data_structures::{CsvWriter, Thread};

impl CsvWriter {
    /// Runs the CSV writer, consuming records from the channel
    pub async fn run(mut self) -> Result<()> {
        let file = File::create(&self.output_path)
            .with_context(|| format!("Failed to create file: {}", self.output_path))?;
        let mut writer = CsvWriterLib::from_writer(BufWriter::new(file));

        // Write headers
        writer.write_record([
            "Thread ID",
            "Date time of first tweet",
            "Number of Tweets in Thread",
            "Likes in first tweet",
            "Retweets in first tweet",
            "Total likes for all tweets",
            "Total retweets for all tweets",
            "Thread Text",
        ])?;

        let mut buffer = Vec::with_capacity(self.buffer_size);

        while let Some(record) = self.receiver.recv().await {
            buffer.push(record);
            if buffer.len() >= self.buffer_size {
                self.flush_buffer(&mut writer, &mut buffer)?;
            }
        }

        if !buffer.is_empty() {
            self.flush_buffer(&mut writer, &mut buffer)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Flushes the buffer to the CSV writer
    fn flush_buffer(&self, writer: &mut CsvWriterLib<BufWriter<File>>, buffer: &mut Vec<Vec<String>>) -> Result<()> {
        for record in buffer.drain(..) {
            writer.write_record(&record)?;
        }
        Ok(())
    }
}

/// Writes threads to a text file
pub async fn write_threads_to_file(threads: &[Thread], screen_name: &str, timestamp: i64, output_dir: &Path) -> Result<()> {
    let file_path = output_dir.join(format!("threads_{}_{}.txt", screen_name, timestamp));
    let file = File::create(&file_path)?;
    let mut writer = BufWriter::new(file);

    for thread in threads {
        writeln!(writer, "--- Start of Thread ---")?;
        writeln!(writer, "Thread ID: {}", thread.id)?;
        writeln!(writer, "Timestamp: {}", thread.tweets[0].created_at)?;
        writeln!(writer, "Public Support: {} retweets, {} likes",
                 thread.tweets[0].retweet_count, thread.tweets[0].favorite_count)?;
        writeln!(writer, "Thread text:")?;

        for (i, tweet) in thread.tweets.iter().enumerate() {
            writeln!(writer, "- Tweet {}:", i + 1)?;
            writeln!(writer, "{}", tweet.full_text)?;
            writeln!(writer)?;
        }

        writeln!(writer, "--- End of Thread ---\n")?;
    }

    writer.flush()?;
    Ok(())
}

/// Writes CSV data for threads
pub async fn write_csv(
    threads: &[Thread],
    _screen_name: &str,
    _timestamp: i64,
    csv_tx: async_mpsc::Sender<Vec<String>>,
) -> Result<()> {
    for thread in threads {
        let first_tweet = &thread.tweets[0];
        let total_likes: u32 = thread.tweets.iter().filter_map(|t| t.favorite_count.parse::<u32>().ok()).sum();
        let total_retweets: u32 = thread.tweets.iter().filter_map(|t| t.retweet_count.parse::<u32>().ok()).sum();
        let thread_text: String = thread.tweets.iter().map(|t| t.full_text.replace('\n', " ")).collect::<Vec<_>>().join(" ");

        let record = vec![
            thread.id.clone(),
            first_tweet.created_at.clone(),
            thread.tweets.len().to_string(),
            first_tweet.favorite_count.clone(),
            first_tweet.retweet_count.clone(),
            total_likes.to_string(),
            total_retweets.to_string(),
            thread_text,
        ];

        csv_tx.send(record).await?;
    }

    Ok(())
}

/// Reads user input from any `BufRead` source.
///
/// This indirection allows us to inject a mock reader in tests so that
/// the function does **not** block waiting for interactive input in CI
/// or during automated `cargo test` runs.
fn prompt_input_from_reader<R: std::io::BufRead>(reader: &mut R, prompt: &str) -> Result<String> {
    use std::io::Write;

    // Print the prompt to stdout so that interactive usage remains unchanged.
    print!("{}", prompt);
    std::io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    reader.read_line(&mut input).context("Failed to read input")?;
    Ok(input.trim().to_string())
}

/// Gets user input with a prompt using `stdin` as the source. This wrapper
/// keeps the original public API intact for production usage while delegating
/// to the injectable function above for easier testing.
pub fn prompt_input(prompt: &str) -> Result<String> {
    use std::io;

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    prompt_input_from_reader(&mut handle, prompt)
}

/// Gets input file path from user
pub fn get_input_file() -> Result<String> {
    prompt_input("📁 Enter path to your tweets.js file: ")
}

/// Gets optional DM file path from user
pub fn get_dm_file() -> Result<Option<String>> {
    let input = prompt_input("💬 Enter path to direct-messages.js (or press Enter to skip): ")?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::sync::mpsc as async_mpsc;

    #[tokio::test]
    async fn test_csv_writer_creation() {
        let temp_dir = tempdir().unwrap();
        let csv_path = temp_dir.path().join("test.csv");
        let (_, rx) = async_mpsc::channel::<Vec<String>>(10);
        
        let writer = CsvWriter::new(csv_path.to_string_lossy().to_string(), rx, 100);
        assert_eq!(writer.buffer_size, 100);
    }

    #[tokio::test]
    async fn test_write_threads_to_file() {
        use super::super::data_structures::{Tweet, Thread, TweetEntities};
        
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();
        
        let tweet = Tweet {
            id_str: "123".to_string(),
            id: "123".to_string(),
            full_text: "Test tweet".to_string(),
            created_at: "Mon Jan 01 12:00:00 +0000 2023".to_string(),
            favorite_count: "5".to_string(),
            retweet_count: "2".to_string(),
            retweeted: false,
            favorited: false,
            truncated: false,
            lang: Some("en".to_string()),
            source: "<a href=\"http://twitter.com\" rel=\"nofollow\">Twitter Web App</a>".to_string(),
            display_text_range: vec!["0".to_string(), "10".to_string()],
            in_reply_to_status_id: None,
            in_reply_to_status_id_str: None,
            in_reply_to_user_id: None,
            in_reply_to_user_id_str: None,
            in_reply_to_screen_name: None,
            edit_info: None,
            entities: Some(TweetEntities {
                hashtags: vec![],
                symbols: vec![],
                user_mentions: vec![],
                urls: vec![],
            }),
            possibly_sensitive: None,
        };

        let thread = Thread {
            id: "thread_123".to_string(),
            tweets: vec![tweet],
            favorite_count: 5,
            retweet_count: 2,
            tweet_count: 1,
        };

        let result = write_threads_to_file(&[thread], "testuser", 1234567890, output_dir).await;
        assert!(result.is_ok());

        let file_path = output_dir.join("threads_testuser_1234567890.txt");
        assert!(file_path.exists());
    }

    #[test]
    fn test_input_functions() {
        use std::io::Cursor;

        // Simulate providing a tweets.js path and ensure the function returns it.
        let mut tweets_reader = Cursor::new("path/to/tweets.js\n");
        let tweets = prompt_input_from_reader(&mut tweets_reader, "Enter path: ").unwrap();
        assert_eq!(tweets, "path/to/tweets.js");

        // Simulate pressing Enter (empty input) for the optional DM file.
        let mut dm_reader = Cursor::new("\n");
        let dm = prompt_input_from_reader(&mut dm_reader, "Enter DM path: ").unwrap();
        assert_eq!(dm, "");
    }
}
