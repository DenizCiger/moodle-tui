use crate::models::{RuntimeConfig, normalize_base_url};
use crate::moodle::MoodleError;
use serde_json::Value;

pub fn token_endpoint(base_url: &str) -> String {
    format!("{}/login/token.php", normalize_base_url(base_url))
}

pub fn rest_endpoint(base_url: &str) -> String {
    format!("{}/webservice/rest/server.php", normalize_base_url(base_url))
}

pub async fn post_form(
    client: &reqwest::Client,
    url: &str,
    params: &[(&str, String)],
) -> Result<Value, MoodleError> {
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(params)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;
    let parsed: Option<Value> = serde_json::from_str(&body).ok();

    if !status.is_success() {
        let detail = if body.trim().is_empty() {
            "no body".to_owned()
        } else {
            body
        };
        return Err(MoodleError::message(format!(
            "HTTP {} while calling Moodle endpoint: {}",
            status.as_u16(),
            detail
        )));
    }

    parsed.ok_or_else(|| MoodleError::message("Moodle endpoint returned non-JSON response"))
}

pub fn extract_moodle_exception(value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let message = object.get("message").and_then(Value::as_str);
    let errorcode = object.get("errorcode").and_then(Value::as_str);
    let exception = object.get("exception").and_then(Value::as_str);
    let debuginfo = object.get("debuginfo").and_then(Value::as_str);

    if message.is_none() && errorcode.is_none() && exception.is_none() {
        return None;
    }

    let mut fragments = Vec::new();
    if let Some(value) = message {
        fragments.push(format!("message={value}"));
    }
    if let Some(value) = errorcode {
        fragments.push(format!("errorcode={value}"));
    }
    if let Some(value) = exception {
        fragments.push(format!("exception={value}"));
    }
    if let Some(value) = debuginfo {
        fragments.push(format!("debuginfo={value}"));
    }

    if fragments.is_empty() {
        Some("Moodle returned an unknown error".to_owned())
    } else {
        Some(fragments.join(" | "))
    }
}

pub async fn call_webservice(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    wsfunction: &str,
    extra: &[(&str, String)],
) -> Result<Value, MoodleError> {
    let mut params: Vec<(&str, String)> = Vec::with_capacity(3 + extra.len());
    params.push(("wstoken", token.to_owned()));
    params.push(("wsfunction", wsfunction.to_owned()));
    params.push(("moodlewsrestformat", "json".to_owned()));
    for (key, value) in extra {
        params.push((key, value.clone()));
    }

    let payload = post_form(client, &rest_endpoint(&config.base_url), &params).await?;
    if let Some(exception) = extract_moodle_exception(&payload) {
        return Err(MoodleError::Message(exception));
    }
    Ok(payload)
}
