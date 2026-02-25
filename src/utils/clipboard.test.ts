import { describe, expect, it } from "bun:test";
import { copyTextToClipboardWithDeps } from "./clipboard.ts";

describe("clipboard copy", () => {
  it("uses wl-copy first on linux when available", () => {
    const calls: string[] = [];
    const result = copyTextToClipboardWithDeps("https://example.test", {
      platform: "linux",
      commandExists: (command) => command === "wl-copy",
      runCommand: (command) => {
        calls.push(command);
      },
      env: {},
    });

    expect(result.ok).toBe(true);
    expect(calls).toEqual(["wl-copy"]);
  });

  it("falls back to xclip when wl-copy fails", () => {
    const calls: string[] = [];
    const result = copyTextToClipboardWithDeps("https://example.test", {
      platform: "linux",
      commandExists: (command) => command === "wl-copy" || command === "xclip",
      runCommand: (command) => {
        calls.push(command);
        if (command === "wl-copy") {
          throw new Error("wl-copy failed");
        }
      },
      env: {},
    });

    expect(result.ok).toBe(true);
    expect(calls).toEqual(["wl-copy", "xclip"]);
  });

  it("returns error when linux has no clipboard backends", () => {
    const result = copyTextToClipboardWithDeps("https://example.test", {
      platform: "linux",
      commandExists: () => false,
      runCommand: () => {},
      env: {},
    });

    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.message.includes("wl-copy")).toBe(true);
    }
  });

  it("uses powershell on windows", () => {
    const calls: Array<{ command: string; args: string[]; envText?: string }> = [];
    const result = copyTextToClipboardWithDeps("hello", {
      platform: "win32",
      commandExists: (command) => command === "pwsh",
      runCommand: (command, args, options) => {
        calls.push({
          command,
          args,
          envText: options?.env?.TUI_MOODLE_CLIPBOARD_TEXT,
        });
      },
      env: {},
    });

    expect(result.ok).toBe(true);
    expect(calls[0]?.command).toBe("pwsh");
    expect(calls[0]?.args.includes("Set-Clipboard -Value $env:TUI_MOODLE_CLIPBOARD_TEXT")).toBe(
      true,
    );
    expect(calls[0]?.envText).toBe("hello");
  });

  it("returns error on windows when no powershell exists", () => {
    const result = copyTextToClipboardWithDeps("hello", {
      platform: "win32",
      commandExists: () => false,
      runCommand: () => {},
      env: {},
    });

    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.message.toLowerCase().includes("powershell")).toBe(true);
    }
  });
});
