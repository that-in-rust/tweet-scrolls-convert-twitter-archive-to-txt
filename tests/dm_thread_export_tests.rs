// TDD for DM thread export: user IDs and relative timestamps

#[cfg(test)]
mod tests {
    use chrono::Utc;

    struct MockMessage {
        sender_id: String,
        text: String,
        timestamp: String,
    }

    fn export_dm_thread(messages: &[MockMessage]) -> Vec<String> {
        let mut output = Vec::new();
        let mut prev_time: Option<chrono::DateTime<Utc>> = None;
        for msg in messages {
            let curr_time = chrono::DateTime::parse_from_rfc3339(&msg.timestamp).unwrap().with_timezone(&Utc);
            let rel = if let Some(prev) = prev_time {
                let delta = curr_time - prev;
                if delta.num_days() > 0 {
                    format!(" ({} days later)", delta.num_days())
                } else if delta.num_hours() > 0 {
                    format!(" ({} hours later)", delta.num_hours())
                } else {
                    format!(" ({} minutes later)", delta.num_minutes())
                }
            } else {
                String::new()
            };
            output.push(format!("{}: {}{}", msg.sender_id, msg.text, rel));
            prev_time = Some(curr_time);
        }
        output
    }

    #[test]
    fn test_dm_thread_export_with_user_ids_and_relative_timestamps() {
        let messages = vec![
            MockMessage {
                sender_id: "1754755789".to_string(),
                text: "Hello!".to_string(),
                timestamp: "2025-08-09T10:00:00Z".to_string(),
            },
            MockMessage {
                sender_id: "1234567890".to_string(),
                text: "Hi there!".to_string(),
                timestamp: "2025-08-09T10:05:00Z".to_string(),
            },
            MockMessage {
                sender_id: "1754755789".to_string(),
                text: "How are you?".to_string(),
                timestamp: "2025-08-09T12:05:00Z".to_string(),
            },
        ];
        let output = export_dm_thread(&messages);
        assert_eq!(output[0], "1754755789: Hello!");
        assert_eq!(output[1], "1234567890: Hi there! (5 minutes later)");
        assert_eq!(output[2], "1754755789: How are you? (2 hours later)");
    }
}
