pub mod browser;
pub mod clipboard;

use std::process::{Command, Stdio};

pub fn command_exists(command: &str) -> bool {
    let checker = if cfg!(windows) { "where" } else { "which" };
    Command::new(checker)
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub type OpResult = Result<(), String>;
