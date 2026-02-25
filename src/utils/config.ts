import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { homedir } from "os";
import { join } from "path";

export const DEFAULT_MOODLE_SERVICE = "moodle_mobile_app";

export interface MoodleSavedConfig {
  baseUrl: string;
  username: string;
  service: string;
}

export interface MoodleRuntimeConfig {
  baseUrl: string;
  username: string;
  service: string;
  password: string;
}

export function getAppConfigDir(): string {
  return process.env.TUI_MOODLE_CONFIG_DIR || join(homedir(), ".config", "tui-moodle");
}

export function getConfigFilePath(): string {
  return join(getAppConfigDir(), "config.json");
}

function normalizeBaseUrl(raw: string): string {
  return raw.trim().replace(/\/+$/, "");
}

export function loadConfig(): MoodleSavedConfig | null {
  try {
    const configFile = getConfigFilePath();
    if (!existsSync(configFile)) return null;

    const raw = readFileSync(configFile, "utf-8");
    const parsed = JSON.parse(raw) as Partial<MoodleSavedConfig> | null;
    if (!parsed || typeof parsed !== "object") return null;

    const baseUrl = typeof parsed.baseUrl === "string" ? normalizeBaseUrl(parsed.baseUrl) : "";
    const username = typeof parsed.username === "string" ? parsed.username.trim() : "";
    const service =
      typeof parsed.service === "string" && parsed.service.trim().length > 0
        ? parsed.service.trim()
        : DEFAULT_MOODLE_SERVICE;

    if (!baseUrl || !username) return null;
    return { baseUrl, username, service };
  } catch {
    return null;
  }
}

export function saveConfig(config: MoodleRuntimeConfig | MoodleSavedConfig): void {
  const persistedConfig: MoodleSavedConfig = {
    baseUrl: normalizeBaseUrl(config.baseUrl),
    username: config.username.trim(),
    service:
      config.service.trim().length > 0 ? config.service.trim() : DEFAULT_MOODLE_SERVICE,
  };

  const configDir = getAppConfigDir();
  mkdirSync(configDir, { recursive: true });
  writeFileSync(getConfigFilePath(), JSON.stringify(persistedConfig, null, 2), {
    mode: 0o600,
  });
}

export function clearConfig(): void {
  try {
    writeFileSync(getConfigFilePath(), "{}", { mode: 0o600 });
  } catch {
    // ignore
  }
}
