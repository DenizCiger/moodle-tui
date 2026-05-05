use crate::models::{
    Course, CourseModule, CourseSection, ModuleContentItem, RuntimeConfig, UpcomingAssignment,
};

pub fn demo_config() -> RuntimeConfig {
    RuntimeConfig {
        base_url: "https://demo.moodle.example".to_owned(),
        username: "demo".to_owned(),
        service: "moodle_mobile_app".to_owned(),
        password: "demo".to_owned(),
    }
}

pub fn demo_courses() -> Vec<Course> {
    vec![
        Course {
            id: 1,
            shortname: "MAT101".into(),
            fullname: "Mathematics 101".into(),
            displayname: Some("Mathematics 101".into()),
            categoryid: Some(1),
            categoryname: Some("Science".into()),
            summary: Some("Algebra and trigonometry".into()),
            visible: Some(1),
            progress: Some(42.0),
            courseurl: Some("https://demo.moodle.example/course/view.php?id=1".into()),
        },
        Course {
            id: 2,
            shortname: "PHY201".into(),
            fullname: "Physics 201".into(),
            displayname: Some("Physics 201".into()),
            categoryid: Some(1),
            categoryname: Some("Science".into()),
            summary: Some("Mechanics and waves".into()),
            visible: Some(1),
            progress: Some(78.0),
            courseurl: Some("https://demo.moodle.example/course/view.php?id=2".into()),
        },
        Course {
            id: 3,
            shortname: "ENG110".into(),
            fullname: "English Literature".into(),
            displayname: None,
            categoryid: Some(2),
            categoryname: Some("Humanities".into()),
            summary: None,
            visible: Some(1),
            progress: None,
            courseurl: Some("https://demo.moodle.example/course/view.php?id=3".into()),
        },
    ]
}

pub fn demo_course_sections(course_id: i64) -> Vec<CourseSection> {
    vec![
        CourseSection {
            id: course_id * 100 + 1,
            name: Some("Week 1: Intro".into()),
            section: Some(1),
            summary: Some("<p>Welcome to the course!</p>".into()),
            visible: Some(1),
            modules: vec![
                CourseModule {
                    id: course_id * 1000 + 1,
                    instance: Some(1),
                    name: "Syllabus".into(),
                    modname: Some("resource".into()),
                    description: Some("Course syllabus PDF".into()),
                    url: Some(format!("https://demo.moodle.example/mod/resource/view.php?id={course_id}")),
                    visible: Some(1),
                    contents: vec![ModuleContentItem {
                        kind: Some("file".into()),
                        filename: Some("syllabus.pdf".into()),
                        fileurl: Some("https://demo.moodle.example/files/syllabus.pdf".into()),
                        ..Default::default()
                    }],
                },
                CourseModule {
                    id: course_id * 1000 + 2,
                    instance: Some(2),
                    name: "Problem set 1".into(),
                    modname: Some("assign".into()),
                    description: Some("First problem set covering chapters 1-2.".into()),
                    url: Some(format!("https://demo.moodle.example/mod/assign/view.php?id={course_id}")),
                    visible: Some(1),
                    contents: vec![],
                },
            ],
        },
        CourseSection {
            id: course_id * 100 + 2,
            name: Some("Week 2: Foundations".into()),
            section: Some(2),
            summary: None,
            visible: Some(1),
            modules: vec![CourseModule {
                id: course_id * 1000 + 3,
                instance: Some(3),
                name: "Discussion forum".into(),
                modname: Some("forum".into()),
                description: None,
                url: Some(format!("https://demo.moodle.example/mod/forum/view.php?id={course_id}")),
                visible: Some(1),
                contents: vec![],
            }],
        },
    ]
}

pub fn demo_upcoming() -> Vec<UpcomingAssignment> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    vec![
        UpcomingAssignment {
            id: 11,
            name: "Problem set 3".into(),
            due_date: now + 86_400 * 2,
            course_id: 1,
            course_short_name: Some("MAT101".into()),
            course_full_name: Some("Mathematics 101".into()),
        },
        UpcomingAssignment {
            id: 12,
            name: "Lab report".into(),
            due_date: now + 86_400 * 5,
            course_id: 2,
            course_short_name: Some("PHY201".into()),
            course_full_name: Some("Physics 201".into()),
        },
        UpcomingAssignment {
            id: 13,
            name: "Essay draft".into(),
            due_date: now + 86_400 * 9,
            course_id: 3,
            course_short_name: Some("ENG110".into()),
            course_full_name: Some("English Literature".into()),
        },
    ]
}
