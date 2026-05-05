use crate::app::state::AppState;
use crate::app::state::types::Screen;
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::ui::shared::centered_rect;
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

pub fn render(frame: &mut Frame, state: &AppState) {
    let outer = frame.area();
    let area = centered_rect(
        (outer.width as f32 * 0.8) as u16,
        (outer.height as f32 * 0.8) as u16,
        outer,
    );
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Keyboard shortcuts ")
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let scroll = match &state.screen {
        Screen::MainShell(main) => main.settings_scroll,
        _ => 0,
    };

    let mut lines: Vec<Line> = Vec::new();
    for section in get_shortcut_sections(TabId::Dashboard) {
        lines.push(Line::from(Span::styled(
            section.title,
            Style::default().fg(theme::BRAND).add_modifier(Modifier::BOLD),
        )));
        for item in section.items {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14}", item.keys), Style::default().fg(theme::WARNING)),
                Span::raw(item.action.to_owned()),
            ]));
        }
        lines.push(Line::from(""));
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);
    let total = lines.len() as u16;
    let viewport = layout[0].height;
    let max_scroll = total.saturating_sub(viewport);
    let clamped = scroll.min(max_scroll);
    frame.render_widget(Paragraph::new(lines).scroll((clamped, 0)), layout[0]);
    let footer = if max_scroll > 0 {
        format!("↑/↓ scroll · {clamped}/{max_scroll} · Esc or ? to close")
    } else {
        "Esc or ? to close".to_owned()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            footer,
            Style::default().fg(theme::NEUTRAL_GRAY),
        ))),
        layout[1],
    );
}
