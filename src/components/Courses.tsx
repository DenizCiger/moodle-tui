import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Box, Text, useInput, useStdout } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import { fitText, truncateText } from "./timetable/text.ts";
import type { MoodleRuntimeConfig } from "../utils/config.ts";
import type { MoodleCourse } from "../utils/moodle.ts";
import { fetchCourses } from "../utils/moodle.ts";
import { getCachedCourses, saveCoursesToCache } from "../utils/cache.ts";
import { isShortcutPressed } from "./shortcuts.ts";

interface CoursesProps {
  config: MoodleRuntimeConfig;
  topInset?: number;
  inputEnabled?: boolean;
}

function stripHtml(value: string | undefined): string {
  if (!value) return "";
  return value.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

function formatCategory(course: MoodleCourse): string {
  const id = course.categoryid !== undefined ? String(course.categoryid) : "";
  const name = course.categoryname?.trim() || "";
  if (id && name) return `${id} (${name})`;
  if (id) return id;
  if (name) return name;
  return "-";
}

function formatProgress(progress: number | null | undefined): string {
  if (progress === undefined || progress === null) return "-";
  return `${Math.round(progress)}%`;
}

function formatVisible(visible: number | undefined): string {
  if (visible === undefined) return "-";
  return visible ? "Yes" : "No";
}

export default function Courses({ config, topInset = 0, inputEnabled = true }: CoursesProps) {
  const { stdout } = useStdout();
  const [courses, setCourses] = useState<MoodleCourse[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [dataSource, setDataSource] = useState<"live" | "cache" | "none">("none");

  const loadCourses = useCallback(
    async ({ forceRefresh }: { forceRefresh: boolean }) => {
      setLoading(true);
      if (!forceRefresh) {
        setError("");
      }

      try {
        const liveCourses = await fetchCourses(config);
        setCourses(liveCourses);
        setSelectedIdx((prev) => Math.min(prev, Math.max(0, liveCourses.length - 1)));
        setDataSource("live");
        setError("");
        saveCoursesToCache(liveCourses);
      } catch (loadError) {
        const message =
          loadError instanceof Error ? loadError.message : "Unknown Moodle API error";

        if (!forceRefresh) {
          const cachedCourses = getCachedCourses();
          if (cachedCourses && cachedCourses.length > 0) {
            setCourses(cachedCourses);
            setSelectedIdx((prev) => Math.min(prev, Math.max(0, cachedCourses.length - 1)));
            setDataSource("cache");
            setError(`Live sync failed; showing cached courses. ${message}`);
          } else {
            setDataSource("none");
            setCourses([]);
            setSelectedIdx(0);
            setError(`Failed to load courses: ${message}`);
          }
        } else {
          setError(`Refresh failed: ${message}`);
        }
      } finally {
        setLoading(false);
      }
    },
    [config],
  );

  useEffect(() => {
    void loadCourses({ forceRefresh: false });
  }, [loadCourses]);

  useEffect(() => {
    setSelectedIdx((prev) => Math.min(prev, Math.max(0, courses.length - 1)));
  }, [courses.length]);

  const selectedCourse = courses[selectedIdx] ?? null;
  const termWidth = Math.max(70, stdout?.columns ?? 120);
  const termHeight = Math.max(18, (stdout?.rows ?? 24) - topInset);
  const bodyHeight = Math.max(8, termHeight - 5);
  const pageJump = Math.max(4, Math.floor(bodyHeight / 3));
  const listRows = Math.max(3, bodyHeight - 3);
  const visibleStart = Math.min(
    Math.max(0, selectedIdx - Math.floor(listRows / 2)),
    Math.max(0, courses.length - listRows),
  );
  const visibleCourses = courses.slice(visibleStart, visibleStart + listRows);

  const shortNameWidth = Math.max(12, Math.floor((termWidth - 8) * 0.24));
  const fullNameWidth = Math.max(18, termWidth - shortNameWidth - 8);

  const selectedMeta = useMemo(() => {
    if (!selectedCourse) return "No course selected.";

    return truncateText(
      `Category: ${formatCategory(selectedCourse)} | Visible: ${formatVisible(selectedCourse.visible)} | Progress: ${formatProgress(selectedCourse.progress)} | URL: ${selectedCourse.courseurl || "-"}`,
      Math.max(16, termWidth - 2),
    );
  }, [selectedCourse, termWidth]);

  const selectedSummary = useMemo(() => {
    if (!selectedCourse) return "";
    const summary = stripHtml(selectedCourse.summary);
    return truncateText(summary || "No course summary available.", Math.max(16, termWidth - 2));
  }, [selectedCourse, termWidth]);

  useInput(
    (input, key) => {
      if (isShortcutPressed("courses-refresh", input, key)) {
        void loadCourses({ forceRefresh: true });
        return;
      }

      if (isShortcutPressed("courses-up", input, key)) {
        setSelectedIdx((prev) => Math.max(0, prev - 1));
        return;
      }

      if (isShortcutPressed("courses-down", input, key)) {
        setSelectedIdx((prev) => Math.min(Math.max(0, courses.length - 1), prev + 1));
        return;
      }

      if (isShortcutPressed("courses-page-up", input, key)) {
        setSelectedIdx((prev) => Math.max(0, prev - pageJump));
        return;
      }

      if (isShortcutPressed("courses-page-down", input, key)) {
        setSelectedIdx((prev) =>
          Math.min(Math.max(0, courses.length - 1), prev + pageJump),
        );
        return;
      }

      if (isShortcutPressed("courses-home", input, key)) {
        setSelectedIdx(0);
        return;
      }

      if (isShortcutPressed("courses-end", input, key)) {
        setSelectedIdx(Math.max(0, courses.length - 1));
      }
    },
    { isActive: inputEnabled && Boolean(process.stdin.isTTY) },
  );

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight}>
      <Box justifyContent="space-between">
        <Text bold color={COLORS.brand}>
          Moodle Courses
        </Text>
        <Text dimColor>{truncateText(`${config.username} @ ${config.baseUrl}`, 56)}</Text>
      </Box>

      <Box justifyContent="space-between">
        <Text dimColor>{`Enrolled courses: ${courses.length}`}</Text>
        <Text color={dataSource === "cache" ? COLORS.warning : COLORS.neutral.gray}>
          {dataSource === "live"
            ? "Source: live"
            : dataSource === "cache"
              ? "Source: cache fallback"
              : "Source: none"}
        </Text>
      </Box>

      <Box
        flexDirection="column"
        marginTop={1}
        height={bodyHeight}
        borderStyle="single"
        borderColor={COLORS.neutral.brightBlack}
      >
        <Box justifyContent="space-between" paddingX={1}>
          <Text bold>Course List</Text>
          <Text dimColor>{courses.length > 0 ? `${selectedIdx + 1}/${courses.length}` : "0/0"}</Text>
        </Box>

        {loading ? (
          <Box justifyContent="center" alignItems="center" flexGrow={1}>
            <Text color={COLORS.warning}>
              <Spinner type="dots" /> Loading courses...
            </Text>
          </Box>
        ) : courses.length === 0 ? (
          <Box justifyContent="center" alignItems="center" flexGrow={1} paddingX={1}>
            <Text color={COLORS.warning}>No enrolled courses returned.</Text>
          </Box>
        ) : (
          <Box flexDirection="column" paddingX={1}>
            <Box>
              <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                {"  "}
              </Text>
              <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                {fitText("Short", shortNameWidth)}
              </Text>
              <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                {" "}
              </Text>
              <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                {fitText("Name", fullNameWidth)}
              </Text>
            </Box>

            {visibleCourses.map((course, index) => {
              const actualIndex = visibleStart + index;
              const isSelected = actualIndex === selectedIdx;
              const rowBackgroundColor = isSelected
                ? COLORS.panel.selected
                : actualIndex % 2 === 1
                  ? COLORS.panel.alternate
                  : undefined;

              return (
                <Box key={course.id}>
                  <Text
                    color={isSelected ? COLORS.brand : COLORS.neutral.brightBlack}
                    backgroundColor={rowBackgroundColor}
                    bold={isSelected}
                  >
                    {isSelected ? "> " : "  "}
                  </Text>
                  <Text
                    color={COLORS.neutral.white}
                    backgroundColor={rowBackgroundColor}
                    bold={isSelected}
                  >
                    {fitText(course.shortname || "-", shortNameWidth)}
                  </Text>
                  <Text backgroundColor={rowBackgroundColor}> </Text>
                  <Text
                    color={COLORS.neutral.white}
                    backgroundColor={rowBackgroundColor}
                    bold={isSelected}
                  >
                    {fitText(course.fullname, fullNameWidth)}
                  </Text>
                </Box>
              );
            })}
          </Box>
        )}
      </Box>

      <Box marginTop={1}>
        <Text dimColor>{selectedMeta}</Text>
      </Box>

      {selectedSummary ? (
        <Box>
          <Text dimColor>{selectedSummary}</Text>
        </Box>
      ) : null}

      {error && (
        <Box marginTop={1}>
          <Text color={COLORS.error}>{truncateText(error, Math.max(16, termWidth - 2))}</Text>
        </Box>
      )}
    </Box>
  );
}
