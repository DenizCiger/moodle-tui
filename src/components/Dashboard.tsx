import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Box, Text, useStdout } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import CourseFinderOverlay from "./CourseFinderOverlay.tsx";
import CoursePage, { buildCourseContentLines } from "./CoursePage.tsx";
import { useInputCapture } from "./inputCapture.tsx";
import { isShortcutPressed } from "./shortcuts.ts";
import { fitText, truncateText } from "./timetable/text.ts";
import { useStableInput } from "./useStableInput.ts";
import { getCachedCourses, saveCoursesToCache } from "../utils/cache.ts";
import type { MoodleRuntimeConfig } from "../utils/config.ts";
import {
  fetchCourseContents,
  fetchCourses,
  fetchUpcomingAssignments,
  type MoodleCourse,
  type MoodleCourseSection,
  type MoodleUpcomingAssignment,
} from "../utils/moodle.ts";

interface DashboardProps {
  config: MoodleRuntimeConfig;
  topInset?: number;
  inputEnabled?: boolean;
  onTabLabelChange?: (label: string) => void;
}

type ViewMode = "dashboard" | "course";

function formatDueDateTime(unixTimestamp: number): string {
  const value = new Date(unixTimestamp * 1000);
  if (Number.isNaN(value.getTime())) return "-";

  const year = String(value.getFullYear());
  const month = String(value.getMonth() + 1).padStart(2, "0");
  const day = String(value.getDate()).padStart(2, "0");
  const hours = String(value.getHours()).padStart(2, "0");
  const minutes = String(value.getMinutes()).padStart(2, "0");
  return `${year}-${month}-${day} ${hours}:${minutes}`;
}

function enrichAssignmentsWithCourseNames(
  assignments: MoodleUpcomingAssignment[],
  courses: MoodleCourse[],
): MoodleUpcomingAssignment[] {
  const courseById = new Map(courses.map((course) => [course.id, course]));

  return assignments.map((assignment) => {
    if (assignment.courseShortName && assignment.courseFullName) {
      return assignment;
    }

    const course = courseById.get(assignment.courseId);
    if (!course) return assignment;

    return {
      ...assignment,
      courseShortName: assignment.courseShortName || course.shortname,
      courseFullName: assignment.courseFullName || course.fullname,
    };
  });
}

export default function Dashboard({
  config,
  topInset = 0,
  inputEnabled = true,
  onTabLabelChange,
}: DashboardProps) {
  const { stdout } = useStdout();
  const [courses, setCourses] = useState<MoodleCourse[]>([]);
  const [upcomingAssignments, setUpcomingAssignments] = useState<MoodleUpcomingAssignment[]>([]);
  const [dashboardScrollOffset, setDashboardScrollOffset] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [dataSource, setDataSource] = useState<"live" | "cache" | "none">("none");

  const [viewMode, setViewMode] = useState<ViewMode>("dashboard");
  const [courseFinderOpen, setCourseFinderOpen] = useState(false);
  const [activeCourseId, setActiveCourseId] = useState<number | null>(null);
  const [courseSectionsById, setCourseSectionsById] = useState<Record<number, MoodleCourseSection[]>>(
    {},
  );
  const [coursePageLoading, setCoursePageLoading] = useState(false);
  const [coursePageError, setCoursePageError] = useState("");
  const [courseScrollOffset, setCourseScrollOffset] = useState(0);

  useInputCapture(courseFinderOpen);

  const loadDashboard = useCallback(
    async ({ forceRefresh }: { forceRefresh: boolean }) => {
      setLoading(true);
      if (!forceRefresh) {
        setError("");
      }

      try {
        const liveCourses = await fetchCourses(config);
        setCourses(liveCourses);
        setDataSource("live");
        saveCoursesToCache(liveCourses);

        try {
          const assignments = await fetchUpcomingAssignments(config);
          setUpcomingAssignments(enrichAssignmentsWithCourseNames(assignments, liveCourses));
          setError("");
        } catch (assignmentError) {
          const message =
            assignmentError instanceof Error ? assignmentError.message : "Unknown Moodle API error";
          setUpcomingAssignments([]);
          setError(`Assignments unavailable: ${message}`);
        }
      } catch (loadError) {
        const message =
          loadError instanceof Error ? loadError.message : "Unknown Moodle API error";

        if (!forceRefresh) {
          const cachedCourses = getCachedCourses();
          if (cachedCourses && cachedCourses.length > 0) {
            setCourses(cachedCourses);
            setUpcomingAssignments([]);
            setDataSource("cache");
            setError(`Live sync failed; showing cached courses. ${message}`);
          } else {
            setDataSource("none");
            setCourses([]);
            setUpcomingAssignments([]);
            setDashboardScrollOffset(0);
            setError(`Failed to load dashboard: ${message}`);
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
    void loadDashboard({ forceRefresh: false });
  }, [loadDashboard]);

  const openCourseFromFinder = useCallback(
    async (course: MoodleCourse) => {
      setActiveCourseId(course.id);
      setViewMode("course");
      setCourseFinderOpen(false);
      setCourseScrollOffset(0);
      await loadCourseContents(course.id, false);
    },
    [loadCourseContents],
  );

  const activeCourse = useMemo(() => {
    if (activeCourseId === null) return null;
    return courses.find((course) => course.id === activeCourseId) ?? null;
  }, [activeCourseId, courses]);

  useEffect(() => {
    if (!onTabLabelChange) return;

    if (viewMode === "course") {
      onTabLabelChange(activeCourse?.shortname || "Course");
      return;
    }

    onTabLabelChange("Dashboard");
  }, [activeCourse?.shortname, onTabLabelChange, viewMode]);

  const activeSections: MoodleCourseSection[] =
    activeCourse && courseSectionsById[activeCourse.id]
      ? (courseSectionsById[activeCourse.id] ?? [])
      : [];

  const courseContentLines = useMemo(
    () => buildCourseContentLines(activeSections),
    [activeSections],
  );

  const termWidth = Math.max(70, stdout?.columns ?? 120);
  const termHeight = Math.max(18, (stdout?.rows ?? 24) - topInset);
  const bodyHeight = Math.max(8, termHeight - 7);
  const pageJump = Math.max(4, Math.floor(bodyHeight / 3));

  const dashboardRows = Math.max(4, bodyHeight - 2);
  const maxDashboardScrollOffset = Math.max(0, upcomingAssignments.length - dashboardRows);
  const visibleAssignments = upcomingAssignments.slice(
    dashboardScrollOffset,
    dashboardScrollOffset + dashboardRows,
  );

  const dueWidth = Math.max(16, Math.floor((termWidth - 8) * 0.2));
  const courseWidth = Math.max(14, Math.floor((termWidth - 8) * 0.28));
  const assignmentWidth = Math.max(18, termWidth - dueWidth - courseWidth - 8);

  const contentRows = Math.max(4, bodyHeight - 2);
  const maxCourseScrollOffset = Math.max(0, courseContentLines.length - contentRows);

  useEffect(() => {
    setCourseScrollOffset((previous) => Math.min(previous, maxCourseScrollOffset));
  }, [maxCourseScrollOffset]);

  useEffect(() => {
    setDashboardScrollOffset((previous) => Math.min(previous, maxDashboardScrollOffset));
  }, [maxDashboardScrollOffset]);

  useStableInput(
    (input, key) => {
      if (!inputEnabled) return;
      if (courseFinderOpen) return;

      if (isShortcutPressed("dashboard-open-finder", input, key)) {
        setCourseFinderOpen(true);
        return;
      }

      if (viewMode === "course") {
        if (isShortcutPressed("dashboard-back", input, key)) {
          setViewMode("dashboard");
          setCoursePageError("");
          return;
        }

        if (isShortcutPressed("dashboard-refresh", input, key) && activeCourseId !== null) {
          void loadCourseContents(activeCourseId, true);
          return;
        }

        if (isShortcutPressed("dashboard-up", input, key)) {
          setCourseScrollOffset((previous) => Math.max(0, previous - 1));
          return;
        }

        if (isShortcutPressed("dashboard-down", input, key)) {
          setCourseScrollOffset((previous) => Math.min(maxCourseScrollOffset, previous + 1));
          return;
        }

        if (isShortcutPressed("dashboard-page-up", input, key)) {
          setCourseScrollOffset((previous) => Math.max(0, previous - pageJump));
          return;
        }

        if (isShortcutPressed("dashboard-page-down", input, key)) {
          setCourseScrollOffset((previous) => Math.min(maxCourseScrollOffset, previous + pageJump));
          return;
        }

        if (isShortcutPressed("dashboard-home", input, key)) {
          setCourseScrollOffset(0);
          return;
        }

        if (isShortcutPressed("dashboard-end", input, key)) {
          setCourseScrollOffset(maxCourseScrollOffset);
        }
        return;
      }

      if (isShortcutPressed("dashboard-refresh", input, key)) {
        void loadDashboard({ forceRefresh: true });
        return;
      }

      if (isShortcutPressed("dashboard-up", input, key)) {
        setDashboardScrollOffset((previous) => Math.max(0, previous - 1));
        return;
      }

      if (isShortcutPressed("dashboard-down", input, key)) {
        setDashboardScrollOffset((previous) =>
          Math.min(maxDashboardScrollOffset, previous + 1),
        );
        return;
      }

      if (isShortcutPressed("dashboard-page-up", input, key)) {
        setDashboardScrollOffset((previous) => Math.max(0, previous - pageJump));
        return;
      }

      if (isShortcutPressed("dashboard-page-down", input, key)) {
        setDashboardScrollOffset((previous) =>
          Math.min(maxDashboardScrollOffset, previous + pageJump),
        );
        return;
      }

      if (isShortcutPressed("dashboard-home", input, key)) {
        setDashboardScrollOffset(0);
        return;
      }

      if (isShortcutPressed("dashboard-end", input, key)) {
        setDashboardScrollOffset(maxDashboardScrollOffset);
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  return (
    <Box flexDirection="column" width={termWidth} height={termHeight}>
      <Box justifyContent="space-between">
        <Text bold color={COLORS.brand}>
          {viewMode === "course" ? "Moodle Course Page" : "Moodle Dashboard"}
        </Text>
        <Text dimColor>{truncateText(`${config.username} @ ${config.baseUrl}`, 56)}</Text>
      </Box>

      {viewMode === "dashboard" ? (
        <>
          <Box justifyContent="space-between">
            <Text dimColor>{`Upcoming: ${upcomingAssignments.length} | Courses: ${courses.length}`}</Text>
            <Text color={dataSource === "cache" ? COLORS.warning : COLORS.neutral.gray}>
              {dataSource === "live"
                ? "Source: live"
                : dataSource === "cache"
                  ? "Source: cache fallback"
                  : "Source: none"}
            </Text>
          </Box>

          <Box minHeight={1}>
            <Text dimColor>Press / to open course finder.</Text>
          </Box>

          <Box
            flexDirection="column"
            marginTop={1}
            height={bodyHeight}
            borderStyle="single"
            borderColor={COLORS.neutral.brightBlack}
          >
            <Box justifyContent="space-between" paddingX={1}>
              <Text bold>Upcoming Assignments</Text>
              <Text dimColor>
                {upcomingAssignments.length > 0
                  ? `${dashboardScrollOffset + 1}-${Math.min(dashboardScrollOffset + visibleAssignments.length, upcomingAssignments.length)}/${upcomingAssignments.length}`
                  : "0/0"}
              </Text>
            </Box>

            {loading ? (
              <Box justifyContent="center" alignItems="center" flexGrow={1}>
                <Text color={COLORS.warning}>
                  <Spinner type="dots" /> Loading dashboard...
                </Text>
              </Box>
            ) : upcomingAssignments.length === 0 ? (
              <Box justifyContent="center" alignItems="center" flexGrow={1} paddingX={1}>
                <Text color={COLORS.warning}>No upcoming assignments found.</Text>
              </Box>
            ) : (
              <Box flexDirection="column" paddingX={1}>
                <Box>
                  <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                    {fitText("Due", dueWidth)}
                  </Text>
                  <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                    {" "}
                  </Text>
                  <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                    {fitText("Course", courseWidth)}
                  </Text>
                  <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                    {" "}
                  </Text>
                  <Text color={COLORS.neutral.gray} backgroundColor={COLORS.panel.header}>
                    {fitText("Assignment", assignmentWidth)}
                  </Text>
                </Box>

                {visibleAssignments.map((assignment, index) => {
                  const rowBackgroundColor =
                    index % 2 === 1 ? COLORS.panel.alternate : undefined;
                  const courseLabel =
                    assignment.courseFullName || assignment.courseShortName || "Unknown course";

                  return (
                    <Box key={`${assignment.courseId}-${assignment.id}`}>
                      <Text color={COLORS.neutral.white} backgroundColor={rowBackgroundColor}>
                        {fitText(formatDueDateTime(assignment.dueDate), dueWidth)}
                      </Text>
                      <Text backgroundColor={rowBackgroundColor}> </Text>
                      <Text color={COLORS.neutral.white} backgroundColor={rowBackgroundColor}>
                        {fitText(courseLabel, courseWidth)}
                      </Text>
                      <Text backgroundColor={rowBackgroundColor}> </Text>
                      <Text color={COLORS.neutral.white} backgroundColor={rowBackgroundColor}>
                        {fitText(assignment.name, assignmentWidth)}
                      </Text>
                    </Box>
                  );
                })}
              </Box>
            )}
          </Box>
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

      {courseFinderOpen && (
        <CourseFinderOverlay
          termWidth={termWidth}
          termHeight={termHeight}
          courses={courses}
          loading={loading}
          onClose={() => {
            setCourseFinderOpen(false);
          }}
          onApplyCourse={(course) => {
            void openCourseFromFinder(course);
          }}
        />
      )}
    </Box>
  );
}
