use tweet_scrolls::processing::data_structures::{Tweet, TweetEntities, Thread};
use tweet_scrolls::utils::enhanced_csv_writer::{EnhancedCsvWriter, CsvRecord};
use tempfile::tempdir;

#[cfg(test)]
mod enhanced_csv_writer_tests {
    use super::*;

    fn create_test_tweet(id: &str, text: &str, reply_to_id: Option<&str>, reply_to_user: Option<&str>) -> Tweet {
        Tweet {
            id_str: id.to_string(),
            id: id.to_string(),
            full_text: text.to_string(),
            created_at: "Mon Jan 01 12:00:00 +0000 2023".to_string(),
            favorite_count: "5".to_string(),
            retweet_count: "2".to_string(),
            retweeted: false,
            favorited: false,
            truncated: false,
            lang: Some("en".to_string()),
            source: "Twitter Web App".to_string(),
            display_text_range: vec!["0".to_string(), text.len().to_string()],
            in_reply_to_status_id: reply_to_id.map(|s| s.to_string()),
            in_reply_to_status_id_str: reply_to_id.map(|s| s.to_string()),
            in_reply_to_user_id: None,
            in_reply_to_user_id_str: None,
            in_reply_to_screen_name: reply_to_user.map(|s| s.to_string()),
            edit_info: None,
            entities: Some(TweetEntities {
                hashtags: vec![],
                symbols: vec![],
                user_mentions: vec![],
                urls: vec![],
            }),
            possibly_sensitive: None,
        }
    }

    fn create_test_thread(id: &str, tweets: Vec<Tweet>) -> Thread {
        let tweet_count = tweets.len();
        let favorite_count = tweets.iter()
            .map(|t| t.favorite_count.parse::<u32>().unwrap_or(0))
            .sum();
        let retweet_count = tweets.iter()
            .map(|t| t.retweet_count.parse::<u32>().unwrap_or(0))
            .sum();

        Thread {
            id: id.to_string(),
            tweets,
            tweet_count,
            favorite_count,
            retweet_count,
        }
    }

    #[tokio::test]
    async fn test_create_csv_record_original_tweet() {
        let tweet = create_test_tweet("1", "Hello world", None, None);
        let thread = create_test_thread("thread1", vec![tweet.clone()]);
        
        let record = CsvRecord::from_tweet_and_thread(&tweet, &thread, "testuser", 1);
        
        assert_eq!(record.tweet_id, "1");
        assert_eq!(record.tweet_text, "Hello world");
        assert_eq!(record.tweet_type, "Original");
        assert_eq!(record.thread_id, "thread1");
        assert_eq!(record.thread_position, 1);
        assert_eq!(record.twitter_url, "https://twitter.com/testuser/status/1");
        assert_eq!(record.reply_context, "");
    }

    #[tokio::test]
    async fn test_create_csv_record_reply_tweet() {
        let tweet = create_test_tweet("2", "Reply to myself", Some("1"), Some("testuser"));
        let thread = create_test_thread("thread1", vec![tweet.clone()]);
        
        let record = CsvRecord::from_tweet_and_thread(&tweet, &thread, "testuser", 2);
        
        assert_eq!(record.tweet_type, "ReplyToUser");
        assert_eq!(record.reply_context, "Reply to @testuser (1)");
    }

    #[tokio::test]
    async fn test_enhanced_csv_writer_creation() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_output.csv");
        
        let writer = EnhancedCsvWriter::new(output_path.to_str().unwrap()).await;
        assert!(writer.is_ok());
    }

    #[tokio::test]
    async fn test_write_single_thread() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_output.csv");
        
        let mut writer = EnhancedCsvWriter::new(output_path.to_str().unwrap()).await.unwrap();
        
        let tweet = create_test_tweet("1", "Hello world", None, None);
        let thread = create_test_thread("thread1", vec![tweet]);
        
        let result = writer.write_thread(&thread, "testuser").await;
        assert!(result.is_ok());
        
        let result = writer.finalize().await;
        assert!(result.is_ok());
        
        // Verify file exists
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_write_multiple_threads() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().join("test_output.csv");
        
        let mut writer = EnhancedCsvWriter::new(output_path.to_str().unwrap()).await.unwrap();
        
        // Create first thread
        let tweet1 = create_test_tweet("1", "First tweet", None, None);
        let thread1 = create_test_thread("thread1", vec![tweet1]);
        
        // Create second thread with multiple tweets
        let tweet2 = create_test_tweet("2", "Second tweet", None, None);
        let tweet3 = create_test_tweet("3", "Reply to second", Some("2"), Some("testuser"));
        let thread2 = create_test_thread("thread2", vec![tweet2, tweet3]);
        
        let result1 = writer.write_thread(&thread1, "testuser").await;
        assert!(result1.is_ok());
        
        let result2 = writer.write_thread(&thread2, "testuser").await;
        assert!(result2.is_ok());
        
        let result = writer.finalize().await;
        assert!(result.is_ok());
        
        // Verify file exists
        assert!(output_path.exists());
    }
}
