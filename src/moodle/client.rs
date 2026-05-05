use crate::models::{
    AssignmentDetail, AssignmentSubmissionStatus, Course, CourseSection, RuntimeConfig,
    UpcomingAssignment,
};
use crate::moodle::MoodleError;
use crate::moodle::{assignments, auth, courses};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct MoodleClient {
    http: reqwest::Client,
}

impl Default for MoodleClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MoodleClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent(concat!("moodle-tui/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build reqwest client");
        Self { http }
    }

    pub async fn test_credentials(&self, config: &RuntimeConfig) -> Result<(), MoodleError> {
        auth::test_credentials(&self.http, config).await
    }

    pub async fn request_token(&self, config: &RuntimeConfig) -> Result<String, MoodleError> {
        auth::request_token(&self.http, config).await
    }

    pub async fn fetch_courses(&self, config: &RuntimeConfig) -> Result<Vec<Course>, MoodleError> {
        let token = self.request_token(config).await?;
        courses::fetch_courses(&self.http, config, &token).await
    }

    pub async fn fetch_course_contents(
        &self,
        config: &RuntimeConfig,
        course_id: i64,
    ) -> Result<Vec<CourseSection>, MoodleError> {
        let token = self.request_token(config).await?;
        courses::fetch_course_contents(&self.http, config, &token, course_id).await
    }

    pub async fn fetch_course_assignments(
        &self,
        config: &RuntimeConfig,
        course_id: i64,
    ) -> Result<Vec<AssignmentDetail>, MoodleError> {
        let token = self.request_token(config).await?;
        assignments::fetch_course_assignments(&self.http, config, &token, course_id).await
    }

    pub async fn fetch_assignment_detail(
        &self,
        config: &RuntimeConfig,
        course_id: i64,
        assignment_id: i64,
    ) -> Result<Option<AssignmentDetail>, MoodleError> {
        let token = self.request_token(config).await?;
        assignments::fetch_assignment_detail(&self.http, config, &token, course_id, assignment_id)
            .await
    }

    pub async fn fetch_assignment_submission_status(
        &self,
        config: &RuntimeConfig,
        assign_id: i64,
    ) -> Result<Option<AssignmentSubmissionStatus>, MoodleError> {
        let token = self.request_token(config).await?;
        assignments::fetch_assignment_submission_status(&self.http, config, &token, assign_id).await
    }

    pub async fn fetch_upcoming_assignments(
        &self,
        config: &RuntimeConfig,
    ) -> Result<Vec<UpcomingAssignment>, MoodleError> {
        let token = self.request_token(config).await?;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        assignments::fetch_upcoming_assignments(&self.http, config, &token, now).await
    }
}
