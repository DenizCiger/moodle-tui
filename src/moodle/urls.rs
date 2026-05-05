use crate::models::normalize_base_url;

pub fn build_assignment_activity_url(base_url: &str, cmid: i64) -> String {
    format!("{}/mod/assign/view.php?id={cmid}", normalize_base_url(base_url))
}

pub fn build_course_view_url(base_url: &str, course_id: i64) -> String {
    format!("{}/course/view.php?id={course_id}", normalize_base_url(base_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_assignment_url() {
        assert_eq!(
            build_assignment_activity_url("https://x.example/", 42),
            "https://x.example/mod/assign/view.php?id=42"
        );
    }
}
