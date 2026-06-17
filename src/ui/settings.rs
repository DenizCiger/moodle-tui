use crate::app::state::types::{MainState, SettingsPane, SettingsPaneState};
use crate::shortcuts::{TabId, get_shortcut_sections};
use crate::ui::theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use serde_json::Value;
use unicode_width::UnicodeWidthStr;

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

pub fn render(frame: &mut Frame, area: Rect, main: &MainState) {
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);

    render_keybind_pane(frame, panes[0], main);
    render_config_pane(frame, panes[1], main);
}

fn render_keybind_pane(frame: &mut Frame, area: Rect, main: &MainState) {
    let rows = filtered_keybind_rows(main);
    let state = &main.settings_keybinds;
    let active = main.settings_active_pane == SettingsPane::Keybinds;
    let pane_bg = pane_background(active);
    fill_area(frame, area, pane_bg);
    render_pane_header(frame, area, "Keybinds", active);
    render_keybind_rows(frame, pane_body(area), &rows, state, active);
}

fn render_config_pane(frame: &mut Frame, area: Rect, main: &MainState) {
    let rows = filtered_config_rows(main);
    let state = &main.settings_config;
    let active = main.settings_active_pane == SettingsPane::Config;
    let pane_bg = pane_background(active);
    fill_area(frame, area, pane_bg);
    render_pane_header(frame, area, "Config", active);
    render_config_rows(frame, pane_body(area), &rows, state, active);
}

fn render_pane_header(frame: &mut Frame, area: Rect, title: &str, active: bool) {
    let pane_bg = pane_background(active);
    let style = if active {
        Style::default()
            .fg(theme::NEUTRAL_WHITE)
            .bg(pane_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme::NEUTRAL_GRAY)
            .bg(pane_bg)
            .add_modifier(Modifier::BOLD)
    };
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.min(1),
    };
    let title = pad_to_width(title, header_area.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(title, style))),
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

fn render_keybind_rows(
    frame: &mut Frame,
    area: Rect,
    rows: &[SettingsKeybindRow],
    state: &SettingsPaneState,
    active: bool,
) {
    let pane_bg = pane_background(active);
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                pad_to_width(" No keybinds match.", area.width as usize),
                Style::default().fg(theme::NEUTRAL_GRAY).bg(pane_bg),
            ))),
            area,
        );
        return;
    }

    let mut lines = Vec::new();
    let mut line_widths = Vec::new();
    let mut visible_index = 0usize;
    let mut section = "";
    let selected = state.cursor.min(rows.len().saturating_sub(1));
    let visible_width = scrolled_visible_width(area.width, state.horizontal_scroll);
    for row in rows {
        if row.section != section {
            section = &row.section;
            line_widths.push(UnicodeWidthStr::width(row.section.as_str()));
            lines.push(Line::from(Span::styled(
                pad_to_width(&row.section, visible_width),
                Style::default()
                    .fg(if active {
                        theme::BRAND
                    } else {
                        theme::NEUTRAL_GRAY
                    })
                    .bg(pane_bg)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        let selected_row = active && visible_index == selected;
        let mut action = row.action.clone();
        if row.conflict {
            action.push_str(" (conflict)");
        }
        line_widths.push(keybind_row_width(&row.keys, &action));
        let key_width = 12usize;
        let keys = truncate_with_ellipsis(&row.keys, key_width);
        lines.push(keybind_row_line(
            &keys,
            &action,
            row.conflict,
            selected_row,
            active,
            visible_width,
        ));
        visible_index += 1;
    }

    let scroll = clamped_scroll(state.scroll, lines.len(), area.height);
    frame.render_widget(
        Paragraph::new(lines).scroll((scroll, state.horizontal_scroll)),
        area,
    );
    render_horizontal_indicators(
        frame,
        area,
        state.horizontal_scroll,
        &line_widths,
        pane_text_color(active),
        scroll,
    );
}

fn render_config_rows(
    frame: &mut Frame,
    area: Rect,
    rows: &[SettingsConfigRow],
    state: &SettingsPaneState,
    active: bool,
) {
    let pane_bg = pane_background(active);
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                pad_to_width(" No configurable settings match.", area.width as usize),
                Style::default().fg(theme::NEUTRAL_GRAY).bg(pane_bg),
            ))),
            area,
        );
        return;
    }

    let mut lines = Vec::new();
    let mut line_widths = Vec::new();
    let mut visible_index = 0usize;
    let mut group = "";
    let selected = state.cursor.min(rows.len().saturating_sub(1));
    let visible_width = scrolled_visible_width(area.width, state.horizontal_scroll);
    for row in rows {
        if row.group_id != group {
            group = &row.group_id;
            line_widths.push(UnicodeWidthStr::width(row.group_name.as_str()));
            lines.push(Line::from(Span::styled(
                pad_to_width(&row.group_name, visible_width),
                Style::default()
                    .fg(if active {
                        theme::BRAND
                    } else {
                        theme::NEUTRAL_GRAY
                    })
                    .bg(pane_bg)
                    .add_modifier(Modifier::BOLD),
            )));
        }
        let selected_row = active && visible_index == selected;
        line_widths.push(config_row_width(&row.label, &row.value));
        lines.push(config_row_line(
            &row.label,
            &row.value,
            selected_row,
            active,
            visible_width,
        ));
        visible_index += 1;
    }

    let scroll = clamped_scroll(state.scroll, lines.len(), area.height);
    frame.render_widget(
        Paragraph::new(lines).scroll((scroll, state.horizontal_scroll)),
        area,
    );
    render_horizontal_indicators(
        frame,
        area,
        state.horizontal_scroll,
        &line_widths,
        pane_text_color(active),
        scroll,
    );
}

fn render_horizontal_indicators(
    frame: &mut Frame,
    area: Rect,
    horizontal_scroll: u16,
    line_widths: &[usize],
    indicator_fg: ratatui::style::Color,
    vertical_scroll: u16,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let style = Style::default()
        .fg(indicator_fg)
        .add_modifier(Modifier::BOLD);

    for visible_y in 0..area.height {
        let Some(line_width) = line_widths.get(vertical_scroll as usize + visible_y as usize)
        else {
            continue;
        };
        let max_scroll = horizontal_max_for_width(*line_width, area.width);
        let (left, right) = horizontal_continuation(horizontal_scroll, max_scroll);
        if left {
            render_edge_indicator(
                frame,
                Rect {
                    x: area.x,
                    y: area.y + visible_y,
                    width: 1,
                    height: 1,
                },
                style,
            );
        }
        if right {
            render_edge_indicator(
                frame,
                Rect {
                    x: area.right().saturating_sub(1),
                    y: area.y + visible_y,
                    width: 1,
                    height: 1,
                },
                style,
            );
        }
    }
}

fn render_edge_indicator(frame: &mut Frame, area: Rect, style: Style) {
    let lines: Vec<Line<'static>> = (0..area.height)
        .map(|_| Line::from(Span::styled("…", style)))
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
}

pub fn settings_pane_width(total_width: u16, pane: SettingsPane) -> u16 {
    let area = Rect {
        x: 0,
        y: 0,
        width: total_width,
        height: 1,
    };
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);
    match pane {
        SettingsPane::Keybinds => panes[0].width,
        SettingsPane::Config => panes[1].width,
    }
}

pub fn max_horizontal_scroll(main: &MainState, pane: SettingsPane, pane_width: u16) -> u16 {
    let max_width = match pane {
        SettingsPane::Keybinds => filtered_keybind_rows(main)
            .iter()
            .flat_map(|row| {
                let mut action = row.action.clone();
                if row.conflict {
                    action.push_str(" (conflict)");
                }
                [
                    UnicodeWidthStr::width(row.section.as_str()),
                    keybind_row_width(&row.keys, &action),
                ]
            })
            .max()
            .unwrap_or(0),
        SettingsPane::Config => filtered_config_rows(main)
            .iter()
            .flat_map(|row| {
                [
                    UnicodeWidthStr::width(row.group_name.as_str()),
                    config_row_width(&row.label, &row.value),
                ]
            })
            .max()
            .unwrap_or(0),
    };
    horizontal_max_for_width(max_width, pane_width)
}

pub fn bounded_horizontal_scroll(current: u16, delta: isize, max: u16) -> u16 {
    if delta < 0 {
        current.saturating_sub(delta.unsigned_abs() as u16)
    } else {
        current.saturating_add(delta as u16).min(max)
    }
}

fn horizontal_max_for_width(content_width: usize, pane_width: u16) -> u16 {
    content_width
        .saturating_sub(pane_width as usize)
        .min(u16::MAX as usize) as u16
}

fn horizontal_continuation(horizontal_scroll: u16, max_scroll: u16) -> (bool, bool) {
    (horizontal_scroll > 0, horizontal_scroll < max_scroll)
}

fn scrolled_visible_width(viewport_width: u16, horizontal_scroll: u16) -> usize {
    viewport_width as usize + horizontal_scroll as usize
}

fn keybind_row_width(keys: &str, action: &str) -> usize {
    let key_width = 12usize;
    key_width
        + 1
        + UnicodeWidthStr::width(action)
        + UnicodeWidthStr::width(truncate_with_ellipsis(keys, key_width).as_str())
            .saturating_sub(key_width)
}

fn config_row_width(label: &str, value: &str) -> usize {
    UnicodeWidthStr::width(format!(" {label:<22}").as_str()) + UnicodeWidthStr::width(value)
}

fn keybind_row_line(
    keys: &str,
    action: &str,
    conflict: bool,
    selected: bool,
    active: bool,
    width: usize,
) -> Line<'static> {
    let pane_bg = pane_background(active);
    let key_width = 12usize;
    let keys = pad_to_width(keys, key_width);
    let text_width = keybind_row_width(keys.trim_end(), action);
    let padding = " ".repeat(width.saturating_sub(text_width));

    if selected {
        let style = selected_style(true);
        return Line::from(vec![
            Span::styled(keys, style),
            Span::styled(" ", style),
            Span::styled(action.to_owned(), style),
            Span::styled(padding, style),
        ]);
    }

    Line::from(vec![
        Span::styled(
            keys,
            Style::default()
                .fg(if active {
                    if conflict {
                        theme::ERROR
                    } else {
                        theme::WARNING
                    }
                } else {
                    theme::NEUTRAL_GRAY
                })
                .bg(pane_bg),
        ),
        Span::styled(" ", Style::default().bg(pane_bg)),
        Span::styled(
            action.to_owned(),
            Style::default().fg(pane_text_color(active)).bg(pane_bg),
        ),
        Span::styled(padding, Style::default().bg(pane_bg)),
    ])
}

fn config_row_line(
    label: &str,
    value: &str,
    selected: bool,
    active: bool,
    width: usize,
) -> Line<'static> {
    let pane_bg = pane_background(active);
    let label = format!(" {label:<22}");
    let text_width = UnicodeWidthStr::width(label.as_str()) + UnicodeWidthStr::width(value);
    let padding = " ".repeat(width.saturating_sub(text_width));

    if selected {
        let style = selected_style(true);
        return Line::from(vec![
            Span::styled(label, style),
            Span::styled(value.to_owned(), style),
            Span::styled(padding, style),
        ]);
    }

    Line::from(vec![
        Span::styled(
            label,
            Style::default().fg(pane_text_color(active)).bg(pane_bg),
        ),
        Span::styled(
            value.to_owned(),
            Style::default()
                .fg(if active {
                    theme::NEUTRAL_GRAY
                } else {
                    theme::NEUTRAL_GRAY
                })
                .bg(pane_bg),
        ),
        Span::styled(padding, Style::default().bg(pane_bg)),
    ])
}

pub fn footer_text(main: &MainState) -> &'static str {
    if main.settings_search_active {
        " Type to search - Enter keep filter - Esc clear "
    } else {
        " Tab switch pane - Left/Right pan - Ctrl+Left reset pan - / search - Up/Down move - Enter edit config - Esc or ? close "
    }
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

fn pane_background(active: bool) -> ratatui::style::Color {
    if active {
        theme::PANEL_ACTIVE_BG
    } else {
        theme::PANEL_INACTIVE_BG
    }
}

fn pane_text_color(active: bool) -> ratatui::style::Color {
    if active {
        theme::NEUTRAL_WHITE
    } else {
        theme::NEUTRAL_GRAY
    }
}

fn fill_area(frame: &mut Frame, area: Rect, bg: ratatui::style::Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let line = " ".repeat(area.width as usize);
    let lines: Vec<Line<'static>> = (0..area.height)
        .map(|_| Line::from(Span::styled(line.clone(), Style::default().bg(bg))))
        .collect();
    frame.render_widget(Paragraph::new(lines), area);
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
    let query = main.settings_search_query.trim().to_lowercase();
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
    let query = main.settings_search_query.trim().to_lowercase();
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

fn pad_to_width(value: &str, width: usize) -> String {
    let current = UnicodeWidthStr::width(value);
    if current >= width {
        value.to_owned()
    } else {
        format!("{value}{}", " ".repeat(width - current))
    }
}

pub fn truncate_with_ellipsis(value: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(value) <= max_width {
        return value.to_owned();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "…".to_owned();
    }
    let mut out = String::new();
    let mut width = 0usize;
    for ch in value.chars() {
        let ch_width = UnicodeWidthStr::width(ch.to_string().as_str());
        if width + ch_width + 1 > max_width {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push('…');
    out
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
        main.settings_search_query = "flash-lite".into();
        let rows = filtered_config_rows(&main);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn shared_filter_applies_to_keybinds_and_config() {
        let mut main = main_with_plugin();
        main.settings_search_query = "quiz-ai-extension".into();
        assert!(
            filtered_keybind_rows(&main)
                .iter()
                .any(|row| row.source == "quiz-ai-extension")
        );
        assert!(
            filtered_config_rows(&main)
                .iter()
                .any(|row| row.group_id == "quiz-ai-extension")
        );
    }

    #[test]
    fn includes_plugin_keybind_rows() {
        let main = main_with_plugin();
        let rows = keybind_rows(&main);
        assert!(rows.iter().any(|row| row.source == "quiz-ai-extension"));
    }

    #[test]
    fn truncates_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("abcdef", 4), "abc…");
        assert_eq!(truncate_with_ellipsis("abcdef", 1), "…");
        assert_eq!(truncate_with_ellipsis("abc", 4), "abc");
    }

    #[test]
    fn long_keybind_label_has_bounded_horizontal_scroll() {
        let mut main = main_with_plugin();
        main.plugin_registry.plugins[0]
            .manifest
            .contributes
            .quiz_actions[0]
            .title =
            "Very long plugin keybind label that should continue past the visible pane".into();

        let max = max_horizontal_scroll(&main, SettingsPane::Keybinds, 32);

        assert!(max > 0);
        assert_eq!(bounded_horizontal_scroll(0, 200, max), max);
        assert_eq!(bounded_horizontal_scroll(max, -200, max), 0);
    }

    #[test]
    fn short_rows_have_no_horizontal_scroll() {
        let main = MainState::default();

        assert_eq!(max_horizontal_scroll(&main, SettingsPane::Keybinds, 500), 0);
        assert_eq!(max_horizontal_scroll(&main, SettingsPane::Config, 500), 0);
    }

    #[test]
    fn settings_pane_width_uses_direct_split_without_divider() {
        assert_eq!(settings_pane_width(100, SettingsPane::Keybinds), 52);
        assert_eq!(settings_pane_width(100, SettingsPane::Config), 48);
    }

    #[test]
    fn selected_settings_rows_fill_visible_width() {
        let keybind = keybind_row_line("Enter", "Edit config", false, true, true, 40);
        assert!(
            keybind
                .spans
                .iter()
                .all(|span| span.style.bg == Some(theme::PANEL_SELECTED))
        );

        let config = config_row_line("Enabled", "yes", true, true, 40);
        assert!(
            config
                .spans
                .iter()
                .all(|span| span.style.bg == Some(theme::PANEL_SELECTED))
        );
    }

    #[test]
    fn inactive_settings_rows_are_dimmed() {
        let keybind = keybind_row_line("Enter", "Edit config", false, false, false, 40);
        assert_eq!(keybind.spans[0].style.fg, Some(theme::NEUTRAL_GRAY));
        assert_eq!(keybind.spans[2].style.fg, Some(theme::NEUTRAL_GRAY));

        let config = config_row_line("Enabled", "yes", false, false, 40);
        assert_eq!(config.spans[0].style.fg, Some(theme::NEUTRAL_GRAY));
        assert_eq!(config.spans[1].style.fg, Some(theme::NEUTRAL_GRAY));
    }

    #[test]
    fn inactive_pane_cursor_match_is_not_selected() {
        let keybind = keybind_row_line("Enter", "Edit config", false, false, false, 40);
        assert!(
            keybind
                .spans
                .iter()
                .all(|span| span.style.bg != Some(theme::PANEL_SELECTED))
        );

        let config = config_row_line("Enabled", "yes", false, false, 40);
        assert!(
            config
                .spans
                .iter()
                .all(|span| span.style.bg != Some(theme::PANEL_SELECTED))
        );
    }

    #[test]
    fn config_values_contribute_to_horizontal_scroll() {
        let mut main = main_with_plugin();
        main.plugin_registry.plugins[0].directory =
            "C:/a/very/long/plugin/path/that/exceeds/the/settings/pane".into();

        let max = max_horizontal_scroll(&main, SettingsPane::Config, 30);

        assert!(max > 0);
    }

    #[test]
    fn horizontal_continuation_indicates_hidden_sides() {
        assert_eq!(horizontal_continuation(0, 0), (false, false));
        assert_eq!(horizontal_continuation(0, 12), (false, true));
        assert_eq!(horizontal_continuation(4, 12), (true, true));
        assert_eq!(horizontal_continuation(12, 12), (true, false));
    }

    #[test]
    fn scrolled_visible_width_includes_horizontal_offset() {
        assert_eq!(scrolled_visible_width(40, 0), 40);
        assert_eq!(scrolled_visible_width(40, 12), 52);
    }
}
