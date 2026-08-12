//! Core data structures for tweet and DM processing

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc as async_mpsc;

/// Represents a tweet from the Twitter archive
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tweet {
    /// Twitter's string representation of the tweet ID
    pub id_str: String,
    /// Numeric tweet ID
    pub id: String,
    /// Full text content of the tweet
    pub full_text: String,
    /// Creation timestamp in Twitter's format
    pub created_at: String,
    /// Number of favorites/likes as a string
    pub favorite_count: String,
    /// Number of retweets as a string
    pub retweet_count: String,
    /// Whether this tweet has been retweeted by the user
    pub retweeted: bool,
    /// Whether this tweet has been favorited/liked by the user
    pub favorited: bool,
    /// Whether the tweet was truncated in the original response
    pub truncated: bool,
    /// Language code of the tweet
    #[serde(default)]
    pub lang: String,
    /// Source application used to post the tweet
    pub source: String,
    /// Display range indices for the tweet text
    pub display_text_range: Vec<String>,

    /// ID of the tweet being replied to (if this is a reply)
    pub in_reply_to_status_id: Option<String>,
    /// String representation of the tweet being replied to
    pub in_reply_to_status_id_str: Option<String>,
    /// ID of the user being replied to
    pub in_reply_to_user_id: Option<String>,
    /// String representation of the user ID being replied to
    pub in_reply_to_user_id_str: Option<String>,
    /// Screen name of the user being replied to
    pub in_reply_to_screen_name: Option<String>,

    /// Edit information for the tweet (may be missing in older tweets)
    #[serde(default)]
    pub edit_info: Option<EditInfo>,

    /// Tweet entities like mentions, hashtags, URLs (always present, but may be empty)
    pub entities: TweetEntities,

    /// Whether the tweet contains sensitive content
    #[serde(default)]
    pub possibly_sensitive: Option<bool>,
}

/// Edit information for tweets
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct EditInfo {
    /// Initial edit information for the tweet
    #[serde(default)]
    pub initial: Option<EditInitial>,
}

/// Initial edit information
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct EditInitial {
    /// IDs of tweets in the edit history
    #[serde(rename = "editTweetIds")]
    pub edit_tweet_ids: Vec<String>,
    /// Timestamp until which the tweet can be edited
    #[serde(rename = "editableUntil")]
    pub editable_until: String,
    /// Number of edits remaining for this tweet
    #[serde(rename = "editsRemaining")]
    pub edits_remaining: String,
    /// Whether the tweet is eligible for editing
    #[serde(rename = "isEditEligible")]
    pub is_edit_eligible: bool,
}

/// Tweet entities (mentions, hashtags, etc.)
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TweetEntities {
    /// Hashtags mentioned in the tweet
    pub hashtags: Vec<Hashtag>,
    /// Symbols (cashtags) mentioned in the tweet
    pub symbols: Vec<Symbol>,
    /// Users mentioned in the tweet
    pub user_mentions: Vec<UserMention>,
    /// URLs included in the tweet
    pub urls: Vec<TweetUrl>,
}

/// Hashtag in tweet
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Hashtag {
    /// Text of the hashtag/symbol without the # or $ symbol
    pub text: String,
    /// Position indices in the tweet text where this hashtag/symbol appears
    pub indices: Vec<String>,
}

/// Symbol in tweet (cashtags)
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Symbol {
    /// Text of the hashtag/symbol without the # or $ symbol
    pub text: String,
    /// Position indices in the tweet text where this hashtag/symbol appears
    pub indices: Vec<String>,
}

/// User mention in tweet
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct UserMention {
    /// Display name of the mentioned user
    pub name: String,
    /// Screen name (handle) of the mentioned user
    pub screen_name: String,
    /// Position indices in the tweet text where this mention appears
    pub indices: Vec<String>,
    /// String representation of the mentioned user's ID
    pub id_str: String,
    /// Numeric ID of the mentioned user
    pub id: String,
}

/// URL in tweet
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct TweetUrl {
    /// Shortened URL as it appears in the tweet
    pub url: String,
    /// Full expanded URL
    #[serde(default)]
    pub expanded_url: String,
    /// Display version of the URL
    #[serde(default)]
    pub display_url: String,
    /// Position indices in the tweet text where this URL appears
    pub indices: Vec<String>,
}

/// Wrapper for tweet data from JSON
#[derive(Deserialize, Debug, Clone)]
pub struct TweetWrapper {
    /// The actual tweet data
    pub tweet: Tweet,
}

/// Represents a conversation thread
#[derive(Debug, Clone)]
pub struct Thread {
    /// Unique identifier for the thread (first tweet's ID)
    pub id: String,
    /// All tweets in this thread in chronological order
    pub tweets: Vec<Tweet>,
    /// Total number of tweets in the thread
    pub tweet_count: usize,
    /// Total number of favorites/likes across all tweets in the thread
    pub favorite_count: u32,
    /// Total number of retweets across all tweets in the thread
    pub retweet_count: u32,
}

/// Represents a processed DM conversation
#[derive(Debug)]
pub struct ProcessedConversation {
    /// Unique identifier for the DM conversation
    pub conversation_id: String,
    /// Total number of messages in the conversation
    pub message_count: u32,
    /// List of participants in the conversation
    pub participants: Vec<String>,
    /// Timestamp of the first message in the conversation
    pub first_message_date: Option<String>,
    /// Timestamp of the last message in the conversation
    pub last_message_date: Option<String>,
}

/// CSV writer for async processing
pub struct CsvWriter {
    /// Path where the CSV file will be written
    pub output_path: String,
    /// Channel receiver for incoming CSV records
    pub receiver: async_mpsc::Receiver<Vec<String>>,
    /// Size of the buffer for batching writes
    pub buffer_size: usize,
}

impl CsvWriter {
    /// Creates a new CsvWriter instance
    pub fn new(
        output_path: String,
        receiver: async_mpsc::Receiver<Vec<String>>,
        buffer_size: usize,
    ) -> Self {
        Self {
            output_path,
            receiver,
            buffer_size,
        }
    }
}
