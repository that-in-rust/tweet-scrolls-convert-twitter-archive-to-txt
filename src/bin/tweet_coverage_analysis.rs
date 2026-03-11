use tweet_scrolls::processing::data_structures::{TweetWrapper, Tweet, Thread};
use chrono::DateTime;
use anyhow::{Result, Context};
use std::collections::{HashMap, HashSet};
use tokio::fs;
use std::env;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Tweet Coverage & Threading Analysis");
    println!("=====================================");

    // ...existing code...

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: Please provide the folder path as a command-line argument.");
        std::process::exit(1);
    }

    let folder_path = &args[1];
    let tweets_file = format!("{}/tweets.js", folder_path);

    // Step 1: Load and parse all tweets
    println!("\n📂 Step 1: Loading tweets from file...");
    let content = fs::read_to_string(&tweets_file).await
        .context("Failed to read tweets file")?;
    
    let json_content = extract_json_content(&content)?;
    let all_tweets: Vec<TweetWrapper> = serde_json::from_str(&json_content)
        .context("Failed to parse tweets JSON")?;
    
    println!("✅ Loaded {} total tweets from file", all_tweets.len());
    
    // Step 2: Analyze tweet types and structure
    println!("\n📊 Step 2: Analyzing tweet structure...");
    analyze_tweet_structure(&all_tweets);
    
    // Step 3: Process tweets through our threading system
    println!("\n🧵 Step 3: Processing tweets through threading system...");
    let mut original_tweet_count = 0;
    let mut reply_count = 0;
    let mut retweet_count = 0;
    let mut total_tweets = 0;

    let threads = create_threads_from_tweets(&all_tweets, "amuldotexe")?.into_iter().map(|thread| {
        for tweet in &thread.tweets {
            total_tweets += 1;
            if tweet.retweeted {
                retweet_count += 1;
            } else if tweet.in_reply_to_status_id_str.is_some() {
                reply_count += 1;
            } else {
                original_tweet_count += 1;
            }
            if total_tweets % 1000 == 0 {
                println!("[PROGRESS] Processed {} tweets: {} originals, {} replies, {} retweets", total_tweets, original_tweet_count, reply_count, retweet_count);
            }
        }
        thread
    }).collect::<Vec<_>>();

    println!("\n✅ Completed processing {} threads.", threads.len());
    println!("Final stats: Originals: {}, Replies: {}, Retweets: {}", original_tweet_count, reply_count, retweet_count);
    
    // Step 4: Coverage analysis
    println!("\n🔍 Step 4: Coverage Analysis...");
    let mut covered = 0;
    let mut missed = 0;
    for tw in &all_tweets {
        let mut found = false;
        for thread in &threads {
            if thread.tweets.iter().any(|t| t.id_str == tw.tweet.id_str) {
                found = true;
                break;
            }
        }
        if found {
            covered += 1;
        } else {
            missed += 1;
        }
    }
    println!("Coverage summary: Covered: {}, Missed: {}", covered, missed);

    // Step 5: Threading quality analysis
    println!("\n🎯 Step 5: Threading Quality Analysis...");
    let mut total_thread_len = 0;
    let mut max_thread_len = 0;
    for thread in &threads {
        let len = thread.tweets.len();
        total_thread_len += len;
        if len > max_thread_len {
            max_thread_len = len;
        }
    }
    let avg_thread_len = if threads.len() > 0 { total_thread_len as f64 / threads.len() as f64 } else { 0.0 };
    println!("Threading quality summary: Total threads: {}, Avg thread length: {:.2}, Max thread length: {}", threads.len(), avg_thread_len, max_thread_len);
    
    // Step 6: Missing tweets investigation
    println!("\n🕵️ Step 6: Missing Tweets Investigation...");
    investigate_missing_tweets(&all_tweets, &threads);
    
    Ok(())
}

fn extract_json_content(content: &str) -> Result<String> {
    let start_marker = "window.YTD.tweets.part0 = ";
    let start_pos = content.find(start_marker)
        .context("Could not find tweets start marker")?;
    let json_start = start_pos + start_marker.len();
    
    let end_pos = content.rfind(']')
        .context("Could not find tweets end marker")?;
    let json_end = end_pos + 1;
    
    Ok(content[json_start..json_end].to_string())
}

fn create_threads_from_tweets(all_tweets: &[TweetWrapper], screen_name: &str) -> Result<Vec<Thread>> {
    // Extract tweets and filter out retweets and RT @ tweets
    let tweets: Vec<Tweet> = all_tweets.iter()
        .map(|tw| tw.tweet.clone())
        .filter(|tweet| !tweet.retweeted && !tweet.full_text.starts_with("RT @"))
        .collect();
    
    // NEW: Include ALL tweets (original + replies to user + replies to others)
    // No filtering - we'll classify them instead
    
    // Create a map for quick lookup
    let tweets_map: HashMap<String, Tweet> = tweets.iter()
        .map(|tweet| (tweet.id_str.clone(), tweet.clone()))
        .collect();
    
    // Build threads
    let mut threads: Vec<Vec<Tweet>> = Vec::new();
    let mut processed_tweets = HashSet::new();
    
    for tweet in tweets_map.values() {
        // Skip if already processed
        if processed_tweets.contains(&tweet.id_str) {
            continue;
        }
        
        // Start a new thread if this is a root tweet (not a reply to our user)
        if tweet.in_reply_to_status_id.is_none() || 
           tweet.in_reply_to_screen_name.as_deref() != Some(screen_name) {
            
            let mut thread = vec![tweet.clone()];
            processed_tweets.insert(tweet.id_str.clone());
            let mut current_id = tweet.id_str.clone();
            
            // Follow the reply chain
            while let Some(reply) = tweets_map.values()
                .find(|t| t.in_reply_to_status_id_str.as_deref() == Some(&current_id)) {
                
                if processed_tweets.contains(&reply.id_str) {
                    break; // Avoid infinite loops
                }
                
                thread.push(reply.clone());
                processed_tweets.insert(reply.id_str.clone());
                current_id = reply.id_str.clone();
            }
            
            threads.push(thread);
        }
    }
    
    // Sort threads by creation date (newest first)
    threads.sort_by(|a, b| {
        let date_a = DateTime::parse_from_str(&a[0].created_at, "%a %b %d %H:%M:%S %z %Y")
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap());
        let date_b = DateTime::parse_from_str(&b[0].created_at, "%a %b %d %H:%M:%S %z %Y")
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap());
        date_b.cmp(&date_a)
    });
    
    // Convert to Thread structs
    let result_threads: Vec<Thread> = threads.into_iter().map(|thread| {
        let tweet_count = thread.len();
        let favorite_count: u32 = thread.iter()
            .map(|t| t.favorite_count.parse().unwrap_or(0))
            .sum();
        let retweet_count: u32 = thread.iter()
            .map(|t| t.retweet_count.parse().unwrap_or(0))
            .sum();
        
        Thread {
            id: thread[0].id_str.clone(),
            tweets: thread,
            tweet_count,
            favorite_count,
            retweet_count,
        }
    }).collect();
    
    Ok(result_threads)
}

fn analyze_tweet_structure(tweets: &[TweetWrapper]) {
    let mut original_tweets = 0;
    let mut replies = 0;
    let mut retweets = 0;
    let mut has_edit_info = 0;
    let mut has_entities = 0;
    
    for tweet_wrapper in tweets {
        let tweet = &tweet_wrapper.tweet;
        
        if tweet.retweeted {
            retweets += 1;
        } else if tweet.in_reply_to_status_id_str.is_some() {
            replies += 1;
        } else {
            original_tweets += 1;
        }
        
        if tweet.edit_info.is_some() {
            has_edit_info += 1;
        }
        
        if tweet.entities.as_ref().is_some_and(|entities| {
            !entities.hashtags.is_empty() ||
            !entities.user_mentions.is_empty() ||
            !entities.urls.is_empty()
        }) {
            has_entities += 1;
        }
    }
    
    println!("📈 Tweet Structure Analysis:");
    println!("   📝 Original tweets: {} ({:.1}%)", original_tweets, 
             (original_tweets as f64 / tweets.len() as f64) * 100.0);
    println!("   💬 Replies: {} ({:.1}%)", replies, 
             (replies as f64 / tweets.len() as f64) * 100.0);
    println!("   🔄 Retweets: {} ({:.1}%)", retweets, 
             (retweets as f64 / tweets.len() as f64) * 100.0);
    println!("   ✏️  With edit info: {} ({:.1}%)", has_edit_info,
             (has_edit_info as f64 / tweets.len() as f64) * 100.0);
    println!("   🏷️  With entities: {} ({:.1}%)", has_entities,
             (has_entities as f64 / tweets.len() as f64) * 100.0);
}

fn analyze_coverage(all_tweets: &[TweetWrapper], threads: &[tweet_scrolls::processing::data_structures::Thread]) {
    // Count tweets in threads
    let mut tweets_in_threads = 0;
    let mut thread_tweet_ids = HashSet::new();
    
    for thread in threads {
        tweets_in_threads += thread.tweets.len();
        for tweet in &thread.tweets {
            thread_tweet_ids.insert(&tweet.id_str);
        }
    }
    
    // Count original tweets (non-retweets)
    let non_retweet_count = all_tweets.iter()
        .filter(|tw| !tw.tweet.retweeted)
        .count();
    
    let coverage_percentage = (tweets_in_threads as f64 / non_retweet_count as f64) * 100.0;
    
    println!("📊 Coverage Analysis:");
    println!("   📁 Total tweets in file: {}", all_tweets.len());
    println!("   🚫 Retweets (excluded): {}", all_tweets.len() - non_retweet_count);
    println!("   ✅ Non-retweet tweets: {}", non_retweet_count);
    println!("   🧵 Tweets in threads: {}", tweets_in_threads);
    println!("   📈 Coverage: {:.2}% ({}/{})", 
             coverage_percentage, tweets_in_threads, non_retweet_count);
    
    if coverage_percentage < 100.0 {
        let missing = non_retweet_count - tweets_in_threads;
        println!("   ⚠️  Missing tweets: {} ({:.2}%)", missing,
                 (missing as f64 / non_retweet_count as f64) * 100.0);
    } else {
        println!("   🎉 Perfect coverage! All non-retweet tweets are captured.");
    }
}

fn analyze_threading_quality(all_tweets: &[TweetWrapper], threads: &[tweet_scrolls::processing::data_structures::Thread]) {
    let mut single_tweet_threads = 0;
    let mut multi_tweet_threads = 0;
    let mut longest_thread = 0;
    let mut total_thread_length = 0;
    let mut reply_chains_found = 0;
    
    // Build a map of tweet ID to tweet for quick lookup
    let _tweet_map: HashMap<String, &Tweet> = all_tweets.iter()
        .map(|tw| (tw.tweet.id_str.clone(), &tw.tweet))
        .collect();
    
    for thread in threads {
        total_thread_length += thread.tweets.len();
        
        if thread.tweets.len() == 1 {
            single_tweet_threads += 1;
        } else {
            multi_tweet_threads += 1;
            
            // Check if this is a proper reply chain
            let mut is_reply_chain = true;
            for i in 1..thread.tweets.len() {
                let current_tweet = &thread.tweets[i];
                let prev_tweet = &thread.tweets[i-1];
                
                // Check if current tweet replies to previous tweet
                if let Some(reply_to_id) = &current_tweet.in_reply_to_status_id_str {
                    if reply_to_id != &prev_tweet.id_str {
                        is_reply_chain = false;
                        break;
                    }
                } else {
                    is_reply_chain = false;
                    break;
                }
            }
            
            if is_reply_chain {
                reply_chains_found += 1;
            }
        }
        
        if thread.tweets.len() > longest_thread {
            longest_thread = thread.tweets.len();
        }
    }
    
    let avg_thread_length = total_thread_length as f64 / threads.len() as f64;
    
    println!("🎯 Threading Quality Analysis:");
    println!("   📊 Total threads created: {}", threads.len());
    println!("   📝 Single-tweet threads: {} ({:.1}%)", single_tweet_threads,
             (single_tweet_threads as f64 / threads.len() as f64) * 100.0);
    println!("   🧵 Multi-tweet threads: {} ({:.1}%)", multi_tweet_threads,
             (multi_tweet_threads as f64 / threads.len() as f64) * 100.0);
    println!("   📏 Average thread length: {:.2} tweets", avg_thread_length);
    println!("   🏆 Longest thread: {} tweets", longest_thread);
    println!("   🔗 Proper reply chains: {} ({:.1}% of multi-tweet threads)", 
             reply_chains_found,
             if multi_tweet_threads > 0 { 
                 (reply_chains_found as f64 / multi_tweet_threads as f64) * 100.0 
             } else { 0.0 });
}

fn investigate_missing_tweets(all_tweets: &[TweetWrapper], threads: &[tweet_scrolls::processing::data_structures::Thread]) {
    // Collect all tweet IDs that are in threads
    let mut threaded_tweet_ids = HashSet::new();
    for thread in threads {
        for tweet in &thread.tweets {
            threaded_tweet_ids.insert(&tweet.id_str);
        }
    }
    
    // Find missing tweets (non-retweets not in threads)
    let mut missing_tweets = Vec::new();
    for tweet_wrapper in all_tweets {
        let tweet = &tweet_wrapper.tweet;
        if !tweet.retweeted && !threaded_tweet_ids.contains(&tweet.id_str) {
            missing_tweets.push(tweet);
        }
    }
    
    if missing_tweets.is_empty() {
        println!("🎉 No missing tweets found! All non-retweets are properly threaded.");
        return;
    }
    
    println!("🕵️ Missing Tweets Investigation:");
    println!("   📊 Found {} missing tweets", missing_tweets.len());
    
    // Analyze why tweets might be missing
    let mut missing_replies = 0;
    let mut missing_originals = 0;
    let mut missing_with_mentions = 0;
    let mut missing_with_urls = 0;
    
    println!("\n   🔍 Sample of missing tweets:");
    for (i, tweet) in missing_tweets.iter().take(5).enumerate() {
        println!("   {}. ID: {} | Created: {}", i + 1, tweet.id_str, tweet.created_at);
        println!("      Text: {}", &tweet.full_text[..tweet.full_text.len().min(100)]);
        
        if tweet.in_reply_to_status_id_str.is_some() {
            missing_replies += 1;
            println!("      Type: Reply to {}", 
                     tweet.in_reply_to_status_id_str.as_ref().unwrap());
        } else {
            missing_originals += 1;
            println!("      Type: Original tweet");
        }
        
        if let Some(entities) = &tweet.entities {
            if !entities.user_mentions.is_empty() {
                missing_with_mentions += 1;
                println!("      Mentions: {}", entities.user_mentions.len());
            }
            
            if !entities.urls.is_empty() {
                missing_with_urls += 1;
                println!("      URLs: {}", entities.urls.len());
            }
        }
        
        println!();
    }
    
    println!("   📈 Missing Tweet Analysis:");
    println!("      💬 Missing replies: {} ({:.1}%)", missing_replies,
             (missing_replies as f64 / missing_tweets.len() as f64) * 100.0);
    println!("      📝 Missing originals: {} ({:.1}%)", missing_originals,
             (missing_originals as f64 / missing_tweets.len() as f64) * 100.0);
    println!("      👥 With mentions: {} ({:.1}%)", missing_with_mentions,
             (missing_with_mentions as f64 / missing_tweets.len() as f64) * 100.0);
    println!("      🔗 With URLs: {} ({:.1}%)", missing_with_urls,
             (missing_with_urls as f64 / missing_tweets.len() as f64) * 100.0);
}
