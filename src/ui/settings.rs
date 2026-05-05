use crate::app::state::AppState;
use crate::app::state::types::Screen;
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::ui::theme;
use ratatui::Frame;
use tui_components::ui::settings::{SettingsItemView, SettingsModal, SettingsSectionView};
use tui_components::ui::theme::Theme;

pub fn render(frame: &mut Frame, state: &AppState) {
    let scroll = match &state.screen {
        Screen::MainShell(main) => main.settings_scroll,
        _ => 0,
    };
    let sections = get_shortcut_sections(TabId::Dashboard)
        .into_iter()
        .map(|section| SettingsSectionView {
            title: section.title.to_owned(),
            items: section
                .items
                .into_iter()
                .map(|item| SettingsItemView {
                    keys: item.keys.to_owned(),
                    action: item.action.to_owned(),
                })
                .collect(),
        })
        .collect();
    let mut modal = SettingsModal::new("Keyboard shortcuts", sections);
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
