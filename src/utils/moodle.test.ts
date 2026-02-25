import { describe, expect, it } from "bun:test";
import {
  normalizeCourse,
  normalizeCourseSection,
  normalizeUpcomingAssignments,
  normalizeTokenResponse,
} from "./moodle.ts";

describe("moodle response normalization", () => {
  it("normalizes token success payload", () => {
    const normalized = normalizeTokenResponse({ token: "abc123" });
    expect(normalized.token).toBe("abc123");
    expect(normalized.error).toBeUndefined();
  });

  it("normalizes token error payload", () => {
    const normalized = normalizeTokenResponse({
      error: "invalidlogin",
      errorcode: "invalidlogin",
      debuginfo: "Wrong username/password",
    });

    expect(normalized.token).toBeUndefined();
    expect(normalized.error).toBe("invalidlogin");
    expect(normalized.errorcode).toBe("invalidlogin");
    expect(normalized.debuginfo).toBe("Wrong username/password");
  });

  it("normalizes course payload", () => {
    const course = normalizeCourse({
      id: 42,
      shortname: "MATH",
      fullname: "Mathematics",
      displayname: "Mathematics 2025",
      categoryid: 5,
      categoryname: "Science &amp; Tech",
      summary: "<p>Summary &amp; Scope</p>",
      visible: 1,
      progress: 75.4,
      courseurl: "https://moodle.school.tld/course/view.php?id=42",
    });

    expect(course).not.toBeNull();
    expect(course?.id).toBe(42);
    expect(course?.shortname).toBe("MATH");
    expect(course?.categoryid).toBe(5);
    expect(course?.categoryname).toBe("Science & Tech");
    expect(course?.summary).toBe("<p>Summary & Scope</p>");
    expect(course?.visible).toBe(1);
    expect(course?.progress).toBe(75.4);
  });

  it("normalizes course section payload", () => {
    const section = normalizeCourseSection({
      id: 10,
      name: "Week 1",
      section: 1,
      summary: "<p>Intro</p>",
      visible: 1,
      modules: [
        {
          id: 55,
          name: "Lecture Notes",
          modname: "resource",
          description: "<p>Read this first</p>",
          url: "https://moodle.school.tld/mod/resource/view.php?id=55",
          visible: 1,
          contents: [
            {
              type: "file",
              filename: "notes.pdf",
              fileurl: "https://moodle.school.tld/pluginfile.php/notes.pdf",
            },
          ],
        },
      ],
    });

    expect(section).not.toBeNull();
    expect(section?.id).toBe(10);
    expect(section?.modules.length).toBe(1);
    expect(section?.modules[0]?.id).toBe(55);
    expect(section?.modules[0]?.contents[0]?.filename).toBe("notes.pdf");
  });

  it("normalizes upcoming assignments and keeps only future due dates", () => {
    const assignments = normalizeUpcomingAssignments(
      {
        courses: [
          {
            id: 10,
            shortname: "PROG",
            fullname: "Programming &amp; Basics",
            assignments: [
              { id: 1, name: "Future &amp; assignment", duedate: 2_000_000_000 },
              { id: 2, name: "Past assignment", duedate: 1_000_000_000 },
              { id: 3, name: "Missing due date" },
            ],
          },
        ],
      },
      1_500_000_000,
    );

    expect(assignments.length).toBe(1);
    expect(assignments[0]?.id).toBe(1);
    expect(assignments[0]?.name).toBe("Future & assignment");
    expect(assignments[0]?.courseId).toBe(10);
    expect(assignments[0]?.courseShortName).toBe("PROG");
    expect(assignments[0]?.courseFullName).toBe("Programming & Basics");
  });

  it("sorts upcoming assignments by due date ascending", () => {
    const assignments = normalizeUpcomingAssignments(
      {
        courses: [
          {
            id: 1,
            shortname: "B",
            fullname: "Beta",
            assignments: [
              { id: 2, name: "Second", duedate: 2_000 },
              { id: 1, name: "First", duedate: 1_000 },
            ],
          },
        ],
      },
      500,
    );

    expect(assignments.map((item) => item.id)).toEqual([1, 2]);
  });
});
