//! MVP Relationship and Timeline Analyzer
//! 
//! Provides simple, immediately useful analysis of Twitter data:
//! - Who you interact with most
//! - When you're most active
//! - Clean, readable output

use anyhow::Result;
use chrono::{DateTime, Timelike};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs as async_fs;

#[allow(unused_imports)]
use super::data_structures::{Thread, Tweet, TweetEntities, UserMention, EditInfo, EditInitial};
use crate::models::direct_message::DmWrapper;

/// Simple relationship statistics
#[derive(Debug, Clone)]
pub struct SimpleRelationship {
    /// The username of the user in this relationship
    pub username: String,
    /// Total number of interactions with this user
    pub interaction_count: u32,
    /// Timestamp of the last interaction
    pub last_interaction: String,
    /// Type of interactions: "tweets", "dms", or "both"
    pub interaction_type: String,
}

/// Simple activity pattern
#[derive(Debug, Clone)]
pub struct ActivityPattern {
    /// Hour of the day (0-23)
    pub hour: u32,
    /// Number of activities in this time period
    pub activity_count: u32,
    /// Day of the week as string
    pub day_of_week: String,
}

/// MVP Analyzer for immediate insights
pub struct MvpAnalyzer {
    /// Map of usernames to their relationship data
    pub relationships: HashMap<String, SimpleRelationship>,
    /// Activity counts by hour of day (0-23)
    pub hourly_activity: HashMap<u32, u32>,
    /// Activity counts by day of week
    pub daily_activity: HashMap<String, u32>,
}

impl Default for MvpAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl MvpAnalyzer {
    /// Create a new MVP analyzer
    pub fn new() -> Self {
        Self {
            relationships: HashMap::new(),
            hourly_activity: HashMap::new(),
            daily_activity: HashMap::new(),
        }
    }

    /// Analyze tweets for relationships and activity patterns
    pub fn analyze_tweets(&mut self, threads: &[Thread]) -> Result<()> {
        for thread in threads {
            for tweet in &thread.tweets {
                // Extract timestamp for activity analysis
                if let Ok(dt) = DateTime::parse_from_str(&tweet.created_at, "%a %b %d %H:%M:%S %z %Y") {
                    let hour = dt.hour();
                    let day = dt.format("%A").to_string();
                    
                    *self.hourly_activity.entry(hour).or_insert(0) += 1;
                    *self.daily_activity.entry(day).or_insert(0) += 1;
                }

                // Extract relationships from mentions
                for mention in &tweet.entities.user_mentions {
                    let username = mention.screen_name.clone();
                    let relationship = self.relationships.entry(username.clone()).or_insert(SimpleRelationship {
                        username: username.clone(),
                        interaction_count: 0,
                        last_interaction: tweet.created_at.clone(),
                        interaction_type: "tweets".to_string(),
                    });
                    
                    relationship.interaction_count += 1;
                    relationship.last_interaction = tweet.created_at.clone();
                }

                // Extract relationships from replies
                if let Some(reply_to_user) = &tweet.in_reply_to_screen_name {
                    let relationship = self.relationships.entry(reply_to_user.clone()).or_insert(SimpleRelationship {
                        username: reply_to_user.clone(),
                        interaction_count: 0,
                        last_interaction: tweet.created_at.clone(),
                        interaction_type: "tweets".to_string(),
                    });
                    
                    relationship.interaction_count += 1;
                    relationship.last_interaction = tweet.created_at.clone();
                }
            }
        }
        Ok(())
    }

    /// Analyze DMs for relationships
    pub fn analyze_dms(&mut self, dm_data: &[DmWrapper]) -> Result<()> {
        for dm_wrapper in dm_data {
            let conversation = &dm_wrapper.dm_conversation;
            
            // Extract participants from conversation ID
            let participants: Vec<&str> = conversation.conversation_id.split('-').collect();
            
            for message in &conversation.messages {
                if let Some(message_create) = &message.message_create {
                    // Extract timestamp for activity analysis
                    if let Ok(dt) = DateTime::parse_from_rfc3339(message_create.created_at.as_ref().unwrap_or(&"".to_string())) {
                        let hour = dt.hour();
                        let day = dt.format("%A").to_string();
                        
                        *self.hourly_activity.entry(hour).or_insert(0) += 1;
                        *self.daily_activity.entry(day).or_insert(0) += 1;
                    }

                    // Track DM relationships
                    if let (Some(sender_id), Some(recipient_id)) = (&message_create.sender_id, &message_create.recipient_id) {
                        // Use a simplified username (just the ID for now)
                        let other_user = if sender_id != recipient_id {
                            format!("user_{}", if participants.len() > 1 { 
                                if participants[0] == sender_id { participants[1] } else { participants[0] }
                            } else { 
                                recipient_id 
                            })
                        } else {
                            continue; // Skip self-messages
                        };

                        let relationship = self.relationships.entry(other_user.clone()).or_insert(SimpleRelationship {
                            username: other_user.clone(),
                            interaction_count: 0,
                            last_interaction: message_create.created_at.as_ref().unwrap_or(&"".to_string()).clone(),
                            interaction_type: "dms".to_string(),
                        });
                        
                        relationship.interaction_count += 1;
                        relationship.last_interaction = message_create.created_at.as_ref().unwrap_or(&"".to_string()).clone();
                        
                        // Update interaction type if we have both tweets and DMs
                        if relationship.interaction_type == "tweets" {
                            relationship.interaction_type = "both".to_string();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Get top relationships by interaction count
    pub fn get_top_relationships(&self, limit: usize) -> Vec<SimpleRelationship> {
        let mut relationships: Vec<SimpleRelationship> = self.relationships.values().cloned().collect();
        relationships.sort_by_key(|relationship| std::cmp::Reverse(relationship.interaction_count));
        relationships.into_iter().take(limit).collect()
    }

    /// Get peak activity hours
    pub fn get_peak_activity_hours(&self, limit: usize) -> Vec<(u32, u32)> {
        let mut hours: Vec<(u32, u32)> = self.hourly_activity.iter().map(|(&h, &c)| (h, c)).collect();
        hours.sort_by_key(|hour| std::cmp::Reverse(hour.1));
        hours.into_iter().take(limit).collect()
    }

    /// Get most active days
    pub fn get_most_active_days(&self) -> Vec<(String, u32)> {
        let mut days: Vec<(String, u32)> = self.daily_activity.iter().map(|(d, &c)| (d.clone(), c)).collect();
        days.sort_by_key(|day| std::cmp::Reverse(day.1));
        days
    }

    /// Generate a clean, readable report
    pub async fn generate_report(&self, output_dir: &Path, screen_name: &str, timestamp: i64) -> Result<()> {
        let mut report = String::new();
        
        report.push_str("🎯 TWITTER RELATIONSHIP & ACTIVITY INTELLIGENCE REPORT\n");
        report.push_str("=====================================================\n\n");

        // Top relationships section
        report.push_str("👥 TOP PEOPLE YOU INTERACT WITH\n");
        report.push_str("--------------------------------\n");
        let top_relationships = self.get_top_relationships(10);
        
        if top_relationships.is_empty() {
            report.push_str("No significant relationships found in the data.\n\n");
        } else {
            for (i, relationship) in top_relationships.iter().enumerate() {
                report.push_str(&format!(
                    "{}. @{} - {} interactions ({})\n",
                    i + 1,
                    relationship.username,
                    relationship.interaction_count,
                    relationship.interaction_type
                ));
            }
            report.push('\n');
        }

        // Activity patterns section
        report.push_str("⏰ WHEN YOU'RE MOST ACTIVE\n");
        report.push_str("---------------------------\n");
        
        let peak_hours = self.get_peak_activity_hours(5);
        if !peak_hours.is_empty() {
            report.push_str("Peak Activity Hours:\n");
            for (hour, count) in peak_hours {
                let time_str = if hour == 0 {
                    "12:00 AM".to_string()
                } else if hour < 12 {
                    format!("{}:00 AM", hour)
                } else if hour == 12 {
                    "12:00 PM".to_string()
                } else {
                    format!("{}:00 PM", hour - 12)
                };
                report.push_str(&format!("  {} - {} activities\n", time_str, count));
            }
            report.push('\n');
        }

        let active_days = self.get_most_active_days();
        if !active_days.is_empty() {
            report.push_str("Most Active Days:\n");
            for (day, count) in active_days {
                report.push_str(&format!("  {} - {} activities\n", day, count));
            }
            report.push('\n');
        }

        // Summary statistics
        report.push_str("📊 SUMMARY STATISTICS\n");
        report.push_str("---------------------\n");
        report.push_str(&format!("Total unique relationships: {}\n", self.relationships.len()));
        report.push_str(&format!("Total activities tracked: {}\n", 
            self.hourly_activity.values().sum::<u32>()));
        
        let most_active_hour = self.hourly_activity.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&hour, &count)| (hour, count));
        
        if let Some((hour, count)) = most_active_hour {
            let time_str = if hour == 0 {
                "12:00 AM".to_string()
            } else if hour < 12 {
                format!("{}:00 AM", hour)
            } else if hour == 12 {
                "12:00 PM".to_string()
            } else {
                format!("{}:00 PM", hour - 12)
            };
            report.push_str(&format!("Peak activity time: {} ({} activities)\n", time_str, count));
        }

        report.push('\n');
        report.push_str("💡 INSIGHTS & RECOMMENDATIONS\n");
        report.push_str("------------------------------\n");
        
        if !top_relationships.is_empty() {
            let top_person = &top_relationships[0];
            report.push_str(&format!("• Your strongest connection is @{} with {} interactions\n", 
                top_person.username, top_person.interaction_count));
        }
        
        if let Some((hour, _)) = most_active_hour {
            let time_str = if hour == 0 {
                "midnight".to_string()
            } else if hour < 12 {
                format!("{}:00 AM", hour)
            } else if hour == 12 {
                "noon".to_string()
            } else {
                format!("{}:00 PM", hour - 12)
            };
            report.push_str(&format!("• You're most active around {}\n", time_str));
        }

        if self.relationships.len() > 5 {
            report.push_str("• You have a diverse network of connections\n");
        } else if !self.relationships.is_empty() {
            report.push_str("• You tend to interact with a focused group of people\n");
        }

        report.push('\n');
        report.push_str("Generated by Tweet-Scrolls Relationship Intelligence System\n");
        report.push_str(&format!("Report generated at: {}\n", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Write the report
        let report_path = output_dir.join(format!("relationship_intelligence_{}_{}.txt", screen_name, timestamp));
        async_fs::write(&report_path, report).await?;
        
        println!("📊 Relationship intelligence report saved to: {}", report_path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn create_test_tweet(id: &str, text: &str, mentions: Vec<&str>, created_at: &str) -> Tweet {
        Tweet {
            id_str: id.to_string(),
            id: id.to_string(),
            full_text: text.to_string(),
            created_at: created_at.to_string(),
            favorite_count: "0".to_string(),
            retweet_count: "0".to_string(),
            retweeted: false,
            favorited: false,
            truncated: false,
            lang: "en".to_string(),
            source: "Twitter Web App".to_string(),
            display_text_range: vec!["0".to_string(), text.len().to_string()],
            in_reply_to_status_id: None,
            in_reply_to_status_id_str: None,
            in_reply_to_user_id: None,
            in_reply_to_user_id_str: None,
            in_reply_to_screen_name: None,
            edit_info: Some(EditInfo {
                initial: Some(EditInitial {
                    edit_tweet_ids: vec![id.to_string()],
                    editable_until: "2025-01-01T00:00:00.000Z".to_string(),
                    edits_remaining: "5".to_string(),
                    is_edit_eligible: false,
                })
            }),
            entities: TweetEntities {
                hashtags: vec![],
                symbols: vec![],
                user_mentions: mentions.into_iter().map(|m| UserMention {
                    name: m.to_string(),
                    screen_name: m.to_string(),
                    indices: vec!["0".to_string(), "10".to_string()],
                    id_str: "123456789".to_string(),
                    id: "123456789".to_string(),
                }).collect(),
                urls: vec![],
            },
            possibly_sensitive: None,
        }
    }

    #[test]
    fn test_mvp_analyzer_creation() {
        let analyzer = MvpAnalyzer::new();
        assert_eq!(analyzer.relationships.len(), 0);
        assert_eq!(analyzer.hourly_activity.len(), 0);
        assert_eq!(analyzer.daily_activity.len(), 0);
    }

    #[test]
    fn test_tweet_analysis() {
        let mut analyzer = MvpAnalyzer::new();
        
        let tweet = create_test_tweet(
            "123",
            "Hello @testuser this is a test",
            vec!["testuser"],
            "Mon Jan 01 12:00:00 +0000 2024"
        );
        
        let thread = Thread {
            id: "123".to_string(),
            tweets: vec![tweet],
            tweet_count: 1,
            favorite_count: 0,
            retweet_count: 0,
        };
        
        let result = analyzer.analyze_tweets(&[thread]);
        assert!(result.is_ok());
        
        // Check that relationship was extracted
        assert_eq!(analyzer.relationships.len(), 1);
        assert!(analyzer.relationships.contains_key("testuser"));
        
        // Check that activity was tracked
        assert!(analyzer.hourly_activity.contains_key(&12)); // 12:00 PM
    }

    #[test]
    fn test_top_relationships() {
        let mut analyzer = MvpAnalyzer::new();
        
        // Add some test relationships
        analyzer.relationships.insert("user1".to_string(), SimpleRelationship {
            username: "user1".to_string(),
            interaction_count: 10,
            last_interaction: "2024-01-01".to_string(),
            interaction_type: "tweets".to_string(),
        });
        
        analyzer.relationships.insert("user2".to_string(), SimpleRelationship {
            username: "user2".to_string(),
            interaction_count: 5,
            last_interaction: "2024-01-01".to_string(),
            interaction_type: "dms".to_string(),
        });
        
        let top = analyzer.get_top_relationships(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].username, "user1");
        assert_eq!(top[0].interaction_count, 10);
        assert_eq!(top[1].username, "user2");
        assert_eq!(top[1].interaction_count, 5);
    }

    #[test]
    fn test_activity_patterns() {
        let mut analyzer = MvpAnalyzer::new();
        
        // Add some test activity
        analyzer.hourly_activity.insert(9, 10);  // 9 AM
        analyzer.hourly_activity.insert(14, 15); // 2 PM
        analyzer.hourly_activity.insert(20, 5);  // 8 PM
        
        let peak_hours = analyzer.get_peak_activity_hours(2);
        assert_eq!(peak_hours.len(), 2);
        assert_eq!(peak_hours[0], (14, 15)); // 2 PM should be first
        assert_eq!(peak_hours[1], (9, 10));  // 9 AM should be second
    }
}