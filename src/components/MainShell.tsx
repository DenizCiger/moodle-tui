import React, { useMemo, useState } from "react";
import { Box, Text, useApp, useInput, useStdout } from "ink";
import type { MoodleRuntimeConfig } from "../utils/config.ts";
import { COLORS } from "./colors.ts";
import Courses from "./Courses.tsx";
import { InputCaptureProvider } from "./inputCapture.tsx";
import SettingsModal from "./SettingsModal.tsx";
import { isShortcutPressed, type TabId } from "./shortcuts.ts";

interface MainShellProps {
  config: MoodleRuntimeConfig;
  onLogout: () => void;
}

const INACTIVE_TAB_BACKGROUND = "ansi256(238)";

function TabButton({ label, active }: { label: string; active: boolean }) {
  return (
    <Text
      color={active ? COLORS.neutral.black : COLORS.neutral.white}
      backgroundColor={active ? COLORS.brand : INACTIVE_TAB_BACKGROUND}
      bold={active}
    >
      {` ${label} `}
    </Text>
  );
}

export default function MainShell({ config, onLogout }: MainShellProps) {
  const { exit } = useApp();
  const { stdout } = useStdout();
  const [activeTab] = useState<TabId>("courses");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [globalShortcutsBlocked, setGlobalShortcutsBlocked] = useState(false);

  useInput(
    (input, key) => {
      if (settingsOpen) {
        if (isShortcutPressed("settings-close", input, key)) {
          setSettingsOpen(false);
        }
        return;
      }

      if (globalShortcutsBlocked) return;

      if (isShortcutPressed("settings-open", input, key)) {
        setSettingsOpen(true);
        return;
      }

      if (isShortcutPressed("quit", input, key)) {
        exit();
        return;
      }

      if (isShortcutPressed("logout", input, key)) {
        onLogout();
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  const termWidth = Math.max(50, stdout?.columns ?? 120);
  const termHeight = Math.max(18, (stdout?.rows ?? 24) - 2);
  const tabs = useMemo(() => [{ id: "courses" as const, label: "Courses" }], []);

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight + 2}>
      <Box justifyContent="space-between">
        <Box flexDirection="row" minWidth={20}>
          {tabs.map((tab) => (
            <Box key={tab.id}>
              <TabButton label={tab.label} active={tab.id === activeTab} />
            </Box>
          ))}
        </Box>
        <Box flexGrow={1} />
        <Box minWidth={22} justifyContent="flex-end">
          <Text
            color={COLORS.neutral.white}
            bold={settingsOpen}
            dimColor={!settingsOpen}
          >
            {settingsOpen ? "Settings" : "Press ? for settings"}
          </Text>
        </Box>
      </Box>

      <InputCaptureProvider onBlockedChange={setGlobalShortcutsBlocked}>
        <Courses config={config} topInset={2} inputEnabled={!settingsOpen} />
      </InputCaptureProvider>

      {settingsOpen && (
        <Box position="absolute" width={termWidth} height={termHeight + 2}>
          <SettingsModal activeTab={activeTab} width={termWidth} height={termHeight + 2} />
        </Box>
      )}
    </Box>
  );
}
