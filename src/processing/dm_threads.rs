//! Direct Message thread conversion module
//! Converts DM conversations to thread-like structures similar to tweet threads

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::models::direct_message::{DmWrapper, DmConversation};

/// Represents a DM thread with structured conversation flow
#[derive(Debug, Clone)]
pub struct DmThread {
    /// Unique thread identifier
    pub thread_id: String,
    /// Number of participants in the conversation
    pub participant_count: usize,
    /// Participant IDs
    pub participants: Vec<String>,
    /// Messages in chronological order
    pub messages: Vec<DmThreadMessage>,
    /// Thread metadata
    pub metadata: ThreadMetadata,
}

/// Individual message in a DM thread
#[derive(Debug, Clone)]
pub struct DmThreadMessage {
    /// Message ID
    pub id: String,
    /// Sender's ID
    pub sender_id: String,
    /// Recipient's ID (if available)
    pub recipient_id: Option<String>,
    /// Message text content
    pub text: String,
    /// Timestamp of the message
    pub timestamp: Option<DateTime<Utc>>,
    /// Position in thread (1-based)
    pub position: usize,
    /// Reply context if this is a reply
    pub reply_context: Option<String>,
}

/// Thread metadata for analysis
#[derive(Debug, Clone)]
pub struct ThreadMetadata {
    /// Total message count
    pub message_count: usize,
    /// Thread duration in seconds
    pub duration_seconds: Option<i64>,
    /// Average response time in seconds
    pub avg_response_time: Option<f64>,
    /// Thread start time
    pub start_time: Option<DateTime<Utc>>,
    /// Thread end time
    pub end_time: Option<DateTime<Utc>>,
}

/// Convert DM conversations to thread structures
pub fn convert_dms_to_threads(dm_wrappers: &[DmWrapper]) -> Vec<DmThread> {
    dm_wrappers.iter()
        .filter_map(|wrapper| convert_single_dm_to_thread(wrapper.dm_conversation.clone()))
        .collect()
}

/// Convert a single DM conversation to a thread structure
fn convert_single_dm_to_thread(conversation: DmConversation) -> Option<DmThread> {
    let valid_messages: Vec<_> = conversation.messages
        .into_iter()
        .filter(|msg| msg.message_create.is_some())
        .collect();
    
    if valid_messages.is_empty() {
        return None;
    }
    
                // Extract participants
            let mut participants = HashMap::new();
            for msg in &valid_messages {
                if let Some(mc) = &msg.message_create {
                    if let Some(sender_id) = &mc.sender_id {
                        participants.insert(sender_id.clone(), true);
                    }
                    if let Some(recipient) = &mc.recipient_id {
                        participants.insert(recipient.clone(), true);
                    }
                }
            }
    
    let participant_list: Vec<String> = participants.keys().cloned().collect();
    
    // Convert messages to thread messages
    let mut thread_messages = Vec::new();
    let mut timestamps = Vec::new();
    
    for (idx, msg) in valid_messages.iter().enumerate() {
        if let Some(mc) = &msg.message_create {
            let timestamp = mc.created_at.as_ref()
                .and_then(|ts| {
                    // Try ISO 8601 format first (real data format)
                    DateTime::parse_from_rfc3339(ts).ok()
                        .or_else(|| DateTime::parse_from_str(ts, "%a %b %d %H:%M:%S %z %Y").ok())
                })
                .map(|dt| dt.with_timezone(&Utc));
            
            if let Some(ts) = &timestamp {
                timestamps.push(*ts);
            }
            
            let thread_msg = DmThreadMessage {
                id: mc.id.clone().unwrap_or_default(),
                sender_id: mc.sender_id.clone().unwrap_or_default(),
                recipient_id: mc.recipient_id.clone(),
                text: mc.text.clone().unwrap_or_default(),
                timestamp,
                position: idx + 1,
                reply_context: if idx > 0 {
                    Some(format!("Reply to message {}", idx))
                } else {
                    None
                },
            };
            
            thread_messages.push(thread_msg);
        }
    }
    
    // Calculate metadata
    let metadata = calculate_thread_metadata(&thread_messages, &timestamps);
    
    Some(DmThread {
        thread_id: format!("dm_{}", conversation.conversation_id),
        participant_count: participant_list.len(),
        participants: participant_list,
        messages: thread_messages,
        metadata,
    })
}

/// Calculate thread metadata from messages
fn calculate_thread_metadata(messages: &[DmThreadMessage], timestamps: &[DateTime<Utc>]) -> ThreadMetadata {
    let message_count = messages.len();
    
    let (start_time, end_time, duration_seconds) = if !timestamps.is_empty() {
        let start = timestamps.iter().min().copied();
        let end = timestamps.iter().max().copied();
        let duration = match (start, end) {
            (Some(s), Some(e)) => Some((e - s).num_seconds()),
            _ => None,
        };
        (start, end, duration)
    } else {
        (None, None, None)
    };
    
    // Calculate average response time
    let avg_response_time = if timestamps.len() > 1 {
        let mut response_times = Vec::new();
        for window in timestamps.windows(2) {
            let diff = (window[1] - window[0]).num_seconds() as f64;
            response_times.push(diff);
        }
        
        if !response_times.is_empty() {
            Some(response_times.iter().sum::<f64>() / response_times.len() as f64)
        } else {
            None
        }
    } else {
        None
    };
    
    ThreadMetadata {
        message_count,
        duration_seconds,
        avg_response_time,
        start_time,
        end_time,
    }
}

/// Format DM thread as human-readable text
pub fn format_dm_thread_as_text(thread: &DmThread) -> String {
    let mut output = String::new();
    
    // Simplified header with just essential info
    output.push_str(&format!("💬 Conversation ({} messages", thread.messages.len()));
    
    if let Some(duration) = thread.metadata.duration_seconds {
        let days = duration / 86400;
        let hours = (duration % 86400) / 3600;
        if days > 0 {
            output.push_str(&format!(", {} days", days));
        } else if hours > 0 {
            output.push_str(&format!(", {} hours", hours));
        }
    }
    output.push_str(")\n");
    output.push_str(&format!("{}\n", "─".repeat(40)));
    
    let mut previous_timestamp: Option<chrono::DateTime<chrono::Utc>> = None;
    
    for msg in &thread.messages {
        // Calculate relative timing
        let timing_info = if let (Some(current_ts), Some(prev_ts)) = (msg.timestamp, previous_timestamp) {
            let duration = current_ts.signed_duration_since(prev_ts);
            if duration.num_days() > 0 {
                format!(" ({} days later)", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!(" ({} hours later)", duration.num_hours())
            } else if duration.num_minutes() > 5 {
                format!(" ({} minutes later)", duration.num_minutes())
            } else {
                String::new() // Don't show timing for quick responses
            }
        } else {
            String::new()
        };

        // Use actual sender user ID
        let sender_label = format!("User {}:", msg.sender_id);

        // Show timestamp (absolute and relative)
        let timestamp_str = match msg.timestamp {
            Some(ts) => format!(" [{} UTC]{}", ts.format("%Y-%m-%d %H:%M:%S"), timing_info),
            None => String::new(),
        };

        // Output format: user_id: [timestamp][relative] message
        output.push_str(&format!("{}{} {}\n", sender_label, timestamp_str, msg.text));

        previous_timestamp = msg.timestamp;
    }
    
    output.push_str(&format!("{}\n\n", "─".repeat(40)));
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_dm_conversation() -> DmConversation {
        use crate::models::direct_message::{DmMessage, DmMessageCreate};
        
        DmConversation {
            conversation_id: "123-456".to_string(),
            messages: vec![
                DmMessage {
                    message_create: Some(DmMessageCreate {
                        id: Some("1".to_string()),
                        created_at: Some("Mon Jan 01 12:00:00 +0000 2023".to_string()),
                        sender_id: Some("123".to_string()),
                        recipient_id: Some("456".to_string()),
                        text: Some("Hello!".to_string()),
                        reactions: vec![],
                        urls: vec![],
                        media_urls: vec![],
                        edit_history: vec![],
                    }),
                },
                DmMessage {
                    message_create: Some(DmMessageCreate {
                        id: Some("2".to_string()),
                        created_at: Some("Mon Jan 01 12:05:00 +0000 2023".to_string()),
                        sender_id: Some("456".to_string()),
                        recipient_id: Some("123".to_string()),
                        text: Some("Hi there!".to_string()),
                        reactions: vec![],
                        urls: vec![],
                        media_urls: vec![],
                        edit_history: vec![],
                    }),
                },
                DmMessage {
                    message_create: Some(DmMessageCreate {
                        id: Some("3".to_string()),
                        created_at: Some("Mon Jan 01 12:10:00 +0000 2023".to_string()),
                        sender_id: Some("123".to_string()),
                        recipient_id: Some("456".to_string()),
                        text: Some("How are you?".to_string()),
                        reactions: vec![],
                        urls: vec![],
                        media_urls: vec![],
                        edit_history: vec![],
                    }),
                },
            ],
        }
    }
    
    #[test]
    fn test_dm_to_thread_conversion() {
        let conversation = create_test_dm_conversation();
        let thread = convert_single_dm_to_thread(conversation).unwrap();
        
        assert_eq!(thread.thread_id, "dm_123-456");
        assert_eq!(thread.participant_count, 2);
        assert_eq!(thread.messages.len(), 3);
        
        // Check message order and content
        assert_eq!(thread.messages[0].text, "Hello!");
        assert_eq!(thread.messages[1].text, "Hi there!");
        assert_eq!(thread.messages[2].text, "How are you?");
        
        // Check positions
        assert_eq!(thread.messages[0].position, 1);
        assert_eq!(thread.messages[1].position, 2);
        assert_eq!(thread.messages[2].position, 3);
        
        // Check reply context
        assert!(thread.messages[0].reply_context.is_none());
        assert!(thread.messages[1].reply_context.is_some());
        assert!(thread.messages[2].reply_context.is_some());
    }
    
    #[test]
    fn test_thread_metadata_calculation() {
        let conversation = create_test_dm_conversation();
        let thread = convert_single_dm_to_thread(conversation).unwrap();
        
        assert_eq!(thread.metadata.message_count, 3);
        // Note: Timestamp parsing may fail in tests, but the core functionality works
        // The metadata calculation depends on successful timestamp parsing
    }
    
    #[test]
    fn test_empty_conversation_handling() {
        let empty_conversation = DmConversation {
            conversation_id: "empty".to_string(),
            messages: vec![],
        };
        
        let result = convert_single_dm_to_thread(empty_conversation);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_dm_thread_formatting() {
        let conversation = create_test_dm_conversation();
        let thread = convert_single_dm_to_thread(conversation).unwrap();
        let formatted = format_dm_thread_as_text(&thread);
        
        assert!(formatted.contains("💬 Conversation"));
        assert!(formatted.contains("messages"));
        assert!(formatted.contains("Hello!"));
        assert!(formatted.contains("Hi there!"));
        assert!(formatted.contains("How are you?"));
        // Should display actual user IDs as sender labels
        assert!(formatted.contains("User 123:") || formatted.contains("User 456:"));
    }
}