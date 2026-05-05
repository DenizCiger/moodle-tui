use crate::app::state::text_input::TextInputState;
use crate::models::{
    AssignmentDetail, AssignmentSubmissionStatus, Course, CourseSection, RuntimeConfig,
    SavedConfig, UpcomingAssignment,
};

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

#[derive(Debug, Clone, Default)]
pub struct MainState {
    pub config: Option<RuntimeConfig>,
    pub dashboard: DashboardData,
    pub view: CourseView,
    pub selected_row: usize,
    pub settings_open: bool,
    pub settings_scroll: u16,
    pub assignment_modal: Option<AssignmentModalData>,
    pub course_finder_open: bool,
    pub content_finder_open: bool,
    pub finder_query: TextInputState,
    pub finder_selected: usize,
    pub finder_target_idx: usize,
    pub toast: Option<String>,
    pub toast_id: u64,
    pub dashboard_focus: DashboardPane,
    pub assignment_list_by_course_id: std::collections::HashMap<i64, Vec<AssignmentDetail>>,
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
    pub storage_warning: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Screen::Loading,
            demo_mode: false,
            terminal_size: (80, 24),
            saved_config: None,
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
    OpenUrl(String),
    CopyToClipboard(String),
    ShowToast(String),
    ResolveUpcomingLink {
        config: RuntimeConfig,
        upcoming_id: i64,
        course_id: i64,
        action: LinkAction,
    },
    Logout,
    ScheduleToastExpire(u64),
    Quit,
}

#[derive(Debug)]
pub enum WorkerEvent {
    BootstrapLoaded {
        saved_config: Option<SavedConfig>,
        password: Option<String>,
        storage_warning: Option<String>,
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
    Toast(String),
    ToastExpire(u64),
    AssignmentListLoaded {
        course_id: i64,
        list: Vec<AssignmentDetail>,
    },
}
