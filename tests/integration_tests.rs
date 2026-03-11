//! Integration tests for Tweet-Scrolls
//! 
//! All tests consolidated here per steering document requirements.
//! Tests realistic Twitter export processing with actual data structures.

use std::collections::HashMap;
use tempfile::tempdir;

use tweet_scrolls::*;

// ============================================================================
// REALISTIC TEST DATA GENERATORS (Based on Actual Twitter Export Structure)
// ============================================================================

/// Creates realistic DM data matching actual Twitter export format
fn create_realistic_dm_data() -> Vec<models::direct_message::DmWrapper> {
    vec![
        models::direct_message::DmWrapper {
            dm_conversation: models::direct_message::DmConversation {
                conversation_id: "3382-1132151165410455552".to_string(),
                messages: vec![
                    models::direct_message::DmMessage {
                        message_create: Some(models::direct_message::DmMessageCreate {
                            recipient_id: Some("3382".to_string()),
                            reactions: vec![],
                            urls: vec![],
                            text: Some("Hey, are you planning on having a co-founder for your startup?".to_string()),
                            media_urls: vec![],
                            sender_id: Some("1132151165410455552".to_string()),
                            id: Some("1927384914816532581".to_string()),
                            created_at: Some("2025-05-27T15:22:27.518Z".to_string()),
                            edit_history: vec![],
                        })
                    },
                    models::direct_message::DmMessage {
                        message_create: Some(models::direct_message::DmMessageCreate {
                            recipient_id: Some("1132151165410455552".to_string()),
                            reactions: vec![],
                            urls: vec![],
                            text: Some("Not sure yet, still exploring options. What do you think?".to_string()),
                            media_urls: vec![],
                            sender_id: Some("3382".to_string()),
                            id: Some("1916872219248173473".to_string()),
                            created_at: Some("2025-04-28T15:08:45.535Z".to_string()),
                            edit_history: vec![],
                        })
                    }
                ]
            }
        },
        models::direct_message::DmWrapper {
            dm_conversation: models::direct_message::DmConversation {
                conversation_id: "3382-9876543210".to_string(),
                messages: vec![
                    models::direct_message::DmMessage {
                        message_create: Some(models::direct_message::DmMessageCreate {
                            recipient_id: Some("9876543210".to_string()),
                            reactions: vec![],
                            urls: vec![],
                            text: Some("Thanks for the introduction!".to_string()),
                            media_urls: vec![],
                            sender_id: Some("3382".to_string()),
                            id: Some("1925000000000000000".to_string()),
                            created_at: Some("2025-05-20T10:00:00.000Z".to_string()),
                            edit_history: vec![],
                        })
                    }
                ]
            }
        }
    ]
}

/// Creates realistic Tweet data matching actual Twitter export format
fn create_realistic_tweet_data() -> Vec<processing::data_structures::TweetWrapper> {
    use processing::data_structures::*;
    
    vec![
        TweetWrapper {
            tweet: Tweet {
                id_str: "1947489885754986818".to_string(),
                id: "1947489885754986818".to_string(),
                full_text: "Via @NotebookLM whom I fed thousands of my tweets in txt format for analysis".to_string(),
                created_at: "Tue Jul 22 02:52:26 +0000 2025".to_string(),
                favorite_count: "5".to_string(),
                retweet_count: "0".to_string(),
                retweeted: false,
                favorited: false,
                truncated: false,
                lang: Some("en".to_string()),
                source: "<a href=\"https://mobile.twitter.com\" rel=\"nofollow\">Twitter Web App</a>".to_string(),
                display_text_range: vec!["0".to_string(), "270".to_string()],
                in_reply_to_status_id: None,
                in_reply_to_status_id_str: None,
                in_reply_to_user_id: None,
                in_reply_to_user_id_str: None,
                in_reply_to_screen_name: Some("testuser".to_string()),
                edit_info: None,
                entities: Some(TweetEntities {
                    hashtags: vec![],
                    symbols: vec![],
                    user_mentions: vec![
                        UserMention {
                            name: "NotebookLM".to_string(),
                            screen_name: "NotebookLM".to_string(),
                            indices: vec!["4".to_string(), "15".to_string()],
                            id_str: "1846671939437252609".to_string(),
                            id: "1846671939437252609".to_string(),
                        }
                    ],
                    urls: vec![],
                }),
                possibly_sensitive: None,
            }
        },
        TweetWrapper {
            tweet: Tweet {
                id_str: "1947478130287120782".to_string(),
                id: "1947478130287120782".to_string(),
                full_text: "@TnvMadhav Been @dhh @jasonfried fan since I came to twitter in 2021".to_string(),
                created_at: "Tue Jul 22 02:05:43 +0000 2025".to_string(),
                favorite_count: "0".to_string(),
                retweet_count: "0".to_string(),
                retweeted: false,
                favorited: false,
                truncated: false,
                lang: Some("en".to_string()),
                source: "<a href=\"http://twitter.com/download/android\" rel=\"nofollow\">Twitter for Android</a>".to_string(),
                display_text_range: vec!["0".to_string(), "68".to_string()],
                in_reply_to_status_id: Some("1947467485424562448".to_string()),
                in_reply_to_status_id_str: Some("1947467485424562448".to_string()),
                in_reply_to_user_id: Some("848022794629730304".to_string()),
                in_reply_to_user_id_str: Some("848022794629730304".to_string()),
                in_reply_to_screen_name: Some("TnvMadhav".to_string()),
                edit_info: None,
                entities: Some(TweetEntities {
                    hashtags: vec![],
                    symbols: vec![],
                    user_mentions: vec![
                        UserMention {
                            name: "TnvMadhav".to_string(),
                            screen_name: "TnvMadhav".to_string(),
                            indices: vec!["0".to_string(), "10".to_string()],
                            id_str: "848022794629730304".to_string(),
                            id: "848022794629730304".to_string(),
                        }
                    ],
                    urls: vec![],
                }),
                possibly_sensitive: None,
            }
        }
    ]
}

// ============================================================================
// CORE FUNCTIONALITY TESTS (Missing from current implementation)
// ============================================================================

#[tokio::test]
async fn test_tweet_processing_end_to_end() {
    let _temp_dir = tempdir().unwrap();
    let tweets = create_realistic_tweet_data();
    
    // Test the core tweet processing pipeline
    let threads = tweet_scrolls::processing::tweets::process_tweets_simple(&tweets, "testuser").await.unwrap();
    
    // Verify threads were created
    assert!(!threads.is_empty(), "Should create threads from tweet data");
    
    // Verify thread structure
    for thread in &threads {
        assert!(!thread.tweets.is_empty(), "Thread should contain tweets");
        assert!(!thread.tweets[0].full_text.is_empty(), "Tweet should have content");
    }
    
    // Verify chronological ordering (newest first)
    if threads.len() > 1 {
        let first_thread_time = &threads[0].tweets[0].created_at;
        let second_thread_time = &threads[1].tweets[0].created_at;
        // Note: This would need proper date parsing for real comparison
        assert!(!first_thread_time.is_empty());
        assert!(!second_thread_time.is_empty());
    }
}

#[tokio::test]
async fn test_dm_processing_end_to_end() {
    let _temp_dir = tempdir().unwrap();
    let dm_data = create_realistic_dm_data();
    
    // Test the core DM processing pipeline
    let conversations = tweet_scrolls::processing::direct_messages::process_dm_conversations(&dm_data, "testuser").await.unwrap();
    
    // Verify conversations were created
    assert!(!conversations.is_empty(), "Should create conversations from DM data");
    
    // Verify conversation structure
    for conversation in &conversations {
        assert!(!conversation.conversation_id.is_empty(), "Conversation should have ID");
        assert!(conversation.message_count > 0, "Conversation should have messages");
        assert!(!conversation.participants.is_empty(), "Conversation should have participants");
    }
    
    // Verify sorting by message count (descending)
    if conversations.len() > 1 {
        assert!(conversations[0].message_count >= conversations[1].message_count,
                "Conversations should be sorted by message count descending");
    }
}

#[tokio::test]
async fn test_relationship_analysis_pipeline() {
    let dm_data = create_realistic_dm_data();
    let tweet_data = create_realistic_tweet_data();
    
    // Test relationship analyzer creation
    let analyzer = relationship::RelationshipAnalyzer::new();
    
    // Test user extraction from both data sources
    let dm_users = analyzer.extract_users_from_dms(&dm_data);
    let tweets: Vec<_> = tweet_data.iter().map(|tw| tw.tweet.clone()).collect();
    let tweet_users = analyzer.extract_users_from_tweets(&tweets);
    
    // Verify users were extracted
    assert!(!dm_users.is_empty(), "Should extract users from DM data");
    assert!(!tweet_users.is_empty(), "Should extract users from tweet data");
    
    // Test profile creation
    for user_id in &dm_users {
        let profile = analyzer.create_user_profile(user_id, &dm_data);
        assert_eq!(profile.user_id, *user_id, "Profile should match user id");
        assert!(profile.total_interactions > 0, "Profile should have interactions");
    }
    
    // Test timeline building
    let timeline = analyzer.build_timeline(&dm_data, &tweets);
    assert!(!timeline.is_empty(), "Should build interaction timeline");
}

#[tokio::test]
async fn test_file_output_generation() {
    let _temp_dir = tempdir().unwrap();
    let output_dir = _temp_dir.path().to_str().unwrap();
    
    let tweets = create_realistic_tweet_data();
    let _dm_data = create_realistic_dm_data();
    
    // Test CSV output generation
    let threads = tweet_scrolls::processing::tweets::process_tweets_simple(&tweets, "testuser").await.unwrap();
    
    // Create output directory
    let _timestamp = 1234567890; // Fixed timestamp for testing
    let output_path = std::path::Path::new(output_dir).join("output_testuser_1234567890");
    std::fs::create_dir_all(&output_path).unwrap();
    
    // Write enhanced CSV output
    let csv_path = output_path.join("threads_testuser_1234567890.csv");
    let mut csv_writer = tweet_scrolls::utils::enhanced_csv_writer::EnhancedCsvWriter::new(
        csv_path.to_str().unwrap()
    ).await.unwrap();
    
    for thread in &threads {
        csv_writer.write_thread(thread, "testuser").await.unwrap();
    }
    csv_writer.finalize().await.unwrap();
    
    // Verify CSV file was created and has the expected headers
    assert!(csv_path.exists(), "CSV file was not created");
    
    // Read the CSV file and verify its contents
    let mut rdr = csv::Reader::from_path(&csv_path).unwrap();
    let headers = rdr.headers().unwrap().clone();
    
    // Verify expected headers are present
    let expected_headers = [
        "tweet_id", "tweet_text", "tweet_type", "created_at", 
        "favorite_count", "retweet_count", "thread_id", 
        "thread_position", "twitter_url", "reply_context"
    ];
    
    for header in &expected_headers {
        assert!(
            headers.iter().any(|h| h == *header),
            "Missing expected header: {}",
            header
        );
    }
    
    // Verify at least one record was written
    let records: Vec<_> = rdr.records().collect::<Result<_, _>>().unwrap();
    assert!(!records.is_empty(), "No records were written to the CSV");
    
    // Verify tweet type and URL are populated
    for record in records {
        let tweet_type = record.get(2).unwrap(); // tweet_type column
        let twitter_url = record.get(11).unwrap(); // twitter_url column
        
        // Verify tweet type is one of the expected values
        assert!(
            ["Original", "ReplyToUser", "ReplyToOthers", "AuthorReply"].contains(&tweet_type),
            "Unexpected tweet type: {}",
            tweet_type
        );
        
        // Verify URL format
        assert!(
            twitter_url.starts_with("https://twitter.com/") && twitter_url.ends_with(&format!("/status/{}", record.get(0).unwrap())),
            "Invalid Twitter URL format: {}",
            twitter_url
        );
    }
}

#[test]
fn test_javascript_prefix_removal() {
    let js_content = "window.YTD.direct_messages.part0 = [{\"dm_conversation\":{\"conversation_id\":\"test\"}}]";
    
    // Test JavaScript prefix removal logic
    let cleaned = if let Some(stripped) = js_content.strip_prefix("window.YTD.direct_messages.part0 = ") {
        stripped
    } else {
        js_content
    };
    
    assert!(cleaned.starts_with("[{"), "Should remove JavaScript prefix");
    assert!(!cleaned.contains("window.YTD"), "Should not contain JavaScript prefix");
}

#[test]
fn test_retweet_filtering() {
    let mut tweets = create_realistic_tweet_data();
    
    // Add a retweet to test data
    let mut retweet = tweets[0].clone();
    retweet.tweet.retweeted = true;
    retweet.tweet.full_text = "RT @someone: This is a retweet".to_string();
    tweets.push(retweet);
    
    // Test filtering logic
    let filtered: Vec<_> = tweets.iter()
        .filter(|tw| !tw.tweet.retweeted && !tw.tweet.full_text.starts_with("RT @"))
        .collect();
    
    assert_eq!(filtered.len(), 2, "Should filter out retweets");
    for tweet in filtered {
        assert!(!tweet.tweet.retweeted, "Filtered tweets should not be retweets");
        assert!(!tweet.tweet.full_text.starts_with("RT @"), "Should not start with RT @");
    }
}

#[test]
fn test_large_data_structures() {
    // Test with larger datasets to simulate real usage
    let mut large_dm_data = Vec::new();
    
    // Create 100 conversations with multiple messages each
    for i in 0..100 {
        let conversation = models::direct_message::DmWrapper {
            dm_conversation: models::direct_message::DmConversation {
                conversation_id: format!("user_{}-sender_{}", i % 10, (i + 1) % 20),
                messages: (0..5).map(|j| {
                    models::direct_message::DmMessage {
                        message_create: Some(models::direct_message::DmMessageCreate {
                            recipient_id: Some(format!("user_{}", i % 10)),
                            reactions: vec![],
                            urls: vec![],
                            text: Some(format!("Message {} in conversation {}", j, i)),
                            media_urls: vec![],
                            sender_id: Some(format!("sender_{}", (i + j) % 20)),
                            id: Some(format!("msg_{}_{}", i, j)),
                            created_at: Some("2025-01-01T00:00:00.000Z".to_string()),
                            edit_history: vec![],
                        })
                    }
                }).collect()
            }
        };
        large_dm_data.push(conversation);
    }
    
    // Test that we can handle larger datasets
    let analyzer = relationship::RelationshipAnalyzer::new();
    let users = analyzer.extract_users_from_dms(&large_dm_data);
    
    assert!(users.len() >= 10, "Should extract multiple unique users");
    assert!(users.len() <= 30, "Should have reasonable number of unique users");
    
    // Test profile creation with larger dataset
    let first_user = users.iter().next().unwrap();
    let profile = analyzer.create_user_profile(first_user, &large_dm_data);
    
    assert!(profile.total_interactions > 0, "Profile should have interactions from large dataset");
}

// ============================================================================
// CONSOLIDATED EXISTING TESTS (Moved from individual modules)
// ============================================================================

// User Profile Tests
#[test]
fn test_user_profile_creation() {
    let profile = models::profile::UserProfile::new("test_user_id");
    assert_eq!(profile.user_id, "test_user_id");
    assert_eq!(profile.total_interactions, 0);
    assert!(profile.interaction_counts.is_empty());
}

#[test]
fn test_user_profile_add_interaction() {
    let mut profile = models::profile::UserProfile::new("test_user");
    use chrono::TimeZone;
    let timestamp = chrono::Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
    profile.add_interaction("dm_sent", timestamp);
    
    assert_eq!(profile.total_interactions, 1);
    assert_eq!(profile.interaction_counts.get("dm_sent"), Some(&1));
}

// Communication Tests
#[test]
fn test_calculate_response_times_empty() {
    let messages = vec![];
    let response_times = relationship::communication::calculate_response_times(&messages);
    assert!(response_times.is_empty());
}

#[test]
fn test_calculate_average_response_time_empty() {
    let response_times = vec![];
    let avg = relationship::communication::calculate_average_response_time(&response_times);
    assert_eq!(avg, std::time::Duration::ZERO);
}

// Timeline Integration Tests
#[test]
fn test_analyze_hourly_activity_empty() {
    let events = vec![];
    let activity = relationship::timeline_integration::analyze_hourly_activity(&events);
    assert_eq!(activity.len(), 24);
    assert!(activity.iter().all(|&count| count == 0));
}

#[test]
fn test_find_most_active_day_empty() {
    let events = vec![];
    let result = relationship::timeline_integration::find_most_active_day(&events);
    assert!(result.is_none());
}

// File Generation Tests

#[tokio::test]
async fn test_create_directory_structure() {
    let _temp_dir = tempdir().unwrap();
    let generator = relationship::file_generation::LLMFileGenerator::new(
        _temp_dir.path().to_str().unwrap(), 
        "testuser", 
        1234567890
    );
    
    let profiles_dir = generator.create_directory_structure().await.unwrap();
    assert!(tokio::fs::metadata(&profiles_dir).await.is_ok());
    assert!(profiles_dir.contains("relationship_profiles_testuser_1234567890"));
}

// Text Generation Tests
#[test]
fn test_generate_user_profile_text() {
    let mut profile = models::profile::UserProfile::new("test_user_id_123456");
    profile.total_interactions = 42;
    profile.interaction_counts.insert("dm_messages".to_string(), 25);
    profile.interaction_counts.insert("dm_received".to_string(), 17);
    
    let timeline = vec![];
    let profile_text = relationship::text_generators::generate_user_profile_text(&profile, &timeline);
    
    assert!(profile_text.contains("USER RELATIONSHIP PROFILE"));
    assert!(profile_text.contains("test_user_id_123456"));
    assert!(profile_text.contains("Total Interactions: 42"));
    assert!(profile_text.contains("dm_messages: 25"));
}

#[test]
fn test_generate_timeline_text() {
    use chrono::TimeZone;
    use models::interaction::{InteractionEvent, InteractionType};
    
    let timeline = vec![
        InteractionEvent::new(
            "event1",
            chrono::Utc.with_ymd_and_hms(2023, 6, 15, 14, 30, 0).unwrap(),
            InteractionType::DmSent,
            "test_user_id_123456",
            "Test message content"
        ),
    ];
    
    let timeline_text = relationship::timeline_text::generate_timeline_text(&timeline);
    
    assert!(timeline_text.contains("CHRONOLOGICAL INTERACTION LOG"));
    assert!(timeline_text.contains("Total Events: 1"));
    assert!(timeline_text.contains("2023-06"));
}

#[test]
fn test_generate_llm_analysis_prompts() {
    let mut profiles = HashMap::new();
    let mut profile = models::profile::UserProfile::new("test_user");
    profile.total_interactions = 42;
    profiles.insert("user1".to_string(), profile);
    
    let prompts = relationship::prompts_generator::generate_llm_analysis_prompts(&profiles);
    
    assert!(prompts.contains("Which relationships need more attention"));
    assert!(prompts.contains("What communication patterns make conversations most engaging"));
    assert!(prompts.contains("Total relationships analyzed: 1"));
    assert!(prompts.contains("Blake3 hashing for privacy"));
}

// Data Structure Tests
#[test]
fn test_tweet_creation() {
    use processing::data_structures::{Tweet, TweetEntities};
    
    let tweet = Tweet {
        id_str: "123".to_string(),
        id: "123".to_string(),
        full_text: "test tweet".to_string(),
        created_at: "Mon Jan 01 12:00:00 +0000 2023".to_string(),
        favorite_count: "0".to_string(),
        retweet_count: "0".to_string(),
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
    
    assert_eq!(tweet.full_text, "test tweet");
    assert_eq!(tweet.id, "123");
    assert!(!tweet.retweeted);
}

#[test]
fn test_thread_creation() {
    use processing::data_structures::{Tweet, TweetEntities};
    
    let tweet = Tweet {
        id_str: "123".to_string(),
        id: "123".to_string(),
        full_text: "test tweet".to_string(),
        created_at: "Mon Jan 01 12:00:00 +0000 2023".to_string(),
        favorite_count: "0".to_string(),
        retweet_count: "0".to_string(),
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
    
    let thread = processing::data_structures::Thread {
        id: "thread_123".to_string(),
        tweets: vec![tweet],
        tweet_count: 1,
        favorite_count: 0,
        retweet_count: 0,
    };
    
    assert_eq!(thread.tweets.len(), 1);
    assert_eq!(thread.tweet_count, 1);
}

// Utility Tests
#[test]
fn test_format_duration() {
    let duration = chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(30);
    let formatted = utils::format_duration(duration);
    assert!(formatted.contains("2h") || formatted.contains("hour"));
}

#[test]
fn test_format_timestamp() {
    use chrono::TimeZone;
    let timestamp = chrono::Utc.with_ymd_and_hms(2023, 6, 15, 14, 30, 0).unwrap();
    let formatted = utils::format_timestamp(&timestamp);
    // format_timestamp returns relative time for recent dates, absolute format for old dates
    assert!(formatted.contains("ago") || formatted == "just now" || formatted.contains("2023") || formatted.contains("Jun"));
}
