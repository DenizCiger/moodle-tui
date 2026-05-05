use crate::app::state::AppState;
use crate::app::state::types::{CourseView, MainState};
use crate::ui::{assignment_modal, course_tree, dashboard, settings, theme};
use chrono::TimeZone;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use tui_components::ui::search::{SearchModal, SearchModalCategory, SearchModalRow};

pub fn render(frame: &mut Frame, main: &MainState, state: &AppState) {
    let area = frame.area();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(frame, layout[0], main, state);

    match &main.view {
        CourseView::Dashboard => dashboard::render_dashboard(frame, layout[1], main),
        CourseView::Course(course) => dashboard::render_course_page(frame, layout[1], course),
    }

    render_footer(frame, layout[2], main);

    if main.settings_open {
        settings::render(frame, state);
    }

    if let Some(modal) = &main.assignment_modal {
        assignment_modal::render(frame, modal);
    }

    if main.course_finder_open || main.content_finder_open {
        render_finder(frame, main);
    }

    if let Some(toast) = &main.toast {
        render_toast(frame, toast);
    }
}

fn render_header(frame: &mut Frame, area: ratatui::layout::Rect, main: &MainState, state: &AppState) {
    let username = state
        .saved_config
        .as_ref()
        .map(|c| c.username.as_str())
        .unwrap_or("(unknown)");
    let title = match &main.view {
        CourseView::Dashboard => "Dashboard".to_owned(),
        CourseView::Course(course) => {
            if !course.course_short_name.is_empty() {
                course.course_short_name.clone()
            } else if !course.course_full_name.is_empty() {
                course.course_full_name.clone()
            } else {
                format!("Course #{}", course.course_id)
            }
        }
    };
    let line = Line::from(vec![
        Span::styled(" moodle-tui ", Style::default().fg(theme::NEUTRAL_BLACK).bg(theme::BRAND).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(title, Style::default().fg(theme::NEUTRAL_WHITE).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("[{username}]"), Style::default().fg(theme::NEUTRAL_GRAY)),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_footer(frame: &mut Frame, area: ratatui::layout::Rect, main: &MainState) {
    let mut hints = vec!["? help".to_owned(), "/ courses".to_owned(), "r refresh".to_owned()];
    if matches!(main.view, CourseView::Course(_)) {
        hints.push("Esc back".to_owned());
        hints.push("f content".to_owned());
    }
    hints.push("q quit".to_owned());
    let line = Line::from(Span::styled(
        format!(" {}", hints.join("  ·  ")),
        Style::default().fg(theme::NEUTRAL_GRAY),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

fn render_finder(frame: &mut Frame, main: &MainState) {
    use crate::search::courses::filter_courses;

    if main.course_finder_open {
        let filtered = filter_courses(&main.dashboard.courses, &main.finder.input.value);
        let rows: Vec<SearchModalRow> = filtered
            .iter()
            .map(|course| {
                SearchModalRow::new(vec![
                    Span::styled(
                        format!("{:<10}", course.shortname),
                        Style::default().fg(theme::BRAND).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::raw(course.fullname.clone()),
                ])
            })
            .collect();
        SearchModal {
            title: " Course finder ",
            hint: "↑/↓ select · Enter open · Esc cancel",
            state: &main.finder,
            rows,
            categories: None,
            empty_text: "No matches",
            theme: theme::components_theme(),
        }
        .render(frame);
        return;
    }

    if let CourseView::Course(course) = &main.view {
        let tree_rows = course_tree::build_course_tree_rows(&course.sections, &course.collapsed);
        let targets = crate::ui::content_finder::build_targets(&tree_rows);
        let target_idx = main.finder_target_idx.min(targets.len() - 1);
        let target = &targets[target_idx];
        let candidate_rows = crate::ui::content_finder::filter_by_target(&tree_rows, target);
        let query = main.finder.input.value.to_lowercase();
        let filtered: Vec<&course_tree::CourseTreeRow> = if query.trim().is_empty() {
            candidate_rows
        } else {
            candidate_rows
                .into_iter()
                .filter(|r| r.text.to_lowercase().contains(&query))
                .collect()
        };
        let categories: Vec<SearchModalCategory> = targets
            .iter()
            .enumerate()
            .map(|(i, t)| SearchModalCategory {
                label: t.label.as_str(),
                active: i == target_idx,
            })
            .collect();
        let rows: Vec<SearchModalRow> = filtered
            .iter()
            .map(|row| {
                SearchModalRow::new(vec![
                    Span::raw(format!("{} ", row.icon)),
                    Span::raw(row.text.clone()),
                ])
            })
            .collect();
        SearchModal {
            title: " Content finder ",
            hint: "↑/↓ select · ←/→ change target · Enter jump · Esc cancel",
            state: &main.finder,
            rows,
            categories: Some(categories),
            empty_text: "No matches",
            theme: theme::components_theme(),
        }
        .render(frame);
    } else {
        SearchModal {
            title: " Content finder ",
            hint: "↑/↓ select · ←/→ change target · Enter jump · Esc cancel",
            state: &main.finder,
            rows: Vec::new(),
            categories: None,
            empty_text: "Content finder requires open course",
            theme: theme::components_theme(),
        }
        .render(frame);
    }
}

fn render_toast(frame: &mut Frame, message: &str) {
    let area = frame.area();
    let toast_area = ratatui::layout::Rect {
        x: area.x + 2,
        y: area.bottom().saturating_sub(2),
        width: area.width.saturating_sub(4).min(message.len() as u16 + 4),
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(" {message} "),
            Style::default().fg(theme::NEUTRAL_BLACK).bg(theme::SUCCESS),
        ))),
        toast_area,
    );
}

pub fn format_timestamp(seconds: i64) -> String {
    chrono::Local
        .timestamp_opt(seconds, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "(unknown)".to_owned())
}
