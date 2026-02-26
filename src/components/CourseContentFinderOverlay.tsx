import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Box, Text } from "ink";
import Spinner from "ink-spinner";
import type { CourseTreeNodeKind, CourseTreeRow } from "./CoursePage.tsx";
import { COLORS } from "./colors.ts";
import { filterCourseContentByFuzzyQuery } from "./courseContentSearch.ts";
import TextInput from "./TextInput.tsx";
import { truncateText } from "./timetable/text.ts";

interface CourseContentFinderOverlayProps {
  termWidth: number;
  termHeight: number;
  rows: CourseTreeRow[];
  loading: boolean;
  onClose: () => void;
  onApplyRow: (rowId: string) => void;
}

export interface CourseContentTarget {
  id: string;
  label: string;
  mode: "all" | "module-type" | "row-kind";
  moduleType?: string;
  rowKind?: CourseTreeNodeKind;
}

const MODULE_TYPE_LABELS: Record<string, string> = {
  assign: "Assignments",
  quiz: "Quizzes",
  forum: "Forums",
  resource: "Resources",
  page: "Pages",
  book: "Books",
  folder: "Folders",
  url: "Link Activities",
};

const MODULE_TYPE_ORDER = ["assign", "quiz", "forum", "resource", "page", "book", "folder", "url"];

const KIND_TARGETS: CourseContentTarget[] = [
  { id: "kind:section", label: "Sections", mode: "row-kind", rowKind: "section" },
  { id: "kind:label", label: "Labels", mode: "row-kind", rowKind: "label" },
  { id: "kind:content-item", label: "Files & Items", mode: "row-kind", rowKind: "content-item" },
  { id: "kind:module-url", label: "URLs", mode: "row-kind", rowKind: "module-url" },
  { id: "kind:module-description", label: "Descriptions", mode: "row-kind", rowKind: "module-description" },
  { id: "kind:summary", label: "Summaries", mode: "row-kind", rowKind: "summary" },
];

function normalizeType(value: string | undefined): string {
  return (value || "").trim().toLowerCase();
}

function moduleTypeLabel(moduleType: string): string {
  const known = MODULE_TYPE_LABELS[moduleType];
  if (known) return known;
  if (!moduleType) return "Other Activities";

  const normalized = moduleType.replace(/[_-]+/g, " ").trim();
  if (!normalized) return "Other Activities";
  return `${normalized.charAt(0).toUpperCase()}${normalized.slice(1)} Activities`;
}

export function buildCourseContentTargets(rows: CourseTreeRow[]): CourseContentTarget[] {
  const moduleTypes = new Set<string>();
  rows.forEach((row) => {
    if (row.kind !== "module") return;
    const moduleType = normalizeType(row.moduleType);
    if (!moduleType) return;
    moduleTypes.add(moduleType);
  });

  const moduleTypeTargets = Array.from(moduleTypes)
    .sort((left, right) => {
      const leftIndex = MODULE_TYPE_ORDER.indexOf(left);
      const rightIndex = MODULE_TYPE_ORDER.indexOf(right);
      const leftRank = leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex;
      const rightRank = rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex;
      if (leftRank !== rightRank) return leftRank - rightRank;
      return left.localeCompare(right, undefined, { sensitivity: "base" });
    })
    .map((moduleType) => ({
      id: `module-type:${moduleType}`,
      label: moduleTypeLabel(moduleType),
      mode: "module-type" as const,
      moduleType,
    }));

  return [{ id: "all", label: "All", mode: "all" }, ...moduleTypeTargets, ...KIND_TARGETS];
}

export function filterRowsByTarget(
  rows: CourseTreeRow[],
  target: CourseContentTarget,
): CourseTreeRow[] {
  if (target.mode === "all") return rows;
  if (target.mode === "module-type") {
    const moduleType = normalizeType(target.moduleType);
    return rows.filter(
      (row) => row.kind === "module" && normalizeType(row.moduleType) === moduleType,
    );
  }
  return rows.filter((row) => row.kind === target.rowKind);
}

export function cycleCourseContentTargetIndex(
  currentIndex: number,
  delta: number,
  length: number,
): number {
  if (length <= 0) return 0;
  const normalized = currentIndex + delta;
  return ((normalized % length) + length) % length;
}

function renderSearchResultLine(row: CourseTreeRow): string {
  const indent = "  ".repeat(Math.max(0, row.depth));
  const indicator = row.collapsible ? "▾" : "•";
  return `${indent}${indicator} ${row.icon} ${row.text}`;
}

export default function CourseContentFinderOverlay({
  termWidth,
  termHeight,
  rows,
  loading,
  onClose,
  onApplyRow,
}: CourseContentFinderOverlayProps) {
  const [draft, setDraft] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [scrollOffset, setScrollOffset] = useState(0);
  const [targetIndex, setTargetIndex] = useState(0);
  const targets = useMemo(() => buildCourseContentTargets(rows), [rows]);
  const activeTarget = targets[targetIndex] ?? targets[0] ?? { id: "all", label: "All", mode: "all" };

  useEffect(() => {
    setTargetIndex((previous) => Math.min(previous, Math.max(targets.length - 1, 0)));
  }, [targets.length]);

  const targetedRows = useMemo(
    () => filterRowsByTarget(rows, activeTarget),
    [activeTarget, rows],
  );

  const searchResults = useMemo(
    () => filterCourseContentByFuzzyQuery(targetedRows, draft),
    [targetedRows, draft],
  );

  const maxModalWidth = Math.max(1, termWidth - 2);
  const maxModalHeight = Math.max(1, termHeight - 2);
  const modalWidth = Math.min(maxModalWidth, Math.max(28, Math.min(112, termWidth - 8)));
  const modalHeight = Math.min(maxModalHeight, Math.max(10, Math.min(30, termHeight - 4)));
  const resultRows = Math.max(1, modalHeight - 7);
  const maxLineWidth = Math.max(1, modalWidth - 6);

  const visibleResults = useMemo(
    () => searchResults.slice(scrollOffset, scrollOffset + resultRows),
    [resultRows, scrollOffset, searchResults],
  );

  useEffect(() => {
    setSelectedIdx((previous) => Math.min(previous, Math.max(searchResults.length - 1, 0)));
  }, [searchResults.length]);

  useEffect(() => {
    const maxScroll = Math.max(searchResults.length - resultRows, 0);
    setScrollOffset((previous) => Math.min(previous, maxScroll));
  }, [resultRows, searchResults.length]);

  useEffect(() => {
    if (selectedIdx < scrollOffset) {
      setScrollOffset(selectedIdx);
      return;
    }

    if (selectedIdx >= scrollOffset + resultRows) {
      setScrollOffset(selectedIdx - resultRows + 1);
    }
  }, [resultRows, scrollOffset, selectedIdx]);

  const moveSelection = useCallback(
    (delta: number) => {
      setSelectedIdx((previous) =>
        Math.max(0, Math.min(previous + delta, Math.max(searchResults.length - 1, 0))),
      );
    },
    [searchResults.length],
  );

  const applySelection = useCallback(
    (query: string) => {
      const instantResults = filterCourseContentByFuzzyQuery(targetedRows, query);
      const boundedIndex = Math.max(
        0,
        Math.min(selectedIdx, Math.max(instantResults.length - 1, 0)),
      );
      const selected = instantResults[boundedIndex];
      if (!selected) {
        onClose();
        return;
      }

      onApplyRow(selected.id);
    },
    [onApplyRow, onClose, selectedIdx, targetedRows],
  );

  const cycleTarget = useCallback((delta: number) => {
    setTargetIndex((previous) =>
      cycleCourseContentTargetIndex(previous, delta, targets.length),
    );
    setSelectedIdx(0);
    setScrollOffset(0);
  }, [targets.length]);

  const visibleStart = searchResults.length > 0 ? scrollOffset + 1 : 0;
  const visibleEnd = Math.min(searchResults.length, scrollOffset + visibleResults.length);

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
            Course Content Finder
          </Text>
          <Text dimColor>
            {truncateText(
              searchResults.length > 0
                ? `${activeTarget?.label || "All"} · ${Math.min(selectedIdx + 1, searchResults.length)}/${searchResults.length}`
                : "0/0",
              Math.max(1, modalWidth - 24),
            )}
          </Text>
        </Box>

        <Box>
          <Text color={COLORS.brand}>{"> "}</Text>
          <TextInput
            value={draft}
            onChange={(value) => {
              setDraft(value);
              setSelectedIdx(0);
              setScrollOffset(0);
            }}
            onSubmit={(value) => {
              applySelection(value);
            }}
            onKey={(_input, key) => {
              if (key.escape) {
                onClose();
                return true;
              }

              if (key.upArrow) {
                moveSelection(-1);
                return true;
              }

              if (key.downArrow) {
                moveSelection(1);
                return true;
              }

              if (key.pageUp) {
                moveSelection(-resultRows);
                return true;
              }

              if (key.pageDown) {
                moveSelection(resultRows);
                return true;
              }

              if (key.home) {
                setSelectedIdx(0);
                return true;
              }

              if (key.leftArrow) {
                cycleTarget(-1);
                return true;
              }

              if (key.rightArrow) {
                cycleTarget(1);
                return true;
              }

              if (key.end) {
                setSelectedIdx(Math.max(searchResults.length - 1, 0));
                return true;
              }

              return false;
            }}
            placeholder="section, activity, file, url, description"
            focus
          />
        </Box>

        <Box minHeight={1}>
          {loading ? (
            <Text color={COLORS.warning}>
              <Spinner type="dots" /> Loading course content...
            </Text>
          ) : (
            <Text dimColor>
              {truncateText(
                "Use ←/→ target type, ↑/↓ move, PgUp/PgDn, Home/End, Enter apply, Esc cancel.",
                Math.max(1, modalWidth - 4),
              )}
            </Text>
          )}
        </Box>

        <Box flexDirection="column" flexGrow={1} overflow="hidden">
          {!loading && searchResults.length === 0 && (
            <Text dimColor>No course content found for this query.</Text>
          )}

          {!loading &&
            visibleResults.map((row, idx) => {
              const absoluteIdx = scrollOffset + idx;
              const selected = absoluteIdx === selectedIdx;
              return (
                <Box key={row.id}>
                  <Text color={selected ? COLORS.brand : COLORS.neutral.gray} bold={selected}>
                    {selected ? "> " : "  "}
                  </Text>
                  <Text>{truncateText(renderSearchResultLine(row), maxLineWidth)}</Text>
                </Box>
              );
            })}
        </Box>

        <Box justifyContent="space-between">
          <Text dimColor>
            {searchResults.length > 0 ? `Showing ${visibleStart}-${visibleEnd}` : "Showing 0-0"}
          </Text>
          <Text dimColor>
            {searchResults.length > resultRows
              ? `Scroll ${scrollOffset}/${Math.max(searchResults.length - resultRows, 0)}`
              : " "}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}
