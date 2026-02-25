import { execFileSync } from "child_process";

export type BrowserOpenResult = { ok: true } | { ok: false; message: string };

interface BrowserCommandOptions {
  env?: NodeJS.ProcessEnv;
}

interface BrowserDeps {
  platform: NodeJS.Platform;
  commandExists: (command: string) => boolean;
  runCommand: (command: string, args: string[], options?: BrowserCommandOptions) => void;
  env: NodeJS.ProcessEnv;
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

function defaultRunCommand(command: string, args: string[], options?: BrowserCommandOptions): void {
  execFileSync(command, args, {
    encoding: "utf-8",
    stdio: ["ignore", "pipe", "pipe"],
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

export function openUrlInBrowserWithDeps(url: string, deps: BrowserDeps): BrowserOpenResult {
  const target = url.trim();
  if (!target) return { ok: false, message: "Cannot open an empty URL." };

  try {
    if (deps.platform === "darwin") {
      if (!deps.commandExists("open")) {
        return { ok: false, message: "Browser launcher 'open' is not available." };
      }

      deps.runCommand("open", [target], { env: deps.env });
      return { ok: true };
    }

    if (deps.platform === "linux") {
      const candidates: Array<{ command: string; args: string[] }> = [
        { command: "xdg-open", args: [target] },
        { command: "sensible-browser", args: [target] },
      ];
      const available = candidates.filter((candidate) => deps.commandExists(candidate.command));
      if (available.length === 0) {
        return {
          ok: false,
          message: "No browser launcher found (tried xdg-open, sensible-browser).",
        };
      }

      let lastError = "Browser launcher failed.";
      for (const candidate of available) {
        try {
          deps.runCommand(candidate.command, candidate.args, { env: deps.env });
          return { ok: true };
        } catch (error) {
          lastError = getErrorMessage(error);
        }
      }

      return { ok: false, message: lastError };
    }

    if (deps.platform === "win32") {
      if (!deps.commandExists("cmd")) {
        return { ok: false, message: "Windows command shell is not available." };
      }

      deps.runCommand("cmd", ["/c", "start", "", target], { env: deps.env });
      return { ok: true };
    }

    return { ok: false, message: `Browser opening is not supported on '${deps.platform}'.` };
  } catch (error) {
    return { ok: false, message: getErrorMessage(error) };
  }
}

export function openUrlInBrowser(url: string): BrowserOpenResult {
  return openUrlInBrowserWithDeps(url, {
    platform: process.platform,
    commandExists: defaultCommandExists,
    runCommand: defaultRunCommand,
    env: process.env,
  });
}
