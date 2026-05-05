use crate::models::{AssignmentDetail, AssignmentSubmissionStatus, RuntimeConfig, UpcomingAssignment};
use crate::moodle::MoodleError;
use crate::moodle::api::call_webservice;
use crate::moodle::normalize::{
    normalize_course_assignments, normalize_submission_status, normalize_upcoming_assignments,
};

pub async fn fetch_course_assignments(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    course_id: i64,
) -> Result<Vec<AssignmentDetail>, MoodleError> {
    let payload = call_webservice(
        client,
        config,
        token,
        "mod_assign_get_assignments",
        &[("courseids[0]", course_id.to_string())],
    )
    .await?;
    Ok(normalize_course_assignments(&payload, Some(course_id)))
}

pub async fn fetch_assignment_submission_status(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    assign_id: i64,
) -> Result<Option<AssignmentSubmissionStatus>, MoodleError> {
    let payload = call_webservice(
        client,
        config,
        token,
        "mod_assign_get_submission_status",
        &[("assignid", assign_id.to_string())],
    )
    .await?;
    Ok(normalize_submission_status(&payload))
}

pub async fn fetch_assignment_detail(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    course_id: i64,
    assignment_id: i64,
) -> Result<Option<AssignmentDetail>, MoodleError> {
    let assignments = fetch_course_assignments(client, config, token, course_id).await?;
    Ok(assignments.into_iter().find(|a| a.id == assignment_id))
}

pub async fn fetch_upcoming_assignments(
    client: &reqwest::Client,
    config: &RuntimeConfig,
    token: &str,
    now_timestamp: i64,
) -> Result<Vec<UpcomingAssignment>, MoodleError> {
    let payload = call_webservice(client, config, token, "mod_assign_get_assignments", &[]).await?;
    Ok(normalize_upcoming_assignments(&payload, now_timestamp))
}
