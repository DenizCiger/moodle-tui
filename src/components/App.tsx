import React, { useEffect, useState } from "react";
import { Box, Text } from "ink";
import {
  clearConfig,
  loadConfig,
  saveConfig,
  type MoodleRuntimeConfig,
  type MoodleSavedConfig,
} from "../utils/config.ts";
import { clearCache } from "../utils/cache.ts";
import {
  clearPassword,
  getSecureStorageDiagnostic,
  loadPassword,
  savePassword,
} from "../utils/secret.ts";
import { testCredentials } from "../utils/moodle.ts";
import Login from "./Login.tsx";
import MainShell from "./MainShell.tsx";

type Screen = "loading" | "login" | "app";

export default function App() {
  const [screen, setScreen] = useState<Screen>("loading");
  const [savedConfig, setSavedConfig] = useState<MoodleSavedConfig | null>(null);
  const [config, setConfig] = useState<MoodleRuntimeConfig | null>(null);
  const [error, setError] = useState("");
  const [secureStorageNotice, setSecureStorageNotice] = useState("");

  useEffect(() => {
    let cancelled = false;

    async function init() {
      const storage = getSecureStorageDiagnostic();
      if (!cancelled) {
        setSecureStorageNotice(storage.available ? "" : storage.message);
      }

      const saved = loadConfig();
      if (!saved) {
        if (!cancelled) setScreen("login");
        return;
      }

      if (cancelled) return;
      setSavedConfig(saved);

      const password = await loadPassword(saved);
      if (cancelled) return;

      if (!password) {
        setScreen("login");
        return;
      }

      const runtimeConfig: MoodleRuntimeConfig = { ...saved, password };
      const loginResult = await testCredentials(runtimeConfig);
      if (cancelled) return;

      if (!loginResult.ok) {
        setError(`Auto-login failed: ${loginResult.message}`);
        setScreen("login");
        return;
      }

      setConfig(runtimeConfig);
      setScreen("app");
    }

    void init();
    return () => {
      cancelled = true;
    };
  }, []);

  const handleLogin = async (newConfig: MoodleRuntimeConfig) => {
    const nextSaved: MoodleSavedConfig = {
      baseUrl: newConfig.baseUrl,
      username: newConfig.username,
      service: newConfig.service,
    };

    try {
      saveConfig(newConfig);
      setSavedConfig(nextSaved);
      setError("");
    } catch {
      setError("Login succeeded, but profile settings could not be saved to disk.");
    }

    try {
      await savePassword(nextSaved, newConfig.password);
    } catch {
      setError(
        "Login succeeded, but secure password storage failed. You will need to log in again next time.",
      );
    }

    setConfig(newConfig);
    setScreen("app");
  };

  const handleLogout = () => {
    const activeProfile =
      config
        ? {
            baseUrl: config.baseUrl,
            username: config.username,
            service: config.service,
          }
        : savedConfig;

    if (activeProfile) {
      void clearPassword(activeProfile);
    }

    clearConfig();
    clearCache();
    setSavedConfig(null);
    setConfig(null);
    setError("");
    setScreen("login");
  };

  if (screen === "loading") {
    return (
      <Box padding={1}>
        <Text dimColor>Loading...</Text>
      </Box>
    );
  }

  if (screen === "login") {
    return (
      <Login
        onLogin={handleLogin}
        initialConfig={savedConfig}
        error={error}
        secureStorageNotice={secureStorageNotice}
      />
    );
  }

  if (screen === "app" && config) {
    return <MainShell config={config} onLogout={handleLogout} />;
  }

  return null;
}
