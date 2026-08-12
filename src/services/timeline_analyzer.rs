//! Timeline analysis service for the Tweet-Scrolls application

use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use std::collections::{HashMap, HashSet};


#[allow(unused_imports)]
use crate::models::interaction::{InteractionEvent, InteractionType};
use crate::models::statistics::calculate_percentiles;
use crate::models::timeline::{
    ResponseTimeStats, TimelineAnalysis, TimelineDensity, TimelinePattern,
};

/// Analyzes a timeline of interaction events and extracts patterns and statistics
pub struct TimelineAnalyzer {
    events: Vec<InteractionEvent>,
}

impl TimelineAnalyzer {
    /// Creates a new TimelineAnalyzer with the given events
    pub fn new(events: Vec<InteractionEvent>) -> Self {
        // Ensure events are sorted by timestamp
        let mut events = events;
        events.sort_by_key(|event| event.timestamp);
        TimelineAnalyzer { events }
    }

    /// Performs timeline analysis and returns the results
    pub fn analyze(&self) -> TimelineAnalysis {
        if self.events.is_empty() {
            return TimelineAnalysis::new(Utc::now(), Utc::now());
        }

        let start_time = self.events.first().unwrap().timestamp;
        let end_time = self.events.last().unwrap().timestamp;
        let total_days = (end_time - start_time).num_days().max(1) as f64;

        let mut analysis = TimelineAnalysis::new(start_time, end_time);
        
        // Calculate basic statistics
        analysis.total_interactions = self.events.len();
        analysis.unique_participants = self.calculate_unique_participants();
        
        // Analyze patterns
        analysis.patterns = self.detect_patterns();
        
        // Calculate density metrics
        analysis.density = self.calculate_density(&start_time, &end_time, total_days);
        
        // Calculate response times
        analysis.response_times = self.calculate_response_times();
        
        analysis
    }

    /// Calculates the number of unique participants in the timeline
    fn calculate_unique_participants(&self) -> usize {
        let mut participants = HashSet::new();
        for event in &self.events {
            participants.insert(&event.user_id);
        }
        participants.len()
    }

    /// Detects patterns in the timeline
    fn detect_patterns(&self) -> Vec<TimelinePattern> {
        let mut patterns = Vec::new();
        
        // Detect daily rhythm (more than 3 events per day on average)
        let avg_events_per_day = self.events.len() as f64 / 7.0;
        if avg_events_per_day > 3.0 {
            patterns.push(TimelinePattern::DailyRhythm);
        }
        
        // Detect time of day patterns
        let active_hours = self.detect_active_hours();
        if !active_hours.is_empty() {
            patterns.push(TimelinePattern::TimeOfDayPattern { active_hours });
        }
        
        // Detect weekly patterns
        let active_days = self.detect_active_days();
        if !active_days.is_empty() {
            patterns.push(TimelinePattern::WeeklyPattern { active_days });
        }
        
        // Detect bursty activity
        if self.is_bursty_activity() {
            patterns.push(TimelinePattern::BurstyActivity);
        }
        
        if patterns.is_empty() {
            patterns.push(TimelinePattern::NoPattern);
        }
        
        patterns
    }

    /// Detects active hours in the timeline
    /// An hour is considered active if it has at least 2 interactions
    fn detect_active_hours(&self) -> Vec<u32> {
        let mut hour_counts = [0; 24];
        
        // Count interactions per hour
        for event in &self.events {
            let hour = event.timestamp.hour();
            hour_counts[hour as usize] += 1;
        }
        
        // Find hours with at least 2 interactions
        hour_counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count >= 2)  // At least 2 interactions
            .map(|(hour, _)| hour as u32)
            .collect()
    }

    /// Detects active days in the timeline
    fn detect_active_days(&self) -> Vec<Weekday> {
        let mut day_counts = [0; 7];
        
        for event in &self.events {
            let weekday = event.timestamp.weekday();
            day_counts[weekday.num_days_from_sunday() as usize] += 1;
        }
        
        // Find days with above-average activity
        let avg = self.events.len() as f64 / 7.0;
        let threshold = avg * 1.5; // 50% more than average
        
        day_counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count as f64 > threshold)
            .filter_map(|(day, _)| {
                match day {
                    0 => Some(Weekday::Sun),
                    1 => Some(Weekday::Mon),
                    2 => Some(Weekday::Tue),
                    3 => Some(Weekday::Wed),
                    4 => Some(Weekday::Thu),
                    5 => Some(Weekday::Fri),
                    6 => Some(Weekday::Sat),
                    _ => None,
                }
            })
            .collect()
    }

    /// Detects if the activity is bursty
    fn is_bursty_activity(&self) -> bool {
        if self.events.len() < 10 {
            return false;
        }
        
        // Calculate time differences between consecutive events
        let mut diffs = Vec::new();
        for window in self.events.windows(2) {
            let diff = (window[1].timestamp - window[0].timestamp)
                .num_seconds() as f64;
            diffs.push(diff);
        }
        
        // Calculate coefficient of variation
        let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
        if mean == 0.0 {
            return false;
        }
        
        let variance = diffs.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / diffs.len() as f64;
        let std_dev = variance.sqrt();
        let cv = std_dev / mean;
        
        // High coefficient of variation indicates bursty activity
        cv > 1.0
    }

    /// Calculates density metrics for the timeline
    fn calculate_density(
        &self,
        _start_time: &DateTime<Utc>,
        _end_time: &DateTime<Utc>,
        total_days: f64,
    ) -> TimelineDensity {
        let mut hour_counts = [0; 24];
        let mut day_counts = [0; 7];
        
        for event in &self.events {
            let hour = event.timestamp.hour();
            let weekday = event.timestamp.weekday();
            
            hour_counts[hour as usize] += 1;
            day_counts[weekday.num_days_from_sunday() as usize] += 1;
        }
        
        // Find peak hour and day
        let (peak_hour, _) = hour_counts.iter().enumerate()
            .max_by_key(|&(_, &count)| count)
            .unwrap_or((0, &0));
            
        let (peak_day, _) = day_counts.iter().enumerate()
            .max_by_key(|&(_, &count)| count)
            .unwrap_or((0, &0));
        
        // Find hours with above-average activity
        let avg_per_hour = self.events.len() as f64 / 24.0;
        let peak_hours = hour_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count as f64 > avg_per_hour * 1.5)
            .map(|(hour, _)| hour as u32)
            .collect();
        
        // Find days with above-average activity
        let avg_per_day = self.events.len() as f64 / 7.0;
        let peak_days = day_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count as f64 > avg_per_day * 1.5)
            .filter_map(|(day, _)| {
                match day {
                    0 => Some(Weekday::Sun),
                    1 => Some(Weekday::Mon),
                    2 => Some(Weekday::Tue),
                    3 => Some(Weekday::Wed),
                    4 => Some(Weekday::Thu),
                    5 => Some(Weekday::Fri),
                    6 => Some(Weekday::Sat),
                    _ => None,
                }
            })
            .collect();
        
        TimelineDensity {
            avg_interactions_per_day: self.events.len() as f64 / total_days,
            peak_hours,
            peak_days,
            peak_hour: peak_hour as u32,
            peak_day: peak_day as u32,
        }
    }

    /// Calculates response time statistics
    fn calculate_response_times(&self) -> ResponseTimeStats {
        let mut response_times = Vec::new();
        
        // Group events by conversation ID from metadata
        let mut conversations: HashMap<String, Vec<&InteractionEvent>> = HashMap::new();
        for event in &self.events {
            if let Some(conv_id) = event.metadata.get("conversation_id") {
                conversations
                    .entry(conv_id.clone())
                    .or_default()
                    .push(event);
            }
        }
        
        // Calculate response times within each conversation
        for (_, events) in conversations {
            if events.len() < 2 {
                continue; // Need at least 2 events for a response time
            }
            
            // Sort conversation events by timestamp
            let mut sorted_events = events;
            sorted_events.sort_by_key(|e| e.timestamp);
            
            // Calculate time between consecutive messages
            for window in sorted_events.windows(2) {
                let duration = (window[1].timestamp - window[0].timestamp)
                    .num_seconds() as f64;
                response_times.push(duration);
            }
        }
        
        if response_times.is_empty() {
            return ResponseTimeStats {
                average: 0.0,
                median: 0.0,
                percentiles: HashMap::new(),
                min: 0.0,
                max: 0.0,
            };
        }
        
        // Sort for percentiles
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Calculate statistics
        let sum: f64 = response_times.iter().sum();
        let count = response_times.len() as f64;
        let average = sum / count;
        
        let median = if count > 0.0 {
            if count % 2.0 == 0.0 {
                let mid = (count / 2.0) as usize;
                (response_times[mid - 1] + response_times[mid]) / 2.0
            } else {
                response_times[(count / 2.0) as usize]
            }
        } else {
            0.0
        };
        
        // Calculate percentiles
        let percentiles = calculate_percentiles(&response_times);
        
        ResponseTimeStats {
            average,
            median,
            percentiles,
            min: *response_times.first().unwrap_or(&0.0),
            max: *response_times.last().unwrap_or(&0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Duration};

    fn create_test_event(
        timestamp: DateTime<Utc>,
        user_id: &str,
        conversation_id: &str,
    ) -> InteractionEvent {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("conversation_id".to_string(), conversation_id.to_string());
        metadata.insert("message_id".to_string(), "1".to_string());
        
        InteractionEvent {
            id: "test_event".to_string(),
            timestamp,
            interaction_type: InteractionType::DmSent,
            user_id: user_id.to_string(),
            content: "Test message".to_string(),
            metadata,
        }
    }

    #[test]
    fn test_timeline_analyzer_empty() {
        let analyzer = TimelineAnalyzer::new(Vec::new());
        let analysis = analyzer.analyze();
        
        assert_eq!(analysis.total_interactions, 0);
        assert_eq!(analysis.unique_participants, 0);
    }

    #[test]
    fn test_timeline_analyzer_basic() {
        let start = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let mut events = Vec::new();
        
        // Create some test events
        for i in 0..5 {
            let timestamp = start + Duration::hours(i * 2);
            events.push(create_test_event(timestamp, "user1", "conv1"));
        }
        
        let analyzer = TimelineAnalyzer::new(events);
        let analysis = analyzer.analyze();
        
        assert_eq!(analysis.total_interactions, 5);
        assert_eq!(analysis.unique_participants, 1);
        assert!(!analysis.patterns.is_empty());
    }

    #[test]
    fn test_detect_active_hours() {
        let start = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let mut events = Vec::new();
        
        // Add events at specific hours
        for &hour in &[10, 10, 11, 11, 11, 14, 22] {
            let timestamp = start + Duration::hours(hour);
            events.push(create_test_event(timestamp, "user1", "conv1"));
        }
        
        let analyzer = TimelineAnalyzer::new(events);
        let active_hours = analyzer.detect_active_hours();
        
        // Should detect hours 10, 11 as active (11 has 3 events, 10 has 2, others have 1)
        assert!(active_hours.contains(&10));
        assert!(active_hours.contains(&11));
        assert!(!active_hours.contains(&14));
        assert!(!active_hours.contains(&22));
    }
}
