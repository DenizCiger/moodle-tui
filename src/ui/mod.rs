pub mod assignment_modal;
pub mod content_finder;
pub mod course_tree;
pub mod dashboard;
pub mod login;
pub mod settings;
pub mod shared;
pub mod shell;
pub mod theme;

use crate::app::state::{AppState, Screen};
use ratatui::Frame;

pub fn render(frame: &mut Frame, state: &AppState) {
    match &state.screen {
        Screen::Loading => render_loading(frame),
        Screen::Login(login) => login::render(frame, login, state),
        Screen::MainShell(main) => shell::render(frame, main, state),
    }
}

fn render_loading(frame: &mut Frame) {
    use ratatui::widgets::{Block, Paragraph};
    let area = frame.area();
    let paragraph = Paragraph::new("Loading…").block(Block::bordered().title("moodle-tui"));
    frame.render_widget(paragraph, area);
}
