use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use moodle_tui::app::state::{AppCommand, AppState, WorkerEvent};
use moodle_tui::moodle::MoodleClient;
use moodle_tui::plugins::protocol::HostMessage;
use moodle_tui::plugins::runtime::PluginRuntime;
use moodle_tui::{platform, storage, ui};

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
enum RuntimeEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Worker(WorkerEvent),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
    std::thread::spawn(move || {
        loop {
            if !event::poll(Duration::from_millis(100)).unwrap_or(false) {
                continue;
            }
            match event::read() {
                Ok(CrosstermEvent::Key(key)) => {
                    if key.kind == KeyEventKind::Press {
                        let _ = tx.send(RuntimeEvent::Key(key));
                    }
                }
                Ok(CrosstermEvent::Mouse(_)) => {}
                Ok(CrosstermEvent::Resize(w, h)) => {
                    let _ = tx.send(RuntimeEvent::Resize(w, h));
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}

fn resolve_plugin_settings(
    manifest: &moodle_tui::plugins::manifest::PluginManifest,
) -> serde_json::Value {
    let mut settings = serde_json::Map::new();
    let Some(properties) = manifest
        .settings_schema
        .as_ref()
        .and_then(|schema| schema.get("properties"))
        .and_then(|properties| properties.as_object())
    else {
        return serde_json::Value::Object(settings);
    };

    for (name, schema) in properties {
        let is_secret = schema
            .get("format")
            .and_then(|value| value.as_str())
            .is_some_and(|format| format == "secret")
            || schema
                .get("secret")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
        let value = if is_secret {
            storage::secret::load_plugin_secret(&manifest.id, name)
                .ok()
                .flatten()
        } else {
            storage::plugin_settings::load_plugin_setting(&manifest.id, name).or_else(|| {
                schema
                    .get("default")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned)
            })
        };
        if let Some(value) = value {
            settings.insert(name.clone(), serde_json::Value::String(value));
        }
    }

    serde_json::Value::Object(settings)
}

fn execute_command(tx: mpsc::UnboundedSender<RuntimeEvent>, command: AppCommand, demo_mode: bool) {
    match command {
        AppCommand::Bootstrap => {
            tokio::spawn(async move {
                let plugin_registry = moodle_tui::plugins::registry::load_registry();
                if demo_mode {
                    let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::BootstrapLoaded {
                        saved_config: None,
                        password: None,
                        storage_warning: None,
                        plugin_registry,
                    }));
                    return;
                }
                let saved_config = storage::config::load_config();
                let password = saved_config
                    .as_ref()
                    .and_then(|c| storage::secret::load_password(c).ok().flatten());
                let diag = storage::secret::get_secure_storage_diagnostic();
                let warning = if diag.available {
                    None
                } else {
                    Some(diag.message)
                };
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::BootstrapLoaded {
                    saved_config,
                    password,
                    storage_warning: warning,
                    plugin_registry,
                }));
            });
        }
        AppCommand::ValidateLogin(config) => {
            tokio::spawn(async move {
                let client = MoodleClient::for_base_url(&config.base_url);
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
                let client = MoodleClient::for_base_url(&config.base_url);
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
                let client = MoodleClient::for_base_url(&config.base_url);
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
                let client = MoodleClient::for_base_url(&config.base_url);
                match client.fetch_course_assignments(&config, course_id).await {
                    Ok(list) => {
                        let detail = list.iter().find(|a| a.id == assignment_id).cloned();
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::AssignmentListLoaded {
                            course_id,
                            list,
                        }));
                        let _ =
                            tx.send(RuntimeEvent::Worker(WorkerEvent::AssignmentDetailLoaded {
                                assignment_id,
                                result: Ok(detail),
                            }));
                    }
                    Err(error) => {
                        let _ =
                            tx.send(RuntimeEvent::Worker(WorkerEvent::AssignmentDetailLoaded {
                                assignment_id,
                                result: Err(error.to_string()),
                            }));
                    }
                }
            });
        }
        AppCommand::LoadAssignmentStatus(config, assignment_id) => {
            tokio::spawn(async move {
                let client = MoodleClient::for_base_url(&config.base_url);
                let result = client
                    .fetch_assignment_submission_status(&config, assignment_id)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::AssignmentStatusLoaded {
                    assignment_id,
                    result,
                }));
            });
        }
        AppCommand::LoadQuizDetail(config, course_id, quiz_id) => {
            tokio::spawn(async move {
                let client = MoodleClient::for_base_url(&config.base_url);
                match client.fetch_course_quizzes(&config, course_id).await {
                    Ok(list) => {
                        let detail = list.iter().find(|q| q.id == quiz_id).cloned();
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::QuizListLoaded {
                            course_id,
                            list,
                        }));
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::QuizDetailLoaded {
                            course_id,
                            quiz_id,
                            result: Ok(detail),
                        }));
                    }
                    Err(error) => {
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::QuizDetailLoaded {
                            course_id,
                            quiz_id,
                            result: Err(error.to_string()),
                        }));
                    }
                }
            });
        }
        AppCommand::StartQuizAttempt(config, quiz_id) => {
            tokio::spawn(async move {
                let client = MoodleClient::for_base_url(&config.base_url);
                let result = client
                    .start_quiz_attempt(&config, quiz_id)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::QuizAttemptStarted {
                    quiz_id,
                    result,
                }));
            });
        }
        AppCommand::LoadQuizAttempt(config, attempt_id) => {
            tokio::spawn(async move {
                let client = MoodleClient::for_base_url(&config.base_url);
                let result = client
                    .fetch_quiz_attempt_data(&config, attempt_id)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::QuizAttemptLoaded {
                    attempt_id,
                    result,
                }));
            });
        }
        AppCommand::SaveQuizAttempt(config, attempt) => {
            tokio::spawn(async move {
                let client = MoodleClient::for_base_url(&config.base_url);
                let attempt_id = attempt.attempt.id;
                let result = client
                    .save_quiz_attempt(&config, &attempt)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::QuizAttemptSaved {
                    attempt_id,
                    result,
                }));
            });
        }
        AppCommand::FinishQuizAttempt(config, attempt) => {
            tokio::spawn(async move {
                let client = MoodleClient::for_base_url(&config.base_url);
                let attempt_id = attempt.attempt.id;
                let result = client
                    .finish_quiz_attempt(&config, &attempt)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::QuizAttemptFinished {
                    attempt_id,
                    result,
                }));
            });
        }
        AppCommand::InvokePluginQuizAction {
            plugin,
            action_id,
            result_kind,
            question_context,
        } => {
            tokio::spawn(async move {
                let action_title = plugin
                    .manifest
                    .contributes
                    .quiz_actions
                    .iter()
                    .find(|action| action.id == action_id)
                    .map(|action| action.title.clone())
                    .unwrap_or_else(|| action_id.clone());
                let settings = resolve_plugin_settings(&plugin.manifest);
                let payload = serde_json::json!({
                    "question": question_context,
                    "settings": settings,
                });

                let message = HostMessage::Invoke {
                    id: format!("plugin-action-{action_id}"),
                    action: action_id,
                    payload,
                };

                let runtime = PluginRuntime::new();
                let result = match runtime.invoke_once(&plugin, &message) {
                    Ok(moodle_tui::plugins::protocol::PluginMessage::Ok { payload, .. }) => {
                        serde_json::from_value::<moodle_tui::plugins::AiFillResponse>(payload)
                            .map_err(|e| format!("Invalid AI response: {e}"))
                    }
                    Ok(moodle_tui::plugins::protocol::PluginMessage::Error { message, .. }) => {
                        Err(format!("Plugin error: {message}"))
                    }
                    Ok(moodle_tui::plugins::protocol::PluginMessage::HostAction { .. }) => {
                        Err("Plugin sent unexpected HostAction".into())
                    }
                    Err(e) => Err(format!("Plugin runtime: {e}")),
                };
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::PluginQuizActionResult {
                    plugin_id: plugin.manifest.id,
                    action_title,
                    result_kind,
                    result,
                }));
            });
        }
        AppCommand::SavePluginSetting {
            plugin_id,
            setting_name,
            secret,
            value,
        } => {
            tokio::spawn(async move {
                let result = if secret {
                    storage::secret::save_plugin_secret(&plugin_id, &setting_name, &value)
                } else {
                    storage::plugin_settings::save_plugin_setting(&plugin_id, &setting_name, &value)
                }
                .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::PluginSettingSaved {
                    plugin_id,
                    setting_name,
                    secret,
                    result,
                }));
            });
        }
        AppCommand::InstallPluginFromDir(path) => {
            tokio::spawn(async move {
                let result = moodle_tui::plugins::registry::install_plugin_from_dir(&path)
                    .map(|_| moodle_tui::plugins::registry::load_registry())
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::PluginRegistryChanged(
                    result,
                )));
            });
        }
        AppCommand::UninstallPlugin(plugin_id) => {
            tokio::spawn(async move {
                let result = moodle_tui::plugins::registry::uninstall_plugin(&plugin_id)
                    .map(|_| {
                        let _ = storage::plugin_settings::clear_plugin_settings(&plugin_id);
                        moodle_tui::plugins::registry::load_registry()
                    })
                    .map_err(|e| e.to_string());
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::PluginRegistryChanged(
                    result,
                )));
            });
        }
        AppCommand::ReloadPlugins => {
            tokio::spawn(async move {
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::PluginRegistryChanged(
                    Ok(moodle_tui::plugins::registry::load_registry()),
                )));
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
                let client = MoodleClient::for_base_url(&config.base_url);
                match client.fetch_course_assignments(&config, course_id).await {
                    Ok(list) => {
                        let cmid = list.iter().find(|a| a.id == upcoming_id).map(|a| a.cmid);
                        let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::AssignmentListLoaded {
                            course_id,
                            list,
                        }));
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
        AppCommand::Logout {
            saved_config,
            password,
            storage_warning,
        } => {
            tokio::spawn(async move {
                let saved_config = saved_config.or_else(storage::config::load_config);
                let _ = storage::cache::clear_cache();
                storage::session::set_auto_login(false);
                let _ = tx.send(RuntimeEvent::Worker(WorkerEvent::BootstrapLoaded {
                    saved_config,
                    password,
                    storage_warning,
                    plugin_registry: moodle_tui::plugins::registry::load_registry(),
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
