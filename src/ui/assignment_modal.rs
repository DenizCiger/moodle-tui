use crate::app::state::types::AssignmentModalData;
use crate::ui::shared::centered_rect;
use crate::ui::shell::format_timestamp;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn render(frame: &mut Frame, modal: &AssignmentModalData) {
    let outer = frame.area();
    let area = centered_rect(
        ((outer.width as f32) * 0.75) as u16,
        ((outer.height as f32) * 0.7) as u16,
        outer,
    );
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Assignment — {} ", modal.assignment_name))
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Course: ", Style::default().fg(theme::NEUTRAL_GRAY)),
        Span::styled(modal.course_name.clone(), Style::default().add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Due:    ", Style::default().fg(theme::NEUTRAL_GRAY)),
        Span::raw(format_timestamp(modal.due_date)),
    ]));

    if let Some(detail) = &modal.detail {
        if let Some(cutoff) = detail.cutoffdate.filter(|v| *v > 0) {
            lines.push(Line::from(vec![
                Span::styled("Cutoff: ", Style::default().fg(theme::NEUTRAL_GRAY)),
                Span::raw(format_timestamp(cutoff)),
            ]));
        }
        if let Some(grading) = detail.gradingduedate.filter(|v| *v > 0) {
            lines.push(Line::from(vec![
                Span::styled("Grading due: ", Style::default().fg(theme::NEUTRAL_GRAY)),
                Span::raw(format_timestamp(grading)),
            ]));
        }
        if let Some(grade) = detail.grade.filter(|v| *v > 0.0) {
            lines.push(Line::from(vec![
                Span::styled("Max grade: ", Style::default().fg(theme::NEUTRAL_GRAY)),
                Span::raw(format!("{grade}")),
            ]));
        }
    }

    if let Some(status) = &modal.status {
        lines.push(Line::from(""));
        if let Some(value) = &status.submission_status {
            lines.push(Line::from(vec![
                Span::styled("Submission ", Style::default().fg(theme::NEUTRAL_GRAY)),
                badge(value, submission_color(value)),
            ]));
        }
        if let Some(value) = &status.grading_status {
            lines.push(Line::from(vec![
                Span::styled("Grading    ", Style::default().fg(theme::NEUTRAL_GRAY)),
                badge(value, grading_color(value)),
            ]));
        }
    }

    if modal.detail_loading || modal.status_loading {
        lines.push(Line::from(Span::styled(
            "Loading…",
            Style::default().fg(theme::NEUTRAL_GRAY),
        )));
    }
    if let Some(error) = &modal.detail_error {
        lines.push(Line::from(Span::styled(
            format!("Detail error: {error}"),
            Style::default().fg(theme::ERROR),
        )));
    }
    if let Some(error) = &modal.status_error {
        lines.push(Line::from(Span::styled(
            format!("Status error: {error}"),
            Style::default().fg(theme::ERROR),
        )));
    }

    let description = modal
        .detail
        .as_ref()
        .and_then(|d| d.intro.as_deref())
        .map(crate::moodle::html::strip_html)
        .filter(|s| !s.is_empty())
        .or_else(|| modal.module_description.clone());

    if let Some(desc) = description {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Description",
            Style::default().fg(theme::BRAND).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(desc));
    }

    if let Some(url) = &modal.module_url {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Link: ", Style::default().fg(theme::NEUTRAL_GRAY)),
            Span::raw(url.clone()),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Esc close · Shift+C open in browser · c copy link",
        Style::default().fg(theme::NEUTRAL_GRAY),
    )));

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn badge(text: &str, bg: Color) -> Span<'static> {
    Span::styled(
        format!(" {text} "),
        Style::default().bg(bg).fg(theme::NEUTRAL_BLACK).add_modifier(Modifier::BOLD),
    )
}

fn submission_color(status: &str) -> Color {
    match status.to_lowercase().as_str() {
        "submitted" | "submitted for grading" => theme::SUCCESS,
        "new" | "draft" => theme::WARNING,
        _ => theme::NEUTRAL_GRAY,
    }
}

fn grading_color(status: &str) -> Color {
    match status.to_lowercase().as_str() {
        "graded" => theme::SUCCESS,
        "notgraded" | "not graded" => theme::WARNING,
        _ => theme::NEUTRAL_GRAY,
    }
}
