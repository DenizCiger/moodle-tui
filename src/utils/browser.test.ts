import { describe, expect, it } from "bun:test";
import { openUrlInBrowserWithDeps } from "./browser.ts";

describe("browser open", () => {
  it("uses open on macOS", () => {
    const calls: string[] = [];
    const result = openUrlInBrowserWithDeps("https://example.test", {
      platform: "darwin",
      commandExists: (command) => command === "open",
      runCommand: (command) => {
        calls.push(command);
      },
      env: {},
    });

    expect(result.ok).toBe(true);
    expect(calls).toEqual(["open"]);
  });

  it("falls back on linux when xdg-open fails", () => {
    const calls: string[] = [];
    const result = openUrlInBrowserWithDeps("https://example.test", {
      platform: "linux",
      commandExists: (command) => command === "xdg-open" || command === "sensible-browser",
      runCommand: (command) => {
        calls.push(command);
        if (command === "xdg-open") throw new Error("xdg-open failed");
      },
      env: {},
    });

    expect(result.ok).toBe(true);
    expect(calls).toEqual(["xdg-open", "sensible-browser"]);
  });

  it("returns error on linux when no launcher exists", () => {
    const result = openUrlInBrowserWithDeps("https://example.test", {
      platform: "linux",
      commandExists: () => false,
      runCommand: () => {},
      env: {},
    });

    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.message.includes("xdg-open")).toBe(true);
    }
  });

  it("uses cmd start on windows", () => {
    const calls: Array<{ command: string; args: string[] }> = [];
    const result = openUrlInBrowserWithDeps("https://example.test", {
      platform: "win32",
      commandExists: (command) => command === "cmd",
      runCommand: (command, args) => {
        calls.push({ command, args });
      },
      env: {},
    });

    expect(result.ok).toBe(true);
    expect(calls[0]?.command).toBe("cmd");
    expect(calls[0]?.args).toEqual(["/c", "start", "", "https://example.test"]);
  });
});
