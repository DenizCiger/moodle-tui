use crate::models::{
    AssignmentDetail, AssignmentSubmissionStatus, Course, CourseModule, CourseSection,
    ModuleContentItem, UpcomingAssignment,
};
use crate::moodle::html::decode_html_entities;
use serde_json::Value;

pub fn as_str(value: &Value) -> Option<String> {
    value.as_str().map(|s| s.to_owned())
}

pub fn as_decoded(value: &Value) -> Option<String> {
    value.as_str().map(decode_html_entities)
}

pub fn as_i64(value: &Value) -> Option<i64> {
    if let Some(n) = value.as_i64() {
        return Some(n);
    }
    if let Some(f) = value.as_f64() {
        if f.is_finite() {
            return Some(f as i64);
        }
    }
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            if let Ok(n) = trimmed.parse::<i64>() {
                return Some(n);
            }
            if let Ok(f) = trimmed.parse::<f64>() {
                if f.is_finite() {
                    return Some(f as i64);
                }
            }
        }
    }
    None
}

pub fn as_f64(value: &Value) -> Option<f64> {
    if let Some(f) = value.as_f64() {
        if f.is_finite() {
            return Some(f);
        }
    }
    if let Some(s) = value.as_str() {
        if let Ok(f) = s.trim().parse::<f64>() {
            if f.is_finite() {
                return Some(f);
            }
        }
    }
    None
}

pub fn as_bool(value: &Value) -> Option<bool> {
    if let Some(b) = value.as_bool() {
        return Some(b);
    }
    if let Some(n) = value.as_i64() {
        return Some(n != 0);
    }
    if let Some(s) = value.as_str() {
        return match s.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        };
    }
    None
}

pub fn visible_number(value: &Value) -> Option<i64> {
    if let Some(b) = value.as_bool() {
        return Some(if b { 1 } else { 0 });
    }
    as_i64(value)
}

pub fn progress_value(value: &Value) -> Option<f64> {
    if value.is_null() {
        return None;
    }
    as_f64(value)
}

pub fn normalize_token_response(value: &Value) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let object = match value.as_object() {
        Some(map) => map,
        None => return (None, None, None, None),
    };
    (
        object.get("token").and_then(|v| as_str(v)),
        object.get("error").and_then(|v| as_str(v)),
        object.get("errorcode").and_then(|v| as_str(v)),
        object.get("debuginfo").and_then(|v| as_str(v)),
    )
}

pub fn normalize_course(value: &Value) -> Option<Course> {
    let object = value.as_object()?;
    let id = as_i64(object.get("id")?)?;
    let shortname = object.get("shortname").and_then(as_str)?;
    let fullname = object.get("fullname").and_then(as_str)?;

    Some(Course {
        id,
        shortname: decode_html_entities(&shortname),
        fullname: decode_html_entities(&fullname),
        displayname: object.get("displayname").and_then(as_decoded),
        categoryid: object.get("categoryid").and_then(as_i64),
        categoryname: object.get("categoryname").and_then(as_decoded),
        summary: object.get("summary").and_then(as_decoded),
        visible: object.get("visible").and_then(visible_number),
        progress: object.get("progress").map(progress_value).unwrap_or(None),
        courseurl: object
            .get("courseurl")
            .and_then(as_str)
            .or_else(|| object.get("viewurl").and_then(as_str)),
    })
}

fn normalize_module_content(value: &Value) -> Option<ModuleContentItem> {
    let object = value.as_object()?;
    Some(ModuleContentItem {
        kind: object.get("type").and_then(as_decoded),
        filename: object.get("filename").and_then(as_decoded),
        filepath: object.get("filepath").and_then(as_str),
        filesize: object.get("filesize").and_then(as_i64),
        fileurl: object.get("fileurl").and_then(as_str),
        mimetype: object.get("mimetype").and_then(as_decoded),
        timemodified: object.get("timemodified").and_then(as_i64),
        url: object.get("url").and_then(as_str),
    })
}

fn normalize_module(value: &Value) -> Option<CourseModule> {
    let object = value.as_object()?;
    let id = object.get("id").and_then(as_i64)?;
    let name = object.get("name").and_then(as_decoded)?;
    let contents = object
        .get("contents")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(normalize_module_content).collect())
        .unwrap_or_default();

    Some(CourseModule {
        id,
        instance: object.get("instance").and_then(as_i64),
        name,
        modname: object.get("modname").and_then(as_decoded),
        description: object.get("description").and_then(as_decoded),
        url: object.get("url").and_then(as_str),
        visible: object.get("visible").and_then(visible_number),
        contents,
    })
}

pub fn normalize_section(value: &Value) -> Option<CourseSection> {
    let object = value.as_object()?;
    let id = object.get("id").and_then(as_i64)?;
    let modules = object
        .get("modules")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(normalize_module).collect())
        .unwrap_or_default();
    Some(CourseSection {
        id,
        name: object.get("name").and_then(as_decoded),
        section: object.get("section").and_then(as_i64),
        summary: object.get("summary").and_then(as_decoded),
        visible: object.get("visible").and_then(visible_number),
        modules,
    })
}

fn normalize_assignment_detail(value: &Value, fallback_course_id: Option<i64>) -> Option<AssignmentDetail> {
    let object = value.as_object()?;
    let id = object.get("id").and_then(as_i64)?;
    let cmid = object.get("cmid").and_then(as_i64)?;
    let course_id = object
        .get("course")
        .and_then(as_i64)
        .or(fallback_course_id)?;
    let name = object.get("name").and_then(as_decoded)?;

    Some(AssignmentDetail {
        id,
        cmid,
        course_id,
        name,
        intro: object.get("intro").and_then(as_decoded),
        intro_format: object.get("introformat").and_then(as_i64),
        always_show_description: object.get("alwaysshowdescription").and_then(as_bool),
        allowsubmissionsfromdate: object.get("allowsubmissionsfromdate").and_then(as_i64),
        duedate: object.get("duedate").and_then(as_i64),
        cutoffdate: object.get("cutoffdate").and_then(as_i64),
        gradingduedate: object.get("gradingduedate").and_then(as_i64),
        grade: object.get("grade").and_then(as_f64),
        teamsubmission: object.get("teamsubmission").and_then(as_bool),
        requireallteammemberssubmit: object.get("requireallteammemberssubmit").and_then(as_bool),
        maxattempts: object.get("maxattempts").and_then(as_i64),
        sendnotifications: object.get("sendnotifications").and_then(as_bool),
    })
}

pub fn normalize_course_assignments(value: &Value, course_id_filter: Option<i64>) -> Vec<AssignmentDetail> {
    let object = match value.as_object() {
        Some(map) => map,
        None => return Vec::new(),
    };
    let courses = match object.get("courses").and_then(Value::as_array) {
        Some(items) => items,
        None => return Vec::new(),
    };

    let mut details = Vec::new();
    for raw_course in courses {
        let course_object = match raw_course.as_object() {
            Some(map) => map,
            None => continue,
        };
        let course_id = match course_object.get("id").and_then(as_i64) {
            Some(id) => id,
            None => continue,
        };
        if let Some(filter) = course_id_filter {
            if course_id != filter {
                continue;
            }
        }
        let assignments = match course_object.get("assignments").and_then(Value::as_array) {
            Some(items) => items,
            None => continue,
        };
        for raw in assignments {
            if let Some(detail) = normalize_assignment_detail(raw, Some(course_id)) {
                details.push(detail);
            }
        }
    }
    details
}

pub fn normalize_submission_status(value: &Value) -> Option<AssignmentSubmissionStatus> {
    let object = value.as_object()?;
    let last_attempt = object.get("lastattempt").and_then(Value::as_object);
    let submission = last_attempt
        .and_then(|map| map.get("submission"))
        .and_then(Value::as_object);

    let submission_status = submission
        .and_then(|map| map.get("status"))
        .and_then(as_decoded)
        .or_else(|| object.get("submissionstatus").and_then(as_decoded));
    let grading_status = last_attempt
        .and_then(|map| map.get("gradingstatus"))
        .and_then(as_decoded)
        .or_else(|| object.get("gradingstatus").and_then(as_decoded));
    let can_submit = object.get("cansubmit").and_then(as_bool);
    let can_edit = object
        .get("caneditowner")
        .and_then(as_bool)
        .or_else(|| object.get("canedit").and_then(as_bool));
    let is_locked = last_attempt
        .and_then(|map| map.get("locked"))
        .and_then(as_bool)
        .or_else(|| object.get("locked").and_then(as_bool));
    let last_modified = submission
        .and_then(|map| map.get("timemodified"))
        .and_then(as_i64)
        .or_else(|| {
            last_attempt
                .and_then(|map| map.get("timemodified"))
                .and_then(as_i64)
        });

    if submission_status.is_none()
        && grading_status.is_none()
        && can_submit.is_none()
        && can_edit.is_none()
        && is_locked.is_none()
        && last_modified.is_none()
    {
        return None;
    }

    Some(AssignmentSubmissionStatus {
        submission_status,
        grading_status,
        can_submit,
        can_edit,
        is_locked,
        last_modified,
    })
}

pub fn normalize_upcoming_assignments(value: &Value, now_timestamp: i64) -> Vec<UpcomingAssignment> {
    let object = match value.as_object() {
        Some(map) => map,
        None => return Vec::new(),
    };
    let courses = match object.get("courses").and_then(Value::as_array) {
        Some(items) => items,
        None => return Vec::new(),
    };

    let mut upcoming = Vec::new();
    for raw_course in courses {
        let course_object = match raw_course.as_object() {
            Some(map) => map,
            None => continue,
        };
        let course_id = match course_object.get("id").and_then(as_i64) {
            Some(id) => id,
            None => continue,
        };
        let course_short_name = course_object.get("shortname").and_then(as_decoded);
        let course_full_name = course_object.get("fullname").and_then(as_decoded);
        let assignments = match course_object.get("assignments").and_then(Value::as_array) {
            Some(items) => items,
            None => continue,
        };
        for raw in assignments {
            let object = match raw.as_object() {
                Some(map) => map,
                None => continue,
            };
            let id = match object.get("id").and_then(as_i64) {
                Some(id) => id,
                None => continue,
            };
            let name = match object.get("name").and_then(as_decoded) {
                Some(name) if !name.is_empty() => name,
                _ => continue,
            };
            let due_date = match object.get("duedate").and_then(as_i64) {
                Some(due) if due > 0 => due,
                _ => continue,
            };
            if due_date < now_timestamp {
                continue;
            }
            upcoming.push(UpcomingAssignment {
                id,
                name,
                due_date,
                course_id,
                course_short_name: course_short_name.clone(),
                course_full_name: course_full_name.clone(),
            });
        }
    }

    upcoming.sort_by(|left, right| {
        left.due_date
            .cmp(&right.due_date)
            .then_with(|| {
                let lkey = left
                    .course_full_name
                    .as_deref()
                    .or(left.course_short_name.as_deref())
                    .unwrap_or("");
                let rkey = right
                    .course_full_name
                    .as_deref()
                    .or(right.course_short_name.as_deref())
                    .unwrap_or("");
                lkey.to_lowercase().cmp(&rkey.to_lowercase())
            })
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });

    upcoming
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn course_normalises_basic_fields() {
        let course = normalize_course(&json!({
            "id": 1,
            "shortname": "MAT &amp; PHY",
            "fullname": "Maths",
            "visible": 1,
            "progress": null,
        }))
        .unwrap();
        assert_eq!(course.shortname, "MAT & PHY");
        assert_eq!(course.fullname, "Maths");
        assert_eq!(course.visible, Some(1));
        assert_eq!(course.progress, None);
    }

    #[test]
    fn upcoming_filters_past_and_sorts() {
        let payload = json!({
            "courses": [{
                "id": 1, "shortname": "C1", "fullname": "Course 1",
                "assignments": [
                    {"id": 1, "name": "B", "duedate": 100},
                    {"id": 2, "name": "A", "duedate": 100},
                    {"id": 3, "name": "Past", "duedate": 5},
                ]
            }]
        });
        let result = normalize_upcoming_assignments(&payload, 50);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "A");
        assert_eq!(result[1].name, "B");
    }
}
