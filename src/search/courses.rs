use crate::models::Course;
use crate::search::fuzzy::{FuzzyMatch, fuzzy_match};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseField {
    Shortname,
    Fullname,
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct CourseHighlight {
    pub field: Option<CourseField>,
    pub indices: Vec<usize>,
}

struct WeightedField<'a> {
    value: &'a str,
    weight: f64,
    field: CourseField,
}

fn rank(course: &Course, query: &str) -> Option<(f64, CourseHighlight)> {
    let fields = [
        WeightedField { value: &course.shortname, weight: 1.2, field: CourseField::Shortname },
        WeightedField { value: &course.fullname, weight: 1.0, field: CourseField::Fullname },
        WeightedField {
            value: course.displayname.as_deref().unwrap_or(""),
            weight: 0.95,
            field: CourseField::Other,
        },
        WeightedField {
            value: course.categoryname.as_deref().unwrap_or(""),
            weight: 0.7,
            field: CourseField::Other,
        },
        WeightedField {
            value: course.summary.as_deref().unwrap_or(""),
            weight: 0.35,
            field: CourseField::Other,
        },
    ];
    let mut best: Option<(f64, CourseField, FuzzyMatch)> = None;
    for field in fields {
        if let Some(m) = fuzzy_match(query, field.value) {
            let weighted = m.score * field.weight;
            if best.as_ref().map_or(true, |(s, _, _)| weighted > *s) {
                best = Some((weighted, field.field, m));
            }
        }
    }
    best.map(|(score, field, m)| {
        (
            score,
            CourseHighlight {
                field: Some(field),
                indices: m.indices,
            },
        )
    })
}

pub fn filter_courses<'a>(
    courses: &'a [Course],
    query_raw: &str,
) -> Vec<(&'a Course, CourseHighlight)> {
    let query = query_raw.trim();
    if query.is_empty() {
        return courses
            .iter()
            .map(|c| (c, CourseHighlight::default()))
            .collect();
    }
    let mut ranked: Vec<(&Course, f64, CourseHighlight)> = courses
        .iter()
        .filter_map(|course| rank(course, query).map(|(score, hi)| (course, score, hi)))
        .collect();
    ranked.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.fullname.to_lowercase().cmp(&right.0.fullname.to_lowercase()))
    });
    ranked.into_iter().map(|(course, _, hi)| (course, hi)).collect()
}
