use crate::app::state::AppState;
use crate::app::state::types::CourseView;
use crate::app::state::types::MainState;
use crate::ui::{assignment_modal, course_tree, dashboard, quiz_modal, settings, theme};
use chrono::TimeZone;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use tui_components::ui::search::{
    SearchModal, SearchModalCategory, SearchModalRow, highlight_spans,
};
use unicode_width::UnicodeWidthStr;

pub fn render(frame: &mut Frame, main: &MainState, state: &AppState) {
    let area = frame.area();
    if !main.settings_open && matches!(main.view, CourseView::Dashboard) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        render_tabs(frame, layout[0], main, state);
        render_dashboard_filter_row(frame, layout[1], main);
        dashboard::render_dashboard(frame, layout[2], main);
        render_footer(frame, layout[3], main);
    } else if main.settings_open {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        render_tabs(frame, layout[0], main, state);
        render_settings_filter_row(frame, layout[1], main);
        settings::render(frame, layout[2], main);
        render_footer(frame, layout[3], main);
    } else {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        render_tabs(frame, layout[0], main, state);

        if let CourseView::Course(course) = &main.view {
            dashboard::render_course_page(frame, layout[1], course);
        }

        render_horizontal_separator(frame, layout[2]);
        render_footer(frame, layout[3], main);
    }

    if let Some(modal) = &main.assignment_modal {
        assignment_modal::render(frame, modal);
    }
    if let Some(modal) = &main.quiz_modal {
        quiz_modal::render(frame, modal);
    }

    if main.course_finder_open || main.content_finder_open {
        render_finder(frame, main);
    }

    render_api_key_input(frame, main);
    render_model_picker(frame, main);
    render_plugin_install_input(frame, main);

    if let Some(toast) = &main.toast {
        render_toast(frame, toast);
    }
}

fn render_tabs(frame: &mut Frame, area: Rect, main: &MainState, state: &AppState) {
    let username = state
        .saved_config
        .as_ref()
        .map(|c| c.username.as_str())
        .unwrap_or("(unknown)");
    let tab_title = active_tab_title(main);
    let username = format!("[{username}]");
    let username_width = UnicodeWidthStr::width(username.as_str());
    let max_tab_width = if area.width as usize > username_width + 2 {
        area.width as usize - username_width - 2
    } else {
        area.width as usize
    };
    let tab = format!(
        " {} ",
        truncate_with_ellipsis(&tab_title, max_tab_width.saturating_sub(2))
    );
    let tab_width = UnicodeWidthStr::width(tab.as_str());
    let show_username = tab_width + 1 + username_width <= area.width as usize;
    let spacer_width = if show_username {
        area.width as usize - tab_width - username_width
    } else {
        area.width as usize - tab_width.min(area.width as usize)
    };

    let mut spans = vec![Span::styled(
        tab,
        Style::default()
            .fg(theme::NEUTRAL_BLACK)
            .bg(theme::BRAND)
            .add_modifier(Modifier::BOLD),
    )];
    spans.push(Span::styled(
        " ".repeat(spacer_width),
        Style::default().bg(theme::NEUTRAL_BLACK),
    ));
    if show_username {
        spans.push(Span::styled(
            username,
            Style::default()
                .fg(theme::NEUTRAL_GRAY)
                .bg(theme::NEUTRAL_BLACK),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn active_tab_title(main: &MainState) -> String {
    if main.settings_open {
        "Settings".to_owned()
    } else {
        match &main.view {
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
        }
    }
}

fn render_dashboard_filter_row(frame: &mut Frame, area: Rect, main: &MainState) {
    let (text, style) = if main.dashboard_search_active || !main.dashboard_search_query.is_empty() {
        (
            format!(" / {}", main.dashboard_search_query),
            Style::default()
                .fg(theme::NEUTRAL_WHITE)
                .bg(theme::CHROME_BG),
        )
    } else {
        (
            " / filter courses and upcoming tasks".to_owned(),
            Style::default()
                .fg(theme::NEUTRAL_GRAY)
                .bg(theme::CHROME_BG),
        )
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            pad_to_width(&text, area.width as usize),
            style,
        ))),
        area,
    );
}

fn render_settings_filter_row(frame: &mut Frame, area: Rect, main: &MainState) {
    let (text, style) = if main.settings_search_active || !main.settings_search_query.is_empty() {
        (
            format!(" / {}", main.settings_search_query),
            Style::default()
                .fg(theme::NEUTRAL_WHITE)
                .bg(theme::CHROME_BG),
        )
    } else {
        (
            " / filter keybinds and config".to_owned(),
            Style::default()
                .fg(theme::NEUTRAL_GRAY)
                .bg(theme::CHROME_BG),
        )
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            pad_to_width(&text, area.width as usize),
            style,
        ))),
        area,
    );
}

fn render_horizontal_separator(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "─".repeat(area.width as usize),
            Style::default()
                .fg(theme::SEPARATOR)
                .bg(theme::NEUTRAL_BLACK),
        ))),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, main: &MainState) {
    if main.settings_open {
        let text = settings::footer_text(main);
        let line = Line::from(Span::styled(
            pad_to_width(text, area.width as usize),
            Style::default()
                .fg(theme::NEUTRAL_GRAY)
                .bg(theme::CHROME_BG),
        ));
        frame.render_widget(Paragraph::new(line), area);
        return;
    }
    let mut hints = vec![
        "? help".to_owned(),
        if matches!(main.view, CourseView::Dashboard) {
            "/ filter".to_owned()
        } else {
            "/ courses".to_owned()
        },
        "r refresh".to_owned(),
    ];
    if matches!(main.view, CourseView::Dashboard) {
        hints.push("←/→ pan".to_owned());
        hints.push("Ctrl+← reset pan".to_owned());
    }
    if matches!(main.view, CourseView::Course(_)) {
        hints.push("Esc back".to_owned());
        hints.push("f content".to_owned());
    }
    hints.push("q quit".to_owned());
    let text = format!(" {}", hints.join("  ·  "));
    let line = Line::from(Span::styled(
        pad_to_width(&text, area.width as usize),
        Style::default()
            .fg(theme::NEUTRAL_GRAY)
            .bg(theme::CHROME_BG),
    ));
    frame.render_widget(Paragraph::new(line), area);
}

fn substring_match_indices(haystack: &str, query_lower: &str) -> Vec<usize> {
    if query_lower.is_empty() {
        return Vec::new();
    }
    let hay_lower = haystack.to_lowercase();
    let Some(byte_start) = hay_lower.find(query_lower) else {
        return Vec::new();
    };
    let char_start = hay_lower[..byte_start].chars().count();
    let q_len = query_lower.chars().count();
    (char_start..char_start + q_len).collect()
}

fn render_finder(frame: &mut Frame, main: &MainState) {
    use crate::search::courses::filter_courses;

    if main.course_finder_open {
        use crate::search::courses::CourseField;
        let filtered = filter_courses(&main.dashboard.courses, &main.finder.input.value);
        let shortname_base = Style::default()
            .fg(theme::BRAND)
            .add_modifier(Modifier::BOLD);
        let highlight_style = Style::default()
            .fg(theme::WARNING)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        let rows: Vec<SearchModalRow> = filtered
            .iter()
            .map(|(course, hi)| {
                let padded_shortname = format!("{:<10}", course.shortname);
                let shortname_spans = if matches!(hi.field, Some(CourseField::Shortname)) {
                    highlight_spans(
                        &padded_shortname,
                        &hi.indices,
                        shortname_base,
                        highlight_style,
                    )
                } else {
                    vec![Span::styled(padded_shortname, shortname_base)]
                };
                let fullname_spans = if matches!(hi.field, Some(CourseField::Fullname)) {
                    highlight_spans(
                        &course.fullname,
                        &hi.indices,
                        Style::default(),
                        highlight_style,
                    )
                } else {
                    vec![Span::raw(course.fullname.clone())]
                };
                let mut spans = shortname_spans;
                spans.push(Span::raw(" "));
                spans.extend(fullname_spans);
                SearchModalRow::new(spans)
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
        let highlight_style = Style::default()
            .fg(theme::WARNING)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
        let rows: Vec<SearchModalRow> = filtered
            .iter()
            .map(|row| {
                let indices = substring_match_indices(&row.text, &query);
                let text_spans = if indices.is_empty() {
                    vec![Span::raw(row.text.clone())]
                } else {
                    highlight_spans(&row.text, &indices, Style::default(), highlight_style)
                };
                let mut spans = vec![Span::raw(format!("{} ", row.icon))];
                spans.extend(text_spans);
                SearchModalRow::new(spans)
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

fn render_api_key_input(frame: &mut Frame, main: &MainState) {
    let Some(input) = &main.api_key_input else {
        return;
    };
    let area = frame.area();
    let width = area.width.min(60);
    let height = 7;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let rect = ratatui::layout::Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(input.title.clone())
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    let is_secret = input.secret;
    let display = if input.input.value.is_empty() {
        if is_secret {
            "(paste or type secret value)".to_string()
        } else {
            format!("({})", input.current_value)
        }
    } else if is_secret {
        "\u{2022}".repeat(input.input.value.len())
    } else {
        input.input.value.clone()
    };
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(display, Style::default().bg(theme::PANEL_SELECTED)),
    ]));
    lines.push(Line::from(""));
    if let Some(error) = &input.error {
        lines.push(Line::from(Span::styled(
            format!(" {}", error),
            Style::default().fg(theme::ERROR),
        )));
    }
    if input.saving {
        lines.push(Line::from(Span::styled(
            " Saving...",
            Style::default().fg(theme::NEUTRAL_GRAY),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            " Enter save · Esc cancel ",
            Style::default().fg(theme::NEUTRAL_GRAY),
        )));
    }

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner);
}

fn render_plugin_install_input(frame: &mut Frame, main: &MainState) {
    let Some(input) = &main.plugin_install_input else {
        return;
    };
    let area = frame.area();
    let width = area.width.min(76);
    let height = 7;
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let rect = ratatui::layout::Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Install Local Plugin ")
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let mut lines = vec![Line::from("")];
    let display = if input.input.value.is_empty() {
        "(path to folder containing plugin.json)".to_owned()
    } else {
        input.input.value.clone()
    };
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled(display, Style::default().bg(theme::PANEL_SELECTED)),
    ]));
    lines.push(Line::from(""));
    if let Some(error) = &input.error {
        lines.push(Line::from(Span::styled(
            format!(" {error}"),
            Style::default().fg(theme::ERROR),
        )));
    }
    lines.push(Line::from(Span::styled(
        if input.saving {
            " Installing..."
        } else {
            " Enter install · Esc cancel "
        },
        Style::default().fg(theme::NEUTRAL_GRAY),
    )));

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner);
}

fn render_model_picker(frame: &mut Frame, main: &MainState) {
    let Some(picker) = &main.model_picker else {
        return;
    };
    let area = frame.area();
    let width = area.width.min(72);
    let option_count = picker.options.len().min(8) as u16;
    let height = (option_count + 5).max(7);
    let x = area.x + (area.width - width) / 2;
    let y = area.y + (area.height - height) / 2;
    let rect = ratatui::layout::Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(picker.title.clone())
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(rect);
    frame.render_widget(block, rect);

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    for (idx, option) in picker.options.iter().enumerate() {
        let style = if idx == picker.selected {
            Style::default()
                .fg(theme::NEUTRAL_WHITE)
                .bg(theme::PANEL_SELECTED)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::NEUTRAL_WHITE)
        };
        lines.push(Line::from(vec![
            Span::raw(" "),
            Span::styled(option.clone(), style),
        ]));
    }
    if let Some(error) = &picker.error {
        lines.push(Line::from(Span::styled(
            format!(" {error}"),
            Style::default().fg(theme::ERROR),
        )));
    } else {
        lines.push(Line::from(""));
    }
    let hint = if picker.saving {
        " Saving..."
    } else {
        " ↑/↓ select · Enter save · Esc cancel "
    };
    lines.push(Line::from(Span::styled(
        hint,
        Style::default().fg(theme::NEUTRAL_GRAY),
    )));

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner);
}

pub fn format_timestamp(seconds: i64) -> String {
    chrono::Local
        .timestamp_opt(seconds, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "(unknown)".to_owned())
}

fn pad_to_width(value: &str, width: usize) -> String {
    let current = UnicodeWidthStr::width(value);
    if current >= width {
        value.to_owned()
    } else {
        format!("{value}{}", " ".repeat(width - current))
    }
}

fn truncate_with_ellipsis(value: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(value) <= max_width {
        return value.to_owned();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "…".to_owned();
    }
    let mut out = String::new();
    let mut width = 0usize;
    for ch in value.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if width + ch_width + 1 > max_width {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::types::CoursePageData;

    #[test]
    fn active_tab_title_uses_current_view() {
        let mut main = MainState::default();
        assert_eq!(active_tab_title(&main), "Dashboard");

        main.view = CourseView::Course(CoursePageData {
            course_short_name: "MATH".into(),
            course_full_name: "Mathematics".into(),
            ..Default::default()
        });
        assert_eq!(active_tab_title(&main), "MATH");

        main.settings_open = true;
        assert_eq!(active_tab_title(&main), "Settings");
    }
}
