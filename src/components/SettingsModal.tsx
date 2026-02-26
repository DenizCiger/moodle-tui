import React, { useMemo } from "react";
import { Box, Text } from "ink";
import { COLORS } from "./colors.ts";
import { getShortcutSections, type TabId } from "./shortcuts.ts";
import { fitText, truncateText } from "./timetable/text.ts";

interface SettingsModalProps {
  activeTab: TabId;
  width: number;
  height: number;
}

export default function SettingsModal({ activeTab, width, height }: SettingsModalProps) {
  const sections = useMemo(() => getShortcutSections(activeTab), [activeTab]);
  const maxModalWidth = Math.max(1, width - 2);
  const maxModalHeight = Math.max(1, height - 1);
  const modalWidth = Math.min(maxModalWidth, Math.max(28, Math.min(96, width - 2)));
  const modalHeight = Math.min(maxModalHeight, Math.max(10, Math.min(30, height - 1)));
  const rowWidth = Math.max(1, modalWidth - 4);
  const separator = rowWidth >= 6 ? " - " : " ";
  const contentWidth = Math.max(1, rowWidth - separator.length);
  const keyColumnWidth = Math.max(1, Math.min(24, Math.floor(contentWidth * 0.34)));
  const actionWidth = Math.max(0, contentWidth - keyColumnWidth);
  const tabLabel = activeTab.charAt(0).toUpperCase() + activeTab.slice(1);

  return (
    <Box flexGrow={1} justifyContent="center" alignItems="center" height={height}>
      <Box
        flexDirection="column"
        width={modalWidth}
        height={modalHeight}
        borderStyle="round"
        borderColor={COLORS.brand}
        backgroundColor={COLORS.neutral.black}
        paddingX={1}
      >
        <Box justifyContent="space-between">
          <Text bold color={COLORS.brand}>
            Settings
          </Text>
          <Text dimColor>{truncateText(`Tab: ${tabLabel}`, Math.max(1, modalWidth - 16))}</Text>
        </Box>

        <Text dimColor>
          {truncateText("Keyboard shortcuts are grouped by context.", Math.max(1, modalWidth - 4))}
        </Text>

        <Box flexDirection="column" marginTop={1} overflow="hidden" flexGrow={1}>
          {sections.map((section) => (
            <Box key={section.title} flexDirection="column" marginBottom={1}>
              <Text bold>{section.title}</Text>
              {section.items.map((item) => (
                <Box key={item.id} width={rowWidth}>
                  <Text color={COLORS.warning}>{fitText(item.keys, keyColumnWidth)}</Text>
                  <Text dimColor>{separator}</Text>
                  <Text>{fitText(item.action, actionWidth)}</Text>
                </Box>
              ))}
            </Box>
          ))}
        </Box>
      </Box>
    </Box>
  );
}
