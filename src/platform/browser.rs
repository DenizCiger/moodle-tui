use crate::platform::{OpResult, command_exists};
use std::process::Command;

pub fn open_url(url: &str) -> OpResult {
    let target = url.trim();
    if target.is_empty() {
        return Err("Cannot open an empty URL.".to_owned());
    }

    if cfg!(target_os = "macos") {
        if !command_exists("open") {
            return Err("Browser launcher 'open' is not available.".to_owned());
        }
        return run("open", &[target]);
    }

    if cfg!(target_os = "linux") {
        let candidates: &[(&str, &[&str])] = &[
            ("xdg-open", &[]),
            ("sensible-browser", &[]),
        ];
        let available: Vec<&(&str, &[&str])> =
            candidates.iter().filter(|(cmd, _)| command_exists(cmd)).collect();
        if available.is_empty() {
            return Err("No browser launcher found (tried xdg-open, sensible-browser).".to_owned());
        }
        let mut last_error = "Browser launcher failed.".to_owned();
        for (cmd, extra) in available {
            let mut args: Vec<&str> = extra.to_vec();
            args.push(target);
            match run(cmd, &args) {
                Ok(()) => return Ok(()),
                Err(error) => last_error = error,
            }
        }
        return Err(last_error);
    }

    if cfg!(windows) {
        if !command_exists("cmd") {
            return Err("Windows command shell is not available.".to_owned());
        }
        return run("cmd", &["/c", "start", "", target]);
    }

    Err(format!("Browser opening is not supported on '{}'.", std::env::consts::OS))
}

fn run(command: &str, args: &[&str]) -> OpResult {
    let output = Command::new(command).args(args).output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(())
}
