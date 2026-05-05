use ratatui::style::Color;

pub const BRAND: Color = Color::Indexed(45);
pub const WARNING: Color = Color::Indexed(220);
pub const ERROR: Color = Color::Indexed(196);
pub const SUCCESS: Color = Color::Indexed(84);

pub const NEUTRAL_WHITE: Color = Color::Indexed(15);
pub const NEUTRAL_BLACK: Color = Color::Indexed(16);
pub const NEUTRAL_GRAY: Color = Color::Indexed(244);
pub const NEUTRAL_BRIGHT_BLACK: Color = Color::Indexed(240);

pub const PANEL_HEADER: Color = Color::Indexed(238);
pub const PANEL_SELECTED: Color = Color::Indexed(24);
pub const PANEL_ALTERNATE: Color = Color::Indexed(236);

pub fn components_theme() -> tui_components::ui::theme::Theme {
    tui_components::ui::theme::Theme {
        brand: BRAND,
        warning: WARNING,
        error: ERROR,
        success: SUCCESS,
        neutral_white: NEUTRAL_WHITE,
        neutral_black: NEUTRAL_BLACK,
        neutral_gray: NEUTRAL_GRAY,
        neutral_bright_black: NEUTRAL_BRIGHT_BLACK,
        panel_header: PANEL_HEADER,
        panel_selected: PANEL_SELECTED,
        panel_alternate: PANEL_ALTERNATE,
    }
}
