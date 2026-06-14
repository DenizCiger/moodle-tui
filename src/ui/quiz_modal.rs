use crate::app::state::types::QuizModalData;
use crate::models::QuizAnswerKind;
use crate::ui::shared::centered_rect;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

pub fn render(frame: &mut Frame, modal: &QuizModalData) {
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

    if modal.attempt.is_none() {
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
        return;
    }

    let attempt = modal.attempt.as_ref().unwrap();
    if attempt.questions.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Moodle returned no visible questions for this attempt.",
        ));
        lines.push(footer_line());
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
        return;
    }

    let qidx = modal.selected_question.min(attempt.questions.len() - 1);
    let question = &attempt.questions[qidx];
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "Question {} of {}: {}",
            qidx + 1,
            attempt.questions.len(),
            question.name
        ),
        Style::default()
            .fg(theme::BRAND)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(question.text.clone()));

    if question.unsupported {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "This question contains controls that v1 cannot safely submit from the TUI.",
            Style::default().fg(theme::WARNING),
        )));
        lines.push(Line::from(
            "Use Shift+C to open the Moodle quiz in a browser.",
        ));
    } else {
        let visible_controls: Vec<_> = question
            .controls
            .iter()
            .filter(|c| c.kind != QuizAnswerKind::Hidden)
            .collect();
        for (control_idx, control) in visible_controls.iter().enumerate() {
            let selected_control = control_idx == modal.selected_control;
            lines.push(Line::from(""));
            match control.kind {
                QuizAnswerKind::Text => {
                    let marker = if selected_control { ">" } else { " " };
                    lines.push(Line::from(vec![
                        Span::styled(marker, Style::default().fg(theme::BRAND)),
                        Span::raw(" "),
                        Span::styled(&control.name, Style::default().fg(theme::NEUTRAL_GRAY)),
                        Span::raw(": "),
                        Span::styled(
                            if control.value.is_empty() {
                                "(empty)"
                            } else {
                                &control.value
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
                    let marker = if selected_control { ">" } else { " " };
                    lines.push(Line::from(vec![
                        Span::styled(marker, Style::default().fg(theme::BRAND)),
                        Span::raw(" "),
                        Span::styled(&control.name, Style::default().fg(theme::NEUTRAL_GRAY)),
                    ]));
                    for (option_idx, option) in control.options.iter().enumerate() {
                        let picked = selected_control && option_idx == modal.selected_option;
                        let box_text = if control.kind == QuizAnswerKind::MultiChoice {
                            if option.selected { "[x]" } else { "[ ]" }
                        } else if option.selected {
                            "(*)"
                        } else {
                            "( )"
                        };
                        let style = if picked {
                            Style::default()
                                .fg(theme::NEUTRAL_WHITE)
                                .bg(theme::PANEL_SELECTED)
                        } else {
                            Style::default()
                        };
                        lines.push(Line::from(Span::styled(
                            format!("    {box_text} {}", option.label),
                            style,
                        )));
                    }
                }
                QuizAnswerKind::Unsupported | QuizAnswerKind::Hidden => {}
            }
        }
    }

    if modal.confirm_finish {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Finish attempt? y submit · n/Esc cancel",
            Style::default()
                .fg(theme::WARNING)
                .add_modifier(Modifier::BOLD),
        )));
    }
    lines.push(footer_line());
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn status_text(modal: &QuizModalData) -> String {
    if modal.loading {
        "loading".into()
    } else if modal.saving {
        "saving".into()
    } else if modal.finishing {
        "finishing".into()
    } else if let Some(attempt) = &modal.attempt {
        format!("attempt #{} {}", attempt.attempt.id, attempt.attempt.state)
    } else {
        "ready".into()
    }
}

fn footer_line() -> Line<'static> {
    Line::from(Span::styled(
        "PgUp/PgDn question · Up/Down field · Left/Right option · Space toggle · s save · F finish · Shift+C browser · Esc close",
        Style::default().fg(theme::NEUTRAL_GRAY),
    ))
}
