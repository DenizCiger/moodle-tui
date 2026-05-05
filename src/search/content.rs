use crate::search::fuzzy::fuzzy_score;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourseTreeNodeKind {
    Section,
    Module,
    Label,
    ContentItem,
    ModuleDescription,
    ModuleUrl,
    Summary,
}

impl CourseTreeNodeKind {
    pub fn weight(self) -> f64 {
        match self {
            CourseTreeNodeKind::Section => 1.15,
            CourseTreeNodeKind::Module => 1.10,
            CourseTreeNodeKind::Label => 1.05,
            CourseTreeNodeKind::ContentItem => 1.00,
            CourseTreeNodeKind::ModuleDescription => 0.85,
            CourseTreeNodeKind::ModuleUrl => 0.80,
            CourseTreeNodeKind::Summary => 0.75,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CourseTreeRow {
    pub id: String,
    pub kind: CourseTreeNodeKind,
    pub text: String,
}

pub fn filter_course_content<'a>(
    rows: &'a [CourseTreeRow],
    query_raw: &str,
) -> Vec<&'a CourseTreeRow> {
    let searchable: Vec<&CourseTreeRow> = rows.iter().filter(|row| row.id != "empty").collect();
    let query = query_raw.trim();
    if query.is_empty() {
        return searchable;
    }
    let mut ranked: Vec<(&CourseTreeRow, f64)> = searchable
        .into_iter()
        .filter_map(|row| {
            fuzzy_score(query, &row.text).map(|score| (row, score * row.kind.weight()))
        })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.text.to_lowercase().cmp(&right.0.text.to_lowercase()))
            .then_with(|| left.0.id.cmp(&right.0.id))
    });
    ranked.into_iter().map(|(row, _)| row).collect()
}
