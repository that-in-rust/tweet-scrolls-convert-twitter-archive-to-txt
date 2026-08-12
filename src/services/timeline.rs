//! Timeline generation and analysis service

use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};
use crate::models::interaction::{InteractionEvent, ConversationThread};

/// Builds a timeline from a set of interaction events
/// 
/// The timeline is sorted in reverse chronological order (newest first)
pub fn build_timeline(events: &mut [InteractionEvent]) -> Vec<&InteractionEvent> {
    // Sort events by timestamp in descending order (newest first)
    events.sort_by_key(|event| std::cmp::Reverse(event.timestamp));
    events.iter().collect()
}

/// Groups interaction events into conversation threads
/// 
/// # Arguments
/// * `events` - Sorted list of interaction events
/// * `time_window_seconds` - Maximum time between events to be considered part of the same conversation
pub fn group_into_conversations(
    events: Vec<InteractionEvent>,
    time_window_seconds: i64,
) -> Vec<ConversationThread> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut conversations = Vec::new();
    let time_window = Duration::seconds(time_window_seconds);
    
    // Sort events chronologically for grouping
    let mut sorted_events = events;
    sorted_events.sort_by_key(|e| e.timestamp);
    
    let mut current_conversation = ConversationThread::new("0");
    let mut last_timestamp = sorted_events[0].timestamp;
    
    for event in sorted_events {
        // If the time gap is too large, start a new conversation
        if (event.timestamp - last_timestamp) > time_window && !current_conversation.events.is_empty() {
            conversations.push(current_conversation);
            current_conversation = ConversationThread::new(conversations.len().to_string());
        }
        
        current_conversation.add_event(event);
        last_timestamp = current_conversation.last_activity;
    }
    
    // Add the last conversation if not empty
    if !current_conversation.events.is_empty() {
        conversations.push(current_conversation);
    }
    
    conversations
}

/// Calculates response times between consecutive messages in a conversation
pub fn calculate_response_times(conversation: &ConversationThread) -> Vec<Duration> {
    let mut response_times = Vec::new();
    let events = &conversation.events;
    
    for window in events.windows(2) {
        let time_diff = window[1].timestamp - window[0].timestamp;
        response_times.push(time_diff);
    }
    
    response_times
}

/// Analyzes conversation patterns in the timeline
pub fn analyze_conversation_patterns(
    conversations: &[ConversationThread],
    _user_hash: &str,  // Currently unused, but kept for future filtering
) -> HashMap<String, f64> {
    let mut patterns = HashMap::new();
    
    // Calculate average response time
    let total_response_time: i64 = conversations
        .iter()
        .flat_map(|conv| calculate_response_times(conv).into_iter())
        .map(|d| d.num_seconds())
        .sum();
    
    let total_responses: i64 = conversations
        .iter()
        .map(|conv| conv.events.len() as i64 - 1)
        .sum();
    
    let avg_response = if total_responses > 0 {
        total_response_time as f64 / total_responses as f64
    } else {
        0.0
    };
    
    patterns.insert("average_response_time_seconds".to_string(), avg_response);
    
    // Count interaction types
    let mut type_counts = HashMap::new();
    for conv in conversations {
        for event in &conv.events {
            let type_str = event.interaction_type.to_string();
            *type_counts.entry(type_str).or_insert(0) += 1;
        }
    }
    
    // Add interaction type percentages
    let total_events: i32 = type_counts.values().sum();
    if total_events > 0 {
        for (t, &count) in &type_counts {
            let percentage = (count as f64 / total_events as f64) * 100.0;
            patterns.insert(format!("percentage_{}", t.to_lowercase().replace(' ', "_")), percentage);
        }
    }
    
    patterns
}

/// Analyzes conversation density over time
pub fn analyze_temporal_patterns(
    conversations: &[ConversationThread],
    time_interval: Duration,
) -> Vec<(DateTime<Utc>, usize)> {
    if conversations.is_empty() {
        return Vec::new();
    }
    
    // Find time range
    let start_time = conversations
        .iter()
        .map(|c| c.started_at)
        .min()
        .unwrap_or_else(Utc::now);
    
    let end_time = conversations
        .iter()
        .map(|c| c.last_activity)
        .max()
        .unwrap_or_else(Utc::now);
    
    // Initialize time buckets
    let mut current_time = start_time;
    let mut time_buckets = Vec::new();
    
    while current_time <= end_time {
        time_buckets.push((current_time, 0));
        current_time += time_interval;
    }
    
    // Count events in each time bucket
    for conv in conversations {
        for event in &conv.events {
            if let Some(bucket) = time_buckets
                .iter_mut()
                .find(|(time, _)| event.timestamp >= *time && event.timestamp < *time + time_interval)
            {
                bucket.1 += 1;
            }
        }
    }
    
    time_buckets
}
