use crate::app::state::types::QuizModalData;
use crate::models::{QuizAnswerControl, QuizAnswerKind};
use crate::ui::shared::{centered_rect, truncate};
use crate::ui::theme;
use chrono::Utc;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

const NAV_WIDTH: u16 = 13;

pub fn render(frame: &mut Frame, modal: &QuizModalData) {
    if modal.attempt.is_some() {
        render_attempt(frame, modal);
    } else {
        render_start_modal(frame, modal);
    }
}

fn render_start_modal(frame: &mut Frame, modal: &QuizModalData) {
    let outer = frame.area();
    let area = centered_rect(
        ((outer.width as f32) * 0.82) as u16,
        ((outer.height as f32) * 0.82) as u16,
        outer,
    );
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Quiz - {} ", modal.quiz_name))
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = status_lines(modal);
    if let Some(summary) = &modal.summary {
        if let Some(intro) = summary
            .intro
            .as_deref()
            .map(crate::moodle::html::strip_html)
            .filter(|s| !s.is_empty())
            .or_else(|| modal.module_description.clone())
        {
            lines.push(Line::from(""));
            lines.push(Line::from(intro));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter start attempt · Shift+C open in browser · c copy link · Esc close",
        Style::default().fg(theme::NEUTRAL_GRAY),
    )));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn render_attempt(frame: &mut Frame, modal: &QuizModalData) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BRAND));
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    render_attempt_header(frame, rows[0], modal);
    frame.render_widget(Block::default().borders(Borders::TOP), rows[1]);
    render_attempt_body(frame, rows[2], modal);
    frame.render_widget(Block::default().borders(Borders::TOP), rows[3]);
    render_attempt_footer(frame, rows[4], modal);
}

fn render_attempt_header(frame: &mut Frame, area: Rect, modal: &QuizModalData) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(26)])
        .split(area);
    let quiz_title = format!(" QUIZ: {}", modal.quiz_name);
    frame.render_widget(
        Paragraph::new(truncate(&quiz_title, chunks[0].width as usize)),
        chunks[0],
    );
    frame.render_widget(
        Paragraph::new(format!("Time Remaining: {}", time_remaining_text(modal)))
            .alignment(Alignment::Right),
        chunks[1],
    );
}

fn render_attempt_body(frame: &mut Frame, area: Rect, modal: &QuizModalData) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(NAV_WIDTH), Constraint::Min(1)])
        .split(area);
    frame.render_widget(Block::default().borders(Borders::RIGHT), columns[0]);
    render_navigation(frame, columns[0], modal);
    render_question(frame, columns[1], modal);
}

fn render_navigation(frame: &mut Frame, area: Rect, modal: &QuizModalData) {
    let Some(attempt) = &modal.attempt else {
        return;
    };
    let mut lines = vec![Line::from(Span::styled(
        " NAVIGATION",
        Style::default().add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(""));
    for chunk in attempt.questions.chunks(2) {
        let mut spans = Vec::new();
        spans.push(Span::raw(" "));
        for question in chunk {
            let idx = attempt
                .questions
                .iter()
                .position(|q| q.slot == question.slot)
                .unwrap_or(0);
            let style = if idx == modal.selected_question {
                Style::default()
                    .fg(theme::NEUTRAL_WHITE)
                    .bg(theme::PANEL_SELECTED)
            } else {
                Style::default()
            };
            spans.push(Span::styled(format!("[{}]", idx + 1), style));
            spans.push(Span::raw(" "));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), inset(area, 1, 0));
}

fn render_question(frame: &mut Frame, area: Rect, modal: &QuizModalData) {
    let Some(attempt) = &modal.attempt else {
        return;
    };
    if attempt.questions.is_empty() {
        let lines = vec![
            Line::from(" QUESTION"),
            Line::from(""),
            Line::from(" Moodle returned no visible questions for this attempt."),
        ];
        frame.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: false }),
            inset(area, 1, 0),
        );
        return;
    }

    let qidx = modal.selected_question.min(attempt.questions.len() - 1);
    let question = &attempt.questions[qidx];
    let mut lines = vec![Line::from(Span::styled(
        format!(
            " QUESTION {}",
            question
                .number
                .as_deref()
                .unwrap_or(&(qidx + 1).to_string())
        ),
        Style::default().add_modifier(Modifier::BOLD),
    ))];
    lines.push(Line::from(""));
    lines.extend(wrapped_text_lines(
        &question.text,
        area.width.saturating_sub(4) as usize,
    ));

    if question.unsupported {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " This question contains controls that v1 cannot safely submit from the TUI.",
            Style::default().fg(theme::WARNING),
        )));
        lines.push(Line::from(
            " Use Shift+C to open the Moodle quiz in a browser.",
        ));
    } else {
        for (control_idx, control) in visible_controls(question.controls.as_slice())
            .iter()
            .enumerate()
        {
            lines.push(Line::from(""));
            render_control_lines(&mut lines, modal, control_idx, control);
        }
    }

    if modal.confirm_finish {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Finish attempt? y submit · n/Esc cancel",
            Style::default()
                .fg(theme::WARNING)
                .add_modifier(Modifier::BOLD),
        )));
    }

    if let Some(error) = &modal.error {
        lines.push(Line::from(""));
        for line in error.lines() {
            lines.push(Line::from(Span::styled(
                format!(" ERROR: {line}"),
                Style::default().fg(theme::ERROR),
            )));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inset(area, 1, 0),
    );
}

fn render_control_lines(
    lines: &mut Vec<Line<'static>>,
    modal: &QuizModalData,
    control_idx: usize,
    control: &QuizAnswerControl,
) {
    let selected_control = control_idx == modal.selected_control;
    match control.kind {
        QuizAnswerKind::Text => {
            let label_style = if selected_control {
                Style::default()
                    .fg(theme::NEUTRAL_WHITE)
                    .bg(theme::PANEL_SELECTED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(
                    if modal.editing_text {
                        " Answer (editing): "
                    } else {
                        " Answer: "
                    },
                    label_style,
                ),
                Span::styled(
                    if control.value.is_empty() {
                        "(empty)".to_owned()
                    } else {
                        control.value.clone()
                    },
                    if selected_control {
                        Style::default().bg(theme::PANEL_SELECTED)
                    } else {
                        Style::default()
                    },
                ),
            ]));
        }
        QuizAnswerKind::SingleChoice | QuizAnswerKind::MultiChoice => {
            let answer_style = if selected_control {
                Style::default()
                    .fg(theme::NEUTRAL_WHITE)
                    .bg(theme::PANEL_SELECTED)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            lines.push(Line::from(vec![
                Span::raw(" "),
                Span::styled(" Answer", answer_style),
            ]));
            for (option_idx, option) in control.options.iter().enumerate() {
                let picked = selected_control && option_idx == modal.selected_option;
                let box_text = if control.kind == QuizAnswerKind::MultiChoice {
                    if option.selected { "[x]" } else { "[ ]" }
                } else if option.selected {
                    "●"
                } else {
                    "○"
                };
                let style = if picked {
                    Style::default()
                        .fg(theme::NEUTRAL_WHITE)
                        .bg(theme::PANEL_SELECTED)
                } else {
                    Style::default()
                };
                lines.push(Line::from(Span::styled(
                    format!("   {box_text} {}", option.label),
                    style,
                )));
            }
        }
        QuizAnswerKind::Unsupported | QuizAnswerKind::Hidden => {}
    }
}

fn render_attempt_footer(frame: &mut Frame, area: Rect, modal: &QuizModalData) {
    let mut left = status_text(modal);
    if let Some(error) = &modal.error {
        left = format!("Error: {error}");
    }
    let hint = if area.width >= 96 {
        "Left/Right question · Tab field · Up/Down option · Space select · Enter edit · F2 save · F10 finish"
    } else if area.width >= 96 {
        "Left/Right question · Tab field · Up/Down option · Space select · Enter edit · F2 save · F10 finish"
    } else if area.width >= 72 {
        "←/→ question · Tab field · ↑/↓ option · Space select · F2 save · F10 finish"
    } else {
        "←/→ · Tab · Space · F2 save · F10 finish"
    };
    let left_text = truncate(
        &format!(" {hint} · {left}"),
        area.width.saturating_sub(22) as usize,
    );
    let footer = Line::from(vec![
        Span::styled(left_text, Style::default().fg(theme::NEUTRAL_GRAY)),
        Span::raw(" "),
        Span::styled(
            " [ Finish Attempt ] ",
            Style::default()
                .fg(theme::NEUTRAL_WHITE)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(footer).alignment(Alignment::Right), area);
}

fn status_lines(modal: &QuizModalData) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Course: ", Style::default().fg(theme::NEUTRAL_GRAY)),
            Span::styled(
                modal.course_name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("State:  ", Style::default().fg(theme::NEUTRAL_GRAY)),
            Span::raw(status_text(modal)),
        ]),
    ];
    if let Some(error) = &modal.error {
        lines.push(Line::from(Span::styled(
            format!("Error: {error}"),
            Style::default().fg(theme::ERROR),
        )));
    }
    lines
}

fn status_text(modal: &QuizModalData) -> String {
    if modal.loading {
        "loading".into()
    } else if modal.saving {
        "saving".into()
    } else if modal.finishing {
        "finishing".into()
    } else if modal.ai_filling {
        "AI filling...".into()
    } else if let Some(attempt) = &modal.attempt {
        format!("attempt #{} {}", attempt.attempt.id, attempt.attempt.state)
    } else {
        "ready".into()
    }
}

fn time_remaining_text(modal: &QuizModalData) -> String {
    if modal.attempt.is_none() {
        return "No limit".into();
    }
    let Some(deadline) = quiz_deadline(modal) else {
        return "No limit".into();
    };
    let remaining = deadline.saturating_sub(Utc::now().timestamp());
    format_duration(remaining)
}

fn quiz_deadline(modal: &QuizModalData) -> Option<i64> {
    let attempt = &modal.attempt.as_ref()?.attempt;
    let summary = modal.summary.as_ref();
    let mut deadline = attempt.timefinish.filter(|value| *value > 0).or_else(|| {
        let timelimit = summary?.timelimit.filter(|value| *value > 0)?;
        Some(attempt.timestart? + timelimit)
    });
    if let Some(timeclose) = summary
        .and_then(|summary| summary.timeclose)
        .filter(|value| *value > 0)
    {
        deadline = Some(deadline.map_or(timeclose, |current| current.min(timeclose)));
    }
    deadline
}

fn format_duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn visible_controls(controls: &[QuizAnswerControl]) -> Vec<&QuizAnswerControl> {
    controls
        .iter()
        .filter(|control| control.kind != QuizAnswerKind::Hidden)
        .collect()
}

fn wrapped_text_lines(text: &str, width: usize) -> Vec<Line<'static>> {
    if width == 0 {
        return vec![Line::from("")];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if next_len > width && !current.is_empty() {
            lines.push(Line::from(format!(" {current}")));
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(Line::from(format!(" {current}")));
    }
    if lines.is_empty() {
        lines.push(Line::from(""));
    }
    lines
}

fn inset(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(horizontal),
        y: area.y.saturating_add(vertical),
        width: area.width.saturating_sub(horizontal.saturating_mul(2)),
        height: area.height.saturating_sub(vertical.saturating_mul(2)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        QuizAnswerOption, QuizAttempt, QuizAttemptData, QuizQuestion, QuizSummary,
    };
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn formats_time_remaining_from_finish_time() {
        let mut modal = test_modal();
        let now = Utc::now().timestamp();
        modal.attempt.as_mut().unwrap().attempt.timefinish = Some(now + 65);
        assert!(matches!(
            time_remaining_text(&modal).as_str(),
            "00:01:04" | "00:01:05"
        ));
    }

    #[test]
    fn clamps_expired_time_remaining() {
        let mut modal = test_modal();
        modal.attempt.as_mut().unwrap().attempt.timefinish = Some(Utc::now().timestamp() - 1);
        assert_eq!(time_remaining_text(&modal), "00:00:00");
    }

    #[test]
    fn reports_no_limit_without_timing_data() {
        let mut modal = test_modal();
        modal.summary.as_mut().unwrap().timelimit = None;
        modal.attempt.as_mut().unwrap().attempt.timefinish = None;
        modal.attempt.as_mut().unwrap().attempt.timestart = None;
        assert_eq!(time_remaining_text(&modal), "No limit");
    }

    #[test]
    fn close_time_caps_timelimit_deadline() {
        let mut modal = test_modal();
        let now = Utc::now().timestamp();
        modal.attempt.as_mut().unwrap().attempt.timestart = Some(now);
        modal.attempt.as_mut().unwrap().attempt.timefinish = None;
        modal.summary.as_mut().unwrap().timelimit = Some(3600);
        modal.summary.as_mut().unwrap().timeclose = Some(now + 30);
        assert!(matches!(
            time_remaining_text(&modal).as_str(),
            "00:00:29" | "00:00:30"
        ));
    }

    #[test]
    fn renders_fullscreen_attempt_layout() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let modal = test_modal();
        terminal.draw(|frame| render(frame, &modal)).unwrap();
        let content = terminal.backend().buffer().content();
        let text = content.iter().map(|cell| cell.symbol()).collect::<String>();
        assert!(text.contains("QUIZ:"));
        assert!(text.contains("Time Remaining:"));
        assert!(text.contains("NAVIGATION"));
        assert!(text.contains("QUESTION 1"));
        assert!(text.contains("Space select"));
        assert!(text.contains("[ Finish Attempt ]"));
    }

    fn test_modal() -> QuizModalData {
        let now = Utc::now().timestamp();
        QuizModalData {
            course_id: 1,
            quiz_id: 2,
            cmid: 3,
            quiz_name: "Introduction to Web Development".into(),
            course_name: "Course".into(),
            module_description: None,
            module_url: None,
            summary: Some(QuizSummary {
                id: 2,
                course_id: 1,
                cmid: 3,
                name: "Introduction to Web Development".into(),
                intro: None,
                timeopen: None,
                timeclose: None,
                timelimit: Some(1500),
                attempts: None,
            }),
            attempt: Some(QuizAttemptData {
                attempt: QuizAttempt {
                    id: 4,
                    quiz: 2,
                    state: "inprogress".into(),
                    currentpage: Some(0),
                    timestart: Some(now),
                    timefinish: Some(now + 1500),
                },
                questions: vec![QuizQuestion {
                    slot: 1,
                    number: Some("1".into()),
                    name: "Question".into(),
                    text: "Which of the following technologies is primarily responsible for styling web pages?".into(),
                    html: String::new(),
                    controls: vec![QuizAnswerControl {
                        name: "q1:1_answer".into(),
                        kind: QuizAnswerKind::SingleChoice,
                        options: vec![
                            QuizAnswerOption {
                                name: None,
                                label: "HTML".into(),
                                value: "0".into(),
                                selected: false,
                            },
                            QuizAnswerOption {
                                name: None,
                                label: "CSS".into(),
                                value: "1".into(),
                                selected: true,
                            },
                            QuizAnswerOption {
                                name: None,
                                label: "JavaScript".into(),
                                value: "2".into(),
                                selected: false,
                            },
                        ],
                        value: String::new(),
                    }],
                    unsupported: false,
                }],
                warnings: Vec::new(),
            }),
            loading: false,
            saving: false,
            finishing: false,
            ai_filling: false,
            confirm_finish: false,
            error: None,
            selected_question: 0,
            selected_control: 0,
            selected_option: 1,
            editing_text: false,
        }
    }
}
