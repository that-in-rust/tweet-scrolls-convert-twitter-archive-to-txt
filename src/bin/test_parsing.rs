#!/usr/bin/env cargo
//! Test parsing with realistic Twitter export data

use anyhow::Result;
use serde_json::from_str;
use std::fs;
use tweet_scrolls::processing::data_structures::TweetWrapper;

fn main() -> Result<()> {
    println!("🧪 Testing Twitter Export Parsing with Realistic Data");
    
    // Test 1: Parse realistic tweets
    println!("\n📋 Test 1: Parsing realistic tweets.js");
    let tweets_content = fs::read_to_string("private_data/sample_tweets_realistic.js")?;
    
    // Remove JavaScript prefix
    let json_start = tweets_content.find('[').expect("Missing opening bracket");
    let json_end = tweets_content.rfind(']').expect("Missing closing bracket");
    let json_content = &tweets_content[json_start..=json_end];
    
    match from_str::<Vec<TweetWrapper>>(json_content) {
        Ok(tweets) => {
            println!("✅ Successfully parsed {} tweets", tweets.len());
            
            // Test specific fields
            for (i, tweet_wrapper) in tweets.iter().enumerate() {
                let tweet = &tweet_wrapper.tweet;
                println!("Tweet {}: ID={}, Text={:.50}...", 
                    i + 1, 
                    tweet.id_str, 
                    tweet.full_text
                );
                
                // Test reply fields
                if let Some(reply_to) = &tweet.in_reply_to_status_id_str {
                    println!("  └─ Reply to: {}", reply_to);
                }
                
                // Test entities
                if let Some(entities) = &tweet.entities {
                    if !entities.user_mentions.is_empty() {
                        println!("  └─ Mentions: {}", 
                            entities.user_mentions.iter()
                                .map(|m| format!("@{}", m.screen_name))
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                    
                    if !entities.urls.is_empty() {
                        println!("  └─ URLs: {}", entities.urls.len());
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to parse tweets: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\n🎉 All parsing tests completed successfully!");
    Ok(())
}
