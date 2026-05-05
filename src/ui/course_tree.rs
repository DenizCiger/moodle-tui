use crate::models::{CourseSection, ModuleContentItem};
use crate::moodle::html::strip_html;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CourseTreeNodeKind {
    Section,
    Module,
    Summary,
    ModuleDescription,
    ModuleUrl,
    ContentItem,
    Label,
}

#[derive(Debug, Clone)]
pub struct CourseTreeRow {
    pub id: String,
    pub kind: CourseTreeNodeKind,
    pub depth: u8,
    pub text: String,
    pub link_url: Option<String>,
    pub module_type: Option<String>,
    pub icon: &'static str,
    pub collapsible: bool,
    pub expanded: bool,
    pub parent_id: Option<String>,
}

pub fn course_section_node_id(section_id: i64) -> String {
    format!("section:{section_id}")
}

fn course_module_node_id(section_id: i64, module_id: i64) -> String {
    format!("module:{section_id}:{module_id}")
}

fn normalize_type(value: Option<&str>) -> String {
    value.map(|v| v.trim().to_lowercase()).unwrap_or_default()
}

fn module_icon(modname: Option<&str>) -> &'static str {
    match normalize_type(modname).as_str() {
        "forum" => "💬",
        "quiz" => "📝",
        "resource" => "📄",
        "assign" => "✅",
        "url" => "🔗",
        "page" => "📃",
        "book" => "📚",
        "folder" => "📁",
        "label" => "🏷",
        _ => "📦",
    }
}

fn content_icon(kind: Option<&str>) -> &'static str {
    match normalize_type(kind).as_str() {
        "folder" => "📁",
        "url" => "🔗",
        _ => "📄",
    }
}

pub fn build_course_tree_rows(
    sections: &[CourseSection],
    collapsed: &HashSet<String>,
) -> Vec<CourseTreeRow> {
    if sections.is_empty() {
        return vec![CourseTreeRow {
            id: "empty".into(),
            kind: CourseTreeNodeKind::Summary,
            depth: 0,
            text: "No visible course content returned by Moodle.".into(),
            link_url: None,
            module_type: None,
            icon: "•",
            collapsible: false,
            expanded: false,
            parent_id: None,
        }];
    }

    let mut rows = Vec::new();
    for (section_index, section) in sections.iter().enumerate() {
        let section_name = section
            .name
            .as_ref()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                let n = section.section.unwrap_or((section_index + 1) as i64);
                format!("Section {n}")
            });
        let section_id = course_section_node_id(section.id);
        let collapsed_section = collapsed.contains(&section_id);

        rows.push(CourseTreeRow {
            id: section_id.clone(),
            kind: CourseTreeNodeKind::Section,
            depth: 0,
            text: section_name,
            link_url: None,
            module_type: None,
            icon: "📁",
            collapsible: true,
            expanded: !collapsed_section,
            parent_id: None,
        });

        if collapsed_section {
            continue;
        }

        let summary = strip_html(section.summary.as_deref().unwrap_or(""));
        if !summary.is_empty() {
            rows.push(CourseTreeRow {
                id: format!("summary:{}", section.id),
                kind: CourseTreeNodeKind::Summary,
                depth: 1,
                text: summary,
                link_url: None,
                module_type: None,
                icon: "·",
                collapsible: false,
                expanded: false,
                parent_id: Some(section_id.clone()),
            });
        }

        if section.modules.is_empty() {
            rows.push(CourseTreeRow {
                id: format!("section-empty:{}", section.id),
                kind: CourseTreeNodeKind::Summary,
                depth: 1,
                text: "(No activities in this section)".into(),
                link_url: None,
                module_type: None,
                icon: "•",
                collapsible: false,
                expanded: false,
                parent_id: Some(section_id.clone()),
            });
            continue;
        }

        for module in &section.modules {
            let modname = normalize_type(module.modname.as_deref());
            if modname == "label" {
                let text = strip_html(module.description.as_deref().unwrap_or(""));
                let text = if text.is_empty() {
                    let fallback = strip_html(&module.name);
                    if fallback.is_empty() {
                        "(Empty label)".to_owned()
                    } else {
                        fallback
                    }
                } else {
                    text
                };
                rows.push(CourseTreeRow {
                    id: format!("label:{}:{}", section.id, module.id),
                    kind: CourseTreeNodeKind::Label,
                    depth: 1,
                    text,
                    link_url: None,
                    module_type: None,
                    icon: "🏷",
                    collapsible: false,
                    expanded: false,
                    parent_id: Some(section_id.clone()),
                });
                continue;
            }

            let module_id = course_module_node_id(section.id, module.id);
            let collapsed_module = collapsed.contains(&module_id);

            rows.push(CourseTreeRow {
                id: module_id.clone(),
                kind: CourseTreeNodeKind::Module,
                depth: 1,
                text: module.name.clone(),
                link_url: module.url.clone().filter(|s| !s.is_empty()),
                module_type: if modname.is_empty() { None } else { Some(modname.clone()) },
                icon: module_icon(module.modname.as_deref()),
                collapsible: true,
                expanded: !collapsed_module,
                parent_id: Some(section_id.clone()),
            });

            if collapsed_module {
                continue;
            }

            let description = strip_html(module.description.as_deref().unwrap_or(""));
            if !description.is_empty() {
                rows.push(CourseTreeRow {
                    id: format!("module-description:{}:{}", section.id, module.id),
                    kind: CourseTreeNodeKind::ModuleDescription,
                    depth: 2,
                    text: description,
                    link_url: None,
                    module_type: if modname.is_empty() { None } else { Some(modname.clone()) },
                    icon: "·",
                    collapsible: false,
                    expanded: false,
                    parent_id: Some(module_id.clone()),
                });
            }

            if let Some(url) = module.url.as_deref().filter(|s| !s.is_empty()) {
                rows.push(CourseTreeRow {
                    id: format!("module-url:{}:{}", section.id, module.id),
                    kind: CourseTreeNodeKind::ModuleUrl,
                    depth: 2,
                    text: url.to_owned(),
                    link_url: Some(url.to_owned()),
                    module_type: if modname.is_empty() { None } else { Some(modname.clone()) },
                    icon: "🔗",
                    collapsible: false,
                    expanded: false,
                    parent_id: Some(module_id.clone()),
                });
            }

            for (idx, content) in module.contents.iter().enumerate() {
                rows.push(content_row(section.id, module.id, idx, content, &modname, &module_id));
            }
        }
    }

    rows
}

fn content_row(
    section_id: i64,
    module_id: i64,
    index: usize,
    content: &ModuleContentItem,
    modname: &str,
    parent_id: &str,
) -> CourseTreeRow {
    let label = content
        .filename
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| content.kind.clone())
        .unwrap_or_else(|| "content item".to_owned());
    let url = content
        .fileurl
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| content.url.clone().filter(|s| !s.is_empty()));
    let text = match &url {
        Some(u) => format!("{label}: {u}"),
        None => label,
    };
    CourseTreeRow {
        id: format!("content:{section_id}:{module_id}:{index}"),
        kind: CourseTreeNodeKind::ContentItem,
        depth: 2,
        text,
        link_url: url,
        module_type: if modname.is_empty() { None } else { Some(modname.to_owned()) },
        icon: content_icon(content.kind.as_deref()),
        collapsible: false,
        expanded: false,
        parent_id: Some(parent_id.to_owned()),
    }
}

pub fn render_tree_prefix(row: &CourseTreeRow) -> String {
    let indent = "  ".repeat(row.depth as usize);
    let indicator = if row.collapsible {
        if row.expanded { "▾" } else { "▸" }
    } else {
        "·"
    };
    format!("{indent}{indicator}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CourseModule, CourseSection};

    #[test]
    fn empty_sections_emit_placeholder() {
        let rows = build_course_tree_rows(&[], &HashSet::new());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "empty");
    }

    #[test]
    fn collapsed_section_hides_children() {
        let section = CourseSection {
            id: 1,
            name: Some("Week 1".into()),
            section: Some(1),
            summary: None,
            visible: Some(1),
            modules: vec![CourseModule {
                id: 10,
                instance: None,
                name: "Forum".into(),
                modname: Some("forum".into()),
                description: None,
                url: Some("https://x".into()),
                visible: Some(1),
                contents: vec![],
            }],
        };
        let mut collapsed = HashSet::new();
        collapsed.insert("section:1".to_owned());
        let rows = build_course_tree_rows(&[section], &collapsed);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, CourseTreeNodeKind::Section);
        assert!(!rows[0].expanded);
    }
}
