import React, { useMemo } from "react";
import { Box, Text } from "ink";
import type {
  MoodleAssignmentDetail,
  MoodleAssignmentSubmissionStatus,
} from "../utils/moodle.ts";
import { COLORS } from "./colors.ts";
import { fitText, truncateText } from "./timetable/text.ts";

interface AssignmentModalContext {
  courseName: string;
  moduleName: string;
  moduleDescription?: string;
}

interface AssignmentModalProps {
  termWidth: number;
  termHeight: number;
  context: AssignmentModalContext;
  loading: boolean;
  detailError: string;
  statusError: string;
  detail: MoodleAssignmentDetail | null;
  statusLoading: boolean;
  status: MoodleAssignmentSubmissionStatus | null;
  onClose: () => void;
}

const NAMED_HTML_ENTITIES: Record<string, string> = {
  amp: "&",
  lt: "<",
  gt: ">",
  quot: "\"",
  apos: "'",
  nbsp: " ",
};

const STATUS_GOOD_BG = "ansi256(22)";
const STATUS_NEUTRAL_BG = "ansi256(236)";
const STATUS_WARN_BG = "ansi256(130)";
const BUTTON_BG = "ansi256(243)";
const BUTTON_FG = "ansi256(16)";

function decodeHtmlEntities(value: string): string {
  return value.replace(/&(#x?[0-9a-fA-F]+|[a-zA-Z]+);/g, (full, entity: string) => {
    if (entity.startsWith("#x") || entity.startsWith("#X")) {
      const codePoint = Number.parseInt(entity.slice(2), 16);
      if (!Number.isFinite(codePoint) || codePoint <= 0) return full;
      return String.fromCodePoint(codePoint);
    }

    if (entity.startsWith("#")) {
      const codePoint = Number.parseInt(entity.slice(1), 10);
      if (!Number.isFinite(codePoint) || codePoint <= 0) return full;
      return String.fromCodePoint(codePoint);
    }

    return NAMED_HTML_ENTITIES[entity.toLowerCase()] ?? full;
  });
}

function stripHtml(value: string | undefined): string {
  if (!value) return "";
  return decodeHtmlEntities(value).replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

function formatDateTime(unixTimestamp: number | undefined): string {
  if (!unixTimestamp || unixTimestamp <= 0) return "-";
  const value = new Date(unixTimestamp * 1000);
  if (Number.isNaN(value.getTime())) return "-";

  const year = String(value.getFullYear());
  const month = String(value.getMonth() + 1).padStart(2, "0");
  const day = String(value.getDate()).padStart(2, "0");
  const hours = String(value.getHours()).padStart(2, "0");
  const minutes = String(value.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day} ${hours}:${minutes}`;
}

function formatBool(value: boolean | undefined): string {
  if (value === undefined) return "-";
  return value ? "Yes" : "No";
}

function prettifyStatus(value: string | undefined): string {
  if (!value) return "-";
  const compact = value.replace(/[\s_-]+/g, "").trim().toLowerCase();
  if (compact === "notgraded") return "Not graded";

  return value
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

function wrapText(value: string, width: number): string[] {
  if (width <= 0) return [""];
  if (!value.trim()) return [""];

  const words = value.trim().split(/\s+/g);
  const lines: string[] = [];
  let current = "";

  for (const word of words) {
    if (word.length > width) {
      if (current) {
        lines.push(current);
        current = "";
      }
      for (let index = 0; index < word.length; index += width) {
        lines.push(word.slice(index, index + width));
      }
      continue;
    }

    if (!current) {
      current = word;
      continue;
    }

    const next = `${current} ${word}`;
    if (next.length <= width) {
      current = next;
      continue;
    }

    lines.push(current);
    current = word;
  }

  if (current) lines.push(current);
  return lines.length > 0 ? lines : [""];
}

function formatRelativeDue(
  dueDate: number | undefined,
  availableWidth: number,
): string {
  if (!dueDate || dueDate <= 0) return "-";

  const reference = Math.floor(Date.now() / 1000);
  const secondsDelta = dueDate - reference;
  const absolute = Math.abs(secondsDelta);
  const days = Math.floor(absolute / 86400);
  const dayHours = Math.floor((absolute % 86400) / 3600);
  const minutes = Math.floor((absolute % 3600) / 60);
  const useDayFormat = days > 0 && availableWidth >= 19;
  const totalHours = Math.floor(absolute / 3600);
  const compact = useDayFormat
    ? `${days}d ${dayHours}h ${minutes}m`
    : `${totalHours}h ${minutes}m`;

  if (secondsDelta >= 0) return `${compact} remaining`;
  return `${compact} late`;
}

function buildStatusBadge(
  loading: boolean,
  detailError: string,
  statusError: string,
  statusLoading: boolean,
): string {
  if (loading) return "Loading";
  if (detailError) return "Error";
  if (statusLoading) return "Loading status";
  if (statusError) return "Partial";
  return "Ready";
}

function getStatusCellStyle(rowKey: string, value: string): { color?: string; backgroundColor?: string } {
  const normalizedValue = value.trim().toLowerCase();

  if (rowKey === "submission") {
    if (normalizedValue.includes("submitted")) {
      return { color: COLORS.neutral.white, backgroundColor: STATUS_GOOD_BG };
    }
    return { color: COLORS.neutral.white, backgroundColor: STATUS_WARN_BG };
  }

  if (rowKey === "grading") {
    const compact = normalizedValue.replace(/\s+/g, "");
    if (compact === "notgraded") {
      return { color: COLORS.neutral.white, backgroundColor: STATUS_NEUTRAL_BG };
    }

    if (normalizedValue.includes("graded")) {
      return { color: COLORS.neutral.white, backgroundColor: STATUS_GOOD_BG };
    }
    return { color: COLORS.neutral.white, backgroundColor: STATUS_NEUTRAL_BG };
  }

  if (rowKey === "time") {
    if (normalizedValue === "-" || normalizedValue.includes("loading")) {
      return { color: COLORS.neutral.white, backgroundColor: STATUS_NEUTRAL_BG };
    }

    if (normalizedValue.includes("late")) {
      return { color: COLORS.neutral.white, backgroundColor: STATUS_WARN_BG };
    }
    return { color: COLORS.neutral.white, backgroundColor: STATUS_GOOD_BG };
  }

  return { color: COLORS.neutral.white, backgroundColor: STATUS_NEUTRAL_BG };
}

export default function AssignmentModal({
  termWidth,
  termHeight,
  context,
  loading,
  detailError,
  statusError,
  detail,
  statusLoading,
  status,
  onClose,
}: AssignmentModalProps) {
  void onClose;

  const maxModalWidth = Math.max(1, termWidth - 2);
  const maxModalHeight = Math.max(1, termHeight - 2);
  const modalWidth = Math.min(maxModalWidth, Math.max(44, Math.min(118, termWidth - 6)));
  const modalHeight = Math.min(maxModalHeight, Math.max(16, Math.min(38, termHeight - 4)));
  const statusBadge = buildStatusBadge(loading, detailError, statusError, statusLoading);
  const title = detail?.name || context.moduleName || "Assignment";
  const tableContentWidth = Math.max(1, modalWidth - 5);
  const tableSeparatorWidth = tableContentWidth >= 3 ? 1 : 0;
  const tableCellWidth = Math.max(1, tableContentWidth - tableSeparatorWidth);
  const tableLabelWidth = Math.max(1, Math.min(26, Math.floor(tableCellWidth * 0.34)));
  const tableValueWidth = Math.max(1, tableCellWidth - tableLabelWidth);
  const divider = "─".repeat(Math.max(1, modalWidth - 4));
  const dateLineWidth = Math.max(1, modalWidth - 4);
  const dateLabelWidth = Math.max(1, Math.min(10, Math.floor(dateLineWidth * 0.2)));
  const dateValueWidth = Math.max(1, dateLineWidth - dateLabelWidth);
  const compactActions = modalWidth < 72;

  const descriptionLines = useMemo(() => {
    const source =
      stripHtml(detail?.intro) ||
      stripHtml(context.moduleDescription) ||
      "No assignment description available.";
    return wrapText(source, Math.max(1, modalWidth - 4)).slice(0, 2);
  }, [context.moduleDescription, detail?.intro, modalWidth]);

  const actionChips = useMemo(() => {
    const chips: string[] = [];
    if (status?.canEdit) chips.push("Edit submission");
    if (status?.canSubmit) chips.push("Submit changes");
    if (chips.length === 0) chips.push("No submission actions available");
    return chips.slice(0, 2);
  }, [status?.canEdit, status?.canSubmit]);

  const tableRows = [
    {
      key: "attempts",
      label: "Attempts allowed",
      value:
        detail?.maxattempts === -1
          ? "∞"
          : detail?.maxattempts !== undefined
            ? String(detail.maxattempts)
            : "-",
    },
    { key: "submission", label: "Submission status", value: prettifyStatus(status?.submissionStatus) },
    { key: "grading", label: "Grading status", value: prettifyStatus(status?.gradingStatus) },
    {
      key: "time",
      label: "Time",
      value: loading || statusLoading
        ? "Loading..."
        : formatRelativeDue(detail?.duedate, tableValueWidth - 1),
    },
    { key: "modified", label: "Last modified", value: formatDateTime(status?.lastModified) },
    { key: "can-submit", label: "Can submit", value: formatBool(status?.canSubmit) },
    { key: "can-edit", label: "Can edit", value: formatBool(status?.canEdit) },
    { key: "locked", label: "Locked", value: formatBool(status?.isLocked) },
  ];

  const headlineMessage = loading
    ? "Loading assignment details..."
    : detailError
      ? detailError
      : statusError
        ? statusError
        : statusLoading
          ? "Loading submission status..."
          : `${context.courseName}`;

  return (
    <Box
      position="absolute"
      width={termWidth}
      height={termHeight}
      justifyContent="center"
      alignItems="center"
    >
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
            {truncateText(title, Math.max(1, modalWidth - 18))}
          </Text>
          <Text color={COLORS.neutral.gray}>{statusBadge}</Text>
        </Box>

        <Box minHeight={1}>
          <Text color={detailError ? COLORS.error : statusError ? COLORS.warning : COLORS.neutral.gray}>
            {truncateText(headlineMessage, Math.max(1, modalWidth - 4))}
          </Text>
        </Box>

        <Text dimColor>{divider}</Text>

        <Box flexDirection="column">
          <Box>
            <Text bold>{fitText("Opened:", dateLabelWidth)}</Text>
            <Text>{truncateText(formatDateTime(detail?.allowsubmissionsfromdate), dateValueWidth)}</Text>
          </Box>
          <Box>
            <Text bold>{fitText("Due:", dateLabelWidth)}</Text>
            <Text>{truncateText(formatDateTime(detail?.duedate), dateValueWidth)}</Text>
          </Box>
          <Box>
            <Text bold>{fitText("Cutoff:", dateLabelWidth)}</Text>
            <Text>{truncateText(formatDateTime(detail?.cutoffdate), dateValueWidth)}</Text>
          </Box>
        </Box>

        <Box marginTop={1} minHeight={1}>
          <Text>{truncateText(descriptionLines[0] || "-", modalWidth - 4)}</Text>
        </Box>
        <Box minHeight={1}>
          <Text dimColor>{truncateText(descriptionLines[1] || "", modalWidth - 4)}</Text>
        </Box>

        <Box marginTop={1} flexDirection={compactActions ? "column" : "row"}>
          {actionChips.map((action, index) => (
            <Box
              key={action}
              marginRight={compactActions || index === actionChips.length - 1 ? 0 : 1}
              marginTop={compactActions && index > 0 ? 1 : 0}
            >
              <Text color={BUTTON_FG} backgroundColor={BUTTON_BG}>
                {` ${truncateText(
                  action,
                  compactActions
                    ? Math.max(1, modalWidth - 6)
                    : Math.max(1, Math.floor((modalWidth - 8) / 2)),
                )} `}
              </Text>
            </Box>
          ))}
        </Box>

        <Box marginTop={1}>
          <Text bold color={COLORS.brand}>
            Submission status
          </Text>
        </Box>

        <Box
          flexDirection="column"
          borderStyle="single"
          borderColor={COLORS.neutral.brightBlack}
          flexGrow={1}
          overflow="hidden"
        >
          {tableRows.map((row, index) => {
            const statusStyle = getStatusCellStyle(row.key, row.value);
            const labelBg = index % 2 === 0 ? COLORS.panel.header : COLORS.panel.alternate;
            return (
              <Box key={row.key}>
                <Text color={COLORS.neutral.white} backgroundColor={labelBg}>
                  {fitText(` ${row.label}`, tableLabelWidth)}
                </Text>
                {tableSeparatorWidth > 0 && <Text backgroundColor={COLORS.neutral.black}> </Text>}
                <Text
                  color={statusStyle.color}
                  backgroundColor={statusStyle.backgroundColor}
                >
                  {fitText(` ${truncateText(row.value, Math.max(1, tableValueWidth - 1))}`, tableValueWidth)}
                </Text>
              </Box>
            );
          })}
        </Box>

        <Box justifyContent="space-between">
          <Text dimColor>
            {modalWidth >= 54 ? (detail ? `Assignment ID ${detail.id}` : "Assignment ID -") : ""}
          </Text>
          <Text dimColor>Esc close</Text>
        </Box>
      </Box>
    </Box>
  );
}
