import { describe, expect, it } from "bun:test";
import type { CourseTreeNodeKind, CourseTreeRow } from "./CoursePage.tsx";
import { filterCourseContentByFuzzyQuery } from "./courseContentSearch.ts";

function row(
  id: string,
  kind: CourseTreeNodeKind,
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

describe("course content fuzzy search", () => {
  it("returns all searchable rows for empty query and excludes synthetic empty row", () => {
    const rows: CourseTreeRow[] = [
      row("section:1", "section", "Week 1"),
      row("module:1:10", "module", "Programming Fundamentals"),
      row("empty", "summary", "No visible course content returned by Moodle."),
    ];

    const result = filterCourseContentByFuzzyQuery(rows, "");

    expect(result.map((entry) => entry.id)).toEqual(["section:1", "module:1:10"]);
  });

  it("matches subsequences with telescope-style fuzzy scoring", () => {
    const rows: CourseTreeRow[] = [
      row("module:1:11", "module", "Programming Fundamentals"),
      row("module:1:12", "module", "Physics"),
    ];

    const result = filterCourseContentByFuzzyQuery(rows, "prg");

    expect(result[0]?.id).toBe("module:1:11");
  });

  it("prioritizes section/module/label/content kinds over summary when scores tie", () => {
    const rows: CourseTreeRow[] = [
      row("summary:1", "summary", "Calendar"),
      row("module:1:20", "module", "Calendar"),
      row("label:1:21", "label", "Calendar"),
    ];

    const result = filterCourseContentByFuzzyQuery(rows, "calendar");

    expect(result[0]?.id).toBe("module:1:20");
    expect(result[1]?.id).toBe("label:1:21");
    expect(result[2]?.id).toBe("summary:1");
  });

  it("sorts deterministically by text then id when weighted scores are equal", () => {
    const rows: CourseTreeRow[] = [
      row("module:1:2", "module", "Alpha"),
      row("module:1:1", "module", "Alpha"),
      row("module:1:3", "module", "beta"),
    ];

    const result = filterCourseContentByFuzzyQuery(rows, "a");

    expect(result.map((entry) => entry.id)).toEqual([
      "module:1:1",
      "module:1:2",
      "module:1:3",
    ]);
  });
});
