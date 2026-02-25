import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Box, Text, useStdout } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import CoursePage, { buildCourseContentLines } from "./CoursePage.tsx";
import { filterCoursesByFuzzyQuery } from "./courseSearch.ts";
import TextInput from "./TextInput.tsx";
import { useInputCapture } from "./inputCapture.tsx";
import { isShortcutPressed } from "./shortcuts.ts";
import { fitText, truncateText } from "./timetable/text.ts";
import { useStableInput } from "./useStableInput.ts";
import { getCachedCourses, saveCoursesToCache } from "../utils/cache.ts";
import type { MoodleRuntimeConfig } from "../utils/config.ts";
import {
  fetchCourseContents,
  fetchCourses,
  type MoodleCourse,
  type MoodleCourseSection,
} from "../utils/moodle.ts";

interface CoursesProps {
  config: MoodleRuntimeConfig;
  topInset?: number;
  inputEnabled?: boolean;
}

type ViewMode = "list" | "course";

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

  const [viewMode, setViewMode] = useState<ViewMode>("list");
  const [activeCourseId, setActiveCourseId] = useState<number | null>(null);
  const [courseSectionsById, setCourseSectionsById] = useState<Record<number, MoodleCourseSection[]>>(
    {},
  );
  const [coursePageLoading, setCoursePageLoading] = useState(false);
  const [coursePageError, setCoursePageError] = useState("");
  const [courseScrollOffset, setCourseScrollOffset] = useState(0);

  const [searchQuery, setSearchQuery] = useState("");
  const [searchMode, setSearchMode] = useState(false);

  useInputCapture(searchMode);

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

  const loadCourseContents = useCallback(
    async (courseId: number, forceRefresh = false) => {
      if (!forceRefresh && courseSectionsById[courseId]) {
        setCoursePageError("");
        return;
      }

      setCoursePageLoading(true);
      setCoursePageError("");
      try {
        const sections = await fetchCourseContents(config, courseId);
        setCourseSectionsById((previous) => ({ ...previous, [courseId]: sections }));
      } catch (loadError) {
        const message =
          loadError instanceof Error ? loadError.message : "Unknown Moodle API error";
        setCoursePageError(`Failed to load course page: ${message}`);
      } finally {
        setCoursePageLoading(false);
      }
    },
    [config, courseSectionsById],
  );

  useEffect(() => {
    void loadCourses({ forceRefresh: false });
  }, [loadCourses]);

  const filteredCourses = useMemo(
    () => filterCoursesByFuzzyQuery(courses, searchQuery),
    [courses, searchQuery],
  );

  useEffect(() => {
    setSelectedIdx((prev) => Math.min(prev, Math.max(0, filteredCourses.length - 1)));
  }, [filteredCourses.length]);

  const selectedCourse = filteredCourses[selectedIdx] ?? null;
  const activeCourse = useMemo(() => {
    if (activeCourseId === null) return null;
    return courses.find((course) => course.id === activeCourseId) ?? null;
  }, [activeCourseId, courses]);

  const termWidth = Math.max(70, stdout?.columns ?? 120);
  const termHeight = Math.max(18, (stdout?.rows ?? 24) - topInset);
  const bodyHeight = Math.max(8, termHeight - 7);
  const pageJump = Math.max(4, Math.floor(bodyHeight / 3));

  const listRows = Math.max(3, bodyHeight - 3);
  const visibleStart = Math.min(
    Math.max(0, selectedIdx - Math.floor(listRows / 2)),
    Math.max(0, filteredCourses.length - listRows),
  );
  const visibleCourses = filteredCourses.slice(visibleStart, visibleStart + listRows);

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

  const activeSections: MoodleCourseSection[] =
    activeCourse && courseSectionsById[activeCourse.id]
      ? (courseSectionsById[activeCourse.id] ?? [])
      : [];
  const courseContentLines = useMemo(
    () => buildCourseContentLines(activeSections),
    [activeSections],
  );

  const contentRows = Math.max(4, bodyHeight - 2);
  const maxScrollOffset = Math.max(0, courseContentLines.length - contentRows);

  useEffect(() => {
    setCourseScrollOffset((previous) => Math.min(previous, maxScrollOffset));
  }, [maxScrollOffset]);

  const openSelectedCourse = useCallback(async () => {
    if (!selectedCourse) return;

    setActiveCourseId(selectedCourse.id);
    setViewMode("course");
    setCourseScrollOffset(0);
    await loadCourseContents(selectedCourse.id, false);
  }, [loadCourseContents, selectedCourse]);

  useStableInput(
    (input, key) => {
      if (!inputEnabled) return;

      if (searchMode) {
        if (isShortcutPressed("courses-search-cancel", input, key)) {
          setSearchMode(false);
        }
        return;
      }

      if (viewMode === "course") {
        if (isShortcutPressed("course-back", input, key)) {
          setViewMode("list");
          setCoursePageError("");
          return;
        }

        if (isShortcutPressed("courses-refresh", input, key) && activeCourseId !== null) {
          void loadCourseContents(activeCourseId, true);
          return;
        }

        if (isShortcutPressed("courses-up", input, key)) {
          setCourseScrollOffset((previous) => Math.max(0, previous - 1));
          return;
        }

        if (isShortcutPressed("courses-down", input, key)) {
          setCourseScrollOffset((previous) => Math.min(maxScrollOffset, previous + 1));
          return;
        }

        if (isShortcutPressed("courses-page-up", input, key)) {
          setCourseScrollOffset((previous) => Math.max(0, previous - pageJump));
          return;
        }

        if (isShortcutPressed("courses-page-down", input, key)) {
          setCourseScrollOffset((previous) => Math.min(maxScrollOffset, previous + pageJump));
          return;
        }

        if (isShortcutPressed("courses-home", input, key)) {
          setCourseScrollOffset(0);
          return;
        }

        if (isShortcutPressed("courses-end", input, key)) {
          setCourseScrollOffset(maxScrollOffset);
        }
        return;
      }

      if (isShortcutPressed("courses-refresh", input, key)) {
        void loadCourses({ forceRefresh: true });
        return;
      }

      if (isShortcutPressed("courses-open", input, key)) {
        void openSelectedCourse();
        return;
      }

      if (isShortcutPressed("courses-search", input, key)) {
        setSearchMode(true);
        return;
      }

      if (isShortcutPressed("courses-search-clear", input, key)) {
        setSearchQuery("");
        setSelectedIdx(0);
        return;
      }

      if (isShortcutPressed("courses-up", input, key)) {
        setSelectedIdx((previous) => Math.max(0, previous - 1));
        return;
      }

      if (isShortcutPressed("courses-down", input, key)) {
        setSelectedIdx((previous) =>
          Math.min(Math.max(0, filteredCourses.length - 1), previous + 1),
        );
        return;
      }

      if (isShortcutPressed("courses-page-up", input, key)) {
        setSelectedIdx((previous) => Math.max(0, previous - pageJump));
        return;
      }

      if (isShortcutPressed("courses-page-down", input, key)) {
        setSelectedIdx((previous) =>
          Math.min(Math.max(0, filteredCourses.length - 1), previous + pageJump),
        );
        return;
      }

      if (isShortcutPressed("courses-home", input, key)) {
        setSelectedIdx(0);
        return;
      }

      if (isShortcutPressed("courses-end", input, key)) {
        setSelectedIdx(Math.max(0, filteredCourses.length - 1));
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight}>
      <Box justifyContent="space-between">
        <Text bold color={COLORS.brand}>
          {viewMode === "course" ? "Moodle Course Page" : "Moodle Courses"}
        </Text>
        <Text dimColor>{truncateText(`${config.username} @ ${config.baseUrl}`, 56)}</Text>
      </Box>

      {viewMode === "list" ? (
        <>
          <Box justifyContent="space-between">
            <Text dimColor>{`Enrolled: ${courses.length} | Visible: ${filteredCourses.length}`}</Text>
            <Text color={dataSource === "cache" ? COLORS.warning : COLORS.neutral.gray}>
              {dataSource === "live"
                ? "Source: live"
                : dataSource === "cache"
                  ? "Source: cache fallback"
                  : "Source: none"}
            </Text>
          </Box>

          <Box minHeight={1}>
            {searchMode ? (
              <>
                <Text color={COLORS.brand}>Search: </Text>
                <TextInput
                  value={searchQuery}
                  onChange={(value) => {
                    setSearchQuery(value);
                    setSelectedIdx(0);
                  }}
                  onSubmit={() => {
                    setSearchMode(false);
                  }}
                  onKey={(input, key) => {
                    if (isShortcutPressed("courses-search-cancel", input, key)) {
                      setSearchMode(false);
                      return true;
                    }
                    return false;
                  }}
                  placeholder="fuzzy: name, shortname, category, summary"
                  focus
                />
              </>
            ) : (
              <Text dimColor>{`Search: ${searchQuery || "none"}`}</Text>
            )}
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
              <Text dimColor>
                {filteredCourses.length > 0 ? `${selectedIdx + 1}/${filteredCourses.length}` : "0/0"}
              </Text>
            </Box>

            {loading ? (
              <Box justifyContent="center" alignItems="center" flexGrow={1}>
                <Text color={COLORS.warning}>
                  <Spinner type="dots" /> Loading courses...
                </Text>
              </Box>
            ) : filteredCourses.length === 0 ? (
              <Box justifyContent="center" alignItems="center" flexGrow={1} paddingX={1}>
                <Text color={COLORS.warning}>
                  {courses.length === 0
                    ? "No enrolled courses returned."
                    : "No courses match current fuzzy search."}
                </Text>
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
        </>
      ) : (
        <CoursePage
          termWidth={termWidth}
          bodyHeight={bodyHeight}
          course={activeCourse}
          sections={activeSections}
          contentLines={courseContentLines}
          scrollOffset={courseScrollOffset}
          loading={coursePageLoading}
          error={coursePageError}
        />
      )}

      {error && (
        <Box marginTop={1}>
          <Text color={COLORS.error}>{truncateText(error, Math.max(16, termWidth - 2))}</Text>
        </Box>
      )}
    </Box>
  );
}
