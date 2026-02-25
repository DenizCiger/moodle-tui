import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Box, Text, useStdout } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import CourseFinderOverlay from "./CourseFinderOverlay.tsx";
import CourseContentFinderOverlay from "./CourseContentFinderOverlay.tsx";
import AssignmentModal from "./AssignmentModal.tsx";
import CoursePage, {
  buildCourseTreeRows,
  type CourseTreeRow,
  courseSectionNodeId,
} from "./CoursePage.tsx";
import { useInputCapture } from "./inputCapture.tsx";
import { isShortcutPressed, type InputKey } from "./shortcuts.ts";
import { fitText, truncateText } from "./timetable/text.ts";
import { useStableInput } from "./useStableInput.ts";
import { getCachedCourses, saveCoursesToCache } from "../utils/cache.ts";
import type { MoodleRuntimeConfig } from "../utils/config.ts";
import {
  fetchAssignmentSubmissionStatus,
  fetchCourseAssignments,
  fetchCourseContents,
  fetchCourses,
  fetchUpcomingAssignments,
  type MoodleAssignmentDetail,
  type MoodleAssignmentSubmissionStatus,
  type MoodleCourse,
  type MoodleCourseModule,
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

interface AssignmentModalContext {
  courseId: number;
  courseName: string;
  moduleId: number;
  moduleName: string;
  moduleDescription?: string;
  assignmentId?: number;
}

function normalizeModname(value: string | undefined): string {
  return (value || "").trim().toLowerCase();
}

export function isAssignmentModule(
  module: MoodleCourseModule | null | undefined,
): module is MoodleCourseModule {
  if (!module) return false;
  return normalizeModname(module.modname) === "assign";
}

export function findCourseModuleForRow(
  sections: MoodleCourseSection[],
  row: CourseTreeRow | undefined,
): MoodleCourseModule | null {
  if (!row || row.kind !== "module") return null;
  const match = /^module:(\d+):(\d+)$/.exec(row.id);
  if (!match) return null;

  const sectionId = Number.parseInt(match[1] || "", 10);
  const moduleId = Number.parseInt(match[2] || "", 10);
  if (!Number.isFinite(sectionId) || !Number.isFinite(moduleId)) return null;

  const section = sections.find((value) => value.id === sectionId);
  if (!section) return null;
  return section.modules.find((module) => module.id === moduleId) ?? null;
}

export function resolveAssignmentForModule(
  module: MoodleCourseModule,
  assignments: MoodleAssignmentDetail[],
): MoodleAssignmentDetail | null {
  if (module.instance !== undefined) {
    const byInstance = assignments.find((assignment) => assignment.id === module.instance);
    if (byInstance) return byInstance;
  }

  return assignments.find((assignment) => assignment.cmid === module.id) ?? null;
}

export function getAssignmentModalInputAction(
  assignmentModalOpen: boolean,
  input: string,
  key: InputKey,
): "close" | "none" {
  if (!assignmentModalOpen) return "none";
  if (isShortcutPressed("assignment-modal-close", input, key)) return "close";
  return "none";
}

function buildDefaultCollapsedNodeIds(sections: MoodleCourseSection[]): string[] {
  const collapsedIds: string[] = [];

  sections.forEach((section) => {
    const sectionId = courseSectionNodeId(section.id);
    collapsedIds.push(sectionId);

    section.modules.forEach((module) => {
      const modname = (module.modname || "").trim().toLowerCase();
      if (modname === "label") return;
      collapsedIds.push(`module:${section.id}:${module.id}`);
    });
  });

  return collapsedIds;
}

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
  const [courseContentFinderOpen, setCourseContentFinderOpen] = useState(false);
  const [activeCourseId, setActiveCourseId] = useState<number | null>(null);
  const [courseSectionsById, setCourseSectionsById] = useState<Record<number, MoodleCourseSection[]>>(
    {},
  );
  const [coursePageLoading, setCoursePageLoading] = useState(false);
  const [coursePageError, setCoursePageError] = useState("");
  const [courseScrollOffset, setCourseScrollOffset] = useState(0);
  const [courseSelectedIndex, setCourseSelectedIndex] = useState(0);
  const [collapsedCourseNodeIds, setCollapsedCourseNodeIds] = useState<string[]>([]);
  const [pendingCourseTreeInitCourseId, setPendingCourseTreeInitCourseId] = useState<number | null>(
    null,
  );
  const [pendingCourseJumpRowId, setPendingCourseJumpRowId] = useState<string | null>(null);
  const [assignmentModalOpen, setAssignmentModalOpen] = useState(false);
  const [assignmentModalContext, setAssignmentModalContext] = useState<AssignmentModalContext | null>(
    null,
  );
  const [assignmentDetailLoading, setAssignmentDetailLoading] = useState(false);
  const [assignmentDetailError, setAssignmentDetailError] = useState("");
  const [assignmentStatusLoading, setAssignmentStatusLoading] = useState(false);
  const [assignmentStatusError, setAssignmentStatusError] = useState("");
  const [assignmentListByCourseId, setAssignmentListByCourseId] = useState<
    Record<number, MoodleAssignmentDetail[]>
  >({});
  const [assignmentDetailByAssignmentId, setAssignmentDetailByAssignmentId] = useState<
    Record<number, MoodleAssignmentDetail>
  >({});
  const [assignmentStatusByAssignmentId, setAssignmentStatusByAssignmentId] = useState<
    Record<number, MoodleAssignmentSubmissionStatus | null>
  >({});
  const assignmentModalRequestIdRef = useRef(0);

  useInputCapture(courseFinderOpen || courseContentFinderOpen || assignmentModalOpen);

  const closeAssignmentModal = useCallback(() => {
    assignmentModalRequestIdRef.current += 1;
    setAssignmentModalOpen(false);
    setAssignmentDetailLoading(false);
    setAssignmentStatusLoading(false);
    setAssignmentDetailError("");
    setAssignmentStatusError("");
  }, []);

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
      setCourseContentFinderOpen(false);
      setCourseScrollOffset(0);
      setCourseSelectedIndex(0);
      setPendingCourseJumpRowId(null);
      closeAssignmentModal();
      setAssignmentModalContext(null);
      setCollapsedCourseNodeIds(buildDefaultCollapsedNodeIds(courseSectionsById[course.id] ?? []));
      setPendingCourseTreeInitCourseId(course.id);
      await loadCourseContents(course.id, false);
    },
    [closeAssignmentModal, courseSectionsById, loadCourseContents],
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

  useEffect(() => {
    if (viewMode === "course") return;
    closeAssignmentModal();
    setAssignmentModalContext(null);
  }, [closeAssignmentModal, viewMode]);

  const activeSections: MoodleCourseSection[] =
    activeCourse && courseSectionsById[activeCourse.id]
      ? (courseSectionsById[activeCourse.id] ?? [])
      : [];

  useEffect(() => {
    if (viewMode !== "course") return;
    if (activeCourseId === null) return;
    if (pendingCourseTreeInitCourseId !== activeCourseId) return;
    if (activeSections.length === 0 && coursePageLoading) return;

    setCollapsedCourseNodeIds(buildDefaultCollapsedNodeIds(activeSections));
    setCourseSelectedIndex(0);
    setCourseScrollOffset(0);
    setPendingCourseTreeInitCourseId(null);
  }, [
    activeCourseId,
    activeSections,
    coursePageLoading,
    pendingCourseTreeInitCourseId,
    viewMode,
  ]);

  const collapsedCourseNodeIdSet = useMemo(
    () => new Set(collapsedCourseNodeIds),
    [collapsedCourseNodeIds],
  );
  const courseRows = useMemo(
    () => buildCourseTreeRows(activeSections, collapsedCourseNodeIdSet),
    [activeSections, collapsedCourseNodeIdSet],
  );
  const allCourseRows = useMemo(
    () => buildCourseTreeRows(activeSections, []),
    [activeSections],
  );

  useEffect(() => {
    if (!pendingCourseJumpRowId) return;
    const rowIndex = courseRows.findIndex((row) => row.id === pendingCourseJumpRowId);
    if (rowIndex === -1) return;

    setCourseSelectedIndex(rowIndex);
    setPendingCourseJumpRowId(null);
  }, [courseRows, pendingCourseJumpRowId]);

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
  const maxCourseScrollOffset = Math.max(0, courseRows.length - contentRows);
  const maxCourseRowIndex = Math.max(courseRows.length - 1, 0);

  useEffect(() => {
    setCourseSelectedIndex((previous) => Math.min(previous, maxCourseRowIndex));
  }, [maxCourseRowIndex]);

  useEffect(() => {
    setCourseScrollOffset((previous) => {
      const clampedPrevious = Math.min(previous, maxCourseScrollOffset);
      if (courseRows.length === 0) return 0;

      if (courseSelectedIndex < clampedPrevious) {
        return courseSelectedIndex;
      }

      if (courseSelectedIndex >= clampedPrevious + contentRows) {
        return Math.min(
          Math.max(0, courseSelectedIndex - contentRows + 1),
          maxCourseScrollOffset,
        );
      }

      return clampedPrevious;
    });
  }, [contentRows, courseRows.length, courseSelectedIndex, maxCourseScrollOffset]);

  useEffect(() => {
    setDashboardScrollOffset((previous) => Math.min(previous, maxDashboardScrollOffset));
  }, [maxDashboardScrollOffset]);

  const moveCourseSelection = useCallback(
    (delta: number) => {
      setCourseSelectedIndex((previous) =>
        Math.max(0, Math.min(previous + delta, maxCourseRowIndex)),
      );
    },
    [maxCourseRowIndex],
  );

  const setCourseNodeCollapsed = useCallback((nodeId: string, collapsed: boolean) => {
    setCollapsedCourseNodeIds((previous) => {
      const alreadyCollapsed = previous.includes(nodeId);
      if (collapsed) {
        if (alreadyCollapsed) return previous;
        return [...previous, nodeId];
      }

      if (!alreadyCollapsed) return previous;
      return previous.filter((id) => id !== nodeId);
    });
  }, []);

  const applyCourseContentSelection = useCallback(
    (targetRowId: string) => {
      const allRowsById = new Map<string, CourseTreeRow>(
        allCourseRows.map((row) => [row.id, row]),
      );
      const target = allRowsById.get(targetRowId);
      if (!target) {
        setCourseContentFinderOpen(false);
        return;
      }

      const rowsToExpand: string[] = [];
      let current: CourseTreeRow | undefined = target;

      while (current?.parentId) {
        const parent = allRowsById.get(current.parentId);
        if (!parent) break;
        if (parent.collapsible) {
          rowsToExpand.push(parent.id);
        }
        current = parent;
      }

      if (rowsToExpand.length > 0) {
        const rowsToExpandSet = new Set(rowsToExpand);
        setCollapsedCourseNodeIds((previous) =>
          previous.filter((rowId) => !rowsToExpandSet.has(rowId)),
        );
      }

      setPendingCourseJumpRowId(targetRowId);
      setCourseContentFinderOpen(false);
    },
    [allCourseRows],
  );

  const openSelectedAssignmentModal = useCallback(async () => {
    if (activeCourseId === null || !activeCourse) return;

    const selectedRow = courseRows[courseSelectedIndex];
    const module = findCourseModuleForRow(activeSections, selectedRow);
    if (!isAssignmentModule(module)) return;

    const requestId = assignmentModalRequestIdRef.current + 1;
    assignmentModalRequestIdRef.current = requestId;

    setAssignmentModalOpen(true);
    setAssignmentModalContext({
      courseId: activeCourse.id,
      courseName: activeCourse.fullname,
      moduleId: module.id,
      moduleName: module.name,
      moduleDescription: module.description,
    });
    setAssignmentDetailLoading(true);
    setAssignmentStatusLoading(false);
    setAssignmentDetailError("");
    setAssignmentStatusError("");

    let courseAssignments = assignmentListByCourseId[activeCourse.id];
    if (!courseAssignments) {
      try {
        courseAssignments = await fetchCourseAssignments(config, activeCourse.id);
        if (assignmentModalRequestIdRef.current !== requestId) return;

        setAssignmentListByCourseId((previous) => ({
          ...previous,
          [activeCourse.id]: courseAssignments || [],
        }));
        setAssignmentDetailByAssignmentId((previous) => {
          const next = { ...previous };
          (courseAssignments || []).forEach((detail) => {
            next[detail.id] = detail;
          });
          return next;
        });
      } catch (loadError) {
        if (assignmentModalRequestIdRef.current !== requestId) return;
        const message = loadError instanceof Error ? loadError.message : "Unknown Moodle API error";
        setAssignmentDetailError(`Assignment details unavailable: ${message}`);
        setAssignmentDetailLoading(false);
        return;
      }
    }

    const resolvedAssignment = resolveAssignmentForModule(module, courseAssignments || []);
    if (!resolvedAssignment) {
      if (assignmentModalRequestIdRef.current !== requestId) return;
      setAssignmentDetailError(
        "Assignment details unavailable for this activity. This Moodle module could not be matched.",
      );
      setAssignmentDetailLoading(false);
      return;
    }

    if (assignmentModalRequestIdRef.current !== requestId) return;
    setAssignmentModalContext((previous) =>
      previous
        ? {
            ...previous,
            assignmentId: resolvedAssignment.id,
          }
        : previous,
    );
    setAssignmentDetailByAssignmentId((previous) => ({
      ...previous,
      [resolvedAssignment.id]: resolvedAssignment,
    }));
    setAssignmentDetailLoading(false);

    const hasCachedStatus = Object.prototype.hasOwnProperty.call(
      assignmentStatusByAssignmentId,
      resolvedAssignment.id,
    );
    if (hasCachedStatus) {
      setAssignmentStatusLoading(false);
      return;
    }

    setAssignmentStatusLoading(true);
    try {
      const status = await fetchAssignmentSubmissionStatus(config, resolvedAssignment.id);
      if (assignmentModalRequestIdRef.current !== requestId) return;
      setAssignmentStatusByAssignmentId((previous) => ({
        ...previous,
        [resolvedAssignment.id]: status,
      }));
    } catch (loadError) {
      if (assignmentModalRequestIdRef.current !== requestId) return;
      const message = loadError instanceof Error ? loadError.message : "Unknown Moodle API error";
      setAssignmentStatusError(`Submission status unavailable: ${message}`);
    } finally {
      if (assignmentModalRequestIdRef.current !== requestId) return;
      setAssignmentStatusLoading(false);
    }
  }, [
    activeCourse,
    activeCourseId,
    activeSections,
    assignmentListByCourseId,
    assignmentStatusByAssignmentId,
    config,
    courseRows,
    courseSelectedIndex,
  ]);

  const activeAssignmentDetail = useMemo(() => {
    if (!assignmentModalContext?.assignmentId) return null;
    return assignmentDetailByAssignmentId[assignmentModalContext.assignmentId] ?? null;
  }, [assignmentDetailByAssignmentId, assignmentModalContext?.assignmentId]);

  const activeAssignmentStatus = useMemo(() => {
    if (!assignmentModalContext?.assignmentId) return null;
    return assignmentStatusByAssignmentId[assignmentModalContext.assignmentId] ?? null;
  }, [assignmentModalContext?.assignmentId, assignmentStatusByAssignmentId]);

  useStableInput(
    (input, key) => {
      if (!inputEnabled) return;
      const assignmentModalAction = getAssignmentModalInputAction(
        assignmentModalOpen,
        input,
        key as InputKey,
      );
      if (assignmentModalAction === "close") {
        closeAssignmentModal();
        return;
      }

      if (assignmentModalOpen) return;
      if (courseFinderOpen || courseContentFinderOpen) return;

      if (isShortcutPressed("dashboard-open-finder", input, key)) {
        setCourseFinderOpen(true);
        return;
      }

      if (viewMode === "course") {
        if (isShortcutPressed("dashboard-open-content-finder", input, key)) {
          setCourseContentFinderOpen(true);
          return;
        }

        if (isShortcutPressed("dashboard-open-assignment-modal", input, key)) {
          void openSelectedAssignmentModal();
          return;
        }

        if (isShortcutPressed("dashboard-back", input, key)) {
          setViewMode("dashboard");
          setCourseContentFinderOpen(false);
          setCoursePageError("");
          closeAssignmentModal();
          setAssignmentModalContext(null);
          return;
        }

        if (isShortcutPressed("dashboard-refresh", input, key) && activeCourseId !== null) {
          void loadCourseContents(activeCourseId, true);
          return;
        }

        if (isShortcutPressed("dashboard-up", input, key)) {
          moveCourseSelection(-1);
          return;
        }

        if (isShortcutPressed("dashboard-down", input, key)) {
          moveCourseSelection(1);
          return;
        }

        if (isShortcutPressed("dashboard-page-up", input, key)) {
          moveCourseSelection(-pageJump);
          return;
        }

        if (isShortcutPressed("dashboard-page-down", input, key)) {
          moveCourseSelection(pageJump);
          return;
        }

        if (isShortcutPressed("dashboard-home", input, key)) {
          setCourseSelectedIndex(0);
          return;
        }

        if (isShortcutPressed("dashboard-end", input, key)) {
          setCourseSelectedIndex(maxCourseRowIndex);
          return;
        }

        if (isShortcutPressed("dashboard-expand", input, key)) {
          const selectedRow = courseRows[courseSelectedIndex];
          if (!selectedRow || !selectedRow.collapsible) return;

          if (!selectedRow.expanded) {
            setCourseNodeCollapsed(selectedRow.id, false);
            return;
          }

          const firstChildIndex = courseRows.findIndex(
            (row, index) => index > courseSelectedIndex && row.parentId === selectedRow.id,
          );
          if (firstChildIndex !== -1) {
            setCourseSelectedIndex(firstChildIndex);
          }
          return;
        }

        if (isShortcutPressed("dashboard-collapse", input, key)) {
          const selectedRow = courseRows[courseSelectedIndex];
          if (!selectedRow) return;

          if (selectedRow.collapsible && selectedRow.expanded) {
            setCourseNodeCollapsed(selectedRow.id, true);
            return;
          }

          if (selectedRow.parentId) {
            const parentIndex = courseRows.findIndex((row) => row.id === selectedRow.parentId);
            if (parentIndex !== -1) {
              setCourseSelectedIndex(parentIndex);
            }
          }
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
          rows={courseRows}
          selectedIndex={courseSelectedIndex}
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

      {assignmentModalOpen && assignmentModalContext && (
        <AssignmentModal
          termWidth={termWidth}
          termHeight={termHeight}
          context={{
            courseName: assignmentModalContext.courseName,
            moduleName: assignmentModalContext.moduleName,
            moduleDescription: assignmentModalContext.moduleDescription,
          }}
          loading={assignmentDetailLoading}
          detailError={assignmentDetailError}
          statusError={assignmentStatusError}
          detail={activeAssignmentDetail}
          statusLoading={assignmentStatusLoading}
          status={activeAssignmentStatus}
          onClose={closeAssignmentModal}
        />
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

      {viewMode === "course" && courseContentFinderOpen && (
        <CourseContentFinderOverlay
          termWidth={termWidth}
          termHeight={termHeight}
          rows={allCourseRows}
          loading={coursePageLoading}
          onClose={() => {
            setCourseContentFinderOpen(false);
          }}
          onApplyRow={(rowId) => {
            applyCourseContentSelection(rowId);
          }}
        />
      )}
    </Box>
  );
}
