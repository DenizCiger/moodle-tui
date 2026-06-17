use crate::app::state::types::{CoursePageData, DashboardPane, MainState};
use crate::models::{Course, UpcomingAssignment};
use crate::search::courses::filter_courses;
use crate::ui::course_tree::{CourseTreeNodeKind, build_course_tree_rows, render_tree_prefix};
use crate::ui::shell::format_timestamp;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

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
    let pane_bg = pane_background(focused);
    let text_fg = pane_text_color(focused);
    let indicator_fg = if focused {
        theme::NEUTRAL_WHITE
    } else {
        text_fg
    };
    fill_area(frame, area, pane_bg);
    render_pane_header(frame, area, &format!("Upcoming{subtitle}"), focused);
    let body = pane_body(area);

    if let Some(error) = &main.dashboard.error {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("Error: {error}"),
                Style::default().fg(theme::ERROR).bg(pane_bg),
            ))),
            body,
        );
        return;
    }

    let upcoming = filtered_upcoming(main);
    let mut line_widths = Vec::new();
    let hscroll = main.dashboard_upcoming_horizontal_scroll;
    let visible_width = scrolled_visible_width(body.width, hscroll);
    let lines: Vec<Line> = if upcoming.is_empty() {
        let message = "No upcoming assignments";
        line_widths.push(UnicodeWidthStr::width(message));
        vec![Line::from(Span::styled(
            pad_to_width("No upcoming assignments", visible_width),
            Style::default().fg(theme::NEUTRAL_GRAY).bg(pane_bg),
        ))]
    } else {
        upcoming
            .iter()
            .enumerate()
            .map(|(idx, assignment)| {
                let selected = focused && idx == main.selected_row;
                let style = if selected {
                    Style::default()
                        .fg(theme::NEUTRAL_WHITE)
                        .bg(theme::PANEL_SELECTED)
                } else {
                    Style::default().fg(text_fg).bg(pane_bg)
                };
                let course = assignment
                    .course_short_name
                    .as_deref()
                    .or(assignment.course_full_name.as_deref())
                    .unwrap_or("?");
                let text = format!(
                    "{}  {:<14}  {}",
                    format_timestamp(assignment.due_date),
                    course,
                    assignment.name
                );
                line_widths.push(UnicodeWidthStr::width(text.as_str()));
                Line::from(Span::styled(pad_to_width(&text, visible_width), style))
            })
            .collect()
    };

    frame.render_widget(Paragraph::new(lines).scroll((0, hscroll)), body);
    render_horizontal_indicators(frame, body, hscroll, &line_widths, indicator_fg, 0);
}

fn render_courses(frame: &mut Frame, area: Rect, main: &MainState) {
    let focused = main.dashboard_focus == DashboardPane::Courses;
    let pane_bg = pane_background(focused);
    let text_fg = pane_text_color(focused);
    let indicator_fg = if focused {
        theme::NEUTRAL_WHITE
    } else {
        text_fg
    };
    fill_area(frame, area, pane_bg);
    let courses = filtered_courses(main);
    render_pane_header(
        frame,
        area,
        &format!("Courses ({})", courses.len()),
        focused,
    );
    let body = pane_body(area);

    let hscroll = main.dashboard_courses_horizontal_scroll;
    let visible_width = scrolled_visible_width(body.width, hscroll);
    let mut line_widths = Vec::new();
    let lines: Vec<Line> = courses
        .iter()
        .enumerate()
        .map(|(idx, course)| {
            let selected = focused && idx == main.selected_row;
            line_widths.push(course_row_width(course));
            course_row_line(course, selected, pane_bg, text_fg, visible_width)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).scroll((0, hscroll)), body);
    render_horizontal_indicators(frame, body, hscroll, &line_widths, indicator_fg, 0);
}

fn render_pane_header(frame: &mut Frame, area: Rect, title: &str, focused: bool) {
    let pane_bg = pane_background(focused);
    let style = if focused {
        Style::default()
            .fg(theme::NEUTRAL_WHITE)
            .bg(pane_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme::NEUTRAL_GRAY)
            .bg(pane_bg)
            .add_modifier(Modifier::BOLD)
    };
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.min(1),
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            pad_to_width(title, header_area.width as usize),
            style,
        ))),
        header_area,
    );
}

fn pane_body(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(1),
    }
}

fn render_horizontal_indicators(
    frame: &mut Frame,
    area: Rect,
    horizontal_scroll: u16,
    line_widths: &[usize],
    indicator_fg: ratatui::style::Color,
    vertical_scroll: u16,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let style = Style::default()
        .fg(indicator_fg)
        .add_modifier(Modifier::BOLD);

    for visible_y in 0..area.height {
        let Some(line_width) = line_widths.get(vertical_scroll as usize + visible_y as usize)
        else {
            continue;
        };
        let max_scroll = horizontal_max_for_width(*line_width, area.width);
        let (left, right) = horizontal_continuation(horizontal_scroll, max_scroll);
        if left {
            render_edge_indicator(
                frame,
                Rect {
                    x: area.x,
                    y: area.y + visible_y,
                    width: 1,
                    height: 1,
                },
                style,
            );
        }
        if right {
            render_edge_indicator(
                frame,
                Rect {
                    x: area.right().saturating_sub(1),
                    y: area.y + visible_y,
                    width: 1,
                    height: 1,
                },
                style,
            );
        }
    }
}

fn render_edge_indicator(frame: &mut Frame, area: Rect, style: Style) {
    let lines: Vec<Line<'static>> = (0..area.height)
        .map(|_| Line::from(Span::styled("…", style)))
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

pub fn dashboard_pane_width(total_width: u16, pane: DashboardPane) -> u16 {
    let area = Rect {
        x: 0,
        y: 0,
        width: total_width,
        height: 1,
    };
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    match pane {
        DashboardPane::Upcoming => panes[0].width,
        DashboardPane::Courses => panes[1].width,
    }
}

pub fn max_horizontal_scroll(main: &MainState, pane: DashboardPane, pane_width: u16) -> u16 {
    let max_width = match pane {
        DashboardPane::Upcoming => filtered_upcoming(main)
            .iter()
            .map(|assignment| {
                let course = assignment
                    .course_short_name
                    .as_deref()
                    .or(assignment.course_full_name.as_deref())
                    .unwrap_or("?");
                UnicodeWidthStr::width(
                    format!(
                        "{}  {:<14}  {}",
                        format_timestamp(assignment.due_date),
                        course,
                        assignment.name
                    )
                    .as_str(),
                )
            })
            .max()
            .unwrap_or_else(|| UnicodeWidthStr::width("No upcoming assignments")),
        DashboardPane::Courses => filtered_courses(main)
            .iter()
            .map(|course| course_row_width(course))
            .max()
            .unwrap_or(0),
    };
    horizontal_max_for_width(max_width, pane_width)
}

pub fn bounded_horizontal_scroll(current: u16, delta: isize, max: u16) -> u16 {
    if delta < 0 {
        current.saturating_sub(delta.unsigned_abs() as u16)
    } else {
        current.saturating_add(delta as u16).min(max)
    }
}

fn horizontal_max_for_width(content_width: usize, pane_width: u16) -> u16 {
    content_width
        .saturating_sub(pane_width as usize)
        .min(u16::MAX as usize) as u16
}

fn horizontal_continuation(horizontal_scroll: u16, max_scroll: u16) -> (bool, bool) {
    (horizontal_scroll > 0, horizontal_scroll < max_scroll)
}

fn scrolled_visible_width(viewport_width: u16, horizontal_scroll: u16) -> usize {
    viewport_width as usize + horizontal_scroll as usize
}

fn course_row_width(course: &Course) -> usize {
    UnicodeWidthStr::width(format!("{:<10}", course.shortname).as_str())
        + 1
        + UnicodeWidthStr::width(course.fullname.as_str())
}

fn course_row_line(
    course: &Course,
    selected: bool,
    pane_bg: ratatui::style::Color,
    text_fg: ratatui::style::Color,
    width: usize,
) -> Line<'static> {
    let shortname = format!("{:<10}", course.shortname);
    let text_width = course_row_width(course);
    let padding = " ".repeat(width.saturating_sub(text_width));
    if selected {
        let style = Style::default()
            .fg(theme::NEUTRAL_WHITE)
            .bg(theme::PANEL_SELECTED)
            .add_modifier(Modifier::BOLD);
        return Line::from(vec![
            Span::styled(shortname, style),
            Span::styled(" ", style),
            Span::styled(course.fullname.clone(), style),
            Span::styled(padding, style),
        ]);
    }

    Line::from(vec![
        Span::styled(
            shortname,
            Style::default()
                .fg(if text_fg == theme::NEUTRAL_WHITE {
                    theme::BRAND
                } else {
                    text_fg
                })
                .bg(pane_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().bg(pane_bg)),
        Span::styled(
            course.fullname.clone(),
            Style::default().fg(text_fg).bg(pane_bg),
        ),
        Span::styled(padding, Style::default().bg(pane_bg)),
    ])
}

fn pane_background(focused: bool) -> ratatui::style::Color {
    if focused {
        theme::PANEL_ACTIVE_BG
    } else {
        theme::PANEL_INACTIVE_BG
    }
}

fn pane_text_color(focused: bool) -> ratatui::style::Color {
    if focused {
        theme::NEUTRAL_WHITE
    } else {
        theme::NEUTRAL_GRAY
    }
}

fn fill_area(frame: &mut Frame, area: Rect, bg: ratatui::style::Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let line = " ".repeat(area.width as usize);
    let lines: Vec<Line<'static>> = (0..area.height)
        .map(|_| Line::from(Span::styled(line.clone(), Style::default().bg(bg))))
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn pad_to_width(value: &str, width: usize) -> String {
    let current = unicode_width::UnicodeWidthStr::width(value);
    if current >= width {
        value.to_owned()
    } else {
        format!("{value}{}", " ".repeat(width - current))
    }
}

pub fn filtered_courses(main: &MainState) -> Vec<&Course> {
    filter_courses(&main.dashboard.courses, &main.dashboard_search_query)
        .into_iter()
        .map(|(course, _)| course)
        .collect()
}

pub fn filtered_upcoming(main: &MainState) -> Vec<&UpcomingAssignment> {
    let query = main.dashboard_search_query.trim().to_lowercase();
    main.dashboard
        .upcoming
        .iter()
        .filter(|assignment| {
            query.is_empty()
                || assignment.name.to_lowercase().contains(&query)
                || assignment
                    .course_short_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&query)
                || assignment
                    .course_full_name
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&query)
                || format_timestamp(assignment.due_date)
                    .to_lowercase()
                    .contains(&query)
        })
        .collect()
}

pub fn render_course_page(frame: &mut Frame, area: Rect, course: &CoursePageData) {
    fill_area(frame, area, theme::PANEL_ACTIVE_BG);
    let title_name = if !course.course_full_name.is_empty() {
        course.course_full_name.clone()
    } else if !course.course_short_name.is_empty() {
        course.course_short_name.clone()
    } else {
        format!("Course #{}", course.course_id)
    };
    render_course_header(frame, area, &title_name);
    let body = pane_body(area);

    if course.loading {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                pad_to_width("Loading sections…", body.width as usize),
                secondary_course_style(theme::PANEL_ACTIVE_BG),
            ))),
            body,
        );
        return;
    }

    if let Some(error) = &course.error {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                pad_to_width(&format!("Error: {error}"), body.width as usize),
                Style::default().fg(theme::ERROR).bg(theme::PANEL_ACTIVE_BG),
            ))),
            body,
        );
        return;
    }

    let rows = build_course_tree_rows(&course.sections, &course.collapsed);
    let total = rows.len();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(body);
    let viewport = layout[0].height as usize;
    let selected = course.selected_row.min(total.saturating_sub(1));
    let scroll = selected.saturating_sub(viewport.saturating_sub(1));

    let lines: Vec<Line> = if rows.is_empty() {
        vec![Line::from(Span::styled(
            pad_to_width("No sections available", layout[0].width as usize),
            secondary_course_style(theme::PANEL_ACTIVE_BG),
        ))]
    } else {
        let mut visual_lines = Vec::new();
        for (idx, row) in rows.iter().skip(scroll).enumerate() {
            visual_lines.extend(course_tree_row_lines(
                row,
                scroll + idx == selected,
                theme::PANEL_ACTIVE_BG,
                layout[0].width as usize,
            ));
            if visual_lines.len() >= viewport {
                break;
            }
        }
        visual_lines.truncate(viewport);
        visual_lines
    };

    let counter = if total > 0 {
        format!("{}/{}", selected + 1, total)
    } else {
        "0/0".into()
    };
    frame.render_widget(Paragraph::new(lines), layout[0]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            pad_to_width(&counter, layout[1].width as usize),
            secondary_course_style(theme::PANEL_ACTIVE_BG),
        ))),
        layout[1],
    );
}

fn render_course_header(frame: &mut Frame, area: Rect, title: &str) {
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.min(1),
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            pad_to_width(title, header_area.width as usize),
            Style::default()
                .fg(theme::NEUTRAL_WHITE)
                .bg(theme::PANEL_ACTIVE_BG)
                .add_modifier(Modifier::BOLD),
        ))),
        header_area,
    );
}

fn course_tree_row_lines(
    row: &crate::ui::course_tree::CourseTreeRow,
    selected: bool,
    pane_bg: Color,
    width: usize,
) -> Vec<Line<'static>> {
    if matches!(row.kind, CourseTreeNodeKind::Label) {
        return label_tree_row_lines(row, selected, pane_bg, width);
    }
    vec![course_tree_row_line(row, selected, pane_bg, width)]
}

fn course_tree_row_line(
    row: &crate::ui::course_tree::CourseTreeRow,
    selected: bool,
    pane_bg: Color,
    width: usize,
) -> Line<'static> {
    let prefix = render_tree_prefix(row);
    let secondary = matches!(
        row.kind,
        CourseTreeNodeKind::Summary
            | CourseTreeNodeKind::ModuleDescription
            | CourseTreeNodeKind::ModuleUrl
    );
    let text = format!("{prefix} {} {}", row.icon, row.text);
    let padding = " ".repeat(width.saturating_sub(UnicodeWidthStr::width(text.as_str())));

    if selected {
        let style = selected_course_style();
        return Line::from(vec![
            Span::styled(text, style),
            Span::styled(padding, style),
        ]);
    }

    let style = if secondary {
        secondary_course_style(pane_bg)
    } else {
        Style::default().fg(theme::NEUTRAL_WHITE).bg(pane_bg)
    };
    Line::from(vec![
        Span::styled(text, style),
        Span::styled(padding, Style::default().bg(pane_bg)),
    ])
}

fn label_tree_row_lines(
    row: &crate::ui::course_tree::CourseTreeRow,
    selected: bool,
    pane_bg: Color,
    width: usize,
) -> Vec<Line<'static>> {
    let prefix = render_tree_prefix(row);
    let first_prefix = format!("{prefix} {} ", row.icon);
    let continuation_prefix = " ".repeat(UnicodeWidthStr::width(first_prefix.as_str()));
    let label_width = width
        .saturating_sub(UnicodeWidthStr::width(first_prefix.as_str()))
        .max(1);

    wrap_text_to_width(&row.text, label_width)
        .into_iter()
        .enumerate()
        .map(|(idx, text)| {
            let prefix = if idx == 0 {
                first_prefix.clone()
            } else {
                continuation_prefix.clone()
            };
            label_line(&prefix, &text, selected, pane_bg, width)
        })
        .collect()
}

fn label_line(
    prefix: &str,
    text: &str,
    selected: bool,
    pane_bg: Color,
    width: usize,
) -> Line<'static> {
    let content_width = UnicodeWidthStr::width(prefix) + UnicodeWidthStr::width(text);
    let padding = " ".repeat(width.saturating_sub(content_width));
    if selected {
        let style = selected_course_style();
        return Line::from(vec![
            Span::styled(prefix.to_owned(), style),
            Span::styled(text.to_owned(), style),
            Span::styled(padding, style),
        ]);
    }

    Line::from(vec![
        Span::styled(
            prefix.to_owned(),
            Style::default()
                .fg(theme::BRAND)
                .bg(pane_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            text.to_owned(),
            Style::default().fg(theme::NEUTRAL_GRAY).bg(pane_bg),
        ),
        Span::styled(padding, Style::default().bg(pane_bg)),
    ])
}

fn wrap_text_to_width(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        for part in split_word_to_width(word, width) {
            if current.is_empty() {
                current.push_str(&part);
                continue;
            }

            let candidate_width = UnicodeWidthStr::width(current.as_str())
                + 1
                + UnicodeWidthStr::width(part.as_str());
            if candidate_width <= width {
                current.push(' ');
                current.push_str(&part);
            } else {
                lines.push(std::mem::take(&mut current));
                current.push_str(&part);
            }
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn split_word_to_width(word: &str, width: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;

    for ch in word.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str()).max(1);
        if current_width + ch_width > width && !current.is_empty() {
            parts.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(ch);
        current_width += ch_width;
    }

    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

fn selected_course_style() -> Style {
    Style::default()
        .fg(theme::NEUTRAL_WHITE)
        .bg(theme::PANEL_SELECTED)
        .add_modifier(Modifier::BOLD)
}

fn secondary_course_style(pane_bg: Color) -> Style {
    Style::default().fg(theme::NEUTRAL_GRAY).bg(pane_bg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Course, UpcomingAssignment};
    use crate::ui::course_tree::CourseTreeRow;

    #[test]
    fn dashboard_filter_applies_to_courses_and_upcoming() {
        let mut main = MainState::default();
        main.dashboard.courses = vec![
            Course {
                id: 1,
                shortname: "MATH".into(),
                fullname: "Mathematics".into(),
                displayname: None,
                categoryid: None,
                categoryname: None,
                summary: None,
                visible: None,
                progress: None,
                courseurl: None,
            },
            Course {
                id: 2,
                shortname: "HIST".into(),
                fullname: "History".into(),
                displayname: None,
                categoryid: None,
                categoryname: None,
                summary: None,
                visible: None,
                progress: None,
                courseurl: None,
            },
        ];
        main.dashboard.upcoming = vec![
            UpcomingAssignment {
                id: 10,
                name: "Algebra worksheet".into(),
                due_date: 1_800_000_000,
                course_id: 1,
                course_short_name: Some("MATH".into()),
                course_full_name: Some("Mathematics".into()),
            },
            UpcomingAssignment {
                id: 11,
                name: "Essay".into(),
                due_date: 1_800_000_000,
                course_id: 2,
                course_short_name: Some("HIST".into()),
                course_full_name: Some("History".into()),
            },
        ];
        main.dashboard_search_query = "math".into();

        assert_eq!(filtered_courses(&main).len(), 1);
        assert_eq!(filtered_courses(&main)[0].shortname, "MATH");
        assert_eq!(filtered_upcoming(&main).len(), 1);
        assert_eq!(filtered_upcoming(&main)[0].id, 10);
    }

    #[test]
    fn long_dashboard_rows_have_bounded_horizontal_scroll() {
        let mut main = MainState::default();
        main.dashboard.courses = vec![Course {
            id: 1,
            shortname: "MATH".into(),
            fullname: "A very long course title that should continue past the visible pane".into(),
            displayname: None,
            categoryid: None,
            categoryname: None,
            summary: None,
            visible: None,
            progress: None,
            courseurl: None,
        }];
        main.dashboard.upcoming = vec![UpcomingAssignment {
            id: 10,
            name: "A very long upcoming assignment name that should continue past the pane".into(),
            due_date: 1_800_000_000,
            course_id: 1,
            course_short_name: Some("MATH".into()),
            course_full_name: Some("Mathematics".into()),
        }];

        let course_max = max_horizontal_scroll(&main, DashboardPane::Courses, 32);
        let upcoming_max = max_horizontal_scroll(&main, DashboardPane::Upcoming, 32);

        assert!(course_max > 0);
        assert!(upcoming_max > 0);
        assert_eq!(bounded_horizontal_scroll(0, 200, course_max), course_max);
        assert_eq!(bounded_horizontal_scroll(course_max, -200, course_max), 0);
    }

    #[test]
    fn long_course_shortnames_contribute_to_horizontal_scroll() {
        let mut main = MainState::default();
        main.dashboard.courses = vec![Course {
            id: 1,
            shortname: "AM1CHIFScheineckerDellinger2223".into(),
            fullname: "AM 1CHIF Scheinecker_Dellinger 2223".into(),
            displayname: None,
            categoryid: None,
            categoryname: None,
            summary: None,
            visible: None,
            progress: None,
            courseurl: None,
        }];

        let rendered_width = UnicodeWidthStr::width(
            "AM1CHIFScheineckerDellinger2223 AM 1CHIF Scheinecker_Dellinger 2223",
        );

        assert_eq!(course_row_width(&main.dashboard.courses[0]), rendered_width);
        assert_eq!(
            max_horizontal_scroll(&main, DashboardPane::Courses, 32),
            (rendered_width - 32) as u16
        );
    }

    #[test]
    fn dashboard_horizontal_continuation_indicates_hidden_sides() {
        assert_eq!(horizontal_continuation(0, 0), (false, false));
        assert_eq!(horizontal_continuation(0, 12), (false, true));
        assert_eq!(horizontal_continuation(4, 12), (true, true));
        assert_eq!(horizontal_continuation(12, 12), (true, false));
    }

    #[test]
    fn scrolled_visible_width_includes_horizontal_offset() {
        assert_eq!(scrolled_visible_width(40, 0), 40);
        assert_eq!(scrolled_visible_width(40, 12), 52);
    }

    #[test]
    fn selected_course_row_styles_shortname_and_fullname() {
        let course = Course {
            id: 1,
            shortname: "MATH".into(),
            fullname: "Mathematics".into(),
            displayname: None,
            categoryid: None,
            categoryname: None,
            summary: None,
            visible: None,
            progress: None,
            courseurl: None,
        };

        let line = course_row_line(
            &course,
            true,
            theme::PANEL_ACTIVE_BG,
            theme::NEUTRAL_WHITE,
            32,
        );

        assert_eq!(line.spans.len(), 4);
        assert!(
            line.spans
                .iter()
                .all(|span| span.style.bg == Some(theme::PANEL_SELECTED))
        );
        assert!(
            line.spans
                .iter()
                .all(|span| span.style.fg == Some(theme::NEUTRAL_WHITE))
        );
    }

    #[test]
    fn selected_course_tree_rows_fill_visible_width() {
        let row = CourseTreeRow {
            id: "module:1:2".into(),
            kind: CourseTreeNodeKind::Module,
            depth: 1,
            text: "Week 1 quiz".into(),
            link_url: None,
            module_type: Some("quiz".into()),
            icon: "Q",
            collapsible: false,
            expanded: false,
            parent_id: None,
        };

        let line = course_tree_row_line(&row, true, theme::PANEL_ACTIVE_BG, 40);

        assert_eq!(test_line_width(&line), 40);
        assert!(
            line.spans
                .iter()
                .all(|span| span.style.bg == Some(theme::PANEL_SELECTED))
        );
    }

    #[test]
    fn secondary_course_tree_rows_are_dimmed_on_pane_background() {
        let row = CourseTreeRow {
            id: "summary:1".into(),
            kind: CourseTreeNodeKind::Summary,
            depth: 0,
            text: "Section summary".into(),
            link_url: None,
            module_type: None,
            icon: "",
            collapsible: false,
            expanded: false,
            parent_id: None,
        };

        let line = course_tree_row_line(&row, false, theme::PANEL_ACTIVE_BG, 40);

        assert_eq!(line.spans[0].style.fg, Some(theme::NEUTRAL_GRAY));
        assert!(
            line.spans
                .iter()
                .all(|span| span.style.bg == Some(theme::PANEL_ACTIVE_BG))
        );
    }

    #[test]
    fn long_label_rows_wrap_without_exceeding_width() {
        let row = label_row("This is a very long Moodle label that should wrap cleanly");

        let lines = course_tree_row_lines(&row, false, theme::PANEL_ACTIVE_BG, 28);

        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| test_line_width(line) == 28));
        assert_eq!(lines[0].spans[0].style.fg, Some(theme::BRAND));
        assert_eq!(lines[0].spans[1].style.fg, Some(theme::NEUTRAL_GRAY));
        assert!(
            lines
                .iter()
                .flat_map(|line| line.spans.iter())
                .all(|span| !span.style.add_modifier.contains(Modifier::UNDERLINED))
        );
    }

    #[test]
    fn selected_wrapped_label_lines_fill_width_with_selected_style() {
        let row = label_row("A selected Moodle label that wraps to another line");

        let lines = course_tree_row_lines(&row, true, theme::PANEL_ACTIVE_BG, 24);

        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| test_line_width(line) == 24));
        assert!(
            lines
                .iter()
                .flat_map(|line| line.spans.iter())
                .all(|span| span.style.bg == Some(theme::PANEL_SELECTED))
        );
    }

    #[test]
    fn long_label_words_are_split_to_fit_width() {
        let row = label_row("Supercalifragilisticexpialidocious");

        let lines = course_tree_row_lines(&row, false, theme::PANEL_ACTIVE_BG, 18);

        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| test_line_width(line) == 18));
    }

    fn label_row(text: &str) -> CourseTreeRow {
        CourseTreeRow {
            id: "label:1:2".into(),
            kind: CourseTreeNodeKind::Label,
            depth: 1,
            text: text.into(),
            link_url: None,
            module_type: None,
            icon: "L",
            collapsible: false,
            expanded: false,
            parent_id: None,
        }
    }

    fn test_line_width(line: &Line) -> usize {
        line.spans
            .iter()
            .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
            .sum()
    }
}
