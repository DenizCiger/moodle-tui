use crate::app::state::AppState;
use crate::app::state::types::Screen;
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::ui::theme;
use ratatui::Frame;
use tui_components::ui::settings::{SettingsItemView, SettingsModal, SettingsSectionView};

pub fn render(frame: &mut Frame, state: &AppState) {
    let scroll = match &state.screen {
        Screen::MainShell(main) => main.settings_scroll,
        _ => 0,
    };
    let mut sections: Vec<SettingsSectionView> = get_shortcut_sections(TabId::Dashboard)
        .into_iter()
        .map(Into::into)
        .collect();
    if let Screen::MainShell(main) = &state.screen {
        sections.push(plugin_settings_section(main));
    }
    let mut modal = SettingsModal::new("Settings", sections);
    modal.scroll = scroll;
    modal.key_width = 14;
    modal.render(frame, frame.area(), theme::components_theme());
}

fn plugin_settings_section(main: &crate::app::state::types::MainState) -> SettingsSectionView {
    let items = if main.plugin_registry.plugins.is_empty() {
        vec![SettingsItemView {
            keys: "Plugins".into(),
            action: "No plugins installed in the local plugin directory".into(),
        }]
    } else {
        main.plugin_registry
            .plugins
            .iter()
            .map(|plugin| {
                let status = if let Some(error) = &plugin.load_error {
                    format!("error: {error}")
                } else if plugin.enabled {
                    "enabled".into()
                } else {
                    "disabled".into()
                };
                SettingsItemView {
                    keys: plugin.manifest.id.clone(),
                    action: format!(
                        "{} {} ({status})",
                        plugin.manifest.name, plugin.manifest.version
                    ),
                }
            })
            .collect()
    };
    SettingsSectionView {
        title: "Plugins".into(),
        items,
    }
}
