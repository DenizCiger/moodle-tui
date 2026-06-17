use crate::app::state::types::{
    AppCommand, AppState, CourseView, DashboardData, LoginState, MainState, Screen,
};
use crate::demo::{demo_courses, demo_upcoming};
use crate::models::{QuizAnswerKind, RuntimeConfig, SavedConfig};
use crate::plugins::AiFillResponse;
use crate::storage;

impl AppState {
    pub fn handle_worker_event(&mut self, event: super::types::WorkerEvent) -> Vec<AppCommand> {
        use super::types::WorkerEvent;

        match event {
            WorkerEvent::BootstrapLoaded {
                saved_config,
                password,
                storage_warning,
                plugin_registry,
            } => self.on_bootstrap(saved_config, password, storage_warning, plugin_registry),
            WorkerEvent::LoginValidated(result) => self.on_login_validated(result),
            WorkerEvent::DashboardLoaded(result) => {
                if let Screen::MainShell(main) = &mut self.screen {
                    main.dashboard.loading = false;
                    match result {
                        Ok((courses, upcoming)) => {
                            main.dashboard.courses = courses;
                            main.dashboard.upcoming = upcoming;
                            main.dashboard.error = None;
                            main.dashboard.from_cache = false;
                        }
                        Err(error) => {
                            main.dashboard.error = Some(error);
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::CoursePageLoaded { course_id, result } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let CourseView::Course(course) = &mut main.view {
                        if course.course_id == course_id {
                            course.loading = false;
                            match result {
                                Ok(sections) => {
                                    if course.sections.is_empty() {
                                        course.collapsed =
                                            crate::ui::course_tree::initial_collapsed_nodes(
                                                &sections,
                                            );
                                    }
                                    course.sections = sections;
                                    course.error = None;
                                    course.from_cache = false;
                                }
                                Err(error) => course.error = Some(error),
                            }
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::AssignmentStatusLoaded {
                assignment_id,
                result,
            } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let Some(modal) = &mut main.assignment_modal {
                        if modal.assignment_id == assignment_id {
                            modal.status_loading = false;
                            match result {
                                Ok(status) => modal.status = status,
                                Err(error) => modal.status_error = Some(error),
                            }
                            modal.loading = modal.detail_loading || modal.status_loading;
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::AssignmentDetailLoaded {
                assignment_id,
                result,
            } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let Some(modal) = &mut main.assignment_modal {
                        if modal.assignment_id == assignment_id {
                            modal.detail_loading = false;
                            match result {
                                Ok(detail) => modal.detail = detail,
                                Err(error) => modal.detail_error = Some(error),
                            }
                            modal.loading = modal.detail_loading || modal.status_loading;
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::QuizDetailLoaded {
                course_id,
                quiz_id,
                result,
            } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let Some(modal) = &mut main.quiz_modal {
                        if modal.course_id == course_id && modal.quiz_id == quiz_id {
                            match result {
                                Ok(summary) => {
                                    modal.summary = summary;
                                    modal.error = None;
                                }
                                Err(error) => modal.error = Some(error),
                            }
                            modal.loading = false;
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::QuizAttemptStarted { quiz_id, result } => {
                match result {
                    Ok(attempt) => {
                        if let Screen::MainShell(main) = &mut self.screen {
                            if let Some(modal) = &mut main.quiz_modal {
                                if modal.quiz_id == quiz_id {
                                    modal.loading = true;
                                    modal.error = None;
                                }
                            }
                            if let Some(config) = main.config.clone() {
                                return vec![AppCommand::LoadQuizAttempt(config, attempt.id)];
                            }
                        }
                    }
                    Err(error) => {
                        if let Screen::MainShell(main) = &mut self.screen {
                            if let Some(modal) = &mut main.quiz_modal {
                                if modal.quiz_id == quiz_id {
                                    modal.loading = false;
                                    modal.error = Some(error);
                                }
                            }
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::QuizAttemptLoaded { attempt_id, result } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let Some(modal) = &mut main.quiz_modal {
                        match result {
                            Ok(attempt) if attempt.attempt.id == attempt_id => {
                                modal.attempt = Some(attempt);
                                modal.loading = false;
                                modal.saving = false;
                                modal.finishing = false;
                                modal.error = None;
                                modal.selected_question = 0;
                                modal.selected_control = 0;
                                modal.selected_option = 0;
                            }
                            Ok(_) => {}
                            Err(error) => {
                                modal.loading = false;
                                modal.error = Some(error);
                            }
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::QuizAttemptSaved { attempt_id, result } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let Some(modal) = &mut main.quiz_modal {
                        modal.saving = false;
                        match result {
                            Ok(attempt) if attempt.attempt.id == attempt_id => {
                                modal.attempt = Some(attempt);
                                modal.error = None;
                                main.toast_id = main.toast_id.wrapping_add(1);
                                main.toast = Some("Quiz answers saved.".into());
                                return vec![AppCommand::ScheduleToastExpire(main.toast_id)];
                            }
                            Ok(_) => {}
                            Err(error) => modal.error = Some(error),
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::QuizAttemptFinished {
                attempt_id: _,
                result,
            } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let Some(modal) = &mut main.quiz_modal {
                        modal.finishing = false;
                        modal.confirm_finish = false;
                        match result {
                            Ok(()) => {
                                main.toast_id = main.toast_id.wrapping_add(1);
                                main.toast = Some("Quiz attempt finished.".into());
                                main.quiz_modal = None;
                                return vec![AppCommand::ScheduleToastExpire(main.toast_id)];
                            }
                            Err(error) => modal.error = Some(error),
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::PluginSettingSaved {
                plugin_id,
                setting_name,
                secret,
                result,
            } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if result.is_ok() {
                        if secret {
                            main.plugin_secret_configured
                                .insert(crate::ui::settings::setting_key(
                                    &plugin_id,
                                    &setting_name,
                                ));
                        }
                        main.api_key_input = None;
                        main.model_picker = None;
                    } else if let Err(error) = &result {
                        if let Some(input) = &mut main.api_key_input {
                            input.saving = false;
                            input.error = Some(error.clone());
                        }
                        if let Some(picker) = &mut main.model_picker {
                            picker.saving = false;
                            picker.error = Some(error.clone());
                        }
                    }
                    main.plugin_settings = load_plugin_settings_cache(&main.plugin_registry);
                    main.toast_id = main.toast_id.wrapping_add(1);
                    main.toast = Some(match result {
                        Ok(()) => format!("{plugin_id} setting saved."),
                        Err(e) => format!("Failed to save setting: {e}"),
                    });
                    return vec![AppCommand::ScheduleToastExpire(main.toast_id)];
                }
                Vec::new()
            }
            WorkerEvent::PluginQuizActionResult {
                plugin_id: _,
                action_title,
                result_kind,
                result,
            } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if let Some(modal) = &mut main.quiz_modal {
                        modal.ai_filling = false;
                        if result_kind.as_deref() != Some("quiz_fill_answers") {
                            modal.error =
                                Some(format!("{action_title} returned unsupported result kind."));
                            return Vec::new();
                        }
                        match result {
                            Ok(response) => {
                                let filled = apply_ai_fill(modal, &response);
                                main.toast_id = main.toast_id.wrapping_add(1);
                                if filled == 0 {
                                    modal.error = Some(
                                        "Plugin returned no applicable answer for this question."
                                            .into(),
                                    );
                                    main.toast = Some(
                                        "Plugin fill error: no applicable answer returned.".into(),
                                    );
                                } else {
                                    main.toast = Some(format!(
                                        "{action_title} filled {filled} answer{} ({} confidence)",
                                        if filled == 1 { "" } else { "s" },
                                        match response.confidence {
                                            crate::plugins::StudyHelpConfidence::High => "high",
                                            crate::plugins::StudyHelpConfidence::Medium => "medium",
                                            crate::plugins::StudyHelpConfidence::Low => "low",
                                        }
                                    ));
                                }
                                return vec![AppCommand::ScheduleToastExpire(main.toast_id)];
                            }
                            Err(error) => {
                                modal.error = Some(error.clone());
                                main.toast_id = main.toast_id.wrapping_add(1);
                                main.toast = Some(format!("{action_title} error: {error}"));
                                return vec![AppCommand::ScheduleToastExpire(main.toast_id)];
                            }
                        }
                    }
                }
                Vec::new()
            }
            WorkerEvent::PluginRegistryChanged(result) => {
                if let Screen::MainShell(main) = &mut self.screen {
                    main.toast_id = main.toast_id.wrapping_add(1);
                    match result {
                        Ok(registry) => {
                            main.plugin_install_input = None;
                            main.plugin_registry = registry;
                            main.plugin_settings =
                                load_plugin_settings_cache(&main.plugin_registry);
                            main.plugin_secret_configured =
                                load_plugin_secret_configured(&main.plugin_registry);
                            main.toast = Some("Plugin registry updated.".into());
                        }
                        Err(error) => {
                            if let Some(input) = &mut main.plugin_install_input {
                                input.saving = false;
                                input.error = Some(error.clone());
                            }
                            main.toast = Some(format!("Plugin error: {error}"));
                        }
                    }
                    return vec![AppCommand::ScheduleToastExpire(main.toast_id)];
                }
                Vec::new()
            }
            WorkerEvent::Toast(message) => {
                let mut id = 0u64;
                if let Screen::MainShell(main) = &mut self.screen {
                    main.toast_id = main.toast_id.wrapping_add(1);
                    id = main.toast_id;
                    main.toast = Some(message);
                }
                vec![AppCommand::ScheduleToastExpire(id)]
            }
            WorkerEvent::AssignmentListLoaded { course_id, list } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    main.assignment_list_by_course_id.insert(course_id, list);
                }
                Vec::new()
            }
            WorkerEvent::QuizListLoaded { course_id, list } => {
                if let Screen::MainShell(main) = &mut self.screen {
                    main.quiz_list_by_course_id.insert(course_id, list);
                }
                Vec::new()
            }
            WorkerEvent::ToastExpire(id) => {
                if let Screen::MainShell(main) = &mut self.screen {
                    if main.toast_id == id {
                        main.toast = None;
                    }
                }
                Vec::new()
            }
        }
    }

    fn on_bootstrap(
        &mut self,
        saved_config: Option<SavedConfig>,
        password: Option<String>,
        storage_warning: Option<String>,
        plugin_registry: crate::plugins::PluginRegistry,
    ) -> Vec<AppCommand> {
        self.storage_warning = storage_warning.clone();
        self.saved_config = saved_config.clone();

        if self.demo_mode {
            let mut main = MainState::default();
            main.config = Some(RuntimeConfig {
                base_url: "https://demo.moodle.example".into(),
                username: "demo".into(),
                service: "moodle_mobile_app".into(),
                password: "demo".into(),
            });
            main.dashboard = DashboardData {
                courses: demo_courses(),
                upcoming: demo_upcoming(),
                loading: false,
                error: None,
                from_cache: false,
            };
            main.plugin_registry = plugin_registry;
            main.plugin_settings = load_plugin_settings_cache(&main.plugin_registry);
            main.plugin_secret_configured = load_plugin_secret_configured(&main.plugin_registry);
            self.screen = Screen::MainShell(main);
            return Vec::new();
        }

        self.saved_password = password.clone();

        let mut login = LoginState::default();
        login.password.mask = true;
        let mut auto_login_command: Option<AppCommand> = None;
        if let Some(config) = saved_config {
            login.base_url.set(&config.base_url);
            login.username.set(&config.username);
            login.service.set(&config.service);
            if storage::session::auto_login_enabled() {
                if let Some(pw) = password.as_ref() {
                    login.busy = true;
                    auto_login_command = Some(AppCommand::ValidateLogin(RuntimeConfig {
                        base_url: config.base_url.clone(),
                        username: config.username.clone(),
                        service: config.service.clone(),
                        password: pw.clone(),
                    }));
                }
            }
            self.saved_config = Some(config);
        } else {
            login.service.set(crate::models::DEFAULT_MOODLE_SERVICE);
        }
        login.storage_warning = storage_warning;
        self.screen = Screen::Login(login);
        auto_login_command.map(|c| vec![c]).unwrap_or_default()
    }

    fn on_login_validated(&mut self, result: Result<RuntimeConfig, String>) -> Vec<AppCommand> {
        match result {
            Ok(config) => {
                let saved = config.saved();
                let mut save_warnings = Vec::new();
                if let Err(error) = storage::config::save_config(&saved) {
                    save_warnings.push(format!("profile settings could not be saved: {error}"));
                }
                if let Err(error) = storage::secret::save_password(&saved, &config.password) {
                    save_warnings.push(format!("password could not be saved securely: {error}"));
                }
                self.saved_config = Some(saved);
                self.saved_password = Some(config.password.clone());
                storage::session::set_auto_login(true);
                let mut main = MainState::default();
                main.config = Some(config.clone());
                main.plugin_registry = crate::plugins::registry::load_registry();
                main.plugin_settings = load_plugin_settings_cache(&main.plugin_registry);
                main.plugin_secret_configured =
                    load_plugin_secret_configured(&main.plugin_registry);
                main.dashboard.loading = true;
                if let Some(cached) = storage::cache::get_cached_dashboard() {
                    main.dashboard.courses = cached.courses;
                    main.dashboard.upcoming = cached.upcoming_assignments;
                    main.dashboard.from_cache = true;
                }
                self.screen = Screen::MainShell(main);
                let mut commands = vec![AppCommand::LoadDashboard(config)];
                if !save_warnings.is_empty() {
                    commands.push(AppCommand::ShowToast(format!(
                        "Login succeeded, but {}",
                        save_warnings.join("; ")
                    )));
                }
                commands
            }
            Err(error) => {
                storage::session::set_auto_login(false);
                if let Screen::Login(login) = &mut self.screen {
                    login.busy = false;
                    login.error = Some(error);
                } else {
                    let mut login = LoginState::default();
                    login.password.mask = true;
                    if let Some(config) = &self.saved_config {
                        login.base_url.set(&config.base_url);
                        login.username.set(&config.username);
                        login.service.set(&config.service);
                    }
                    login.error = Some(error);
                    self.screen = Screen::Login(login);
                }
                Vec::new()
            }
        }
    }
}

fn apply_ai_fill(
    modal: &mut crate::app::state::types::QuizModalData,
    response: &AiFillResponse,
) -> usize {
    let mut filled = 0usize;
    for answer in &response.answers {
        let control = modal
            .attempt
            .as_mut()
            .and_then(|a| a.questions.get_mut(modal.selected_question))
            .and_then(|q| {
                q.controls
                    .iter_mut()
                    .find(|c| c.name == answer.control_name)
            });

        let Some(control) = control else { continue };

        match control.kind {
            QuizAnswerKind::SingleChoice => {
                if let Some(value) = answer.selected_values.first() {
                    let mut changed = false;
                    for option in control.options.iter_mut() {
                        let selected = option.value == *value;
                        changed |= option.selected != selected;
                        option.selected = option.value == *value;
                    }
                    if changed || control.options.iter().any(|option| option.selected) {
                        filled += 1;
                    }
                }
            }
            QuizAnswerKind::MultiChoice => {
                let mut changed = false;
                for option in control.options.iter_mut() {
                    let selected = answer.selected_values.contains(&option.value)
                        || option
                            .name
                            .as_ref()
                            .is_some_and(|name| answer.selected_values.contains(name));
                    changed |= option.selected != selected;
                    option.selected = selected;
                }
                if changed || control.options.iter().any(|option| option.selected) {
                    filled += 1;
                }
            }
            QuizAnswerKind::Text => {
                if let Some(text) = &answer.text_value {
                    let text = text.trim();
                    if !text.is_empty() {
                        control.value = text.to_owned();
                        filled += 1;
                    }
                }
            }
            QuizAnswerKind::Hidden | QuizAnswerKind::Unsupported => {}
        }
    }
    filled
}

fn load_plugin_settings_cache(
    registry: &crate::plugins::PluginRegistry,
) -> std::collections::HashMap<String, std::collections::HashMap<String, String>> {
    let all = storage::plugin_settings::load_all();
    let mut cache = std::collections::HashMap::new();
    for plugin in &registry.plugins {
        let mut values = all
            .plugins
            .get(&plugin.manifest.id)
            .cloned()
            .unwrap_or_default();
        if plugin.manifest.id == "quiz-ai-extension" && !values.contains_key("gemini_model") {
            if let Ok(Some(model)) =
                storage::secret::load_plugin_secret(&plugin.manifest.id, "gemini_model")
            {
                values.insert("gemini_model".into(), model);
            }
        }
        cache.insert(plugin.manifest.id.clone(), values);
    }
    cache
}

fn load_plugin_secret_configured(
    registry: &crate::plugins::PluginRegistry,
) -> std::collections::HashSet<String> {
    let mut configured = std::collections::HashSet::new();
    for plugin in &registry.plugins {
        for (name, schema) in
            crate::ui::settings::plugin_settings_schema(plugin.manifest.settings_schema.as_ref())
        {
            if crate::ui::settings::schema_is_secret(schema)
                && storage::secret::load_plugin_secret(&plugin.manifest.id, &name)
                    .ok()
                    .flatten()
                    .is_some_and(|value| !value.trim().is_empty())
            {
                configured.insert(crate::ui::settings::setting_key(&plugin.manifest.id, &name));
            }
        }
    }
    configured
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::types::QuizModalData;
    use crate::models::{
        QuizAnswerControl, QuizAnswerKind, QuizAnswerOption, QuizAttempt, QuizAttemptData,
        QuizQuestion,
    };
    use crate::plugins::{ControlAnswer, StudyHelpConfidence};

    #[test]
    fn applies_multi_choice_by_checkbox_input_name() {
        let mut modal = QuizModalData {
            course_id: 1,
            quiz_id: 1,
            cmid: 1,
            quiz_name: "Quiz".into(),
            course_name: "Course".into(),
            module_description: None,
            module_url: None,
            summary: None,
            attempt: Some(QuizAttemptData {
                attempt: QuizAttempt {
                    id: 1,
                    quiz: 1,
                    state: "inprogress".into(),
                    currentpage: None,
                    timestart: None,
                    timefinish: None,
                },
                questions: vec![QuizQuestion {
                    slot: 1,
                    number: Some("3".into()),
                    name: "Q".into(),
                    text: "Compiled languages?".into(),
                    html: String::new(),
                    unsupported: false,
                    controls: vec![QuizAnswerControl {
                        name: "q10:3".into(),
                        kind: QuizAnswerKind::MultiChoice,
                        value: String::new(),
                        options: (0..5)
                            .map(|idx| QuizAnswerOption {
                                name: Some(format!("q10:3_choice{idx}")),
                                label: idx.to_string(),
                                value: "1".into(),
                                selected: false,
                            })
                            .collect(),
                    }],
                }],
                warnings: Vec::new(),
            }),
            loading: false,
            saving: false,
            finishing: false,
            ai_filling: false,
            confirm_finish: false,
            error: None,
            selected_question: 0,
            selected_control: 0,
            selected_option: 0,
            editing_text: false,
        };
        let response = AiFillResponse {
            answers: vec![ControlAnswer {
                control_name: "q10:3".into(),
                selected_values: vec![
                    "q10:3_choice0".into(),
                    "q10:3_choice3".into(),
                    "q10:3_choice4".into(),
                ],
                text_value: None,
            }],
            explanation: String::new(),
            confidence: StudyHelpConfidence::High,
        };

        assert_eq!(apply_ai_fill(&mut modal, &response), 1);
        let options = &modal.attempt.as_ref().unwrap().questions[0].controls[0].options;
        assert_eq!(
            options
                .iter()
                .enumerate()
                .filter_map(|(idx, option)| option.selected.then_some(idx))
                .collect::<Vec<_>>(),
            vec![0, 3, 4]
        );
    }
}
