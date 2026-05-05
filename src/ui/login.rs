use crate::app::state::AppState;
use crate::app::state::types::{LoginFocus, LoginState};
use crate::ui::shared::centered_rect;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, login: &LoginState, _state: &AppState) {
    let area = centered_rect(72, 18, frame.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" moodle-tui — Login ")
        .border_style(Style::default().fg(theme::BRAND));
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(inner);

    render_field(frame, layout[0], "Base URL", &login.base_url.display(), login.focus == LoginFocus::BaseUrl);
    render_field(frame, layout[1], "Username", &login.username.display(), login.focus == LoginFocus::Username);
    render_field(frame, layout[2], "Password", &login.password.display(), login.focus == LoginFocus::Password);
    render_field(frame, layout[3], "Service", &login.service.display(), login.focus == LoginFocus::Service);

    let submit_style = if login.focus == LoginFocus::Submit {
        Style::default().fg(theme::NEUTRAL_WHITE).bg(theme::BRAND).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::NEUTRAL_GRAY)
    };
    let submit_line = Line::from(Span::styled(
        if login.busy { "  Logging in…  " } else { "  [ Submit ]  " },
        submit_style,
    ));
    frame.render_widget(Paragraph::new(submit_line), layout[4]);

    let mut footer_lines = Vec::new();
    if let Some(error) = &login.error {
        footer_lines.push(Line::from(Span::styled(
            format!("Error: {error}"),
            Style::default().fg(theme::ERROR),
        )));
    }
    if let Some(warning) = &login.storage_warning {
        footer_lines.push(Line::from(Span::styled(
            warning.clone(),
            Style::default().fg(theme::WARNING),
        )));
    }
    footer_lines.push(Line::from(Span::styled(
        "Tab/Shift+Tab to switch fields · Enter to submit · Esc to quit",
        Style::default().fg(theme::NEUTRAL_GRAY),
    )));
    frame.render_widget(Paragraph::new(footer_lines), layout[5]);
}

fn render_field(frame: &mut Frame, area: ratatui::layout::Rect, label: &str, value: &str, focused: bool) {
    let label_style = Style::default().fg(theme::NEUTRAL_GRAY);
    let value_style = if focused {
        Style::default().fg(theme::NEUTRAL_WHITE).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::NEUTRAL_WHITE)
    };
    let prefix = if focused { "▌ " } else { "  " };
    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(theme::BRAND)),
        Span::styled(format!("{label:<10}"), label_style),
        Span::styled(value.to_owned(), value_style),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
