use crate::models::{
    AssignmentDetail, AssignmentSubmissionStatus, Course, CourseSection, QuizAttempt,
    QuizAttemptData, QuizSummary, RuntimeConfig, UpcomingAssignment,
};
use crate::moodle::MoodleError;
use crate::moodle::{assignments, auth, courses, quizzes};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct MoodleClient {
    http: Option<reqwest::Client>,
    configuration_error: Option<String>,
}

#[derive(Debug, Clone)]
struct ExpectedOrigin {
    scheme: String,
    host: String,
    port: Option<u16>,
}

impl ExpectedOrigin {
    fn from_url(url: &reqwest::Url) -> Option<Self> {
        Some(Self {
            scheme: url.scheme().to_owned(),
            host: url.host_str()?.to_owned(),
            port: url.port_or_known_default(),
        })
    }

    fn matches(&self, url: &reqwest::Url) -> bool {
        self.scheme == url.scheme()
            && url.host_str() == Some(self.host.as_str())
            && self.port == url.port_or_known_default()
    }
}

impl Default for MoodleClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MoodleClient {
    pub fn new() -> Self {
        Self::build(Vec::new(), None)
    }

    pub fn for_base_url(base_url: &str) -> Self {
        let url = match reqwest::Url::parse(base_url) {
            Ok(url) => url,
            Err(error) => return Self::with_error(format!("Invalid Moodle base URL: {error}")),
        };
        let origin = match ExpectedOrigin::from_url(&url) {
            Some(origin) => origin,
            None => return Self::with_error("Moodle base URL has no hostname"),
        };
        match crate::storage::tls::load_trusted_certificates(base_url) {
            Ok(certificates) => {
                let expected_origin = (!certificates.is_empty()).then_some(origin);
                Self::build(certificates, expected_origin)
            }
            Err(error) => Self::with_error(error.to_string()),
        }
    }

    fn build(
        certificates: Vec<reqwest::Certificate>,
        expected_origin: Option<ExpectedOrigin>,
    ) -> Self {
        let mut builder = reqwest::Client::builder()
            .user_agent(concat!("moodle-tui/", env!("CARGO_PKG_VERSION")));
        for certificate in certificates {
            builder = builder.add_root_certificate(certificate);
        }
        if let Some(expected_origin) = expected_origin {
            builder = builder.redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() >= 10 {
                    attempt.error("too many redirects")
                } else if expected_origin.matches(attempt.url()) {
                    attempt.follow()
                } else {
                    attempt.error("cross-origin redirect blocked")
                }
            }));
        }
        match builder.build() {
            Ok(http) => Self {
                http: Some(http),
                configuration_error: None,
            },
            Err(error) => Self::with_error(format!("Failed to build HTTP client: {error}")),
        }
    }

    fn with_error(error: impl Into<String>) -> Self {
        Self {
            http: None,
            configuration_error: Some(error.into()),
        }
    }

    fn http(&self) -> Result<&reqwest::Client, MoodleError> {
        if let Some(error) = &self.configuration_error {
            return Err(MoodleError::message(error.clone()));
        }
        self.http
            .as_ref()
            .ok_or_else(|| MoodleError::message("HTTP client is unavailable"))
    }

    pub async fn test_credentials(&self, config: &RuntimeConfig) -> Result<(), MoodleError> {
        auth::test_credentials(self.http()?, config).await
    }

    pub async fn request_token(&self, config: &RuntimeConfig) -> Result<String, MoodleError> {
        auth::request_token(self.http()?, config).await
    }

    pub async fn fetch_courses(&self, config: &RuntimeConfig) -> Result<Vec<Course>, MoodleError> {
        let token = self.request_token(config).await?;
        courses::fetch_courses(self.http()?, config, &token).await
    }

    pub async fn fetch_course_contents(
        &self,
        config: &RuntimeConfig,
        course_id: i64,
    ) -> Result<Vec<CourseSection>, MoodleError> {
        let token = self.request_token(config).await?;
        courses::fetch_course_contents(self.http()?, config, &token, course_id).await
    }

    pub async fn fetch_course_assignments(
        &self,
        config: &RuntimeConfig,
        course_id: i64,
    ) -> Result<Vec<AssignmentDetail>, MoodleError> {
        let token = self.request_token(config).await?;
        assignments::fetch_course_assignments(self.http()?, config, &token, course_id).await
    }

    pub async fn fetch_assignment_detail(
        &self,
        config: &RuntimeConfig,
        course_id: i64,
        assignment_id: i64,
    ) -> Result<Option<AssignmentDetail>, MoodleError> {
        let token = self.request_token(config).await?;
        assignments::fetch_assignment_detail(self.http()?, config, &token, course_id, assignment_id)
            .await
    }

    pub async fn fetch_assignment_submission_status(
        &self,
        config: &RuntimeConfig,
        assign_id: i64,
    ) -> Result<Option<AssignmentSubmissionStatus>, MoodleError> {
        let token = self.request_token(config).await?;
        assignments::fetch_assignment_submission_status(self.http()?, config, &token, assign_id)
            .await
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
        assignments::fetch_upcoming_assignments(self.http()?, config, &token, now).await
    }

    pub async fn fetch_course_quizzes(
        &self,
        config: &RuntimeConfig,
        course_id: i64,
    ) -> Result<Vec<QuizSummary>, MoodleError> {
        let token = self.request_token(config).await?;
        quizzes::fetch_course_quizzes(self.http()?, config, &token, course_id).await
    }

    pub async fn start_quiz_attempt(
        &self,
        config: &RuntimeConfig,
        quiz_id: i64,
    ) -> Result<QuizAttempt, MoodleError> {
        let token = self.request_token(config).await?;
        quizzes::start_or_resume_attempt(self.http()?, config, &token, quiz_id).await
    }

    pub async fn fetch_quiz_attempt_data(
        &self,
        config: &RuntimeConfig,
        attempt_id: i64,
    ) -> Result<QuizAttemptData, MoodleError> {
        let token = self.request_token(config).await?;
        quizzes::fetch_attempt_data(self.http()?, config, &token, attempt_id).await
    }

    pub async fn save_quiz_attempt(
        &self,
        config: &RuntimeConfig,
        attempt: &QuizAttemptData,
    ) -> Result<QuizAttemptData, MoodleError> {
        let token = self.request_token(config).await?;
        quizzes::save_attempt(self.http()?, config, &token, attempt).await
    }

    pub async fn finish_quiz_attempt(
        &self,
        config: &RuntimeConfig,
        attempt: &QuizAttemptData,
    ) -> Result<(), MoodleError> {
        let token = self.request_token(config).await?;
        quizzes::finish_attempt(self.http()?, config, &token, attempt).await
    }
}

#[cfg(test)]
mod tests {
    use super::ExpectedOrigin;

    #[test]
    fn origin_accepts_same_host_and_effective_port() {
        let base = reqwest::Url::parse("https://moodle.example.edu/moodle/").unwrap();
        let origin = ExpectedOrigin::from_url(&base).unwrap();
        let endpoint =
            reqwest::Url::parse("https://moodle.example.edu:443/login/token.php").unwrap();
        assert!(origin.matches(&endpoint));
    }

    #[test]
    fn origin_rejects_cross_host_redirect() {
        let base = reqwest::Url::parse("https://moodle.example.edu/moodle/").unwrap();
        let origin = ExpectedOrigin::from_url(&base).unwrap();
        let redirect = reqwest::Url::parse("https://login.example.edu/").unwrap();
        assert!(!origin.matches(&redirect));
    }

    #[test]
    fn origin_rejects_different_port() {
        let base = reqwest::Url::parse("https://moodle.example.edu:8443/").unwrap();
        let origin = ExpectedOrigin::from_url(&base).unwrap();
        let redirect = reqwest::Url::parse("https://moodle.example.edu/").unwrap();
        assert!(!origin.matches(&redirect));
    }
}
