use crate::plugins::protocol::{HostMessage, PluginMessage};
use crate::plugins::registry::InstalledPlugin;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const DEFAULT_PLUGIN_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone)]
pub struct PluginRuntime {
    timeout: Duration,
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_PLUGIN_TIMEOUT,
        }
    }
}

impl PluginRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invoke_once(
        &self,
        plugin: &InstalledPlugin,
        message: &HostMessage,
    ) -> Result<PluginMessage, String> {
        if !plugin.enabled {
            return Err(format!("plugin '{}' is disabled", plugin.manifest.id));
        }
        if let Some(error) = &plugin.load_error {
            return Err(format!(
                "plugin '{}' failed to load: {error}",
                plugin.manifest.id
            ));
        }

        let entry = plugin.manifest.entry_path(&plugin.directory)?;
        let command = plugin_command(&entry);
        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("failed to start plugin '{}': {error}", plugin.manifest.id))?;

        if let Some(mut stdin) = child.stdin.take() {
            let line = serde_json::to_string(message)
                .map_err(|error| format!("failed to serialize plugin message: {error}"))?;
            stdin
                .write_all(line.as_bytes())
                .and_then(|_| stdin.write_all(b"\n"))
                .map_err(|error| format!("failed to write plugin input: {error}"))?;
        }

        let started = std::time::Instant::now();
        loop {
            if started.elapsed() > self.timeout {
                let _ = child.kill();
                return Err(format!("plugin '{}' timed out", plugin.manifest.id));
            }
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(error) => return Err(format!("failed to wait for plugin: {error}")),
            }
        }

        let output = child
            .wait_with_output()
            .map_err(|error| format!("failed to read plugin output: {error}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "plugin '{}' exited with {}: {}",
                plugin.manifest.id,
                output.status,
                stderr.trim()
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout
            .lines()
            .find(|line| !line.trim().is_empty())
            .ok_or_else(|| format!("plugin '{}' returned no response", plugin.manifest.id))?;
        serde_json::from_str(first_line)
            .map_err(|error| format!("invalid plugin response JSON: {error}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PluginCommand {
    program: PathBuf,
    args: Vec<String>,
}

fn plugin_command(entry: &Path) -> PluginCommand {
    if entry
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
    {
        PluginCommand {
            program: PathBuf::from("node"),
            args: vec![entry.to_string_lossy().to_string()],
        }
    } else {
        PluginCommand {
            program: entry.to_owned(),
            args: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_javascript_plugins_through_node() {
        let command = plugin_command(Path::new("plugin.js"));
        assert_eq!(command.program, PathBuf::from("node"));
        assert_eq!(command.args, vec!["plugin.js"]);
    }

    #[test]
    fn runs_native_plugins_directly() {
        let command = plugin_command(Path::new("plugin.exe"));
        assert_eq!(command.program, PathBuf::from("plugin.exe"));
        assert!(command.args.is_empty());
    }
}
