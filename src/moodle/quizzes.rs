use crate::models::{QuizAttempt, QuizAttemptData, QuizSummary, RuntimeConfig};
use crate::moodle::MoodleError;
use crate::moodle::api::call_webservice;
use crate::moodle::normalize::{
    as_i64, normalize_course_quizzes, normalize_quiz_attempt, normalize_quiz_attempt_data,
};
use crate::moodle::quiz_html::build_answer_params;
use serde_json::Value;

pub async fn fetch_course_quizzes(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    course_id: i64,
) -> Result<Vec<QuizSummary>, MoodleError> {
    let payload = call_webservice(
        client,
        config,
        token,
        "mod_quiz_get_quizzes_by_courses",
        &[("courseids[0]", course_id.to_string())],
    )
    .await?;
    Ok(normalize_course_quizzes(&payload, Some(course_id)))
}

pub async fn start_attempt(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    quiz_id: i64,
) -> Result<QuizAttempt, MoodleError> {
    let payload = call_webservice(
        client,
        config,
        token,
        "mod_quiz_start_attempt",
        &[("quizid", quiz_id.to_string())],
    )
    .await?;
    normalize_quiz_attempt(&payload)
        .ok_or_else(|| MoodleError::message("Unexpected Moodle response for quiz attempt"))
}

pub async fn start_or_resume_attempt(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    quiz_id: i64,
) -> Result<QuizAttempt, MoodleError> {
    match start_attempt(client, config, token, quiz_id).await {
        Ok(attempt) => Ok(attempt),
        Err(MoodleError::Message(message)) if message.contains("attemptstillinprogress") => {
            fetch_unfinished_attempt(client, config, token, quiz_id).await
        }
        Err(error) => Err(error),
    }
}

async fn fetch_unfinished_attempt(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    quiz_id: i64,
) -> Result<QuizAttempt, MoodleError> {
    let site = call_webservice(client, config, token, "core_webservice_get_site_info", &[]).await?;
    let user_id = site
        .get("userid")
        .and_then(as_i64)
        .ok_or_else(|| MoodleError::message("Could not resolve current Moodle user id"))?;
    let payload = call_webservice(
        client,
        config,
        token,
        "mod_quiz_get_user_attempts",
        &[
            ("quizid", quiz_id.to_string()),
            ("userid", user_id.to_string()),
            ("status", "unfinished".to_owned()),
        ],
    )
    .await?;
    let attempts = payload
        .get("attempts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| MoodleError::message("Unexpected Moodle response for quiz attempts"))?;
    attempts
        .iter()
        .filter_map(normalize_quiz_attempt)
        .find(|attempt| attempt.state == "inprogress")
        .or_else(|| attempts.iter().filter_map(normalize_quiz_attempt).next())
        .ok_or_else(|| {
            MoodleError::message("Moodle reported an unfinished attempt, but none was returned")
        })
}

pub async fn fetch_attempt_data(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    attempt_id: i64,
) -> Result<QuizAttemptData, MoodleError> {
    let mut payload = fetch_attempt_page(client, config, token, attempt_id, 0).await?;
    let mut page = payload.get("nextpage").and_then(as_i64);
    while let Some(next) = page {
        if next < 0 {
            break;
        }
        let next_payload = match fetch_attempt_page(client, config, token, attempt_id, next).await {
            Ok(payload) => payload,
            Err(error) if next > 0 && is_invalid_quiz_page_error(&error) => break,
            Err(error) => return Err(error),
        };
        let next_questions = next_payload
            .get("questions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if let Some(questions) = payload.get_mut("questions").and_then(Value::as_array_mut) {
            questions.extend(next_questions);
        }
        page = next_payload.get("nextpage").and_then(as_i64);
    }
    normalize_quiz_attempt_data(&payload)
        .ok_or_else(|| MoodleError::message("Unexpected Moodle response for quiz attempt data"))
}

fn is_invalid_quiz_page_error(error: &MoodleError) -> bool {
    match error {
        MoodleError::Message(message) => message.contains("Invalid page number"),
        MoodleError::Http(_) => false,
    }
}

async fn fetch_attempt_page(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    attempt_id: i64,
    page: i64,
) -> Result<Value, MoodleError> {
    call_webservice(
        client,
        config,
        token,
        "mod_quiz_get_attempt_data",
        &[
            ("attemptid", attempt_id.to_string()),
            ("page", page.to_string()),
        ],
    )
    .await
}

pub async fn save_attempt(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    attempt: &QuizAttemptData,
) -> Result<QuizAttemptData, MoodleError> {
    let mut params = vec![("attemptid".to_owned(), attempt.attempt.id.to_string())];
    append_answer_data(&mut params, attempt);
    let extra = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect::<Vec<_>>();
    let _ = call_webservice(client, config, token, "mod_quiz_process_attempt", &extra).await?;
    fetch_attempt_data(client, config, token, attempt.attempt.id).await
}

pub async fn finish_attempt(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    attempt: &QuizAttemptData,
) -> Result<(), MoodleError> {
    let mut params = vec![
        ("attemptid".to_owned(), attempt.attempt.id.to_string()),
        ("finishattempt".to_owned(), "1".to_owned()),
    ];
    append_answer_data(&mut params, attempt);
    let extra = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect::<Vec<_>>();
    let _ = call_webservice(client, config, token, "mod_quiz_process_attempt", &extra).await?;
    Ok(())
}

fn append_answer_data(params: &mut Vec<(String, String)>, attempt: &QuizAttemptData) {
    let mut idx = 0usize;
    for question in &attempt.questions {
        for (name, value) in build_answer_params(&question.controls) {
            params.push((format!("data[{idx}][name]"), name));
            params.push((format!("data[{idx}][value]"), value));
            idx += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        QuizAnswerControl, QuizAnswerKind, QuizAttempt, QuizAttemptData, QuizQuestion,
    };

    #[test]
    fn appends_moodle_answer_payload_names() {
        let mut params = Vec::new();
        let attempt = QuizAttemptData {
            attempt: QuizAttempt {
                id: 1,
                quiz: 2,
                state: "inprogress".into(),
                currentpage: None,
                timestart: None,
                timefinish: None,
            },
            questions: vec![QuizQuestion {
                slot: 1,
                number: None,
                name: "Q".into(),
                text: String::new(),
                html: String::new(),
                unsupported: false,
                controls: vec![QuizAnswerControl {
                    name: "q1:1_answer".into(),
                    kind: QuizAnswerKind::Text,
                    options: Vec::new(),
                    value: "answer".into(),
                }],
            }],
            warnings: Vec::new(),
        };
        append_answer_data(&mut params, &attempt);
        assert_eq!(params[0], ("data[0][name]".into(), "q1:1_answer".into()));
        assert_eq!(params[1], ("data[0][value]".into(), "answer".into()));
    }
}
