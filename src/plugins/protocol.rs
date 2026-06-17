use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostMessage {
    Initialize {
        protocol_version: u32,
        app: String,
    },
    Event {
        event: PluginEvent,
    },
    Invoke {
        id: String,
        action: String,
        payload: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginEvent {
    QuizCurrentQuestion(QuizQuestionContext),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuizQuestionContext {
    pub quiz_id: i64,
    pub attempt_id: i64,
    pub quiz_name: String,
    pub question_index: usize,
    pub question_number: Option<String>,
    pub question_text: String,
    #[serde(default)]
    pub controls: Vec<QuizControlContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuizOptionContext {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuizControlContext {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub options: Vec<QuizOptionContext>,
    #[serde(default)]
    pub current_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PluginMessage {
    Ok {
        id: Option<String>,
        payload: Value,
    },
    Error {
        id: Option<String>,
        message: String,
    },
    HostAction {
        id: Option<String>,
        action: HostAction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HostAction {
    ShowToast { message: String },
    CopyToClipboard { text: String },
    OpenUrl { url: String },
    ShowPanel { title: String, markdown: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StudyHelpResponse {
    pub summary: String,
    #[serde(default)]
    pub hints: Vec<String>,
    pub confidence: StudyHelpConfidence,
    #[serde(default)]
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StudyHelpConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiFillResponse {
    pub answers: Vec<ControlAnswer>,
    pub explanation: String,
    pub confidence: StudyHelpConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlAnswer {
    pub control_name: String,
    #[serde(default)]
    pub selected_values: Vec<String>,
    #[serde(default)]
    pub text_value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_quiz_context_with_control_names() {
        let context = QuizQuestionContext {
            quiz_id: 1,
            attempt_id: 2,
            quiz_name: "Quiz".into(),
            question_index: 0,
            question_number: Some("1".into()),
            question_text: "Pick the stylesheet language.".into(),
            controls: vec![QuizControlContext {
                name: "q1:1_answer".into(),
                kind: "single_choice".into(),
                options: vec![
                    QuizOptionContext {
                        label: "HTML".into(),
                        value: "0".into(),
                        name: None,
                    },
                    QuizOptionContext {
                        label: "CSS".into(),
                        value: "1".into(),
                        name: None,
                    },
                ],
                current_text: None,
            }],
        };
        let json = serde_json::to_string(&HostMessage::Event {
            event: PluginEvent::QuizCurrentQuestion(context),
        })
        .unwrap();
        assert!(json.contains("stylesheet language"));
        assert!(json.contains("CSS"));
        assert!(json.contains("q1:1_answer"));
    }
    #[test]
    fn parses_ai_fill_response() {
        let response: AiFillResponse = serde_json::from_str(
            r#"{
              "answers": [{"control_name": "q1:1_answer", "selected_values": ["1"]}],
              "explanation": "CSS is the correct answer for styling.",
              "confidence": "high"
            }"#,
        )
        .unwrap();
        assert_eq!(response.answers.len(), 1);
        assert_eq!(response.answers[0].control_name, "q1:1_answer");
        assert_eq!(response.answers[0].selected_values, vec!["1"]);
        assert_eq!(response.confidence, StudyHelpConfidence::High);
    }

    #[test]
    fn parses_study_help_response() {
        let response: StudyHelpResponse = serde_json::from_str(
            r#"{
              "summary": "CSS controls presentation.",
              "hints": ["Look for styling terms."],
              "confidence": "high",
              "limitations": []
            }"#,
        )
        .unwrap();
        assert_eq!(response.confidence, StudyHelpConfidence::High);
    }
}
