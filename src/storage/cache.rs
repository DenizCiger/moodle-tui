use crate::models::{Course, CourseSection, UpcomingAssignment};
use crate::storage::{StorageError, config_dir};
use serde_json::{Value, json};
use std::path::PathBuf;
use tui_components::storage::json::{named_file, read_json_or_default, write_json_pretty};
use tui_components::storage::time::{is_expired as timestamp_is_expired, now_ms};

const CACHE_TTL_MS: u64 = 1000 * 60 * 60 * 24 * 21;
const MAX_CACHED_COURSE_PAGES: usize = 48;

#[derive(Debug, Clone, Default)]
pub struct DashboardCacheData {
    pub courses: Vec<Course>,
    pub upcoming_assignments: Vec<UpcomingAssignment>,
}

pub fn cache_file() -> Result<PathBuf, StorageError> {
    Ok(named_file(config_dir()?, "cache.json"))
}

fn is_expired(timestamp: u64) -> bool {
    timestamp_is_expired(timestamp, CACHE_TTL_MS)
}

fn read_cache_value() -> Value {
    cache_file()
        .map(read_json_or_default)
        .unwrap_or_else(|_| Value::Object(Default::default()))
}

fn write_cache_value(value: &Value) -> Result<(), StorageError> {
    let pruned = prune_course_pages(value.clone());
    write_json_pretty(cache_file()?, &pruned)
}

fn prune_course_pages(mut value: Value) -> Value {
    let pages = value.get("coursePages").cloned();
    if let Some(Value::Object(map)) = pages {
        let mut entries: Vec<(String, Value)> = map
            .into_iter()
            .filter(|(_, entry)| {
                entry
                    .get("timestamp")
                    .and_then(Value::as_u64)
                    .map(|ts| !is_expired(ts))
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by(|left, right| {
            right
                .1
                .get("timestamp")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .cmp(&left.1.get("timestamp").and_then(Value::as_u64).unwrap_or(0))
        });
        entries.truncate(MAX_CACHED_COURSE_PAGES);
        let map: serde_json::Map<String, Value> = entries.into_iter().collect();
        value["coursePages"] = Value::Object(map);
    }
    value
}

fn parse_courses(value: &Value) -> Vec<Course> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|entry| serde_json::from_value::<Course>(entry.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn parse_upcoming(value: &Value) -> Vec<UpcomingAssignment> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|entry| serde_json::from_value::<UpcomingAssignment>(entry.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn parse_sections(value: &Value) -> Vec<CourseSection> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|entry| serde_json::from_value::<CourseSection>(entry.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_cached_dashboard() -> Option<DashboardCacheData> {
    let cache = read_cache_value();
    if let Some(dashboard) = cache.get("dashboard") {
        let timestamp = dashboard.get("timestamp").and_then(Value::as_u64)?;
        if is_expired(timestamp) {
            return None;
        }
        let data = dashboard.get("data")?;
        return Some(DashboardCacheData {
            courses: parse_courses(data.get("courses").unwrap_or(&Value::Null)),
            upcoming_assignments: parse_upcoming(
                data.get("upcomingAssignments").unwrap_or(&Value::Null),
            ),
        });
    }

    let legacy_timestamp = cache.get("timestamp").and_then(Value::as_u64);
    if let Some(ts) = legacy_timestamp {
        if is_expired(ts) {
            return None;
        }
    }
    let courses = parse_courses(cache.get("courses").unwrap_or(&Value::Null));
    if courses.is_empty() && !cache.get("courses").map(Value::is_array).unwrap_or(false) {
        return None;
    }
    Some(DashboardCacheData {
        courses,
        upcoming_assignments: Vec::new(),
    })
}

pub fn save_dashboard_to_cache(
    courses: &[Course],
    upcoming: &[UpcomingAssignment],
) -> Result<(), StorageError> {
    let mut cache = read_cache_value();
    let now = now_ms();
    cache["timestamp"] = json!(now);
    cache["courses"] = serde_json::to_value(courses)?;
    cache["dashboard"] = json!({
        "timestamp": now,
        "data": {
            "courses": courses,
            "upcomingAssignments": upcoming,
        }
    });
    write_cache_value(&cache)
}

pub fn save_courses_to_cache(courses: &[Course]) -> Result<(), StorageError> {
    let cached = get_cached_dashboard();
    let upcoming = cached.map(|d| d.upcoming_assignments).unwrap_or_default();
    save_dashboard_to_cache(courses, &upcoming)
}

pub fn get_cached_course_sections(course_id: i64) -> Option<Vec<CourseSection>> {
    let mut cache = read_cache_value();
    let key = course_id.to_string();
    let entry = cache.get("coursePages")?.get(&key)?.clone();
    let timestamp = entry.get("timestamp").and_then(Value::as_u64)?;
    if is_expired(timestamp) {
        if let Some(Value::Object(pages)) = cache.get_mut("coursePages") {
            pages.remove(&key);
        }
        let _ = write_cache_value(&cache);
        return None;
    }
    let data = entry.get("data")?;
    let sections = parse_sections(data);
    if sections.is_empty() && !data.is_array() {
        return None;
    }
    Some(sections)
}

pub fn save_course_sections_to_cache(
    course_id: i64,
    sections: &[CourseSection],
) -> Result<(), StorageError> {
    let mut cache = read_cache_value();
    let key = course_id.to_string();
    let pages = cache.get_mut("coursePages");
    let pages_obj = match pages {
        Some(Value::Object(map)) => map,
        _ => {
            cache["coursePages"] = Value::Object(Default::default());
            cache.get_mut("coursePages").unwrap().as_object_mut().unwrap()
        }
    };
    pages_obj.insert(
        key,
        json!({
            "timestamp": now_ms(),
            "data": sections,
        }),
    );
    write_cache_value(&cache)
}

pub fn clear_cache() -> Result<(), StorageError> {
    let now = now_ms();
    let payload = json!({
        "timestamp": now,
        "courses": [],
        "dashboard": {
            "timestamp": now,
            "data": { "courses": [], "upcomingAssignments": [] }
        },
        "coursePages": {},
    });
    write_json_pretty(cache_file()?, &payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_dir<F: FnOnce()>(test: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let tempdir = tempfile::tempdir().unwrap();
        // SAFETY: tests serialised by ENV_LOCK
        unsafe { std::env::set_var(crate::storage::CONFIG_DIR_ENV, tempdir.path()); }
        test();
        unsafe { std::env::remove_var(crate::storage::CONFIG_DIR_ENV); }
    }

    #[test]
    fn round_trip_dashboard() {
        with_temp_dir(|| {
            let courses = vec![Course {
                id: 1,
                shortname: "MAT".into(),
                fullname: "Mathematics".into(),
                displayname: None,
                categoryid: None,
                categoryname: None,
                summary: None,
                visible: Some(1),
                progress: None,
                courseurl: None,
            }];
            let upcoming = vec![UpcomingAssignment {
                id: 9,
                name: "Hw1".into(),
                due_date: 1_700_000_000,
                course_id: 1,
                course_short_name: Some("MAT".into()),
                course_full_name: Some("Mathematics".into()),
            }];
            save_dashboard_to_cache(&courses, &upcoming).unwrap();
            let loaded = get_cached_dashboard().unwrap();
            assert_eq!(loaded.courses, courses);
            assert_eq!(loaded.upcoming_assignments, upcoming);
        });
    }

    #[test]
    fn course_pages_round_trip() {
        with_temp_dir(|| {
            let sections = vec![CourseSection {
                id: 10,
                name: Some("Week 1".into()),
                section: Some(1),
                summary: None,
                visible: Some(1),
                modules: vec![],
            }];
            save_course_sections_to_cache(42, &sections).unwrap();
            let loaded = get_cached_course_sections(42).unwrap();
            assert_eq!(loaded, sections);
        });
    }
}
