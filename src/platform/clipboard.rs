use crate::platform::{OpResult, command_exists};
use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_text(text: &str) -> OpResult {
    if cfg!(target_os = "macos") {
        if !command_exists("pbcopy") {
            return Err("Clipboard backend 'pbcopy' is not available.".to_owned());
        }
        return run_with_input("pbcopy", &[], text);
    }

    if cfg!(target_os = "linux") {
        let candidates: &[(&str, &[&str])] = &[
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--input"]),
        ];
        let available: Vec<&(&str, &[&str])> =
            candidates.iter().filter(|(cmd, _)| command_exists(cmd)).collect();
        if available.is_empty() {
            return Err("No clipboard utility found (tried wl-copy, xclip, xsel).".to_owned());
        }
        let mut last_error = "Clipboard command failed.".to_owned();
        for (cmd, args) in available {
            match run_with_input(cmd, args, text) {
                Ok(()) => return Ok(()),
                Err(error) => last_error = error,
            }
        }
        return Err(last_error);
    }

    if cfg!(windows) {
        let shells = ["powershell.exe", "powershell", "pwsh.exe", "pwsh"];
        let shell = shells.iter().copied().find(|c| command_exists(c));
        let shell = match shell {
            Some(value) => value,
            None => return Err("No PowerShell executable found for clipboard access.".to_owned()),
        };
        let mut command = Command::new(shell);
        command
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Set-Clipboard -Value $env:TUI_MOODLE_CLIPBOARD_TEXT",
            ])
            .env("TUI_MOODLE_CLIPBOARD_TEXT", text);
        let output = command.output().map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
        }
        return Ok(());
    }

    Err(format!("Clipboard copy is not supported on '{}'.", std::env::consts::OS))
}

fn run_with_input(command: &str, args: &[&str], input: &str) -> OpResult {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(input.as_bytes()).map_err(|e| e.to_string())?;
    }
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(())
}
