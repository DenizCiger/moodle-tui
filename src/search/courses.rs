use crate::models::Course;
use crate::search::fuzzy::fuzzy_score;

struct WeightedField<'a> {
    value: &'a str,
    weight: f64,
}

fn rank(course: &Course, query: &str) -> Option<f64> {
    let fields = [
        WeightedField { value: &course.shortname, weight: 1.2 },
        WeightedField { value: &course.fullname, weight: 1.0 },
        WeightedField { value: course.displayname.as_deref().unwrap_or(""), weight: 0.95 },
        WeightedField { value: course.categoryname.as_deref().unwrap_or(""), weight: 0.7 },
        WeightedField { value: course.summary.as_deref().unwrap_or(""), weight: 0.35 },
    ];
    let mut best: Option<f64> = None;
    for field in fields {
        if let Some(score) = fuzzy_score(query, field.value) {
            let weighted = score * field.weight;
            best = Some(best.map_or(weighted, |b| b.max(weighted)));
        }
    }
    best
}

pub fn filter_courses<'a>(courses: &'a [Course], query_raw: &str) -> Vec<&'a Course> {
    let query = query_raw.trim();
    if query.is_empty() {
        return courses.iter().collect();
    }
    let mut ranked: Vec<(&Course, f64)> = courses
        .iter()
        .filter_map(|course| rank(course, query).map(|score| (course, score)))
        .collect();
    ranked.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.fullname.to_lowercase().cmp(&right.0.fullname.to_lowercase()))
    });
    ranked.into_iter().map(|(course, _)| course).collect()
}
