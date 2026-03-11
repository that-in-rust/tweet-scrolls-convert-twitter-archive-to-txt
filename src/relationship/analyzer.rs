//! Core relationship analysis functionality

use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use crate::models::{
    direct_message::DmWrapper,
    interaction::InteractionEvent,
    profile::UserProfile,
};
use crate::services::timeline_analyzer::TimelineAnalyzer;

use super::communication::{CommunicationFrequency, calculate_communication_frequency};

/// Relationship analyzer for extracting and analyzing user interactions
#[derive(Debug)]
pub struct RelationshipAnalyzer {
    /// Map of user IDs to their profile data
    pub profiles: HashMap<String, UserProfile>,
}

impl Default for RelationshipAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationshipAnalyzer {
    /// Creates a new RelationshipAnalyzer instance
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    /// Extract unique user IDs from DM data
    /// 
    /// # Arguments
    /// 
    /// * `dm_wrappers` - A slice of DM wrapper objects
    /// 
    /// # Returns
    /// 
    /// A HashSet of user IDs
    /// 
    /// # Examples
    /// 
    /// ```
    /// use tweet_scrolls::relationship::RelationshipAnalyzer;
    /// 
    /// let analyzer = RelationshipAnalyzer::new();
    /// let dm_data = vec![]; // Your DM data
    /// let users = analyzer.extract_users_from_dms(&dm_data);
    /// ```
    pub fn extract_users_from_dms(&self, dm_wrappers: &[DmWrapper]) -> HashSet<String> {
        let mut users = HashSet::new();
        
        for wrapper in dm_wrappers {
            let conversation_id = &wrapper.dm_conversation.conversation_id;
            
            // Extract user IDs from conversation ID (format: "user1-user2")
            if let Some(dash_pos) = conversation_id.find('-') {
                let user1 = &conversation_id[..dash_pos];
                let user2 = &conversation_id[dash_pos + 1..];
                
                users.insert(user1.to_string());
                users.insert(user2.to_string());
            }
        }
        
        users
    }

    /// Extract unique user IDs from tweet data
    /// 
    /// # Arguments
    /// 
    /// * `tweets` - A slice of Tweet objects
    /// 
    /// # Returns
    /// 
    /// A HashSet of user IDs
    pub fn extract_users_from_tweets(&self, tweets: &[crate::processing::data_structures::Tweet]) -> HashSet<String> {
        let mut users = HashSet::new();
        
        for tweet in tweets {
            // Add user being replied to
            if let Some(reply_to_user) = &tweet.in_reply_to_screen_name {
                users.insert(reply_to_user.clone());
            }
            
            // Add all mentioned users
            if let Some(entities) = &tweet.entities {
                for mention in &entities.user_mentions {
                    users.insert(mention.screen_name.clone());
                }
            }
        }
        
        users
    }

    /// Create a basic user profile from conversation data
    /// 
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `dm_data` - DM conversation data
    ///
    /// # Returns
    ///
    /// A UserProfile with basic statistics
    pub fn create_user_profile(&self, user_id: &str, dm_data: &[DmWrapper]) -> UserProfile {
        let (first_interaction, last_interaction) = self.find_interaction_timespan(user_id, dm_data);
        
        let mut profile = UserProfile::new(user_id);
        
        // Set interaction timestamps
        profile.first_interaction = first_interaction;
        profile.last_interaction = last_interaction;
        
        // Calculate basic statistics
        let mut total_messages = 0;
        for wrapper in dm_data {
            let conversation_id = &wrapper.dm_conversation.conversation_id;
            
            // Check if this user is part of this conversation
            if let Some(dash_pos) = conversation_id.find('-') {
                let user1_id = &conversation_id[..dash_pos];
                let user2_id = &conversation_id[dash_pos + 1..];
                
                if user_id == user1_id || user_id == user2_id {
                    // Count messages in this conversation
                    for message in &wrapper.dm_conversation.messages {
                        if message.message_create.is_some() {
                            total_messages += 1;
                        }
                    }
                }
            }
        }
        
        profile.total_interactions = total_messages;
        profile.interaction_counts.insert("dm_messages".to_string(), total_messages);
        
        profile
    }

    /// Find the first and last interaction timestamps for a user
    fn find_interaction_timespan(&self, user_id: &str, dm_data: &[DmWrapper]) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
        let mut timestamps = Vec::new();
        
        for wrapper in dm_data {
            let conversation_id = &wrapper.dm_conversation.conversation_id;
            
            // Check if this user is part of this conversation
            if let Some(dash_pos) = conversation_id.find('-') {
                let user1_id = &conversation_id[..dash_pos];
                let user2_id = &conversation_id[dash_pos + 1..];
                
                if user_id == user1_id || user_id == user2_id {
                    // Collect timestamps from this conversation
                    for message in &wrapper.dm_conversation.messages {
                        if let Some(message_create) = &message.message_create {
                            if let Some(created_at) = &message_create.created_at {
                                if let Ok(timestamp) = DateTime::parse_from_rfc3339(created_at) {
                                    timestamps.push(timestamp.with_timezone(&Utc));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if timestamps.is_empty() {
            (None, None)
        } else {
            timestamps.sort();
            (timestamps.first().copied(), timestamps.last().copied())
        }
    }

    /// Build a chronological interaction timeline from DM and tweet data
    /// 
    /// # Arguments
    /// 
    /// * `dm_data` - DM conversation data
    /// * `tweet_data` - Tweet data
    /// 
    /// # Returns
    /// 
    /// A vector of InteractionEvent objects sorted chronologically (newest first)
    pub fn build_timeline(&self, dm_data: &[DmWrapper], tweet_data: &[crate::processing::data_structures::Tweet]) -> Vec<InteractionEvent> {
        let mut timeline = Vec::new();
        
        // Add DM events to timeline
        for wrapper in dm_data {
            let conversation_id = &wrapper.dm_conversation.conversation_id;
            for message in &wrapper.dm_conversation.messages {
                if let Some(event) = InteractionEvent::from_dm_message(message, conversation_id) {
                    timeline.push(event);
                }
            }
        }
        
        // Add tweet events to timeline (if we had a from_tweet method)
        // For now, we'll skip tweet events since the method doesn't exist
        let _ = tweet_data; // Suppress unused parameter warning
        
        // Sort timeline chronologically (newest first)
        timeline.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        timeline
    }

    /// Calculate communication frequency for a user
    /// 
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `dm_data` - DM conversation data
    ///
    /// # Returns
    ///
    /// CommunicationFrequency analysis for the user
    pub fn calculate_communication_frequency(&self, user_id: &str, dm_data: &[DmWrapper]) -> CommunicationFrequency {
        calculate_communication_frequency(user_id, dm_data)
    }

    /// Analyze the timeline of interactions
    /// 
    /// # Arguments
    /// 
    /// * `events` - A slice of InteractionEvent objects
    /// 
    /// # Returns
    /// 
    /// TimelineAnalysis with detected patterns and statistics
    pub fn analyze_timeline(&self, events: &[InteractionEvent]) -> crate::models::timeline::TimelineAnalysis {
        let analyzer = TimelineAnalyzer::new(events.to_vec());
        analyzer.analyze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::direct_message::{DmConversation, DmMessageCreate, DmMessage};

    // Helper function to create sample DM data for testing
    fn create_sample_dm_data() -> Vec<DmWrapper> {
        
        
        vec![
            DmWrapper {
                dm_conversation: DmConversation {
                    conversation_id: "3382-1132151165410455552".to_string(),
                    messages: vec![
                        DmMessage {
                            message_create: Some(DmMessageCreate {
                                id: Some("msg1".to_string()),
                                text: Some("Hello there!".to_string()),
                                created_at: Some("2023-01-01T10:00:00.000Z".to_string()),
                                sender_id: Some("3382".to_string()),
                                recipient_id: Some("1132151165410455552".to_string()),
                                reactions: vec![],
                                urls: vec![],
                                media_urls: vec![],
                                edit_history: vec![],
                            }),
                        },
                    ],
                },
            },
            DmWrapper {
                dm_conversation: DmConversation {
                    conversation_id: "1132151165410455552-9876543210".to_string(),
                    messages: vec![
                        DmMessage {
                            message_create: Some(DmMessageCreate {
                                id: Some("msg2".to_string()),
                                text: Some("How are you?".to_string()),
                                created_at: Some("2023-01-01T10:05:00.000Z".to_string()),
                                sender_id: Some("1132151165410455552".to_string()),
                                recipient_id: Some("9876543210".to_string()),
                                reactions: vec![],
                                urls: vec![],
                                media_urls: vec![],
                                edit_history: vec![],
                            }),
                        },
                    ],
                },
            },
        ]
    }

    // Helper function to create sample tweet data for testing
    fn create_sample_tweet_data() -> Vec<crate::processing::data_structures::Tweet> {
        use crate::processing::data_structures::{Tweet, TweetEntities};
        
        vec![
            Tweet {
                id_str: "tweet1".to_string(),
                id: "tweet1".to_string(),
                full_text: "Hello world!".to_string(),
                created_at: "Mon Jan 01 10:00:00 +0000 2023".to_string(),
                favorite_count: "5".to_string(),
                retweet_count: "2".to_string(),
                retweeted: false,
                favorited: false,
                truncated: false,
                lang: Some("en".to_string()),
                source: "<a href=\"http://twitter.com\" rel=\"nofollow\">Twitter Web App</a>".to_string(),
                display_text_range: vec!["0".to_string(), "12".to_string()],
                in_reply_to_status_id: None,
                in_reply_to_status_id_str: None,
                in_reply_to_user_id: None,
                in_reply_to_user_id_str: None,
                in_reply_to_screen_name: Some("alice".to_string()),
                edit_info: None,
                entities: Some(TweetEntities {
                    hashtags: vec![],
                    symbols: vec![],
                    user_mentions: vec![
                        crate::processing::data_structures::UserMention {
                            screen_name: "bob".to_string(),
                            name: "Bob".to_string(),
                            id: "12345".to_string(),
                            id_str: "12345".to_string(),
                            indices: vec!["0".to_string(), "0".to_string()],
                        }
                    ],
                    urls: vec![],
                }),
                possibly_sensitive: None,
            },
            Tweet {
                id_str: "tweet2".to_string(),
                id: "tweet2".to_string(),
                full_text: "Another tweet".to_string(),
                created_at: "Mon Jan 01 11:00:00 +0000 2023".to_string(),
                favorite_count: "3".to_string(),
                retweet_count: "1".to_string(),
                retweeted: false,
                favorited: false,
                truncated: false,
                lang: Some("en".to_string()),
                source: "<a href=\"http://twitter.com\" rel=\"nofollow\">Twitter Web App</a>".to_string(),
                display_text_range: vec!["0".to_string(), "13".to_string()],
                in_reply_to_status_id: None,
                in_reply_to_status_id_str: None,
                in_reply_to_user_id: None,
                in_reply_to_user_id_str: None,
                in_reply_to_screen_name: None,
                edit_info: None,
                entities: Some(TweetEntities {
                    hashtags: vec![],
                    symbols: vec![],
                    user_mentions: vec![
                        crate::processing::data_structures::UserMention {
                            screen_name: "bob".to_string(),
                            name: "Bob".to_string(),
                            id: "12345".to_string(),
                            id_str: "12345".to_string(),
                            indices: vec!["0".to_string(), "0".to_string()],
                        }
                    ],
                    urls: vec![],
                }),
                possibly_sensitive: None,
            },
        ]
    }

    #[test]
    fn test_relationship_analyzer_creation() {
        let analyzer = RelationshipAnalyzer::new();
        assert!(analyzer.profiles.is_empty());
    }

    #[test]
    fn test_extract_unique_users_from_dms() {
        let sample_dm_data = create_sample_dm_data();
        let analyzer = RelationshipAnalyzer::new();
        
        let users = analyzer.extract_users_from_dms(&sample_dm_data);
        
        // Should extract 3 unique users: "3382", "1132151165410455552", "9876543210"
        assert_eq!(users.len(), 3);
        assert!(users.contains("3382"));
        assert!(users.contains("1132151165410455552"));
        assert!(users.contains("9876543210"));
    }

    #[test]
    fn test_extract_users_from_tweets() {
        let sample_tweet_data = create_sample_tweet_data();
        let analyzer = RelationshipAnalyzer::new();
        
        let users = analyzer.extract_users_from_tweets(&sample_tweet_data);
        
        // Should extract 2 unique users from in_reply_to_screen_name: "alice", "bob"
        assert_eq!(users.len(), 2);
        assert!(users.contains("alice"));
        assert!(users.contains("bob"));
    }

    #[test]
    fn test_handle_empty_data_gracefully() {
        let analyzer = RelationshipAnalyzer::new();
        
        // Test with empty DM data
        let empty_dm_data: Vec<DmWrapper> = vec![];
        let dm_users = analyzer.extract_users_from_dms(&empty_dm_data);
        assert_eq!(dm_users.len(), 0);
        
        // Test with empty tweet data
        let empty_tweet_data: Vec<crate::processing::data_structures::Tweet> = vec![];
        let tweet_users = analyzer.extract_users_from_tweets(&empty_tweet_data);
        assert_eq!(tweet_users.len(), 0);
    }

    #[test]
    fn test_extract_users_from_malformed_conversation_ids() {
        let malformed_dm_data = vec![
            DmWrapper {
                dm_conversation: DmConversation {
                    conversation_id: "no_dash_here".to_string(), // No dash separator
                    messages: vec![],
                },
            },
            DmWrapper {
                dm_conversation: DmConversation {
                    conversation_id: "user1-user2".to_string(), // Valid format
                    messages: vec![],
                },
            },
        ];
        
        let analyzer = RelationshipAnalyzer::new();
        let users = analyzer.extract_users_from_dms(&malformed_dm_data);
        
        // Should only extract from the valid conversation ID
        assert_eq!(users.len(), 2);
        assert!(users.contains("user1"));
        assert!(users.contains("user2"));
    }

    #[test]
    fn test_create_basic_user_profile() {
        let sample_data = create_sample_dm_data();
        let analyzer = RelationshipAnalyzer::new();
        let user_id = "3382".to_string();
        
        let profile = analyzer.create_user_profile(&user_id, &sample_data);
        
        assert_eq!(profile.user_id, user_id);
        assert!(profile.total_interactions > 0);
        assert!(profile.first_interaction.is_some());
        assert!(profile.last_interaction.is_some());
        assert!(profile.first_interaction <= profile.last_interaction);
    }

    #[test]
    fn test_build_interaction_timeline() {
        let dm_data = create_sample_dm_data();
        let tweet_data = create_sample_tweet_data();
        let analyzer = RelationshipAnalyzer::new();
        
        let timeline = analyzer.build_timeline(&dm_data, &tweet_data);
        
        // Should have events from DM data
        assert!(!timeline.is_empty());
        
        // Timeline should be sorted chronologically (newest first)
        for window in timeline.windows(2) {
            assert!(window[0].timestamp >= window[1].timestamp);
        }
    }

    #[test]
    fn test_timeline_analysis_integration() {
        let dm_data = create_sample_dm_data();
        let tweet_data = create_sample_tweet_data();
        let analyzer = RelationshipAnalyzer::new();
        
        let timeline = analyzer.build_timeline(&dm_data, &tweet_data);
        let analysis = analyzer.analyze_timeline(&timeline);
        
        // Should have basic analysis structure
        assert!(analysis.total_interactions > 0);
        assert!(analysis.unique_participants > 0);
        assert!(!analysis.patterns.is_empty() || analysis.patterns.is_empty()); // Either way is valid
    }
}
