use crate::app::state::types::{
    AppCommand, AppState, AssignmentModalData, CourseView, DashboardPane, LinkAction, LoginFocus,
    MainState, Screen,
};
use crate::moodle::urls::{build_assignment_activity_url, build_course_view_url};
use crate::ui::course_tree::{
    CourseTreeNodeKind, CourseTreeRow, build_course_tree_rows,
};
use crate::models::RuntimeConfig;
use crate::shortcuts::is_shortcut_pressed;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use tui_components::input::login::{handle_login_key as handle_shared_login_key, LoginKeyBindings, LoginKeyOutcome};

impl AppState {
    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<AppCommand> {
        match &mut self.screen {
            Screen::Loading => Vec::new(),
            Screen::Login(_) => handle_login_key(self, key),
            Screen::MainShell(_) => handle_main_key(self, key),
        }
    }

    pub fn handle_mouse(&mut self, _mouse: MouseEvent) -> Vec<AppCommand> {
        Vec::new()
    }
}

fn handle_login_key(state: &mut AppState, key: KeyEvent) -> Vec<AppCommand> {
    let login = match &mut state.screen {
        Screen::Login(login) => login,
        _ => return Vec::new(),
    };

    if login.busy {
        return Vec::new();
    }

    let outcome = match login.focus {
        LoginFocus::BaseUrl => handle_shared_login_key(
            key,
            &mut login.focus,
            LoginFocus::Password,
            LoginFocus::Submit,
            Some(&mut login.base_url),
            LoginFocus::next,
            LoginFocus::prev,
            LoginKeyBindings::default(),
        ),
        LoginFocus::Username => handle_shared_login_key(
            key,
            &mut login.focus,
            LoginFocus::Password,
            LoginFocus::Submit,
            Some(&mut login.username),
            LoginFocus::next,
            LoginFocus::prev,
            LoginKeyBindings::default(),
        ),
        LoginFocus::Password => handle_shared_login_key(
            key,
            &mut login.focus,
            LoginFocus::Password,
            LoginFocus::Submit,
            Some(&mut login.password),
            LoginFocus::next,
            LoginFocus::prev,
            LoginKeyBindings::default(),
        ),
        LoginFocus::Service => handle_shared_login_key(
            key,
            &mut login.focus,
            LoginFocus::Password,
            LoginFocus::Submit,
            Some(&mut login.service),
            LoginFocus::next,
            LoginFocus::prev,
            LoginKeyBindings::default(),
        ),
        LoginFocus::Submit => handle_shared_login_key(
            key,
            &mut login.focus,
            LoginFocus::Password,
            LoginFocus::Submit,
            None,
            LoginFocus::next,
            LoginFocus::prev,
            LoginKeyBindings::default(),
        ),
    };

    match outcome {
        LoginKeyOutcome::Submit => {
            let config = RuntimeConfig {
                base_url: login.base_url.value.clone(),
                username: login.username.value.clone(),
                service: login.service.value.clone(),
                password: login.password.value.clone(),
            };
            login.busy = true;
            login.error = None;
            vec![AppCommand::ValidateLogin(config)]
        }
        LoginKeyOutcome::SavedLogin => {
            if let (Some(saved), Some(password)) = (state.saved_config.clone(), state.saved_password.clone()) {
                let config = RuntimeConfig {
                    base_url: saved.base_url,
                    username: saved.username,
                    service: saved.service,
                    password,
                };
                login.busy = true;
                login.error = None;
                vec![AppCommand::ValidateLogin(config)]
            } else {
                Vec::new()
            }
        }
        LoginKeyOutcome::TogglePassword => {
            login.show_password = !login.show_password;
            login.password.mask = !login.show_password;
            Vec::new()
        }
        LoginKeyOutcome::Quit => vec![AppCommand::Quit],
        _ => Vec::new(),
    }
}

fn handle_main_key(state: &mut AppState, key: KeyEvent) -> Vec<AppCommand> {
    let main = match &mut state.screen {
        Screen::MainShell(main) => main,
        _ => return Vec::new(),
    };

    if main.settings_open {
        if is_shortcut_pressed("settings-close", key) {
            main.settings_open = false;
            main.settings_scroll = 0;
            return Vec::new();
        }
        let total = settings_total_lines();
        let (term_w, term_h) = state.terminal_size;
        let modal_h = ((term_h as f32) * 0.8) as u16;
        let viewport = modal_h.saturating_sub(3);
        let max_scroll = total.saturating_sub(viewport);
        let _ = term_w;
        match key.code {
            crossterm::event::KeyCode::Up => {
                main.settings_scroll = main.settings_scroll.min(max_scroll).saturating_sub(1);
            }
            crossterm::event::KeyCode::Down => {
                main.settings_scroll = (main.settings_scroll + 1).min(max_scroll);
            }
            crossterm::event::KeyCode::PageUp => {
                main.settings_scroll = main.settings_scroll.min(max_scroll).saturating_sub(10);
            }
            crossterm::event::KeyCode::PageDown => {
                main.settings_scroll = (main.settings_scroll + 10).min(max_scroll);
            }
            crossterm::event::KeyCode::Home => main.settings_scroll = 0,
            crossterm::event::KeyCode::End => main.settings_scroll = max_scroll,
            _ => {}
        }
        return Vec::new();
    }

    if main.assignment_modal.is_some() {
        return handle_assignment_modal_key(main, key);
    }

    if main.course_finder_open {
        return handle_finder_key(main, key, true);
    }
    if main.content_finder_open {
        return handle_finder_key(main, key, false);
    }

    if let CourseView::Course(_) = &main.view {
        if let Some(commands) = handle_course_view_key(main, key) {
            return commands;
        }
    }

    if is_shortcut_pressed("quit", key) {
        return vec![AppCommand::Quit];
    }
    if is_shortcut_pressed("settings-open", key) {
        main.settings_open = true;
        return Vec::new();
    }
    if is_shortcut_pressed("logout", key) {
        let saved_config = main.config.as_ref().map(|config| config.saved());
        let password = main.config.as_ref().map(|config| config.password.clone());
        let storage_warning = state.storage_warning.clone();
        return vec![AppCommand::Logout {
            saved_config,
            password,
            storage_warning,
        }];
    }
    if is_shortcut_pressed("dashboard-refresh", key) {
        if let Some(config) = main.config.clone() {
            return match &main.view {
                CourseView::Dashboard => vec![AppCommand::LoadDashboard(config)],
                CourseView::Course(course) => {
                    vec![AppCommand::LoadCoursePage(config, course.course_id)]
                }
            };
        }
    }
    if is_shortcut_pressed("dashboard-open-finder", key) {
        main.course_finder_open = true;
        main.finder_query = Default::default();
        main.finder_selected = 0;
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-open-content-finder", key)
        && matches!(main.view, CourseView::Course(_))
    {
        main.content_finder_open = true;
        main.finder_query = Default::default();
        main.finder_selected = 0;
        main.finder_target_idx = 0;
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-back", key) {
        if matches!(main.view, CourseView::Course(_)) {
            main.view = CourseView::Dashboard;
            main.selected_row = 0;
        }
        return Vec::new();
    }
    if key.code == crossterm::event::KeyCode::Tab
        && matches!(main.view, CourseView::Dashboard)
    {
        main.dashboard_focus = main.dashboard_focus.toggle();
        main.selected_row = 0;
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-open-assignment-modal", key)
        && main.dashboard_focus == DashboardPane::Upcoming
    {
        if let Some(assignment) = main.dashboard.upcoming.get(main.selected_row).cloned() {
            return open_modal_for_upcoming(main, assignment);
        }
        return Vec::new();
    }
    if (is_shortcut_pressed("dashboard-open-link", key)
        || is_shortcut_pressed("dashboard-copy-link", key))
        && matches!(main.view, CourseView::Dashboard)
    {
        let action = if is_shortcut_pressed("dashboard-open-link", key) {
            LinkAction::Open
        } else {
            LinkAction::Copy
        };
        return match main.dashboard_focus {
            DashboardPane::Upcoming => resolve_upcoming_link_action(main, action),
            DashboardPane::Courses => resolve_course_link_action(main, action),
        };
    }
    if key.code == crossterm::event::KeyCode::Enter
        && main.dashboard_focus == DashboardPane::Courses
    {
        if let Some(course) = main.dashboard.courses.get(main.selected_row).cloned() {
            let course_id = course.id;
            main.view = CourseView::Course(crate::app::state::types::CoursePageData {
                course_id,
                course_short_name: course.shortname.clone(),
                course_full_name: course.fullname.clone(),
                loading: true,
                ..Default::default()
            });
            if let Some(config) = main.config.clone() {
                return vec![AppCommand::LoadCoursePage(config, course_id)];
            }
        }
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-up", key) {
        main.selected_row = main.selected_row.saturating_sub(1);
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-down", key) {
        let max = match main.dashboard_focus {
            DashboardPane::Upcoming => main.dashboard.upcoming.len(),
            DashboardPane::Courses => main.dashboard.courses.len(),
        }
        .saturating_sub(1);
        if main.selected_row < max {
            main.selected_row += 1;
        }
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-home", key) {
        main.selected_row = 0;
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-end", key) {
        main.selected_row = match main.dashboard_focus {
            DashboardPane::Upcoming => main.dashboard.upcoming.len(),
            DashboardPane::Courses => main.dashboard.courses.len(),
        }
        .saturating_sub(1);
        return Vec::new();
    }

    Vec::new()
}

fn resolve_course_link_action(main: &MainState, action: LinkAction) -> Vec<AppCommand> {
    let course = match main.dashboard.courses.get(main.selected_row) {
        Some(c) => c,
        None => return vec![AppCommand::ShowToast("No link available on this item.".into())],
    };
    let url = course.courseurl.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| {
        let base = main
            .config
            .as_ref()
            .map(|c| c.base_url.as_str())
            .unwrap_or("");
        build_course_view_url(base, course.id)
    });
    match action {
        LinkAction::Open => vec![AppCommand::OpenUrl(url)],
        LinkAction::Copy => vec![AppCommand::CopyToClipboard(url)],
    }
}

fn resolve_upcoming_link_action(main: &mut MainState, action: LinkAction) -> Vec<AppCommand> {
    let assignment = match main.dashboard.upcoming.get(main.selected_row).cloned() {
        Some(a) => a,
        None => {
            return vec![AppCommand::ShowToast("No link available on this item.".into())];
        }
    };
    if let Some(list) = main.assignment_list_by_course_id.get(&assignment.course_id) {
        if let Some(detail) = list.iter().find(|a| a.id == assignment.id) {
            let base = main
                .config
                .as_ref()
                .map(|c| c.base_url.as_str())
                .unwrap_or("");
            let url = build_assignment_activity_url(base, detail.cmid);
            return match action {
                LinkAction::Open => vec![AppCommand::OpenUrl(url)],
                LinkAction::Copy => vec![AppCommand::CopyToClipboard(url)],
            };
        }
    }
    let config = match main.config.clone() {
        Some(c) => c,
        None => return vec![AppCommand::ShowToast("No link available on this item.".into())],
    };
    vec![AppCommand::ResolveUpcomingLink {
        config,
        upcoming_id: assignment.id,
        course_id: assignment.course_id,
        action,
    }]
}

fn open_modal_for_upcoming(
    main: &mut MainState,
    assignment: crate::models::UpcomingAssignment,
) -> Vec<AppCommand> {
    let course_name = assignment
        .course_full_name
        .clone()
        .or_else(|| assignment.course_short_name.clone())
        .unwrap_or_default();
    main.assignment_modal = Some(AssignmentModalData {
        course_id: assignment.course_id,
        assignment_id: assignment.id,
        assignment_name: assignment.name.clone(),
        course_name,
        due_date: assignment.due_date,
        module_description: None,
        module_url: None,
        status: None,
        status_loading: true,
        status_error: None,
        detail: None,
        detail_loading: true,
        detail_error: None,
        loading: true,
        error: None,
    });
    if let Some(config) = main.config.clone() {
        return vec![
            AppCommand::LoadAssignmentDetail(config.clone(), assignment.course_id, assignment.id),
            AppCommand::LoadAssignmentStatus(config, assignment.id),
        ];
    }
    Vec::new()
}

fn handle_course_view_key(main: &mut MainState, key: KeyEvent) -> Option<Vec<AppCommand>> {
    let (rows, selected) = {
        let course = match &main.view {
            CourseView::Course(c) => c,
            _ => return None,
        };
        let rows = build_course_tree_rows(&course.sections, &course.collapsed);
        if rows.is_empty() {
            return None;
        }
        let selected = course.selected_row.min(rows.len() - 1);
        (rows, selected)
    };
    let course = match &mut main.view {
        CourseView::Course(c) => c,
        _ => return None,
    };

    if is_shortcut_pressed("dashboard-up", key) {
        course.selected_row = selected.saturating_sub(1);
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-down", key) {
        if selected + 1 < rows.len() {
            course.selected_row = selected + 1;
        }
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-home", key) {
        course.selected_row = 0;
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-end", key) {
        course.selected_row = rows.len() - 1;
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-page-up", key) {
        course.selected_row = selected.saturating_sub(10);
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-page-down", key) {
        course.selected_row = (selected + 10).min(rows.len() - 1);
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-expand", key) {
        let row = &rows[selected];
        if row.collapsible && !row.expanded {
            course.collapsed.remove(&row.id);
        }
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-collapse", key) {
        let row = &rows[selected];
        if row.collapsible && row.expanded {
            course.collapsed.insert(row.id.clone());
        } else if let Some(parent_id) = row.parent_id.clone() {
            if let Some(parent_idx) = rows.iter().position(|r| r.id == parent_id) {
                course.selected_row = parent_idx;
            }
        }
        return Some(Vec::new());
    }
    if is_shortcut_pressed("dashboard-open-link", key) {
        return Some(match rows[selected].link_url.clone() {
            Some(url) => vec![AppCommand::OpenUrl(url)],
            None => vec![AppCommand::ShowToast("No link available on this item.".into())],
        });
    }
    if is_shortcut_pressed("dashboard-copy-link", key) {
        return Some(match rows[selected].link_url.clone() {
            Some(url) => vec![AppCommand::CopyToClipboard(url)],
            None => vec![AppCommand::ShowToast("No link available on this item.".into())],
        });
    }
    if is_shortcut_pressed("dashboard-open-assignment-modal", key) {
        let row = &rows[selected];
        if matches!(row.kind, CourseTreeNodeKind::Module)
            && row.module_type.as_deref() == Some("assign")
        {
            return Some(open_modal_for_course_module(main, &rows, selected));
        }
        return Some(Vec::new());
    }
    None
}

fn open_modal_for_course_module(
    main: &mut MainState,
    rows: &[CourseTreeRow],
    selected: usize,
) -> Vec<AppCommand> {
    let row = &rows[selected];
    let course = match &main.view {
        CourseView::Course(c) => c,
        _ => return Vec::new(),
    };
    let course_id = course.course_id;
    let course_name = if !course.course_full_name.is_empty() {
        course.course_full_name.clone()
    } else {
        course.course_short_name.clone()
    };
    let parts: Vec<&str> = row.id.split(':').collect();
    let section_id: Option<i64> = parts.get(1).and_then(|s| s.parse().ok());
    let cmid: Option<i64> = parts.get(2).and_then(|s| s.parse().ok());
    let (Some(section_id), Some(cmid)) = (section_id, cmid) else {
        return Vec::new();
    };
    let module = course
        .sections
        .iter()
        .find(|s| s.id == section_id)
        .and_then(|s| s.modules.iter().find(|m| m.id == cmid));
    let assign_id = match module.and_then(|m| m.instance) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let description = rows
        .iter()
        .find(|r| {
            matches!(r.kind, CourseTreeNodeKind::ModuleDescription)
                && r.parent_id.as_deref() == Some(row.id.as_str())
        })
        .map(|r| r.text.clone());
    main.assignment_modal = Some(AssignmentModalData {
        course_id,
        assignment_id: assign_id,
        assignment_name: row.text.clone(),
        course_name,
        due_date: 0,
        module_description: description,
        module_url: row.link_url.clone(),
        status: None,
        status_loading: true,
        status_error: None,
        detail: None,
        detail_loading: true,
        detail_error: None,
        loading: true,
        error: None,
    });
    if let Some(config) = main.config.clone() {
        return vec![
            AppCommand::LoadAssignmentDetail(config.clone(), course_id, assign_id),
            AppCommand::LoadAssignmentStatus(config, assign_id),
        ];
    }
    Vec::new()
}

fn settings_total_lines() -> u16 {
    use crate::shortcuts::{TabId, get_shortcut_sections};
    let mut count = 0u16;
    for section in get_shortcut_sections(TabId::Dashboard) {
        count += 1;
        count += section.items.len() as u16;
        count += 1;
    }
    count
}

fn handle_assignment_modal_key(main: &mut MainState, key: KeyEvent) -> Vec<AppCommand> {
    if is_shortcut_pressed("assignment-modal-close", key) {
        main.assignment_modal = None;
        return Vec::new();
    }
    if is_shortcut_pressed("dashboard-open-link", key)
        || is_shortcut_pressed("dashboard-copy-link", key)
    {
        let action = if is_shortcut_pressed("dashboard-open-link", key) {
            LinkAction::Open
        } else {
            LinkAction::Copy
        };
        let modal = match &main.assignment_modal {
            Some(m) => m,
            None => return Vec::new(),
        };
        let base = main.config.as_ref().map(|c| c.base_url.as_str()).unwrap_or("");
        let url = modal
            .module_url
            .clone()
            .or_else(|| modal.detail.as_ref().map(|d| build_assignment_activity_url(base, d.cmid)));
        return match url {
            Some(url) => match action {
                LinkAction::Open => vec![AppCommand::OpenUrl(url)],
                LinkAction::Copy => vec![AppCommand::CopyToClipboard(url)],
            },
            None => vec![AppCommand::ShowToast("No link available on this item.".into())],
        };
    }
    Vec::new()
}

fn handle_finder_key(main: &mut MainState, key: KeyEvent, is_course_finder: bool) -> Vec<AppCommand> {
    use crate::search::courses::filter_courses;

    let cancel_id = if is_course_finder {
        "course-finder-cancel"
    } else {
        "course-content-finder-cancel"
    };
    let submit_id = if is_course_finder {
        "course-finder-submit"
    } else {
        "course-content-finder-submit"
    };
    if is_shortcut_pressed(cancel_id, key) {
        if is_course_finder {
            main.course_finder_open = false;
        } else {
            main.content_finder_open = false;
        }
        return Vec::new();
    }
    if is_shortcut_pressed(submit_id, key) && is_course_finder {
        let filtered = filter_courses(&main.dashboard.courses, &main.finder_query.value);
        if let Some(course) = filtered.get(main.finder_selected).copied() {
            main.course_finder_open = false;
            let course_id = course.id;
            main.view = CourseView::Course(crate::app::state::types::CoursePageData {
                course_id,
                course_short_name: course.shortname.clone(),
                course_full_name: course.fullname.clone(),
                loading: true,
                ..Default::default()
            });
            main.selected_row = 0;
            if let Some(config) = main.config.clone() {
                return vec![AppCommand::LoadCoursePage(config, course_id)];
            }
        }
        return Vec::new();
    }
    if is_shortcut_pressed(submit_id, key) && !is_course_finder {
        if let CourseView::Course(course) = &main.view {
            let rows = build_course_tree_rows(&course.sections, &course.collapsed);
            let targets = crate::ui::content_finder::build_targets(&rows);
            let target = &targets[main.finder_target_idx.min(targets.len() - 1)];
            let by_target = crate::ui::content_finder::filter_by_target(&rows, target);
            let q = main.finder_query.value.to_lowercase();
            let filtered: Vec<&CourseTreeRow> = if q.trim().is_empty() {
                by_target
            } else {
                by_target
                    .into_iter()
                    .filter(|r| r.text.to_lowercase().contains(&q))
                    .collect()
            };
            if let Some(picked) = filtered.get(main.finder_selected) {
                let picked_id = picked.id.clone();
                main.content_finder_open = false;
                if let Some(idx) = rows.iter().position(|r| r.id == picked_id) {
                    if let CourseView::Course(course_mut) = &mut main.view {
                        course_mut.selected_row = idx;
                    }
                }
            }
        }
        return Vec::new();
    }
    if !is_course_finder
        && (is_shortcut_pressed("course-content-finder-target-prev", key)
            || is_shortcut_pressed("course-content-finder-target-next", key))
    {
        if let CourseView::Course(course) = &main.view {
            let rows = build_course_tree_rows(&course.sections, &course.collapsed);
            let targets = crate::ui::content_finder::build_targets(&rows);
            let delta: isize =
                if is_shortcut_pressed("course-content-finder-target-prev", key) { -1 } else { 1 };
            main.finder_target_idx =
                crate::ui::content_finder::cycle(main.finder_target_idx, delta, targets.len());
            main.finder_selected = 0;
        }
        return Vec::new();
    }

    match key.code {
        KeyCode::Up => {
            main.finder_selected = main.finder_selected.saturating_sub(1);
            return Vec::new();
        }
        KeyCode::Down => {
            let max = if is_course_finder {
                filter_courses(&main.dashboard.courses, &main.finder_query.value).len()
            } else if let CourseView::Course(course) = &main.view {
                let rows = build_course_tree_rows(&course.sections, &course.collapsed);
                let targets = crate::ui::content_finder::build_targets(&rows);
                let target = &targets[main.finder_target_idx.min(targets.len() - 1)];
                let by_target = crate::ui::content_finder::filter_by_target(&rows, target);
                let q = main.finder_query.value.to_lowercase();
                by_target
                    .into_iter()
                    .filter(|r| q.trim().is_empty() || r.text.to_lowercase().contains(&q))
                    .count()
            } else {
                0
            };
            if main.finder_selected + 1 < max {
                main.finder_selected += 1;
            }
            return Vec::new();
        }
        KeyCode::Char(ch) => {
            main.finder_query.insert(ch);
            main.finder_selected = 0;
        }
        KeyCode::Backspace => {
            main.finder_query.backspace();
            main.finder_selected = 0;
        }
        KeyCode::Left => main.finder_query.move_left(),
        KeyCode::Right => main.finder_query.move_right(),
        _ => {}
    }
    Vec::new()
}
