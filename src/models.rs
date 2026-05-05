use serde::{Deserialize, Serialize};

pub const DEFAULT_MOODLE_SERVICE: &str = "moodle_mobile_app";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConfig {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub username: String,
    #[serde(default = "default_service")]
    pub service: String,
}

fn default_service() -> String {
    DEFAULT_MOODLE_SERVICE.to_owned()
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub base_url: String,
    pub username: String,
    pub service: String,
    pub password: String,
}

impl RuntimeConfig {
    pub fn saved(&self) -> SavedConfig {
        SavedConfig {
            base_url: normalize_base_url(&self.base_url),
            username: self.username.trim().to_owned(),
            service: pick_service(&self.service),
        }
    }
}

pub fn normalize_base_url(raw: &str) -> String {
    raw.trim().trim_end_matches('/').to_owned()
}

pub fn pick_service(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        DEFAULT_MOODLE_SERVICE.to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Course {
    pub id: i64,
    pub shortname: String,
    pub fullname: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub displayname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub categoryid: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub categoryname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub visible: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub progress: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub courseurl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ModuleContentItem {
    #[serde(skip_serializing_if = "Option::is_none", default, rename = "type")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub filepath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub filesize: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub fileurl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub mimetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub timemodified: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseModule {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub instance: Option<i64>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub modname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub visible: Option<i64>,
    #[serde(default)]
    pub contents: Vec<ModuleContentItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CourseSection {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub section: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub visible: Option<i64>,
    #[serde(default)]
    pub modules: Vec<CourseModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpcomingAssignment {
    pub id: i64,
    pub name: String,
    #[serde(rename = "dueDate")]
    pub due_date: i64,
    #[serde(rename = "courseId")]
    pub course_id: i64,
    #[serde(rename = "courseShortName", skip_serializing_if = "Option::is_none", default)]
    pub course_short_name: Option<String>,
    #[serde(rename = "courseFullName", skip_serializing_if = "Option::is_none", default)]
    pub course_full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssignmentDetail {
    pub id: i64,
    pub cmid: i64,
    #[serde(rename = "courseId")]
    pub course_id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub intro: Option<String>,
    #[serde(rename = "introFormat", skip_serializing_if = "Option::is_none", default)]
    pub intro_format: Option<i64>,
    #[serde(rename = "alwaysShowDescription", skip_serializing_if = "Option::is_none", default)]
    pub always_show_description: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allowsubmissionsfromdate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub duedate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cutoffdate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gradingduedate: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub grade: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub teamsubmission: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub requireallteammemberssubmit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub maxattempts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sendnotifications: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AssignmentSubmissionStatus {
    #[serde(rename = "submissionStatus", skip_serializing_if = "Option::is_none", default)]
    pub submission_status: Option<String>,
    #[serde(rename = "gradingStatus", skip_serializing_if = "Option::is_none", default)]
    pub grading_status: Option<String>,
    #[serde(rename = "canSubmit", skip_serializing_if = "Option::is_none", default)]
    pub can_submit: Option<bool>,
    #[serde(rename = "canEdit", skip_serializing_if = "Option::is_none", default)]
    pub can_edit: Option<bool>,
    #[serde(rename = "isLocked", skip_serializing_if = "Option::is_none", default)]
    pub is_locked: Option<bool>,
    #[serde(rename = "lastModified", skip_serializing_if = "Option::is_none", default)]
    pub last_modified: Option<i64>,
}
