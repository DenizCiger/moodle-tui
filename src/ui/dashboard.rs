use crate::app::state::types::{CoursePageData, DashboardPane, MainState};
use crate::ui::course_tree::{
    CourseTreeNodeKind, build_course_tree_rows, render_tree_prefix,
};
use crate::ui::shell::format_timestamp;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

pub fn render_dashboard(frame: &mut Frame, area: Rect, main: &MainState) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_upcoming(frame, layout[0], main);
    render_courses(frame, layout[1], main);
}

fn render_upcoming(frame: &mut Frame, area: Rect, main: &MainState) {
    let mut subtitle = String::new();
    if main.dashboard.loading {
        subtitle.push_str(" (loading)");
    } else if main.dashboard.from_cache {
        subtitle.push_str(" (cached)");
    }

    let focused = main.dashboard_focus == DashboardPane::Upcoming;
    let border = if focused { theme::BRAND } else { theme::NEUTRAL_BRIGHT_BLACK };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Upcoming{subtitle} "))
        .border_style(Style::default().fg(border));

    if let Some(error) = &main.dashboard.error {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("Error: {error}"),
                Style::default().fg(theme::ERROR),
            ))),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = if main.dashboard.upcoming.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No upcoming assignments",
            Style::default().fg(theme::NEUTRAL_GRAY),
        )))]
    } else {
        main.dashboard
            .upcoming
            .iter()
            .enumerate()
            .map(|(idx, assignment)| {
                let selected = focused && idx == main.selected_row;
                let style = if selected {
                    Style::default().fg(theme::NEUTRAL_WHITE).bg(theme::PANEL_SELECTED)
                } else {
                    Style::default()
                };
                let course = assignment
                    .course_short_name
                    .as_deref()
                    .or(assignment.course_full_name.as_deref())
                    .unwrap_or("?");
                ListItem::new(Line::from(Span::styled(
                    format!(
                        "{}  {:<14}  {}",
                        format_timestamp(assignment.due_date),
                        course,
                        assignment.name
                    ),
                    style,
                )))
            })
            .collect()
    };

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_courses(frame: &mut Frame, area: Rect, main: &MainState) {
    let focused = main.dashboard_focus == DashboardPane::Courses;
    let border = if focused { theme::BRAND } else { theme::NEUTRAL_BRIGHT_BLACK };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Courses ({}) ", main.dashboard.courses.len()))
        .border_style(Style::default().fg(border));

    let items: Vec<ListItem> = main
        .dashboard
        .courses
        .iter()
        .enumerate()
        .map(|(idx, course)| {
            let selected = focused && idx == main.selected_row;
            let style = if selected {
                Style::default().fg(theme::NEUTRAL_WHITE).bg(theme::PANEL_SELECTED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<10}", course.shortname),
                    Style::default().fg(theme::BRAND).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(course.fullname.clone(), style),
            ]))
        })
        .collect();

    frame.render_widget(List::new(items).block(block), area);
}

pub fn render_course_page(frame: &mut Frame, area: Rect, course: &CoursePageData) {
    let title_name = if !course.course_full_name.is_empty() {
        course.course_full_name.clone()
    } else if !course.course_short_name.is_empty() {
        course.course_short_name.clone()
    } else {
        format!("Course #{}", course.course_id)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title_name} "))
        .border_style(Style::default().fg(theme::BRAND));

    if course.loading {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Loading sections…",
                Style::default().fg(theme::NEUTRAL_GRAY),
            ))),
            inner,
        );
        return;
    }

    if let Some(error) = &course.error {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("Error: {error}"),
                Style::default().fg(theme::ERROR),
            ))),
            inner,
        );
        return;
    }

    let rows = build_course_tree_rows(&course.sections, &course.collapsed);
    let total = rows.len();
    let inner = block.inner(area);
    let viewport = inner.height.saturating_sub(1) as usize;
    let selected = course.selected_row.min(total.saturating_sub(1));
    let scroll = selected.saturating_sub(viewport.saturating_sub(1));

    let mut items: Vec<ListItem> = Vec::new();
    for (idx, row) in rows.iter().skip(scroll).take(viewport).enumerate() {
        let absolute = scroll + idx;
        let is_selected = absolute == selected;
        let prefix = render_tree_prefix(row);
        let secondary = matches!(
            row.kind,
            CourseTreeNodeKind::Summary
                | CourseTreeNodeKind::ModuleDescription
                | CourseTreeNodeKind::ModuleUrl
        );
        let fg = if secondary { theme::NEUTRAL_GRAY } else { theme::NEUTRAL_WHITE };
        let mut style = Style::default().fg(fg);
        if is_selected {
            style = style.bg(theme::PANEL_SELECTED).fg(theme::NEUTRAL_WHITE).add_modifier(Modifier::BOLD);
        }
        let line = if matches!(row.kind, CourseTreeNodeKind::Label) {
            Line::from(vec![
                Span::styled(format!("{prefix} "), style),
                Span::styled(row.text.clone(), style.add_modifier(Modifier::UNDERLINED)),
            ])
        } else {
            Line::from(Span::styled(
                format!("{prefix} {} {}", row.icon, row.text),
                style,
            ))
        };
        items.push(ListItem::new(line));
    }

    let counter = if total > 0 {
        format!("{}/{}", selected + 1, total)
    } else {
        "0/0".into()
    };
    frame.render_widget(block, area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);
    frame.render_widget(List::new(items), layout[0]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            counter,
            Style::default().fg(theme::NEUTRAL_GRAY),
        ))),
        layout[1],
    );
}
