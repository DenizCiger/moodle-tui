use crate::app::state::types::{
    AppCommand, AppState, CourseView, DashboardData, LoginState, MainState, Screen,
};
use crate::demo::{demo_courses, demo_upcoming};
use crate::models::{RuntimeConfig, SavedConfig};
use crate::storage;

impl AppState {
    pub fn handle_worker_event(
        &mut self,
        event: super::types::WorkerEvent,
    ) -> Vec<AppCommand> {
        use super::types::WorkerEvent;

        match event {
            WorkerEvent::BootstrapLoaded {
                saved_config,
                password,
                storage_warning,
            } => self.on_bootstrap(saved_config, password, storage_warning),
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
    ) -> Vec<AppCommand> {
        self.storage_warning = storage_warning.clone();

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
            self.screen = Screen::MainShell(main);
            return Vec::new();
        }

        if let (Some(config), Some(password)) = (saved_config.clone(), password) {
            let runtime = RuntimeConfig {
                base_url: config.base_url.clone(),
                username: config.username.clone(),
                service: config.service.clone(),
                password,
            };
            self.saved_config = Some(config);
            return vec![AppCommand::ValidateLogin(runtime)];
        }

        let mut login = LoginState::default();
        if let Some(config) = saved_config {
            login.base_url.set(&config.base_url);
            login.username.set(&config.username);
            login.service.set(&config.service);
            self.saved_config = Some(config);
        } else {
            login.service.set(crate::models::DEFAULT_MOODLE_SERVICE);
        }
        login.storage_warning = storage_warning;
        self.screen = Screen::Login(login);
        Vec::new()
    }

    fn on_login_validated(&mut self, result: Result<RuntimeConfig, String>) -> Vec<AppCommand> {
        match result {
            Ok(config) => {
                let saved = config.saved();
                let _ = storage::config::save_config(&saved);
                let _ = storage::secret::save_password(&saved, &config.password);
                self.saved_config = Some(saved);
                let mut main = MainState::default();
                main.config = Some(config.clone());
                main.dashboard.loading = true;
                if let Some(cached) = storage::cache::get_cached_dashboard() {
                    main.dashboard.courses = cached.courses;
                    main.dashboard.upcoming = cached.upcoming_assignments;
                    main.dashboard.from_cache = true;
                }
                self.screen = Screen::MainShell(main);
                vec![AppCommand::LoadDashboard(config)]
            }
            Err(error) => {
                if let Screen::Login(login) = &mut self.screen {
                    login.busy = false;
                    login.error = Some(error);
                } else {
                    let mut login = LoginState::default();
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
