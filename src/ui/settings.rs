use crate::app::state::AppState;
use crate::app::state::types::Screen;
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::ui::theme;
use ratatui::Frame;
use tui_components::ui::settings::SettingsModal;

pub fn render(frame: &mut Frame, state: &AppState) {
    let scroll = match &state.screen {
        Screen::MainShell(main) => main.settings_scroll,
        _ => 0,
    };
    let mut modal = SettingsModal::from_shortcuts(
        "Keyboard shortcuts",
        get_shortcut_sections(TabId::Dashboard),
    );
    modal.scroll = scroll;
    modal.key_width = 14;
    modal.render(frame, frame.area(), theme::components_theme());
}
