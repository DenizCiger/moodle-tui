import { describe, expect, it } from "bun:test";
import {
  normalizeCourse,
  normalizeCourseSection,
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
      categoryname: "STEM",
      summary: "<p>Summary</p>",
      visible: 1,
      progress: 75.4,
      courseurl: "https://moodle.school.tld/course/view.php?id=42",
    });

    expect(course).not.toBeNull();
    expect(course?.id).toBe(42);
    expect(course?.shortname).toBe("MATH");
    expect(course?.categoryid).toBe(5);
    expect(course?.categoryname).toBe("STEM");
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
});
