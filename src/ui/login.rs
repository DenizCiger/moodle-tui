use crate::app::state::AppState;
use crate::app::state::types::{LoginFocus, LoginState};
use crate::ui::theme;
use ratatui::Frame;
use tui_components::ui::login::{LoginFieldView, LoginModal};
use tui_components::ui::theme::Theme;

pub fn render(frame: &mut Frame, login: &LoginState, state: &AppState) {
    LoginModal {
        title: "moodle-tui — Login",
        help_lines: Vec::new(),
        fields: vec![
            LoginFieldView {
                label: "Base URL",
                value: &login.base_url.value,
                placeholder: "https://moodle.example.edu",
                focused: login.focus == LoginFocus::BaseUrl,
                masked: false,
            },
            LoginFieldView {
                label: "Username",
                value: &login.username.value,
                placeholder: "Moodle username",
                focused: login.focus == LoginFocus::Username,
                masked: false,
            },
            LoginFieldView {
                label: "Password",
                value: &login.password.value,
                placeholder: "Moodle password",
                focused: login.focus == LoginFocus::Password,
                masked: !login.show_password,
            },
            LoginFieldView {
                label: "Service",
                value: &login.service.value,
                placeholder: crate::models::DEFAULT_MOODLE_SERVICE,
                focused: login.focus == LoginFocus::Service,
                masked: false,
            },
        ],
        submit_focused: login.focus == LoginFocus::Submit,
        saved_account: state
            .saved_config
            .as_ref()
            .filter(|_| state.saved_password.is_some())
            .map(|saved| format!("{}@{}", saved.username, saved.base_url)),
        error: login.error.as_deref(),
        warning: login.storage_warning.as_deref(),
        busy: login.busy,
        busy_label: "Logging in...",
        submit_label: "Submit",
        footer: "Tab/Shift+Tab or ↑/↓ fields · Enter submit
Alt+V show password · Ctrl+L saved login · Esc quit",
        width: 72,
        min_height: 18,
    }
    .render(frame, frame.area(), app_theme());
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
