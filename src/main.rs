use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEvent, KeyEventKind,
    MouseEvent,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use moodle_tui::app::state::{AppCommand, AppState, WorkerEvent};
use moodle_tui::moodle::MoodleClient;
use moodle_tui::{platform, storage, ui};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
enum RuntimeEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Worker(WorkerEvent),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<RuntimeEvent>();
    spawn_input_thread(tx.clone());

    let demo_mode = std::env::args().skip(1).any(|arg| arg == "--demo");
    let mut state = if demo_mode {
        AppState::new_demo()
    } else {
        AppState::new()
    };
    if let Ok((width, height)) = crossterm::terminal::size() {
        state.update_terminal_size(width, height);
    }
    for command in state.initial_commands() {
        execute_command(tx.clone(), command, demo_mode);
    }

    let client = MoodleClient::new();
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    loop {
        terminal.draw(|frame| ui::render(frame, &state))?;
        tokio::select! {
            Some(event) = rx.recv() => {
                let commands = match event {
                    RuntimeEvent::Key(key) => state.handle_key(key),
                    RuntimeEvent::Mouse(mouse) => state.handle_mouse(mouse),
                    RuntimeEvent::Resize(width, height) => { state.update_terminal_size(width, height); Vec::new() }
                    RuntimeEvent::Worker(event) => state.handle_worker_event(event),
                };
                if dispatch(&tx, commands, demo_mode, &client) {
                    break;
                }
            }
            _ = tick.tick() => {}
        }
    }
    Ok(())
}

fn dispatch(
    tx: &mpsc::UnboundedSender<RuntimeEvent>,
    commands: Vec<AppCommand>,
    demo_mode: bool,
    _client: &MoodleClient,
) -> bool {
    let mut quit = false;
    for command in commands {
        match command {
            AppCommand::Quit => quit = true,
            command => execute_command(tx.clone(), command, demo_mode),
        }
    }
    quit
}

fn spawn_input_thread(tx: mpsc::UnboundedSender<RuntimeEvent>) {
    std::thread::spawn(move || loop {
        if !event::poll(Duration::from_millis(100)).unwrap_or(false) {
            continue;
        }
        match event::read() {
            Ok(CrosstermEvent::Key(key)) => {
                if key.kind == KeyEventKind::Press {
                    let _ = tx.send(RuntimeEvent::Key(key));
                }
            }
            Ok(CrosstermEvent::Mouse(mouse)) => {
                let _ = tx.send(RuntimeEvent::Mouse(mouse));
            }
            Ok(CrosstermEvent::Resize(w, h)) => {
                let _ = tx.send(RuntimeEvent::Resize(w, h));
            }
            Ok(_) => {}
            Err(_) => break,
        }
    });
}

fn execute_command(tx: mpsc::UnboundedSender<RuntimeEvent>, command: AppCommand, demo_mode: bool) {
    match command {
        AppCommand::Bootstrap => {
            tokio::spawn(async move {
                if demo_mode {
                    let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::BootstrapLoaded {
                        saved_config: None,
                        password: None,
                        storage_warning: None,
                    }));
                    return;
                }
                let saved_config = storage::config::load_config();
                let password = saved_config
                    .as_ref()
                    .and_then(|c| storage::secret::load_password(c).ok().flatten());
                let diag = storage::secret::get_secure_storage_diagnostic();
                let warning = if diag.available { None } else { Some(diag.message) };
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::BootstrapLoaded {
                    saved_config,
                    password,
                    storage_warning: warning,
                }));
            });
        }
        AppCommand::ValidateLogin(config) => {
            tokio::spawn(async move {
                let client = MoodleClient::new();
                let result = client
                    .test_credentials(&config)
                    .await
                    .map(|_| config)
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::LoginValidated(result)));
            });
        }
        AppCommand::LoadDashboard(config) => {
            tokio::spawn(async move {
                let client = MoodleClient::new();
                let courses = client.fetch_courses(&config).await;
                let upcoming = client.fetch_upcoming_assignments(&config).await;
                let result = match (courses, upcoming) {
                    (Ok(c), Ok(u)) => {
                        let _ = storage::cache::save_dashboard_to_cache(&c, &u);
                        Ok((c, u))
                    }
                    (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
                };
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::DashboardLoaded(result)));
            });
        }
        AppCommand::LoadCoursePage(config, course_id) => {
            tokio::spawn(async move {
                if demo_mode {
                    let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::CoursePageLoaded {
                        course_id,
                        result: Ok(moodle_tui::demo::demo_course_sections(course_id)),
                    }));
                    return;
                }
                let client = MoodleClient::new();
                let result = client
                    .fetch_course_contents(&config, course_id)
                    .await
                    .map_err(|e| e.to_string());
                if let Ok(sections) = &result {
                    let _ = storage::cache::save_course_sections_to_cache(course_id, sections);
                }
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::CoursePageLoaded {
                    course_id,
                    result,
                }));
            });
        }
        AppCommand::LoadAssignmentDetail(config, course_id, assignment_id) => {
            tokio::spawn(async move {
                let client = MoodleClient::new();
                match client.fetch_course_assignments(&config, course_id).await {
                    Ok(list) => {
                        let detail = list.iter().find(|a| a.id == assignment_id).cloned();
                        let _ = tx.send(RuntimeEvent::Worker(
                            WorkerEvent::AssignmentListLoaded { course_id, list },
                        ));
                        let _ = tx.send(RuntimeEvent::Worker(
                            WorkerEvent::AssignmentDetailLoaded {
                                assignment_id,
                                result: Ok(detail),
                            },
                        ));
                    }
                    Err(error) => {
                        let _ = tx.send(RuntimeEvent::Worker(
                            WorkerEvent::AssignmentDetailLoaded {
                                assignment_id,
                                result: Err(error.to_string()),
                            },
                        ));
                    }
                }
            });
        }
        AppCommand::LoadAssignmentStatus(config, assignment_id) => {
            tokio::spawn(async move {
                let client = MoodleClient::new();
                let result = client
                    .fetch_assignment_submission_status(&config, assignment_id)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(
                    WorkerEvent::AssignmentStatusLoaded { assignment_id, result },
                ));
            });
        }
        AppCommand::OpenUrl(url) => {
            tokio::spawn(async move {
                match platform::browser::open_url(&url) {
                    Ok(()) => {
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(
                            "Opened link in browser.".to_owned(),
                        )));
                    }
                    Err(error) => {
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(format!(
                            "Browser error: {error}"
                        ))));
                    }
                }
            });
        }
        AppCommand::CopyToClipboard(text) => {
            tokio::spawn(async move {
                match platform::clipboard::copy_text(&text) {
                    Ok(()) => {
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(
                            "Copied link to clipboard.".to_owned(),
                        )));
                    }
                    Err(error) => {
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(format!(
                            "Clipboard error: {error}"
                        ))));
                    }
                }
            });
        }
        AppCommand::ShowToast(message) => {
            let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(message)));
        }
        AppCommand::ResolveUpcomingLink {
            config,
            upcoming_id,
            course_id,
            action,
        } => {
            let base_url = config.base_url.clone();
            tokio::spawn(async move {
                let client = MoodleClient::new();
                match client.fetch_course_assignments(&config, course_id).await {
                    Ok(list) => {
                        let cmid = list.iter().find(|a| a.id == upcoming_id).map(|a| a.cmid);
                        let _ = tx.send(RuntimeEvent::Worker(
                            WorkerEvent::AssignmentListLoaded {
                                course_id,
                                list,
                            },
                        ));
                        match cmid {
                            Some(cmid) => {
                                let url = moodle_tui::moodle::urls::build_assignment_activity_url(
                                    &base_url, cmid,
                                );
                                let result = match action {
                                    moodle_tui::app::state::types::LinkAction::Open => {
                                        platform::browser::open_url(&url)
                                    }
                                    moodle_tui::app::state::types::LinkAction::Copy => {
                                        platform::clipboard::copy_text(&url)
                                    }
                                };
                                let message = match (action, result) {
                                    (moodle_tui::app::state::types::LinkAction::Open, Ok(())) => {
                                        "Opened link in browser.".to_owned()
                                    }
                                    (moodle_tui::app::state::types::LinkAction::Copy, Ok(())) => {
                                        "Copied link to clipboard.".to_owned()
                                    }
                                    (moodle_tui::app::state::types::LinkAction::Open, Err(e)) => {
                                        format!("Browser error: {e}")
                                    }
                                    (moodle_tui::app::state::types::LinkAction::Copy, Err(e)) => {
                                        format!("Clipboard error: {e}")
                                    }
                                };
                                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(message)));
                            }
                            None => {
                                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(
                                    "No link available on this item.".to_owned(),
                                )));
                            }
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::Toast(format!(
                            "Link lookup error: {error}"
                        ))));
                    }
                }
            });
        }
        AppCommand::Logout => {
            tokio::spawn(async move {
                if let Some(saved) = storage::config::load_config() {
                    let _ = storage::secret::clear_password(&saved);
                }
                let _ = storage::config::clear_config();
                let _ = storage::cache::clear_cache();
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::BootstrapLoaded {
                    saved_config: None,
                    password: None,
                    storage_warning: None,
                }));
            });
        }
        AppCommand::ScheduleToastExpire(id) => {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(2500)).await;
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::ToastExpire(id)));
            });
        }
        AppCommand::Quit => {}
    }
}
