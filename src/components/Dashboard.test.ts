import { describe, expect, it } from "bun:test";
import type { CourseTreeRow } from "./CoursePage.tsx";
import {
  findCourseModuleForRow,
  getAssignmentModalInputAction,
  isAssignmentModule,
  resolveAssignmentForModule,
} from "./Dashboard.tsx";
import type { InputKey } from "./shortcuts.ts";
import type {
  MoodleAssignmentDetail,
  MoodleCourseModule,
  MoodleCourseSection,
} from "../utils/moodle.ts";

function key(overrides: Partial<InputKey> = {}): InputKey {
  return {
    upArrow: false,
    downArrow: false,
    leftArrow: false,
    rightArrow: false,
    pageDown: false,
    pageUp: false,
    home: false,
    end: false,
    return: false,
    escape: false,
    ctrl: false,
    shift: false,
    tab: false,
    backspace: false,
    delete: false,
    meta: false,
    ...overrides,
  };
}

function moduleRow(id: string): CourseTreeRow {
  return {
    id,
    kind: "module",
    depth: 1,
    text: "Assignment",
    icon: "✅",
    collapsible: true,
    expanded: false,
  };
}

describe("dashboard assignment helpers", () => {
  it("resolves selected module row to a course module", () => {
    const sections: MoodleCourseSection[] = [
      {
        id: 10,
        modules: [{ id: 55, instance: 7, name: "Essay 1", modname: "assign", contents: [] }],
      },
    ];

    const resolved = findCourseModuleForRow(sections, moduleRow("module:10:55"));

    expect(resolved?.id).toBe(55);
    expect(resolved?.instance).toBe(7);
  });

  it("returns null for non-module rows", () => {
    const sections: MoodleCourseSection[] = [{ id: 10, modules: [] }];
    const row: CourseTreeRow = {
      id: "summary:10",
      kind: "summary",
      depth: 1,
      text: "Summary",
      icon: "•",
      collapsible: false,
      expanded: false,
    };

    expect(findCourseModuleForRow(sections, row)).toBeNull();
  });

  it("detects assignment modules by modname", () => {
    const assignmentModule: MoodleCourseModule = {
      id: 1,
      name: "Assign",
      modname: "assign",
      contents: [],
    };
    const forumModule: MoodleCourseModule = {
      id: 2,
      name: "Forum",
      modname: "forum",
      contents: [],
    };

    expect(isAssignmentModule(assignmentModule)).toBe(true);
    expect(isAssignmentModule(forumModule)).toBe(false);
    expect(isAssignmentModule(null)).toBe(false);
  });

  it("prefers module.instance over cmid when resolving assignment details", () => {
    const module: MoodleCourseModule = {
      id: 90,
      instance: 7,
      name: "Essay 1",
      modname: "assign",
      contents: [],
    };
    const assignments: MoodleAssignmentDetail[] = [
      { id: 8, cmid: 90, courseId: 10, name: "Fallback by cmid" },
      { id: 7, cmid: 12, courseId: 10, name: "Primary by instance" },
    ];

    const resolved = resolveAssignmentForModule(module, assignments);
    expect(resolved?.id).toBe(7);
  });

  it("falls back to cmid when instance match is unavailable", () => {
    const module: MoodleCourseModule = {
      id: 90,
      instance: 999,
      name: "Essay 1",
      modname: "assign",
      contents: [],
    };
    const assignments: MoodleAssignmentDetail[] = [
      { id: 8, cmid: 90, courseId: 10, name: "Fallback by cmid" },
    ];

    const resolved = resolveAssignmentForModule(module, assignments);
    expect(resolved?.id).toBe(8);
  });

  it("maps Esc to close action only when modal is open", () => {
    expect(getAssignmentModalInputAction(true, "", key({ escape: true }))).toBe("close");
    expect(getAssignmentModalInputAction(false, "", key({ escape: true }))).toBe("none");
    expect(getAssignmentModalInputAction(true, "x", key())).toBe("none");
  });
});
