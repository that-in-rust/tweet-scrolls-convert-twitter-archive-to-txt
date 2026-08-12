//! User profile data structures and related functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a user's profile with interaction statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// User ID
    pub user_id: String,
    /// Total number of interactions
    pub total_interactions: u32,
    /// First interaction timestamp
    pub first_interaction: Option<DateTime<Utc>>,
    /// Last interaction timestamp
    pub last_interaction: Option<DateTime<Utc>>,
    /// Count of interactions by type
    pub interaction_counts: HashMap<String, u32>,
    /// Additional profile metadata
    pub metadata: HashMap<String, String>,
}

impl UserProfile {
    /// Creates a new empty user profile
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            total_interactions: 0,
            first_interaction: None,
            last_interaction: None,
            interaction_counts: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Updates the profile with a new interaction
    pub fn add_interaction(&mut self, interaction_type: impl Into<String>, timestamp: DateTime<Utc>) {
        self.total_interactions += 1;
        
        // Update interaction type count
        let type_str = interaction_type.into();
        *self.interaction_counts.entry(type_str).or_insert(0) += 1;
        
        // Update timestamps
        if self.first_interaction.is_none_or(|t| timestamp < t) {
            self.first_interaction = Some(timestamp);
        }
        if self.last_interaction.is_none_or(|t| timestamp > t) {
            self.last_interaction = Some(timestamp);
        }
    }
}


