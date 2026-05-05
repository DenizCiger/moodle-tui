use crate::app::state::AppState;
use crate::app::state::types::Screen;
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::ui::theme;
use ratatui::Frame;
use tui_components::ui::settings::SettingsModal;
use tui_components::ui::theme::Theme;

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
    modal.render(frame, frame.area(), app_theme());
}

fn app_theme() -> Theme {
    Theme {
        brand: theme::BRAND,
        warning: theme::WARNING,
        error: theme::ERROR,
        success: theme::SUCCESS,
        neutral_white: theme::NEUTRAL_WHITE,
        neutral_black: theme::NEUTRAL_BLACK,
        neutral_gray: theme::NEUTRAL_GRAY,
        neutral_bright_black: theme::NEUTRAL_BRIGHT_BLACK,
        panel_header: theme::PANEL_HEADER,
        panel_selected: theme::PANEL_SELECTED,
        panel_alternate: theme::PANEL_ALTERNATE,
    }
}
