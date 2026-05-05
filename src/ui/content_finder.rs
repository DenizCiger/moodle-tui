use crate::ui::course_tree::{CourseTreeNodeKind, CourseTreeRow};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetMode {
    All,
    ModuleType(String),
    RowKind(CourseTreeNodeKind),
}

#[derive(Debug, Clone)]
pub struct ContentTarget {
    pub id: String,
    pub label: String,
    pub mode: TargetMode,
}

const MODULE_TYPE_ORDER: &[&str] = &[
    "assign", "quiz", "forum", "resource", "page", "book", "folder", "url",
];

fn module_type_label(kind: &str) -> String {
    match kind {
        "assign" => "Assignments".into(),
        "quiz" => "Quizzes".into(),
        "forum" => "Forums".into(),
        "resource" => "Resources".into(),
        "page" => "Pages".into(),
        "book" => "Books".into(),
        "folder" => "Folders".into(),
        "url" => "Link Activities".into(),
        other => {
            let normalized = other.replace(['_', '-'], " ");
            let normalized = normalized.trim();
            if normalized.is_empty() {
                "Other Activities".into()
            } else {
                let mut chars = normalized.chars();
                let head = chars.next().map(|c| c.to_uppercase().to_string()).unwrap_or_default();
                format!("{head}{} Activities", chars.as_str())
            }
        }
    }
}

pub fn build_targets(rows: &[CourseTreeRow]) -> Vec<ContentTarget> {
    let mut module_types: BTreeSet<String> = BTreeSet::new();
    for row in rows {
        if !matches!(row.kind, CourseTreeNodeKind::Module) {
            continue;
        }
        if let Some(t) = row.module_type.as_deref() {
            let trimmed = t.trim().to_lowercase();
            if !trimmed.is_empty() {
                module_types.insert(trimmed);
            }
        }
    }

    let mut sorted: Vec<String> = module_types.into_iter().collect();
    sorted.sort_by(|a, b| {
        let ai = MODULE_TYPE_ORDER.iter().position(|m| *m == a).unwrap_or(usize::MAX);
        let bi = MODULE_TYPE_ORDER.iter().position(|m| *m == b).unwrap_or(usize::MAX);
        ai.cmp(&bi).then_with(|| a.cmp(b))
    });

    let mut targets: Vec<ContentTarget> = vec![ContentTarget {
        id: "all".into(),
        label: "All".into(),
        mode: TargetMode::All,
    }];
    for kind in sorted {
        targets.push(ContentTarget {
            id: format!("module-type:{kind}"),
            label: module_type_label(&kind),
            mode: TargetMode::ModuleType(kind),
        });
    }
    for (id, label, kind) in [
        ("kind:section", "Sections", CourseTreeNodeKind::Section),
        ("kind:label", "Labels", CourseTreeNodeKind::Label),
        ("kind:content-item", "Files & Items", CourseTreeNodeKind::ContentItem),
        ("kind:module-url", "URLs", CourseTreeNodeKind::ModuleUrl),
        ("kind:module-description", "Descriptions", CourseTreeNodeKind::ModuleDescription),
        ("kind:summary", "Summaries", CourseTreeNodeKind::Summary),
    ] {
        targets.push(ContentTarget {
            id: id.into(),
            label: label.into(),
            mode: TargetMode::RowKind(kind),
        });
    }
    targets
}

pub fn filter_by_target<'a>(
    rows: &'a [CourseTreeRow],
    target: &ContentTarget,
) -> Vec<&'a CourseTreeRow> {
    rows.iter()
        .filter(|row| match &target.mode {
            TargetMode::All => true,
            TargetMode::ModuleType(t) => {
                matches!(row.kind, CourseTreeNodeKind::Module)
                    && row.module_type.as_deref().map(|s| s.trim().to_lowercase()).as_deref()
                        == Some(t.as_str())
            }
            TargetMode::RowKind(kind) => row.kind == *kind,
        })
        .collect()
}

pub fn cycle(current: usize, delta: isize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let l = len as isize;
    (((current as isize + delta) % l + l) % l) as usize
}
