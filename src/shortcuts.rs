use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    Dashboard,
}

#[derive(Debug, Clone)]
pub struct ShortcutDisplay {
    pub id: &'static str,
    pub keys: &'static str,
    pub action: &'static str,
}

#[derive(Debug, Clone)]
pub struct ShortcutSection {
    pub title: &'static str,
    pub items: Vec<ShortcutDisplay>,
}

fn char_key(key: KeyEvent, expected: char) -> bool {
    matches!(key.code, KeyCode::Char(value) if value == expected)
}

fn plain_char(key: KeyEvent, expected: char) -> bool {
    char_key(key, expected)
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
}

fn shifted_char(key: KeyEvent, expected: char) -> bool {
    char_key(key, expected) && key.modifiers.contains(KeyModifiers::SHIFT)
}

pub fn is_shortcut_pressed(id: &str, key: KeyEvent) -> bool {
    match id {
        "settings-open" => plain_char(key, '?'),
        "settings-close" => key.code == KeyCode::Esc || plain_char(key, '?'),
        "quit" => plain_char(key, 'q'),
        "logout" => plain_char(key, 'l'),
        "dashboard-refresh" => plain_char(key, 'r'),
        "dashboard-open-finder" => plain_char(key, '/'),
        "dashboard-open-content-finder" => plain_char(key, 'f'),
        "dashboard-open-assignment-modal" => key.code == KeyCode::Enter,
        "dashboard-copy-link" => plain_char(key, 'c'),
        "dashboard-open-link" => char_key(key, 'C') || shifted_char(key, 'c'),
        "dashboard-back" => key.code == KeyCode::Esc,
        "dashboard-up" => key.code == KeyCode::Up,
        "dashboard-down" => key.code == KeyCode::Down,
        "dashboard-expand" => key.code == KeyCode::Right,
        "dashboard-collapse" => key.code == KeyCode::Left,
        "dashboard-page-up" => key.code == KeyCode::PageUp,
        "dashboard-page-down" => key.code == KeyCode::PageDown,
        "dashboard-home" => key.code == KeyCode::Home,
        "dashboard-end" => key.code == KeyCode::End,
        "course-finder-submit" => key.code == KeyCode::Enter,
        "course-finder-cancel" => key.code == KeyCode::Esc,
        "course-content-finder-submit" => key.code == KeyCode::Enter,
        "course-content-finder-target-prev" => key.code == KeyCode::Left,
        "course-content-finder-target-next" => key.code == KeyCode::Right,
        "course-content-finder-cancel" => key.code == KeyCode::Esc,
        "assignment-modal-close" => key.code == KeyCode::Esc,
        _ => false,
    }
}

fn d(id: &'static str, keys: &'static str, action: &'static str) -> ShortcutDisplay {
    ShortcutDisplay { id, keys, action }
}

fn pick(ids: &[&'static str]) -> Vec<ShortcutDisplay> {
    ids.iter()
        .map(|id| match *id {
            "settings-open" => d(id, "?", "Open settings/help"),
            "settings-close" => d(id, "Esc or ?", "Close settings/help"),
            "quit" => d(id, "q", "Quit app"),
            "logout" => d(id, "l", "Logout and clear saved credentials"),
            "dashboard-refresh" => d(id, "r", "Refresh dashboard or active course page"),
            "dashboard-open-finder" => d(id, "/", "Open course finder"),
            "dashboard-open-content-finder" => d(id, "f", "Open course content finder"),
            "dashboard-open-assignment-modal" => d(id, "Enter", "Open selected assignment details"),
            "dashboard-copy-link" => d(id, "c", "Copy selected link to clipboard"),
            "dashboard-open-link" => d(id, "Shift+C", "Open selected link in browser"),
            "dashboard-back" => d(id, "Esc", "Back to dashboard"),
            "dashboard-up" => d(id, "Up", "Scroll up"),
            "dashboard-down" => d(id, "Down", "Scroll down"),
            "dashboard-expand" => d(id, "Right", "Expand selected node or move to first child"),
            "dashboard-collapse" => d(id, "Left", "Collapse selected node or move to parent"),
            "dashboard-page-up" => d(id, "PageUp", "Jump up by one page"),
            "dashboard-page-down" => d(id, "PageDown", "Jump down by one page"),
            "dashboard-home" => d(id, "Home", "Jump to top"),
            "dashboard-end" => d(id, "End", "Jump to bottom"),
            "course-finder-submit" => d(id, "Enter", "Open highlighted course"),
            "course-finder-cancel" => d(id, "Esc", "Close course finder"),
            "course-content-finder-submit" => d(id, "Enter", "Jump to highlighted content"),
            "course-content-finder-target-prev" => d(id, "Left", "Target previous content type"),
            "course-content-finder-target-next" => d(id, "Right", "Target next content type"),
            "course-content-finder-cancel" => d(id, "Esc", "Close content finder"),
            "assignment-modal-close" => d(id, "Esc", "Close assignment details modal"),
            _ => d(id, "", ""),
        })
        .collect()
}

pub fn get_shortcut_sections(_active_tab: TabId) -> Vec<ShortcutSection> {
    vec![
        ShortcutSection {
            title: "Global",
            items: pick(&["settings-open", "dashboard-refresh", "logout", "quit"]),
        },
        ShortcutSection {
            title: "Settings Modal",
            items: pick(&["settings-close"]),
        },
        ShortcutSection {
            title: "Dashboard / Course Page",
            items: pick(&[
                "dashboard-open-finder",
                "dashboard-open-content-finder",
                "dashboard-open-assignment-modal",
                "dashboard-copy-link",
                "dashboard-open-link",
                "dashboard-back",
                "dashboard-up",
                "dashboard-down",
                "dashboard-expand",
                "dashboard-collapse",
                "dashboard-page-up",
                "dashboard-page-down",
                "dashboard-home",
                "dashboard-end",
            ]),
        },
        ShortcutSection {
            title: "Course Finder",
            items: pick(&["course-finder-submit", "course-finder-cancel"]),
        },
        ShortcutSection {
            title: "Course Content Finder",
            items: pick(&[
                "course-content-finder-submit",
                "course-content-finder-target-prev",
                "course-content-finder-target-next",
                "course-content-finder-cancel",
            ]),
        },
        ShortcutSection {
            title: "Assignment Modal",
            items: pick(&[
                "dashboard-copy-link",
                "dashboard-open-link",
                "assignment-modal-close",
            ]),
        },
    ]
}
