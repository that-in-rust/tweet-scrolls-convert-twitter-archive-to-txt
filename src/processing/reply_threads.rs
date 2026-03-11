//! Reply thread processing module
//! Treats all replies as potential thread starters

use std::collections::{HashMap, HashSet};
use crate::processing::data_structures::Tweet;

/// Process tweets to identify and build reply threads
///
/// This function treats all replies as potential threads, not just self-replies
///
/// # Arguments
/// * `tweets` - Vector of tweets to process
/// * `_screen_name` - The user's screen name for context (currently unused)
///
/// # Returns
/// Vector of thread vectors, where each thread is a vector of related tweets
pub fn process_reply_threads(tweets: &[Tweet], _screen_name: &str) -> Vec<Vec<Tweet>> {
    let mut threads = Vec::new();
    let mut tweet_map: HashMap<String, &Tweet> = HashMap::new();
    let mut reply_map: HashMap<String, Vec<&Tweet>> = HashMap::new();
    let mut processed_ids = HashSet::new();
    
    // Build lookup maps once so thread expansion does not rescan every tweet.
    for tweet in tweets {
        tweet_map.insert(tweet.id_str.clone(), tweet);
        if let Some(parent_id) = &tweet.in_reply_to_status_id {
            reply_map.entry(parent_id.clone()).or_default().push(tweet);
        }
    }
    
    // Process each tweet
    for tweet in tweets {
        if processed_ids.contains(&tweet.id_str) {
            continue;
        }
        
        // Build thread starting from this tweet
        let thread = build_thread_from_tweet(tweet, &tweet_map, &reply_map, &mut processed_ids);
        
        if !thread.is_empty() {
            threads.push(thread);
        }
    }
    
    // Sort threads by first tweet timestamp (newest first)
    threads.sort_by(|a, b| {
        b.first().map(|t| &t.created_at)
            .cmp(&a.first().map(|t| &t.created_at))
    });
    
    threads
}

/// Build a complete thread starting from a given tweet
fn build_thread_from_tweet(
    start_tweet: &Tweet,
    tweet_map: &HashMap<String, &Tweet>,
    reply_map: &HashMap<String, Vec<&Tweet>>,
    processed_ids: &mut HashSet<String>,
) -> Vec<Tweet> {
    let mut thread = Vec::new();
    
    // First, trace back to find the root of the thread
    let mut root_tweet = start_tweet;
    while let Some(parent_id) = &root_tweet.in_reply_to_status_id {
        if let Some(parent) = tweet_map.get(parent_id) {
            root_tweet = parent;
        } else {
            break;
        }
    }
    
    // Now build the thread forward from the root
    let mut stack = vec![root_tweet];
    let mut visited_ids = HashSet::new();
    
    while let Some(tweet) = stack.pop() {
        if !visited_ids.insert(tweet.id_str.clone()) {
            continue;
        }
        
        processed_ids.insert(tweet.id_str.clone());
        thread.push(tweet.clone());
        
        if let Some(reply_tweets) = reply_map.get(&tweet.id_str) {
            for reply_tweet in reply_tweets.iter().rev() {
                if !visited_ids.contains(&reply_tweet.id_str) {
                    stack.push(reply_tweet);
                }
            }
        }
    }
    
    // Sort thread chronologically
    thread.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    
    thread
}

/// Convert a thread of tweets into a human-readable format
pub fn format_thread_as_text(thread: &[Tweet], _screen_name: &str) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("🧵 Thread with {} tweets\n", thread.len()));
    output.push_str(&format!("{}\n", "─".repeat(50)));
    
    for (idx, tweet) in thread.iter().enumerate() {
        // Add thread position indicator
        if idx == 0 {
            output.push_str("🔹 [Thread Start]\n");
        } else if let Some(reply_to) = &tweet.in_reply_to_screen_name {
            output.push_str(&format!("↳ Reply to @{}\n", reply_to));
        }
        
        // Add tweet content
        output.push_str(&format!("{}\n", tweet.full_text));
        
        // Add metadata
        output.push_str(&format!("📅 {} | ❤️ {} | 🔁 {}\n", 
            tweet.created_at, 
            tweet.favorite_count, 
            tweet.retweet_count
        ));
        
        // Add separator between tweets
        if idx < thread.len() - 1 {
            output.push('\n');
        }
    }
    
    output.push_str(&format!("{}\n\n", "─".repeat(50)));
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processing::data_structures::TweetEntities;
    
    fn create_test_tweet(id: &str, text: &str, reply_to_id: Option<&str>, reply_to_user: Option<&str>) -> Tweet {
        Tweet {
            id_str: id.to_string(),
            id: id.to_string(),
            full_text: text.to_string(),
            created_at: format!("2023-01-01 12:{:02}:00", id.parse::<u32>().unwrap_or(0)),
            favorite_count: "0".to_string(),
            retweet_count: "0".to_string(),
            retweeted: false,
            favorited: false,
            truncated: false,
            lang: Some("en".to_string()),
            source: "test".to_string(),
            display_text_range: vec!["0".to_string(), text.len().to_string()],
            in_reply_to_status_id: reply_to_id.map(|s| s.to_string()),
            in_reply_to_status_id_str: reply_to_id.map(|s| s.to_string()),
            in_reply_to_user_id: None,
            in_reply_to_user_id_str: None,
            in_reply_to_screen_name: reply_to_user.map(|s| s.to_string()),
            edit_info: None,
            entities: Some(TweetEntities::default()),
            possibly_sensitive: None,
        }
    }
    
    #[test]
    fn test_simple_reply_thread() {
        let tweets = vec![
            create_test_tweet("1", "Original tweet", None, None),
            create_test_tweet("2", "@user Reply to original", Some("1"), Some("testuser")),
            create_test_tweet("3", "@user Another reply", Some("2"), Some("testuser")),
        ];
        
        let threads = process_reply_threads(&tweets, "testuser");
        
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].len(), 3);
        assert_eq!(threads[0][0].id_str, "1");
        assert_eq!(threads[0][1].id_str, "2");
        assert_eq!(threads[0][2].id_str, "3");
    }
    
    #[test]
    fn test_multiple_reply_threads() {
        let tweets = vec![
            create_test_tweet("1", "First thread", None, None),
            create_test_tweet("2", "@user Reply to first", Some("1"), Some("testuser")),
            create_test_tweet("3", "Second thread", None, None),
            create_test_tweet("4", "@other Reply to other user", Some("100"), Some("other")),
            create_test_tweet("5", "@user Reply to second", Some("3"), Some("testuser")),
        ];
        
        let threads = process_reply_threads(&tweets, "testuser");
        
        // Should have 3 threads:
        // 1. Thread starting with tweet 1 (tweets 1, 2)
        // 2. Thread starting with tweet 3 (tweets 3, 5)
        // 3. Orphan reply thread (tweet 4)
        assert_eq!(threads.len(), 3);
    }
    
    #[test]
    fn test_thread_formatting() {
        let thread = vec![
            create_test_tweet("1", "Starting a thread", None, None),
            create_test_tweet("2", "Continuing the thought", Some("1"), Some("testuser")),
        ];
        
        let formatted = format_thread_as_text(&thread, "testuser");
        
        assert!(formatted.contains("Thread with 2 tweets"));
        assert!(formatted.contains("[Thread Start]"));
        assert!(formatted.contains("Reply to @testuser"));
        assert!(formatted.contains("Starting a thread"));
        assert!(formatted.contains("Continuing the thought"));
    }
}
