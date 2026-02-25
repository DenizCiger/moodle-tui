import { afterEach, describe, expect, it } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";
import {
  DEFAULT_MOODLE_SERVICE,
  getConfigFilePath,
  loadConfig,
  saveConfig,
  type MoodleRuntimeConfig,
} from "./config.ts";

const tempDirs: string[] = [];

function withTempConfigDir(): string {
  const dir = mkdtempSync(join(tmpdir(), "tui-moodle-config-"));
  tempDirs.push(dir);
  process.env.TUI_MOODLE_CONFIG_DIR = dir;
  return dir;
}

afterEach(() => {
  delete process.env.TUI_MOODLE_CONFIG_DIR;
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) rmSync(dir, { recursive: true, force: true });
  }
});

describe("config storage", () => {
  it("saves and loads config values", () => {
    withTempConfigDir();
    const config: MoodleRuntimeConfig = {
      baseUrl: "https://moodle.school.tld/",
      username: "student1",
      password: "secret",
      service: "moodle_mobile_app",
    };

    saveConfig(config);
    const loaded = loadConfig();

    expect(loaded).not.toBeNull();
    expect(loaded?.baseUrl).toBe("https://moodle.school.tld");
    expect(loaded?.username).toBe("student1");
    expect(loaded?.service).toBe("moodle_mobile_app");
  });

  it("injects default service when missing in config file", () => {
    withTempConfigDir();
    writeFileSync(
      getConfigFilePath(),
      JSON.stringify({ baseUrl: "https://moodle.school.tld", username: "student1" }),
    );

    const loaded = loadConfig();
    expect(loaded).not.toBeNull();
    expect(loaded?.service).toBe(DEFAULT_MOODLE_SERVICE);
  });
});
