use crate::app::state::AppState;
use crate::app::state::types::{MainState, Screen, SettingsPane, SettingsPaneState};
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsKeybindRow {
    pub section: String,
    pub keys: String,
    pub action: String,
    pub source: String,
    pub conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsConfigField {
    InstallPlugin,
    ReloadPlugins,
    TogglePlugin {
        plugin_id: String,
    },
    UninstallPlugin {
        plugin_id: String,
    },
    PluginSetting {
        plugin_id: String,
        setting_name: String,
        label: String,
        secret: bool,
        options: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsConfigRow {
    pub group_id: String,
    pub group_name: String,
    pub label: String,
    pub value: String,
    pub field: SettingsConfigField,
}

pub fn render(frame: &mut Frame, state: &AppState) {
    let Screen::MainShell(main) = &state.screen else {
        return;
    };

    let area = frame.area();
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Settings ")
        .border_style(Style::default().fg(theme::BRAND));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(52),
            Constraint::Length(1),
            Constraint::Percentage(48),
        ])
        .split(vertical[0]);

    render_keybind_pane(frame, panes[0], main);
    render_separator(frame, panes[1]);
    render_config_pane(frame, panes[2], main);
    render_footer(frame, vertical[1], main);
}

fn render_keybind_pane(frame: &mut Frame, area: Rect, main: &MainState) {
    let rows = filtered_keybind_rows(main);
    let state = &main.settings_keybinds;
    let active = main.settings_active_pane == SettingsPane::Keybinds;
    render_pane_header(frame, area, "Keybinds", active, state);
    render_keybind_rows(frame, pane_body(area), &rows, state);
}

fn render_config_pane(frame: &mut Frame, area: Rect, main: &MainState) {
    let rows = filtered_config_rows(main);
    let state = &main.settings_config;
    let active = main.settings_active_pane == SettingsPane::Config;
    render_pane_header(frame, area, "Config", active, state);
    render_config_rows(frame, pane_body(area), &rows, state);
}

fn pane_title(base: &str, active: bool, state: &SettingsPaneState) -> String {
    let focus = if active { "> " } else { "  " };
    if state.search_active || !state.search_query.is_empty() {
        format!("{focus}{base} /{}", state.search_query)
    } else {
        format!("{focus}{base}")
    }
}

fn render_pane_header(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    active: bool,
    state: &SettingsPaneState,
) {
    let style = if active {
        Style::default()
            .fg(theme::BRAND)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::NEUTRAL_GRAY)
    };
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.min(1),
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            pane_title(title, active, state),
            style,
        ))),
        header_area,
    );
}

fn pane_body(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(1),
    }
}

fn render_separator(frame: &mut Frame, area: Rect) {
    let lines: Vec<Line<'static>> = (0..area.height)
        .map(|_| Line::from(Span::styled("|", Style::default().fg(theme::NEUTRAL_GRAY))))
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

fn render_keybind_rows(
    frame: &mut Frame,
    area: Rect,
    rows: &[SettingsKeybindRow],
    state: &SettingsPaneState,
) {
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " No keybinds match.",
                Style::default().fg(theme::NEUTRAL_GRAY),
            ))),
            area,
        );
        return;
    }

    let mut lines = Vec::new();
    let mut visible_index = 0usize;
    let mut section = "";
    let selected = state.cursor.min(rows.len().saturating_sub(1));
    for row in rows {
        if row.section != section {
            section = &row.section;
            lines.push(Line::from(Span::styled(
                row.section.clone(),
                Style::default()
                    .fg(theme::BRAND)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        let selected_row = visible_index == selected;
        let mut action = row.action.clone();
        if row.conflict {
            action.push_str(" (conflict)");
        }
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<12}", row.keys),
                Style::default().fg(if row.conflict {
                    theme::ERROR
                } else {
                    theme::WARNING
                }),
            ),
            Span::styled(format!("{:<36}", action), selected_style(selected_row)),
            Span::styled(row.source.clone(), Style::default().fg(theme::NEUTRAL_GRAY)),
        ]));
        visible_index += 1;
    }

    let scroll = clamped_scroll(state.scroll, lines.len(), area.height);
    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), area);
}

fn render_config_rows(
    frame: &mut Frame,
    area: Rect,
    rows: &[SettingsConfigRow],
    state: &SettingsPaneState,
) {
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " No configurable settings match.",
                Style::default().fg(theme::NEUTRAL_GRAY),
            ))),
            area,
        );
        return;
    }

    let mut lines = Vec::new();
    let mut visible_index = 0usize;
    let mut group = "";
    let selected = state.cursor.min(rows.len().saturating_sub(1));
    for row in rows {
        if row.group_id != group {
            group = &row.group_id;
            lines.push(Line::from(Span::styled(
                row.group_name.clone(),
                Style::default()
                    .fg(theme::BRAND)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        let selected_row = visible_index == selected;
        let marker = if selected_row { ">" } else { " " };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {marker} {:<22}", row.label),
                selected_style(selected_row),
            ),
            Span::styled(row.value.clone(), value_style(selected_row)),
        ]));
        visible_index += 1;
    }

    let scroll = clamped_scroll(state.scroll, lines.len(), area.height);
    frame.render_widget(Paragraph::new(lines).scroll((scroll, 0)), area);
}

fn render_footer(frame: &mut Frame, area: Rect, main: &MainState) {
    let active = match main.settings_active_pane {
        SettingsPane::Keybinds => &main.settings_keybinds,
        SettingsPane::Config => &main.settings_config,
    };
    let text = if active.search_active {
        " Type to search - Enter keep filter - Esc clear "
    } else {
        " Tab switch pane - / search - Up/Down move - Enter edit config - Esc or ? close "
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(theme::NEUTRAL_GRAY),
        ))),
        area,
    );
}

fn selected_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(theme::NEUTRAL_WHITE)
            .bg(theme::PANEL_SELECTED)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme::NEUTRAL_WHITE)
    }
}

fn value_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(theme::NEUTRAL_WHITE)
            .bg(theme::PANEL_SELECTED)
    } else {
        Style::default().fg(theme::NEUTRAL_GRAY)
    }
}

fn clamped_scroll(scroll: u16, line_count: usize, viewport_height: u16) -> u16 {
    scroll.min((line_count as u16).saturating_sub(viewport_height))
}

pub fn keybind_rows(main: &MainState) -> Vec<SettingsKeybindRow> {
    let mut rows = Vec::new();
    let mut occupied = std::collections::HashSet::new();
    for section in get_shortcut_sections(TabId::Dashboard) {
        for item in section.items {
            occupied.insert(item.keys.to_owned());
            rows.push(SettingsKeybindRow {
                section: section.title.to_owned(),
                keys: item.keys.to_owned(),
                action: item.action.to_owned(),
                source: "core".into(),
                conflict: false,
            });
        }
    }
    for plugin in &main.plugin_registry.plugins {
        for action in &plugin.manifest.contributes.quiz_actions {
            let Some(key) = action
                .default_key
                .as_ref()
                .filter(|key| !key.trim().is_empty())
            else {
                continue;
            };
            let conflict = occupied.contains(key);
            rows.push(SettingsKeybindRow {
                section: "Plugin Quiz Actions".into(),
                keys: key.clone(),
                action: action.title.clone(),
                source: plugin.manifest.id.clone(),
                conflict,
            });
            occupied.insert(key.clone());
        }
    }
    rows
}

pub fn filtered_keybind_rows(main: &MainState) -> Vec<SettingsKeybindRow> {
    let query = main.settings_keybinds.search_query.trim().to_lowercase();
    keybind_rows(main)
        .into_iter()
        .filter(|row| {
            query.is_empty()
                || row.section.to_lowercase().contains(&query)
                || row.keys.to_lowercase().contains(&query)
                || row.action.to_lowercase().contains(&query)
                || row.source.to_lowercase().contains(&query)
        })
        .collect()
}

pub fn config_rows(main: &MainState) -> Vec<SettingsConfigRow> {
    let mut rows = vec![
        SettingsConfigRow {
            group_id: "plugin-manager".into(),
            group_name: "Plugin Manager".into(),
            label: "Install local plugin".into(),
            value: "open path prompt".into(),
            field: SettingsConfigField::InstallPlugin,
        },
        SettingsConfigRow {
            group_id: "plugin-manager".into(),
            group_name: "Plugin Manager".into(),
            label: "Reload plugins".into(),
            value: "refresh registry".into(),
            field: SettingsConfigField::ReloadPlugins,
        },
    ];
    for plugin in &main.plugin_registry.plugins {
        let group_id = plugin.manifest.id.clone();
        let group_name = format!(
            "{} {} ({})",
            plugin.manifest.name, plugin.manifest.version, plugin.manifest.id
        );
        rows.push(SettingsConfigRow {
            group_id: group_id.clone(),
            group_name: group_name.clone(),
            label: "Enabled".into(),
            value: if plugin.enabled { "yes" } else { "no" }.into(),
            field: SettingsConfigField::TogglePlugin {
                plugin_id: plugin.manifest.id.clone(),
            },
        });
        rows.push(SettingsConfigRow {
            group_id: group_id.clone(),
            group_name: group_name.clone(),
            label: "Uninstall".into(),
            value: plugin.directory.display().to_string(),
            field: SettingsConfigField::UninstallPlugin {
                plugin_id: plugin.manifest.id.clone(),
            },
        });
        for (name, schema) in plugin_settings_schema(plugin.manifest.settings_schema.as_ref()) {
            let label = schema_label(&name, schema);
            let secret = schema_is_secret(schema);
            let options = schema_options(schema);
            let value = if secret {
                if main
                    .plugin_secret_configured
                    .contains(&setting_key(&plugin.manifest.id, &name))
                {
                    "configured".into()
                } else {
                    "not configured".into()
                }
            } else {
                current_setting_value(main, &plugin.manifest.id, &name, schema)
            };
            rows.push(SettingsConfigRow {
                group_id: group_id.clone(),
                group_name: group_name.clone(),
                label: label.clone(),
                value,
                field: SettingsConfigField::PluginSetting {
                    plugin_id: plugin.manifest.id.clone(),
                    setting_name: name,
                    label,
                    secret,
                    options,
                },
            });
        }
    }
    rows
}

pub fn filtered_config_rows(main: &MainState) -> Vec<SettingsConfigRow> {
    let query = main.settings_config.search_query.trim().to_lowercase();
    config_rows(main)
        .into_iter()
        .filter(|row| {
            query.is_empty()
                || row.group_id.to_lowercase().contains(&query)
                || row.group_name.to_lowercase().contains(&query)
                || row.label.to_lowercase().contains(&query)
                || row.value.to_lowercase().contains(&query)
        })
        .collect()
}

pub fn setting_key(plugin_id: &str, name: &str) -> String {
    format!("{plugin_id}:{name}")
}

pub fn current_setting_value(
    main: &MainState,
    plugin_id: &str,
    name: &str,
    schema: &Value,
) -> String {
    main.plugin_settings
        .get(plugin_id)
        .and_then(|settings| settings.get(name))
        .cloned()
        .or_else(|| {
            schema
                .get("default")
                .and_then(|value| value.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

pub fn plugin_settings_schema(schema: Option<&Value>) -> Vec<(String, &Value)> {
    let Some(properties) = schema
        .and_then(|schema| schema.get("properties"))
        .and_then(|properties| properties.as_object())
    else {
        return Vec::new();
    };
    let mut rows: Vec<(String, &Value)> = properties
        .iter()
        .map(|(name, schema)| (name.clone(), schema))
        .collect();
    rows.sort_by(|left, right| left.0.cmp(&right.0));
    rows
}

pub fn schema_is_secret(schema: &Value) -> bool {
    schema
        .get("format")
        .and_then(|value| value.as_str())
        .is_some_and(|format| format == "secret")
        || schema
            .get("secret")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
}

pub fn schema_options(schema: &Value) -> Vec<String> {
    schema
        .get("enum")
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn schema_label(name: &str, schema: &Value) -> String {
    schema
        .get("title")
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            name.split('_')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => {
                            format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
                        }
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::registry::InstalledPlugin;

    fn main_with_plugin() -> MainState {
        let mut main = MainState::default();
        main.plugin_secret_configured
            .insert(setting_key("quiz-ai-extension", "gemini_api_key"));
        main.plugin_registry.plugins.push(InstalledPlugin {
            manifest: crate::plugins::manifest::PluginManifest {
                id: "quiz-ai-extension".into(),
                name: "Quiz AI Extension".into(),
                version: "0.2.0".into(),
                entry: "plugin.js".into(),
                permissions: Vec::new(),
                settings_schema: Some(serde_json::json!({
                    "properties": {
                        "gemini_api_key": {"type": "string", "format": "secret"},
                        "gemini_model": {
                            "default": "gemini-2.5-flash-lite",
                            "enum": ["gemini-2.5-flash-lite", "gemini-2.5-flash"]
                        }
                    }
                })),
                contributes: crate::plugins::manifest::PluginContributions {
                    quiz_actions: vec![crate::plugins::manifest::QuizActionContribution {
                        id: "fill_answers".into(),
                        title: "AI Fill Answer".into(),
                        description: None,
                        result_kind: Some("quiz_fill_answers".into()),
                        default_key: Some("F6".into()),
                    }],
                },
            },
            directory: ".".into(),
            enabled: true,
            load_error: None,
        });
        main
    }

    #[test]
    fn config_rows_redact_secret_and_include_enum_default() {
        let main = main_with_plugin();
        let rows = config_rows(&main);
        assert!(rows.iter().any(|row| row.value == "configured"));
        assert!(rows.iter().any(|row| row.value == "gemini-2.5-flash-lite"));
    }

    #[test]
    fn filters_config_rows_by_value() {
        let mut main = main_with_plugin();
        main.settings_config.search_query = "flash-lite".into();
        let rows = filtered_config_rows(&main);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn includes_plugin_keybind_rows() {
        let main = main_with_plugin();
        let rows = keybind_rows(&main);
        assert!(rows.iter().any(|row| row.source == "quiz-ai-extension"));
    }
}
