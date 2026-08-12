//! Direct message processing pipeline

use anyhow::{Context, Result};
use csv::Writer as CsvWriterLib;
use serde_json::from_str;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;
use tokio::fs as async_fs;

use crate::models::direct_message::DmWrapper;
use crate::relationship::RelationshipAnalyzer;
use super::data_structures::ProcessedConversation;
use super::dm_threads::{convert_dms_to_threads, format_dm_thread_as_text};

/// Processes direct messages from a JSON file and generates analysis
/// 
/// # Arguments
/// 
/// * `dm_file` - Path to the DM JSON file
/// * `screen_name` - Twitter handle for output file naming
/// * `output_dir` - Directory to write output files
/// * `timestamp` - Timestamp for file naming
/// 
/// # Returns
/// 
/// Result indicating success or failure of the processing
pub async fn process_dm_file(dm_file: &str, screen_name: &str, output_dir: &Path, timestamp: i64) -> Result<()> {
    let start_time = Instant::now();
    
    println!("📱 Reading DM file...");
    let dm_content = async_fs::read_to_string(dm_file).await
        .with_context(|| format!("Failed to read DM file: {}", dm_file))?;
    
    println!("🔍 Parsing DM data...");
    // Remove JavaScript assignment prefix if present (handle both formats)
    let json_content = if let Some(stripped) = dm_content.strip_prefix("window.YTD.direct_messages.part0 = ") {
        stripped
    } else if let Some(stripped) = dm_content.strip_prefix("window.YTD.direct_message_headers.part0 = ") {
        stripped
    } else {
        &dm_content
    };
    
    let dm_wrappers: Vec<DmWrapper> = from_str(json_content)
        .context("Failed to parse DM JSON")?;
        
    // Create relationship analyzer for timeline analysis
    let analyzer = RelationshipAnalyzer::new();
    
    // Build interaction timeline from DM data
    let timeline = analyzer.build_timeline(&dm_wrappers, &[]);
    
    // Perform timeline analysis
    let timeline_analysis = analyzer.analyze_timeline(&timeline);
    
    // Print timeline analysis summary
    println!("\n📊 Timeline Analysis Results:");
    println!("  • Total interactions: {}", timeline_analysis.total_interactions);
    println!("  • Unique participants: {}", timeline_analysis.unique_participants);
    println!("  • Analysis patterns: {} detected", timeline_analysis.patterns.len());
    println!("  • Average response time: {:.2} minutes", 
             timeline_analysis.response_times.average / 60.0);
    println!("  • Interactions per day: {:.2}", 
             timeline_analysis.density.avg_interactions_per_day);
    
    println!("💬 Processing {} conversations...", dm_wrappers.len());
    
    let mut conversations: Vec<ProcessedConversation> = dm_wrappers
        .iter()
        .map(|wrapper| {
            let conv = &wrapper.dm_conversation;
            let valid_messages: Vec<_> = conv.messages
                .iter()
                .filter(|msg| msg.message_create.is_some())
                .collect();
            
            let first_date = valid_messages.first()
                .and_then(|msg| msg.message_create.as_ref())
                .and_then(|mc| mc.created_at.clone());
            
            let last_date = valid_messages.last()
                .and_then(|msg| msg.message_create.as_ref())
                .and_then(|mc| mc.created_at.clone());
            
            ProcessedConversation {
                conversation_id: conv.conversation_id.clone(),
                message_count: valid_messages.len() as u32,
                participants: vec![], // Will be filled properly later
                first_message_date: first_date,
                last_message_date: last_date,
            }
        })
        .filter(|conv| conv.message_count > 0)
        .collect();
    
    // Sort by message count (descending)
    conversations.sort_by_key(|conversation| std::cmp::Reverse(conversation.message_count));
    
    println!("📊 Writing DM results...");
    
    // Write conversations CSV file
    write_dm_csv(&conversations, screen_name, timestamp, output_dir).await?;
    
    // Convert DMs to threads and write thread files
    write_dm_threads(&dm_wrappers, screen_name, timestamp, output_dir).await?;
    
    // Write timeline analysis to a separate CSV
    write_timeline_analysis_csv(&timeline_analysis, screen_name, timestamp, output_dir).await?;
    
    // Write timeline analysis to TXT file
    write_timeline_analysis_txt(&timeline_analysis, screen_name, timestamp, output_dir).await?;
    
    // Write summary file
    write_dm_summary(&conversations, &timeline_analysis, screen_name, timestamp, output_dir, start_time).await?;
    
    println!("✅ DM processing completed successfully!");
    Ok(())
}

/// Writes DM conversations to CSV file
async fn write_dm_csv(
    conversations: &[ProcessedConversation], 
    screen_name: &str, 
    timestamp: i64, 
    output_dir: &Path
) -> Result<()> {
    let csv_path = output_dir.join(format!("dm_conversations_{}_{}.csv", screen_name, timestamp));
    let csv_file = File::create(&csv_path)?;
    let mut csv_writer = CsvWriterLib::from_writer(BufWriter::new(csv_file));
    
    // Write conversations data
    csv_writer.write_record([
        "Conversation ID",
        "Message Count", 
        "First Message Date",
        "Last Message Date",
    ])?;
    
    for conv in conversations {
        csv_writer.write_record([
            &conv.conversation_id,
            &conv.message_count.to_string(),
            conv.first_message_date.as_deref().unwrap_or("N/A"),
            conv.last_message_date.as_deref().unwrap_or("N/A"),
        ])?;
    }
    csv_writer.flush()?;
    
    Ok(())
}

/// Writes DM threads to CSV and TXT files
async fn write_dm_threads(
    dm_wrappers: &[DmWrapper],
    screen_name: &str,
    timestamp: i64,
    output_dir: &Path
) -> Result<()> {
    // Convert DMs to threads
    let dm_threads = convert_dms_to_threads(dm_wrappers);
    
    if dm_threads.is_empty() {
        println!("⚠️  No DM threads to write");
        return Ok(());
    }
    
    // Write CSV file
    let csv_path = output_dir.join(format!("dm_threads_{}_{}.csv", screen_name, timestamp));
    let csv_file = File::create(&csv_path)?;
    let mut csv_writer = CsvWriterLib::from_writer(BufWriter::new(csv_file));
    
    // Write CSV headers
    csv_writer.write_record([
        "Thread ID",
        "Participant Count",
        "Message Count",
        "Duration (seconds)",
        "Avg Response Time (seconds)",
        "Start Time",
        "End Time",
        "Participants"
    ])?;
    
    // Write thread data
    for thread in &dm_threads {
        csv_writer.write_record([
            &thread.thread_id,
            &thread.participant_count.to_string(),
            &thread.metadata.message_count.to_string(),
            &thread.metadata.duration_seconds.map_or("N/A".to_string(), |d| d.to_string()),
            &thread.metadata.avg_response_time.map_or("N/A".to_string(), |t| format!("{:.2}", t)),
            &thread.metadata.start_time.map_or("N/A".to_string(), |t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            &thread.metadata.end_time.map_or("N/A".to_string(), |t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            &thread.participants.join(";")
        ])?;
    }
    csv_writer.flush()?;
    
    // Write TXT file
    let txt_path = output_dir.join(format!("dm_threads_{}_{}.txt", screen_name, timestamp));
    let mut txt_content = String::new();
    
    txt_content.push_str("📱 DIRECT MESSAGE THREADS\n");
    txt_content.push_str(&format!("{}\n\n", "=".repeat(50)));
    txt_content.push_str(&format!("Total threads: {}\n", dm_threads.len()));
    txt_content.push_str(&format!("Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")));
    txt_content.push_str(&format!("{}\n\n", "=".repeat(50)));
    
    for thread in &dm_threads {
        txt_content.push_str(&format_dm_thread_as_text(thread));
        txt_content.push('\n');
    }
    
    async_fs::write(&txt_path, txt_content).await
        .context("Failed to write DM threads TXT file")?;
    
    println!("📝 Generated {} DM thread files", dm_threads.len());
    Ok(())
}

/// Writes timeline analysis to CSV file
async fn write_timeline_analysis_csv(
    timeline_analysis: &crate::models::timeline::TimelineAnalysis,
    screen_name: &str, 
    timestamp: i64, 
    output_dir: &Path
) -> Result<()> {
    let timeline_csv_path = output_dir.join(format!("timeline_analysis_{}_{}.csv", screen_name, timestamp));
    let timeline_csv_file = File::create(&timeline_csv_path)?;
    let mut timeline_writer = CsvWriterLib::from_writer(BufWriter::new(timeline_csv_file));
    
    // Write timeline analysis header
    timeline_writer.write_record([
        "Analysis Type",
        "Total Interactions",
        "Unique Participants",
        "Patterns Detected",
        "Avg Response Time (min)",
        "Median Response Time (min)",
        "Interactions Per Day",
        "Peak Hour",
        "Peak Day"
    ])?;
    
    // Write timeline analysis data
    timeline_writer.write_record([
        "Summary",
        &timeline_analysis.total_interactions.to_string(),
        &timeline_analysis.unique_participants.to_string(),
        &timeline_analysis.patterns.len().to_string(),
        &format!("{:.2}", timeline_analysis.response_times.average / 60.0),
        &format!("{:.2}", timeline_analysis.response_times.median / 60.0),
        &format!("{:.2}", timeline_analysis.density.avg_interactions_per_day),
        &timeline_analysis.density.peak_hour.to_string(),
        &timeline_analysis.density.peak_day.to_string()
    ])?;
    
    timeline_writer.flush()?;
    Ok(())
}

/// Writes timeline analysis to TXT file
async fn write_timeline_analysis_txt(
    timeline_analysis: &crate::models::timeline::TimelineAnalysis,
    screen_name: &str, 
    timestamp: i64, 
    output_dir: &Path
) -> Result<()> {
    let timeline_txt_path = output_dir.join(format!("timeline_analysis_{}_{}.txt", screen_name, timestamp));
    let timeline_txt_file = File::create(&timeline_txt_path)?;
    let mut timeline_txt_writer = BufWriter::new(timeline_txt_file);
    
    use std::io::Write;
    
    // Format timeline analysis as a table
    writeln!(timeline_txt_writer, "{:=<80}", "")?;
    writeln!(timeline_txt_writer, "TIMELINE ANALYSIS SUMMARY")?;
    writeln!(timeline_txt_writer, "{:=<80}\n", "")?;
    
    // Basic Statistics
    writeln!(timeline_txt_writer, "{:-<40}", " Basic Statistics ")?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35} |", "Total Interactions", timeline_analysis.total_interactions)?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35} |", "Unique Participants", timeline_analysis.unique_participants)?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35} |", "Patterns Detected", timeline_analysis.patterns.len())?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35.2} min |", "Avg Response Time", timeline_analysis.response_times.average / 60.0)?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35.2} min |", "Median Response Time", timeline_analysis.response_times.median / 60.0)?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35.2} |", "Interactions Per Day", timeline_analysis.density.avg_interactions_per_day)?;
    
    // Peak Activity
    writeln!(timeline_txt_writer, "\n{:-<40}", " Peak Activity ")?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35} |", "Peak Hour", format!("{}:00", timeline_analysis.density.peak_hour))?;
    writeln!(timeline_txt_writer, "| {:<36} | {:>35} |", "Peak Day", timeline_analysis.density.peak_day)?;
    
    timeline_txt_writer.flush()?;
    Ok(())
}

/// Writes DM processing summary
async fn write_dm_summary(
    conversations: &[ProcessedConversation],
    timeline_analysis: &crate::models::timeline::TimelineAnalysis,
    screen_name: &str, 
    timestamp: i64, 
    output_dir: &Path,
    start_time: Instant
) -> Result<()> {
    let total_messages: u32 = conversations.iter().map(|c| c.message_count).sum();
    let duration = start_time.elapsed();
    
    let summary_content = format!(
        "DM Processing Summary\n\
         ====================\n\
         Screen Name: {}\n\
         Total Conversations: {}\n\
         Total Messages: {}\n\
         Total Interactions: {}\n\
         Unique Participants: {}\n\
         Processing Duration: {:.2} seconds\n\
         ====================\n\
         Timeline Analysis:\n\
         - Average Response Time: {:.2} minutes\n\
         - Interactions Per Day: {:.2}\n\
         - Peak Activity Hour: {}:00\n\
         - Patterns Detected: {}\n\
         ====================\n\
         Status: Processing Completed Successfully",
        screen_name,
        conversations.len(),
        total_messages,
        timeline_analysis.total_interactions,
        timeline_analysis.unique_participants,
        duration.as_secs_f64(),
        timeline_analysis.response_times.average / 60.0,
        timeline_analysis.density.avg_interactions_per_day,
        timeline_analysis.density.peak_hour,
        timeline_analysis.patterns.len()
    );

    let summary_path = output_dir.join(format!("dm_results_{}_{}.txt", screen_name, timestamp));
    async_fs::write(&summary_path, summary_content).await.context("Failed to write DM summary file")?;
    
    Ok(())
}

/// Simple DM processing function for testing
pub async fn process_dm_conversations(dm_data: &[DmWrapper], _screen_name: &str) -> Result<Vec<ProcessedConversation>> {
    let mut conversations = Vec::new();
    
    for dm_wrapper in dm_data {
        let conversation = &dm_wrapper.dm_conversation;
        
        // Skip empty conversations
        if conversation.messages.is_empty() {
            continue;
        }
        
        // Extract participants from conversation ID
        let participants: Vec<String> = conversation.conversation_id
            .split('-')
            .map(|s| s.to_string())
            .collect();
        
        let processed = ProcessedConversation {
            conversation_id: conversation.conversation_id.clone(),
            message_count: conversation.messages.len() as u32,
            participants,
            first_message_date: conversation.messages.first()
                .and_then(|m| m.message_create.as_ref())
                .and_then(|mc| mc.created_at.clone()),
            last_message_date: conversation.messages.last()
                .and_then(|m| m.message_create.as_ref())
                .and_then(|mc| mc.created_at.clone()),
        };
        
        conversations.push(processed);
    }
    
    // Sort by message count (descending)
    conversations.sort_by_key(|conversation| std::cmp::Reverse(conversation.message_count));
    
    Ok(conversations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_dm_processing_structure() {
        // Test that the function signature is correct
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();
        
        // This would fail with actual processing due to missing file,
        // but tests the function signature and basic structure
        let result = process_dm_file(
            "nonexistent_file.js",
            "testuser",
            output_dir,
            1234567890
        ).await;
        
        // Should fail due to missing file, but not due to compilation issues
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_javascript_prefix_removal() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();
        
        // Create test DM content with JavaScript prefix
        let test_dm_content = r#"window.YTD.direct_messages.part0 = [
  {
    "dmConversation": {
      "conversationId": "test-123",
      "messages": [
        {
          "messageCreate": {
            "id": "msg1",
            "text": "Test message",
            "createdAt": "2023-01-01T10:00:00.000Z",
            "senderId": "user1",
            "recipientId": "user2"
          }
        }
      ]
    }
  }
]"#;
        
        let dm_file_path = output_dir.join("test_dm.js");
        fs::write(&dm_file_path, test_dm_content).unwrap();
        
        let result = process_dm_file(
            dm_file_path.to_str().unwrap(),
            "testuser",
            output_dir,
            1234567890
        ).await;
        
        // Should succeed with proper JavaScript prefix handling
        assert!(result.is_ok());
        
        // Verify output files were created
        let csv_file = output_dir.join("dm_conversations_testuser_1234567890.csv");
        let summary_file = output_dir.join("dm_results_testuser_1234567890.txt");
        
        assert!(csv_file.exists());
        assert!(summary_file.exists());
    }

    #[test]
    fn test_processed_conversation_creation() {
        let conversation = ProcessedConversation {
            conversation_id: "test-conversation".to_string(),
            message_count: 5,
            participants: vec!["user1".to_string(), "user2".to_string()],
            first_message_date: Some("2023-01-01T10:00:00.000Z".to_string()),
            last_message_date: Some("2023-01-01T11:00:00.000Z".to_string()),
        };
        
        assert_eq!(conversation.conversation_id, "test-conversation");
        assert_eq!(conversation.message_count, 5);
        assert_eq!(conversation.participants.len(), 2);
        assert!(conversation.first_message_date.is_some());
        assert!(conversation.last_message_date.is_some());
    }

    #[tokio::test]
    async fn test_empty_dm_file() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path();
        
        // Create empty DM content
        let test_dm_content = "[]";
        
        let dm_file_path = output_dir.join("empty_dm.js");
        fs::write(&dm_file_path, test_dm_content).unwrap();
        
        let result = process_dm_file(
            dm_file_path.to_str().unwrap(),
            "testuser",
            output_dir,
            1234567890
        ).await;
        
        // Should handle empty files gracefully
        assert!(result.is_ok());
    }
}