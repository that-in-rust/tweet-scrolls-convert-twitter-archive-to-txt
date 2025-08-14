use tweet_scrolls::relationship::file_generation::LLMFileGenerator;

use tweet_scrolls::models::profile::UserProfile;
use tweet_scrolls::models::interaction::InteractionEvent;
use std::collections::HashMap;
use tempfile::tempdir;
use anyhow::Result;

/// Main orchestration function for relationship analysis
pub async fn analyze_relationships(
    output_path: &str,
    screen_name: &str,
    timestamp: u64,
    profiles: &[UserProfile],
    interactions: &[InteractionEvent]
) -> Result<()> {
    // Validate output directory
    validate_output_directory(output_path)?;
    
    // Create file generator
    let generator = LLMFileGenerator::new(output_path, screen_name, timestamp);
    
    // Generate all files
    generator.generate_all_files(profiles, interactions)?;
    
    println!("✅ Relationship intelligence analysis complete!");
    println!("📁 Files generated in: {}/relationship_profiles_{}_{}", 
             output_path, screen_name, timestamp);
    println!("📊 {}", format_relationship_summary(profiles));
    
    Ok(())
}

/// Format a summary of the relationship analysis
pub fn format_relationship_summary(profiles: &[UserProfile]) -> String {
    let total_interactions: u32 = profiles.iter().map(|p| p.total_interactions).sum();
    let avg_interactions = if profiles.is_empty() { 
        0.0 
    } else { 
        total_interactions as f64 / profiles.len() as f64 
    };
    
    // Find most active relationship
    let most_active = profiles.iter()
        .max_by_key(|p| p.total_interactions)
        .map(|p| format!("{} ({} interactions)", p.user_id, p.total_interactions))
        .unwrap_or_else(|| "None".to_string());
    
    // Calculate activity distribution
    let high_activity = profiles.iter().filter(|p| p.total_interactions > 20).count();
    let medium_activity = profiles.iter().filter(|p| p.total_interactions >= 10 && p.total_interactions <= 20).count();
    let low_activity = profiles.iter().filter(|p| p.total_interactions < 10).count();
    
    format!(
        "=== Relationship Analysis Summary ===\n\
        Total Relationships Analyzed: {}\n\
        Total Interactions: {}\n\
        Average Interactions per Relationship: {:.1}\n\
        Most Active Relationship: {}\n\
        \n\
        Activity Distribution:\n\
        - High Activity (>20): {} relationships\n\
        - Medium Activity (10-20): {} relationships\n\
        - Low Activity (<10): {} relationships",
        profiles.len(),
        total_interactions,
        avg_interactions,
        most_active,
        high_activity,
        medium_activity,
        low_activity
    )
}

/// Validate that the output directory is accessible
pub fn validate_output_directory(path: &str) -> Result<()> {
    use std::fs;
    use anyhow::Context;
    
    // Try to create the directory to test permissions
    fs::create_dir_all(path)
        .with_context(|| format!("Cannot create output directory: {}", path))?;
    
    // Test write permissions by creating a temporary file
    let test_file = std::path::Path::new(path).join(".test_write_permissions");
    fs::write(&test_file, "test")
        .with_context(|| format!("Cannot write to output directory: {}", path))?;
    
    // Clean up test file
    let _ = fs::remove_file(test_file);
    
    Ok(())
}

#[cfg(test)]
mod main_integration_tests {
    use super::*;

    fn create_sample_profiles() -> Vec<UserProfile> {
        let mut profile1 = UserProfile::new("user1".to_string());
        profile1.total_interactions = 15;
        profile1.first_interaction = Some("2023-01-01T10:00:00Z".parse().unwrap());
        profile1.last_interaction = Some("2023-12-31T15:30:00Z".parse().unwrap());
        profile1.interaction_counts = HashMap::from([
            ("dm_sent".to_string(), 8),
            ("dm_received".to_string(), 7),
        ]);

        let mut profile2 = UserProfile::new("user2".to_string());
        profile2.total_interactions = 23;
        profile2.first_interaction = Some("2023-02-15T14:20:00Z".parse().unwrap());
        profile2.last_interaction = Some("2023-11-20T09:45:00Z".parse().unwrap());
        profile2.interaction_counts = HashMap::from([
            ("tweet_reply".to_string(), 12),
            ("dm_sent".to_string(), 6),
            ("dm_received".to_string(), 5),
        ]);

        vec![profile1, profile2]
    }

    fn create_sample_interactions() -> Vec<InteractionEvent> {
        vec![
            InteractionEvent::new(
                "interaction_001",
                "2023-06-15T10:30:00Z".parse().unwrap(),
                tweet_scrolls::models::interaction::InteractionType::DmSent,
                "user1".to_string(),
                "Hey, how's the project going?"
            ),
            InteractionEvent::new(
                "interaction_002",
                "2023-06-15T14:45:00Z".parse().unwrap(),
                tweet_scrolls::models::interaction::InteractionType::DmReceived,
                "user2".to_string(),
                "Going well! Just finished the first milestone."
            ),
        ]
    }

    #[tokio::test]
    async fn test_analyze_relationships_function() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().to_str().unwrap();
        
        let profiles = create_sample_profiles();
        let interactions = create_sample_interactions();
        
        let result = analyze_relationships(
            output_path,
            "testuser",
            1234567890,
            &profiles,
            &interactions
        ).await;
        
        assert!(result.is_ok());
        
        // Verify directory was created
        let expected_dir = format!("{}/relationship_profiles_testuser_1234567890", output_path);
        assert!(std::path::Path::new(&expected_dir).exists());
        
        // Verify key files were created
        assert!(std::path::Path::new(&expected_dir).join("interaction_timeline.txt").exists());
        assert!(std::path::Path::new(&expected_dir).join("communication_patterns.txt").exists());
        assert!(std::path::Path::new(&expected_dir).join("relationship_network.txt").exists());
        assert!(std::path::Path::new(&expected_dir).join("llm_analysis_prompts.txt").exists());
        
        // Verify individual profile files were created (full user ID in filename)
        for profile in &profiles {
            let profile_filename = format!("user_{}_profile.txt", &profile.user_id);
            assert!(std::path::Path::new(&expected_dir).join(profile_filename).exists());
        }
    }

    #[tokio::test]
    async fn test_relationship_analysis_with_empty_data() {
        let temp_dir = tempdir().unwrap();
        let output_path = temp_dir.path().to_str().unwrap();
        
        let profiles = vec![];
        let interactions = vec![];
        
        let result = analyze_relationships(
            output_path,
            "testuser",
            1234567890,
            &profiles,
            &interactions
        ).await;
        
        assert!(result.is_ok());
        
        // Should still create directory and basic files
        let expected_dir = format!("{}/relationship_profiles_testuser_1234567890", output_path);
        assert!(std::path::Path::new(&expected_dir).exists());
    }

    #[tokio::test]
    async fn test_relationship_analysis_error_handling() {
        // Test with invalid path
        let result = analyze_relationships(
            "/invalid/path/that/does/not/exist",
            "testuser",
            1234567890,
            &create_sample_profiles(),
            &create_sample_interactions()
        ).await;
        
        // Should handle error gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_format_relationship_summary() {
        let profiles = create_sample_profiles();
        let summary = format_relationship_summary(&profiles);
        
        assert!(summary.contains("Total Relationships Analyzed: 2"));
        assert!(summary.contains("Total Interactions: 38"));
        assert!(summary.contains("Average Interactions per Relationship: 19.0"));
        assert!(summary.len() > 100); // Should be a substantial summary
    }

    #[test]
    fn test_validate_output_directory() {
        let temp_dir = tempdir().unwrap();
        let valid_path = temp_dir.path().to_str().unwrap();
        
        // Valid directory should pass
        assert!(validate_output_directory(valid_path).is_ok());
        
        // Invalid directory should fail
        assert!(validate_output_directory("/invalid/path").is_err());
    }
}