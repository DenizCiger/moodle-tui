import React, { useMemo, useState } from "react";
import { Box, Text, useApp, useInput, useStdout } from "ink";
import type { MoodleRuntimeConfig } from "../utils/config.ts";
import { COLORS } from "./colors.ts";
import Dashboard from "./Dashboard.tsx";
import { InputCaptureProvider } from "./inputCapture.tsx";
import SettingsModal from "./SettingsModal.tsx";
import { isShortcutPressed, type TabId } from "./shortcuts.ts";
import { truncateText } from "./timetable/text.ts";

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
  const [activeTab] = useState<TabId>("dashboard");
  const [tabLabel, setTabLabel] = useState("Dashboard");
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

  const termWidth = stdout?.columns && stdout.columns > 0 ? stdout.columns : 120;
  const availableRows = stdout?.rows && stdout.rows > 0 ? stdout.rows : 24;
  const termHeight = Math.max(1, availableRows - 2);
  const tabs = useMemo(() => [{ id: "dashboard" as const, label: tabLabel }], [tabLabel]);
  const settingsHint = settingsOpen
    ? "Settings"
    : termWidth < 64
      ? "? settings"
      : "Press ? for settings";
  const tabLabelWidth = Math.max(1, Math.min(Math.floor(termWidth * 0.45), termWidth - 2));
  const settingsWidth = Math.max(1, Math.min(Math.floor(termWidth * 0.5), termWidth - 2));

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight + 2}>
      <Box justifyContent="space-between">
        <Box flexDirection="row">
          {tabs.map((tab) => (
            <Box key={tab.id}>
              <TabButton
                label={truncateText(tab.label, tabLabelWidth)}
                active={tab.id === activeTab}
              />
            </Box>
          ))}
        </Box>
        <Box flexGrow={1} justifyContent="flex-end">
          <Text
            color={COLORS.neutral.white}
            bold={settingsOpen}
            dimColor={!settingsOpen}
          >
            {truncateText(settingsHint, settingsWidth)}
          </Text>
        </Box>
      </Box>

      <InputCaptureProvider onBlockedChange={setGlobalShortcutsBlocked}>
        <Dashboard
          config={config}
          topInset={2}
          inputEnabled={!settingsOpen}
          onTabLabelChange={setTabLabel}
        />
      </InputCaptureProvider>

      {settingsOpen && (
        <Box position="absolute" width={termWidth} height={termHeight + 2}>
          <SettingsModal activeTab={activeTab} width={termWidth} height={termHeight + 2} />
        </Box>
      )}
    </Box>
  );
}
