use crate::models::tweet_classification::TweetType;
use crate::processing::data_structures::Tweet;

/// Classify a tweet based on its reply status
pub fn classify_tweet_type(tweet: &Tweet, screen_name: &str) -> TweetType {
    if tweet.in_reply_to_status_id_str.is_none() {
        TweetType::Original
    } else if tweet.in_reply_to_screen_name.as_deref() == Some(screen_name) {
        TweetType::ReplyToUser
    } else {
        TweetType::ReplyToOthers
    }
}

/// Generate Twitter URL for a tweet
pub fn generate_twitter_url(tweet: &Tweet, screen_name: &str) -> String {
    format!("https://twitter.com/{}/status/{}", screen_name, tweet.id_str)
}

/// Create reply context string
pub fn create_reply_context(tweet: &Tweet) -> Option<String> {
    if let Some(reply_to_id) = &tweet.in_reply_to_status_id_str {
        if let Some(reply_to_user) = &tweet.in_reply_to_screen_name {
            Some(format!("Reply to @{} ({})", reply_to_user, reply_to_id))
        } else {
            Some(format!("Reply to tweet {}", reply_to_id))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processing::data_structures::TweetEntities;

    fn create_test_tweet(id: &str, reply_to_id: Option<&str>, reply_to_user: Option<&str>) -> Tweet {
        Tweet {
            id_str: id.to_string(),
            id: id.to_string(),
            full_text: "Test tweet".to_string(),
            created_at: "Mon Jan 01 12:00:00 +0000 2023".to_string(),
            favorite_count: "0".to_string(),
            retweet_count: "0".to_string(),
            retweeted: false,
            favorited: false,
            truncated: false,
            lang: Some("en".to_string()),
            source: "Twitter Web App".to_string(),
            display_text_range: vec!["0".to_string(), "10".to_string()],
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

    #[test]
    fn test_classify_original_tweet() {
        let tweet = create_test_tweet("1", None, None);
        assert_eq!(classify_tweet_type(&tweet, "testuser"), TweetType::Original);
    }

    #[test]
    fn test_classify_reply_to_user() {
        let tweet = create_test_tweet("2", Some("1"), Some("testuser"));
        assert_eq!(classify_tweet_type(&tweet, "testuser"), TweetType::ReplyToUser);
    }

    #[test]
    fn test_classify_reply_to_others() {
        let tweet = create_test_tweet("3", Some("1"), Some("otheruser"));
        assert_eq!(classify_tweet_type(&tweet, "testuser"), TweetType::ReplyToOthers);
    }

    #[test]
    fn test_generate_twitter_url() {
        let tweet = create_test_tweet("123456789", None, None);
        assert_eq!(
            generate_twitter_url(&tweet, "testuser"), 
            "https://twitter.com/testuser/status/123456789"
        );
    }

    #[test]
    fn test_create_reply_context() {
        let tweet = create_test_tweet("2", Some("1"), Some("otheruser"));
        assert_eq!(
            create_reply_context(&tweet),
            Some("Reply to @otheruser (1)".to_string())
        );
    }

    #[test]
    fn test_create_reply_context_none() {
        let tweet = create_test_tweet("1", None, None);
        assert_eq!(create_reply_context(&tweet), None);
    }
}
