use crate::app::state::AppState;
use crate::app::state::types::{CourseView, MainState};
use crate::ui::shared::centered_rect;
use crate::ui::{assignment_modal, course_tree, dashboard, settings, theme};
use chrono::TimeZone;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

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

    let outer = frame.area();
    let area = centered_rect(
        (outer.width as f32 * 0.7) as u16,
        (outer.height as f32 * 0.6) as u16,
        outer,
    );
    frame.render_widget(Clear, area);
    let title = if main.course_finder_open {
        " Course finder "
    } else {
        " Content finder "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint = if main.course_finder_open {
        "↑/↓ select · Enter open · Esc cancel"
    } else {
        "↑/↓ select · ←/→ change target · Enter jump · Esc cancel"
    };
    let mut lines: Vec<Line> = vec![
        Line::from(format!("> {}", main.finder_query.value)),
        Line::from(Span::styled(hint, Style::default().fg(theme::NEUTRAL_GRAY))),
        Line::from(""),
    ];

    if main.course_finder_open {
        let filtered = filter_courses(&main.dashboard.courses, &main.finder_query.value);
        let row_capacity = inner.height.saturating_sub(lines.len() as u16) as usize;
        let selected = main
            .finder_selected
            .min(filtered.len().saturating_sub(1));
        let visible_start = selected.saturating_sub(row_capacity.saturating_sub(1));
        if filtered.is_empty() {
            lines.push(Line::from(Span::styled(
                "No matches",
                Style::default().fg(theme::NEUTRAL_GRAY),
            )));
        } else {
            for (local, course) in filtered.iter().skip(visible_start).take(row_capacity).enumerate() {
                let idx = visible_start + local;
                let is_selected = idx == selected;
                lines.push(Line::from(vec![
                    Span::styled(
                        if is_selected { "> " } else { "  " },
                        Style::default().fg(if is_selected { theme::BRAND } else { theme::NEUTRAL_GRAY }),
                    ),
                    Span::styled(
                        format!("{:<10}", course.shortname),
                        Style::default().fg(theme::BRAND).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::raw(course.fullname.clone()),
                ]));
            }
        }
    } else if let CourseView::Course(course) = &main.view {
        let tree_rows = course_tree::build_course_tree_rows(&course.sections, &course.collapsed);
        let targets = crate::ui::content_finder::build_targets(&tree_rows);
        let target_idx = main.finder_target_idx.min(targets.len() - 1);
        let target = &targets[target_idx];
        let max_w = inner.width as usize;
        let mut row_spans: Vec<Span> = Vec::new();
        let mut row_w = 0usize;
        let mut ribbon_lines: Vec<Line> = Vec::new();
        for (i, t) in targets.iter().enumerate() {
            let active = i == target_idx;
            let style = if active {
                Style::default().fg(theme::NEUTRAL_BLACK).bg(theme::BRAND).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::NEUTRAL_GRAY)
            };
            let chunk = format!(" {} ", t.label);
            let w = chunk.chars().count() + 1;
            if row_w + w > max_w && !row_spans.is_empty() {
                ribbon_lines.push(Line::from(std::mem::take(&mut row_spans)));
                row_w = 0;
            }
            row_spans.push(Span::styled(chunk, style));
            row_spans.push(Span::raw(" "));
            row_w += w;
        }
        if !row_spans.is_empty() {
            ribbon_lines.push(Line::from(row_spans));
        }
        for (offset, line) in ribbon_lines.into_iter().enumerate() {
            lines.insert(1 + offset, line);
        }
        let candidate_rows = crate::ui::content_finder::filter_by_target(&tree_rows, target);
        let query = main.finder_query.value.to_lowercase();
        let filtered: Vec<&course_tree::CourseTreeRow> = if query.trim().is_empty() {
            candidate_rows
        } else {
            candidate_rows
                .into_iter()
                .filter(|r| r.text.to_lowercase().contains(&query))
                .collect()
        };
        let row_capacity = inner.height.saturating_sub(lines.len() as u16) as usize;
        let selected = main
            .finder_selected
            .min(filtered.len().saturating_sub(1));
        let visible_start = selected.saturating_sub(row_capacity.saturating_sub(1));
        if filtered.is_empty() {
            lines.push(Line::from(Span::styled(
                "No matches",
                Style::default().fg(theme::NEUTRAL_GRAY),
            )));
        } else {
            for (local, row) in filtered.iter().skip(visible_start).take(row_capacity).enumerate() {
                let idx = visible_start + local;
                let is_selected = idx == selected;
                lines.push(Line::from(vec![
                    Span::styled(
                        if is_selected { "> " } else { "  " },
                        Style::default().fg(if is_selected { theme::BRAND } else { theme::NEUTRAL_GRAY }),
                    ),
                    Span::raw(format!("{} ", row.icon)),
                    Span::raw(row.text.clone()),
                ]));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "Content finder requires open course",
            Style::default().fg(theme::NEUTRAL_GRAY),
        )));
    }

    frame.render_widget(Paragraph::new(lines), inner);
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
