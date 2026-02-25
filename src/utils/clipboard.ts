import { execFileSync } from "child_process";

export type ClipboardCopyResult = { ok: true } | { ok: false; message: string };

interface ClipboardCommandOptions {
  input?: string;
  env?: NodeJS.ProcessEnv;
}

interface ClipboardDeps {
  platform: NodeJS.Platform;
  commandExists: (command: string) => boolean;
  runCommand: (command: string, args: string[], options?: ClipboardCommandOptions) => void;
  env: NodeJS.ProcessEnv;
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

function defaultRunCommand(command: string, args: string[], options?: ClipboardCommandOptions): void {
  execFileSync(command, args, {
    encoding: "utf-8",
    stdio: ["pipe", "pipe", "pipe"],
    input: options?.input,
    env: options?.env,
  });
}

function defaultCommandExists(command: string): boolean {
  try {
    if (process.platform === "win32") {
      execFileSync("where", [command], { stdio: "ignore" });
      return true;
    }

    execFileSync("which", [command], { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

export function copyTextToClipboardWithDeps(
  text: string,
  deps: ClipboardDeps,
): ClipboardCopyResult {
  try {
    if (deps.platform === "darwin") {
      if (!deps.commandExists("pbcopy")) {
        return { ok: false, message: "Clipboard backend 'pbcopy' is not available." };
      }

      deps.runCommand("pbcopy", [], { input: text, env: deps.env });
      return { ok: true };
    }

    if (deps.platform === "linux") {
      const candidates: Array<{ command: string; args: string[] }> = [
        { command: "wl-copy", args: [] },
        { command: "xclip", args: ["-selection", "clipboard"] },
        { command: "xsel", args: ["--clipboard", "--input"] },
      ];

      const available = candidates.filter((candidate) => deps.commandExists(candidate.command));
      if (available.length === 0) {
        return {
          ok: false,
          message: "No clipboard utility found (tried wl-copy, xclip, xsel).",
        };
      }

      let lastError = "Clipboard command failed.";
      for (const candidate of available) {
        try {
          deps.runCommand(candidate.command, candidate.args, { input: text, env: deps.env });
          return { ok: true };
        } catch (error) {
          lastError = getErrorMessage(error);
        }
      }

      return { ok: false, message: lastError };
    }

    if (deps.platform === "win32") {
      const shells = ["powershell.exe", "powershell", "pwsh.exe", "pwsh"];
      const shell = shells.find((candidate) => deps.commandExists(candidate));
      if (!shell) {
        return {
          ok: false,
          message: "No PowerShell executable found for clipboard access.",
        };
      }

      deps.runCommand(
        shell,
        ["-NoProfile", "-NonInteractive", "-Command", "Set-Clipboard -Value $env:TUI_MOODLE_CLIPBOARD_TEXT"],
        {
          env: {
            ...deps.env,
            TUI_MOODLE_CLIPBOARD_TEXT: text,
          },
        },
      );
      return { ok: true };
    }

    return { ok: false, message: `Clipboard copy is not supported on '${deps.platform}'.` };
  } catch (error) {
    return { ok: false, message: getErrorMessage(error) };
  }
}

export function copyTextToClipboard(text: string): ClipboardCopyResult {
  return copyTextToClipboardWithDeps(text, {
    platform: process.platform,
    commandExists: defaultCommandExists,
    runCommand: defaultRunCommand,
    env: process.env,
  });
}
