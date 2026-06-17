use crate::app::state::text_input::{SearchModalState, TextInputState};
use crate::models::{
    AssignmentDetail, AssignmentSubmissionStatus, Course, CourseSection, QuizAttemptData,
    QuizSummary, RuntimeConfig, SavedConfig, UpcomingAssignment,
};
use crate::plugins::PluginRegistry;
use crate::plugins::protocol::QuizQuestionContext;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct LoginState {
    pub base_url: TextInputState,
    pub username: TextInputState,
    pub password: TextInputState,
    pub service: TextInputState,
    pub focus: LoginFocus,
    pub busy: bool,
    pub error: Option<String>,
    pub storage_warning: Option<String>,
    pub show_password: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LoginFocus {
    #[default]
    BaseUrl,
    Username,
    Password,
    Service,
    Submit,
}

impl LoginFocus {
    pub fn next(self) -> Self {
        match self {
            LoginFocus::BaseUrl => LoginFocus::Username,
            LoginFocus::Username => LoginFocus::Password,
            LoginFocus::Password => LoginFocus::Service,
            LoginFocus::Service => LoginFocus::Submit,
            LoginFocus::Submit => LoginFocus::BaseUrl,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            LoginFocus::BaseUrl => LoginFocus::Submit,
            LoginFocus::Username => LoginFocus::BaseUrl,
            LoginFocus::Password => LoginFocus::Username,
            LoginFocus::Service => LoginFocus::Password,
            LoginFocus::Submit => LoginFocus::Service,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DashboardData {
    pub courses: Vec<Course>,
    pub upcoming: Vec<UpcomingAssignment>,
    pub loading: bool,
    pub error: Option<String>,
    pub from_cache: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CoursePageData {
    pub course_id: i64,
    pub course_short_name: String,
    pub course_full_name: String,
    pub sections: Vec<CourseSection>,
    pub loading: bool,
    pub error: Option<String>,
    pub from_cache: bool,
    pub collapsed: std::collections::HashSet<String>,
    pub selected_row: usize,
}

#[derive(Debug, Clone, Default)]
pub enum CourseView {
    #[default]
    Dashboard,
    Course(CoursePageData),
}

#[derive(Debug, Clone)]
pub struct PluginApiKeyConfig {
    pub input: TextInputState,
    pub plugin_id: String,
    pub setting_name: String,
    pub secret: bool,
    pub saving: bool,
    pub error: Option<String>,
    pub title: String,
    pub current_value: String,
}

#[derive(Debug, Clone)]
pub struct PluginModelPickerConfig {
    pub plugin_id: String,
    pub setting_name: String,
    pub secret: bool,
    pub title: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub saving: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PluginInstallConfig {
    pub input: TextInputState,
    pub saving: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsPane {
    #[default]
    Keybinds,
    Config,
}

impl SettingsPane {
    pub fn toggle(self) -> Self {
        match self {
            SettingsPane::Keybinds => SettingsPane::Config,
            SettingsPane::Config => SettingsPane::Keybinds,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SettingsPaneState {
    pub cursor: usize,
    pub scroll: u16,
    pub horizontal_scroll: u16,
}

#[derive(Debug, Clone, Default)]
pub struct MainState {
    pub config: Option<RuntimeConfig>,
    pub dashboard: DashboardData,
    pub view: CourseView,
    pub selected_row: usize,
    pub settings_open: bool,
    pub settings_active_pane: SettingsPane,
    pub settings_keybinds: SettingsPaneState,
    pub settings_config: SettingsPaneState,
    pub settings_search_query: String,
    pub settings_search_active: bool,
    pub assignment_modal: Option<AssignmentModalData>,
    pub quiz_modal: Option<QuizModalData>,
    pub course_finder_open: bool,
    pub content_finder_open: bool,
    pub api_key_input: Option<PluginApiKeyConfig>,
    pub model_picker: Option<PluginModelPickerConfig>,
    pub plugin_install_input: Option<PluginInstallConfig>,
    pub finder: SearchModalState,
    pub finder_target_idx: usize,
    pub toast: Option<String>,
    pub toast_id: u64,
    pub dashboard_focus: DashboardPane,
    pub dashboard_search_query: String,
    pub dashboard_search_active: bool,
    pub dashboard_upcoming_horizontal_scroll: u16,
    pub dashboard_courses_horizontal_scroll: u16,
    pub assignment_list_by_course_id: std::collections::HashMap<i64, Vec<AssignmentDetail>>,
    pub quiz_list_by_course_id: std::collections::HashMap<i64, Vec<QuizSummary>>,
    pub plugin_registry: PluginRegistry,
    pub plugin_settings:
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    pub plugin_secret_configured: std::collections::HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkAction {
    Open,
    Copy,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DashboardPane {
    #[default]
    Upcoming,
    Courses,
}

impl DashboardPane {
    pub fn toggle(self) -> Self {
        match self {
            DashboardPane::Upcoming => DashboardPane::Courses,
            DashboardPane::Courses => DashboardPane::Upcoming,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssignmentModalData {
    pub course_id: i64,
    pub assignment_id: i64,
    pub assignment_name: String,
    pub course_name: String,
    pub due_date: i64,
    pub module_description: Option<String>,
    pub module_url: Option<String>,
    pub status: Option<AssignmentSubmissionStatus>,
    pub status_loading: bool,
    pub status_error: Option<String>,
    pub detail: Option<AssignmentDetail>,
    pub detail_loading: bool,
    pub detail_error: Option<String>,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QuizModalData {
    pub course_id: i64,
    pub quiz_id: i64,
    pub cmid: i64,
    pub quiz_name: String,
    pub course_name: String,
    pub module_description: Option<String>,
    pub module_url: Option<String>,
    pub summary: Option<QuizSummary>,
    pub attempt: Option<QuizAttemptData>,
    pub loading: bool,
    pub saving: bool,
    pub finishing: bool,
    pub ai_filling: bool,
    pub confirm_finish: bool,
    pub error: Option<String>,
    pub selected_question: usize,
    pub selected_control: usize,
    pub selected_option: usize,
    pub editing_text: bool,
}

#[derive(Debug, Clone, Default)]
pub enum Screen {
    #[default]
    Loading,
    Login(LoginState),
    MainShell(MainState),
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub demo_mode: bool,
    pub terminal_size: (u16, u16),
    pub saved_config: Option<SavedConfig>,
    pub saved_password: Option<String>,
    pub storage_warning: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Screen::Loading,
            demo_mode: false,
            terminal_size: (80, 24),
            saved_config: None,
            saved_password: None,
            storage_warning: None,
        }
    }

    pub fn new_demo() -> Self {
        let mut state = Self::new();
        state.demo_mode = true;
        state
    }

    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    pub fn initial_commands(&self) -> Vec<AppCommand> {
        vec![AppCommand::Bootstrap]
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum AppCommand {
    Bootstrap,
    ValidateLogin(RuntimeConfig),
    LoadDashboard(RuntimeConfig),
    LoadCoursePage(RuntimeConfig, i64),
    LoadAssignmentDetail(RuntimeConfig, i64, i64),
    LoadAssignmentStatus(RuntimeConfig, i64),
    LoadQuizDetail(RuntimeConfig, i64, i64),
    StartQuizAttempt(RuntimeConfig, i64),
    LoadQuizAttempt(RuntimeConfig, i64),
    SaveQuizAttempt(RuntimeConfig, QuizAttemptData),
    FinishQuizAttempt(RuntimeConfig, QuizAttemptData),
    OpenUrl(String),
    CopyToClipboard(String),
    ShowToast(String),
    ResolveUpcomingLink {
        config: RuntimeConfig,
        upcoming_id: i64,
        course_id: i64,
        action: LinkAction,
    },
    InvokePluginQuizAction {
        plugin: crate::plugins::InstalledPlugin,
        action_id: String,
        result_kind: Option<String>,
        question_context: QuizQuestionContext,
    },
    SavePluginSetting {
        plugin_id: String,
        setting_name: String,
        secret: bool,
        value: String,
    },
    InstallPluginFromDir(PathBuf),
    UninstallPlugin(String),
    ReloadPlugins,
    Logout {
        saved_config: Option<SavedConfig>,
        password: Option<String>,
        storage_warning: Option<String>,
    },
    ScheduleToastExpire(u64),
    Quit,
}

#[derive(Debug)]
pub enum WorkerEvent {
    BootstrapLoaded {
        saved_config: Option<SavedConfig>,
        password: Option<String>,
        storage_warning: Option<String>,
        plugin_registry: PluginRegistry,
    },
    LoginValidated(Result<RuntimeConfig, String>),
    DashboardLoaded(Result<(Vec<Course>, Vec<UpcomingAssignment>), String>),
    CoursePageLoaded {
        course_id: i64,
        result: Result<Vec<CourseSection>, String>,
    },
    AssignmentDetailLoaded {
        assignment_id: i64,
        result: Result<Option<AssignmentDetail>, String>,
    },
    AssignmentStatusLoaded {
        assignment_id: i64,
        result: Result<Option<AssignmentSubmissionStatus>, String>,
    },
    QuizDetailLoaded {
        course_id: i64,
        quiz_id: i64,
        result: Result<Option<QuizSummary>, String>,
    },
    QuizAttemptStarted {
        quiz_id: i64,
        result: Result<crate::models::QuizAttempt, String>,
    },
    QuizAttemptLoaded {
        attempt_id: i64,
        result: Result<QuizAttemptData, String>,
    },
    QuizAttemptSaved {
        attempt_id: i64,
        result: Result<QuizAttemptData, String>,
    },
    QuizAttemptFinished {
        attempt_id: i64,
        result: Result<(), String>,
    },
    PluginQuizActionResult {
        plugin_id: String,
        action_title: String,
        result_kind: Option<String>,
        result: Result<crate::plugins::AiFillResponse, String>,
    },
    PluginSettingSaved {
        plugin_id: String,
        setting_name: String,
        secret: bool,
        result: Result<(), String>,
    },
    PluginRegistryChanged(Result<PluginRegistry, String>),
    Toast(String),
    ToastExpire(u64),
    AssignmentListLoaded {
        course_id: i64,
        list: Vec<AssignmentDetail>,
    },
    QuizListLoaded {
        course_id: i64,
        list: Vec<QuizSummary>,
    },
}
