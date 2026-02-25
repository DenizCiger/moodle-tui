import { describe, expect, it } from "bun:test";
import {
  normalizeAssignmentSubmissionStatus,
  normalizeCourse,
  normalizeCourseAssignments,
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

  it("normalizes assignment details for a specific course", () => {
    const details = normalizeCourseAssignments(
      {
        courses: [
          {
            id: 10,
            assignments: [
              {
                id: 7,
                cmid: 77,
                course: 10,
                name: "Essay &amp; Draft",
                intro: "<p>Submit a draft.</p>",
                introformat: 1,
                alwaysshowdescription: 1,
                allowsubmissionsfromdate: 1_700_000_000,
                duedate: 1_700_010_000,
                cutoffdate: 1_700_020_000,
                gradingduedate: 1_700_030_000,
                grade: 100,
                teamsubmission: 0,
                requireallteammemberssubmit: false,
                maxattempts: -1,
                sendnotifications: true,
              },
            ],
          },
          {
            id: 11,
            assignments: [{ id: 8, cmid: 88, course: 11, name: "Other course assignment" }],
          },
        ],
      },
      10,
    );

    expect(details.length).toBe(1);
    expect(details[0]?.id).toBe(7);
    expect(details[0]?.cmid).toBe(77);
    expect(details[0]?.courseId).toBe(10);
    expect(details[0]?.name).toBe("Essay & Draft");
    expect(details[0]?.alwaysShowDescription).toBe(true);
    expect(details[0]?.teamsubmission).toBe(false);
    expect(details[0]?.sendnotifications).toBe(true);
  });

  it("normalizes assignment submission status payload", () => {
    const status = normalizeAssignmentSubmissionStatus({
      cansubmit: 1,
      caneditowner: 0,
      lastattempt: {
        gradingstatus: "graded",
        locked: true,
        submission: {
          status: "submitted",
          timemodified: 1_700_000_111,
        },
      },
    });

    expect(status).not.toBeNull();
    expect(status?.submissionStatus).toBe("submitted");
    expect(status?.gradingStatus).toBe("graded");
    expect(status?.canSubmit).toBe(true);
    expect(status?.canEdit).toBe(false);
    expect(status?.isLocked).toBe(true);
    expect(status?.lastModified).toBe(1_700_000_111);
  });

  it("returns null submission status when payload has no meaningful values", () => {
    const status = normalizeAssignmentSubmissionStatus({ warnings: [] });
    expect(status).toBeNull();
  });
});
