use crate::models::{DEFAULT_MOODLE_SERVICE, RuntimeConfig};
use crate::moodle::MoodleError;
use crate::moodle::api::{post_form, token_endpoint};
use crate::moodle::normalize::normalize_token_response;

pub async fn request_token(
    client: &reqwest::Client,
    config: &RuntimeConfig,
) -> Result<String, MoodleError> {
    let service = if config.service.trim().is_empty() {
        DEFAULT_MOODLE_SERVICE.to_owned()
    } else {
        config.service.trim().to_owned()
    };

    let payload = post_form(
        client,
        &token_endpoint(&config.base_url),
        &[
            ("username", config.username.clone()),
            ("password", config.password.clone()),
            ("service", service),
        ],
    )
    .await?;

    let (token, error, errorcode, debuginfo) = normalize_token_response(&payload);
    if let Some(token) = token {
        return Ok(token);
    }
    let reason: Vec<String> = [error, errorcode, debuginfo].into_iter().flatten().collect();
    let message = if reason.is_empty() {
        "Token request failed".to_owned()
    } else {
        reason.join(" | ")
    };
    Err(MoodleError::Message(message))
}

pub async fn test_credentials(
    client: &reqwest::Client,
    config: &RuntimeConfig,
) -> Result<(), MoodleError> {
    request_token(client, config).await.map(|_| ())
}
