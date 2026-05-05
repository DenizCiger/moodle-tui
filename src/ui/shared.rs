use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1])[1]
}

pub fn truncate(value: &str, max_chars: usize) -> String {
    use unicode_width::UnicodeWidthChar;
    let mut width = 0usize;
    let mut out = String::new();
    for ch in value.chars() {
        let w = ch.width().unwrap_or(0);
        if width + w > max_chars.saturating_sub(1) {
            out.push('…');
            return out;
        }
        out.push(ch);
        width += w;
    }
    out
}
