use crate::models::{Course, CourseSection, RuntimeConfig};
use crate::moodle::MoodleError;
use crate::moodle::api::call_webservice;
use crate::moodle::normalize::{as_i64, normalize_course, normalize_section};
use serde_json::Value;

pub async fn fetch_courses(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
) -> Result<Vec<Course>, MoodleError> {
    let site_info = call_webservice(client, config, token, "core_webservice_get_site_info", &[]).await?;
    let user_id = site_info
        .get("userid")
        .and_then(as_i64)
        .ok_or_else(|| MoodleError::message("Could not resolve current user id from Moodle site info"))?;

    let raw = call_webservice(
        client,
        config,
        token,
        "core_enrol_get_users_courses",
        &[("userid", user_id.to_string())],
    )
    .await?;

    let array = raw
        .as_array()
        .ok_or_else(|| MoodleError::message("Unexpected Moodle response for enrolled courses"))?;

    let mut courses: Vec<Course> = array.iter().filter_map(normalize_course).collect();
    courses.sort_by(|left, right| {
        left.fullname.to_lowercase().cmp(&right.fullname.to_lowercase())
    });
    Ok(courses)
}

pub async fn fetch_course_contents(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    course_id: i64,
) -> Result<Vec<CourseSection>, MoodleError> {
    let raw = call_webservice(
        client,
        config,
        token,
        "core_course_get_contents",
        &[("courseid", course_id.to_string())],
    )
    .await?;

    let array = raw
        .as_array()
        .ok_or_else(|| MoodleError::message("Unexpected Moodle response for course contents"))?;
    Ok(array.iter().filter_map(normalize_section).collect())
}

#[allow(dead_code)]
fn _coerce_value(_: &Value) {}
