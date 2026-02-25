import { describe, expect, it } from "bun:test";
import type { CourseTreeRow } from "./CoursePage.tsx";
import {
  buildCourseContentTargets,
  cycleCourseContentTargetIndex,
  filterRowsByTarget,
} from "./CourseContentFinderOverlay.tsx";

function row(
  id: string,
  kind: CourseTreeRow["kind"],
  text: string,
  overrides?: Partial<CourseTreeRow>,
): CourseTreeRow {
  return {
    id,
    kind,
    depth: 0,
    text,
    icon: "•",
    collapsible: false,
    expanded: false,
    ...overrides,
  };
}

describe("course content finder target filtering", () => {
  it("builds module-type targets including assignments", () => {
    const rows: CourseTreeRow[] = [
      row("module:1:11", "module", "Assignment 1", { moduleType: "assign" }),
      row("module:1:12", "module", "Forum", { moduleType: "forum" }),
    ];

    const targets = buildCourseContentTargets(rows);

    expect(targets.some((target) => target.id === "module-type:assign")).toBe(true);
    expect(targets.some((target) => target.label === "Assignments")).toBe(true);
    expect(targets.some((target) => target.id === "module-type:forum")).toBe(true);
  });

  it("filters rows by selected assignment target", () => {
    const rows: CourseTreeRow[] = [
      row("section:1", "section", "Week 1"),
      row("module:1:11", "module", "Essay", { moduleType: "assign" }),
      row("module:1:12", "module", "Forum", { moduleType: "forum" }),
      row("content:1:11:0", "content-item", "slides.pdf"),
      row("module-url:1:11", "module-url", "https://example.test/resource"),
    ];
    const targets = buildCourseContentTargets(rows);
    const assignmentTarget = targets.find((target) => target.id === "module-type:assign");
    const filesTarget = targets.find((target) => target.id === "kind:content-item");

    expect(assignmentTarget).toBeDefined();
    expect(filesTarget).toBeDefined();
    if (!assignmentTarget || !filesTarget) return;

    expect(filterRowsByTarget(rows, targets[0]!).map((value) => value.id)).toEqual([
      "section:1",
      "module:1:11",
      "module:1:12",
      "content:1:11:0",
      "module-url:1:11",
    ]);
    expect(filterRowsByTarget(rows, assignmentTarget).map((value) => value.id)).toEqual([
      "module:1:11",
    ]);
    expect(filterRowsByTarget(rows, filesTarget).map((value) => value.id)).toEqual([
      "content:1:11:0",
    ]);
  });

  it("cycles target indices with wrap-around", () => {
    expect(cycleCourseContentTargetIndex(0, -1, 8)).toBe(7);
    expect(cycleCourseContentTargetIndex(7, 1, 8)).toBe(0);
    expect(cycleCourseContentTargetIndex(3, 2, 8)).toBe(5);
    expect(cycleCourseContentTargetIndex(2, 0, 8)).toBe(2);
  });
});
