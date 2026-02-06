//! System message content generators
//!
//! System messages store structured JSON content that the frontend
//! formats according to the user's locale.

use serde::{Deserialize, Serialize};
use serde_json::json;

/// Participant info for system messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
}

/// Generate content for "chat created" system message
pub fn chat_created_content(participants: Vec<ParticipantInfo>) -> String {
    json!({
        "event": "chat_created",
        "participants": participants
    })
    .to_string()
}

/// Generate content for "participant joined" system message
pub fn participant_joined_content(name: &str, company: Option<&str>) -> String {
    let mut content = json!({
        "event": "participant_joined",
        "name": name
    });
    if let Some(c) = company {
        content["company"] = json!(c);
    }
    content.to_string()
}

/// Generate content for "participant left" system message
pub fn participant_left_content(name: &str) -> String {
    json!({
        "event": "participant_left",
        "name": name
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_created_content() {
        let participants = vec![
            ParticipantInfo {
                name: "Иван Иванов".to_string(),
                company: Some("ООО Ромашка".to_string()),
            },
            ParticipantInfo {
                name: "Пётр Петров".to_string(),
                company: None,
            },
        ];
        let content = chat_created_content(participants);
        assert!(content.contains("chat_created"));
        assert!(content.contains("Иван Иванов"));
        assert!(content.contains("ООО Ромашка"));
        assert!(content.contains("Пётр Петров"));
    }

    #[test]
    fn test_participant_joined_content_with_company() {
        let content = participant_joined_content("Алексей", Some("ООО Василёк"));
        assert!(content.contains("participant_joined"));
        assert!(content.contains("Алексей"));
        assert!(content.contains("ООО Василёк"));
    }

    #[test]
    fn test_participant_joined_content_without_company() {
        let content = participant_joined_content("Алексей", None);
        assert!(content.contains("participant_joined"));
        assert!(content.contains("Алексей"));
        assert!(!content.contains("company"));
    }

    #[test]
    fn test_participant_left_content() {
        let content = participant_left_content("Алексей");
        assert!(content.contains("participant_left"));
        assert!(content.contains("Алексей"));
    }

    #[test]
    fn test_json_format() {
        // Verify JSON can be parsed
        let content = chat_created_content(vec![ParticipantInfo {
            name: "Test".to_string(),
            company: None,
        }]);
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["event"], "chat_created");
        assert!(parsed["participants"].is_array());
    }
}
