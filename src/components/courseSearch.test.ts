import { describe, expect, it } from "bun:test";
import type { MoodleCourse } from "../utils/moodle.ts";
import { filterCoursesByFuzzyQuery } from "./courseSearch.ts";

function course(
  id: number,
  shortname: string,
  fullname: string,
  extras?: Partial<MoodleCourse>,
): MoodleCourse {
  return {
    id,
    shortname,
    fullname,
    ...extras,
  };
}

describe("course fuzzy search", () => {
  const courses: MoodleCourse[] = [
    course(1, "MATH", "Mathematics"),
    course(2, "PHYS", "Physics"),
    course(3, "PROG", "Programming Fundamentals"),
    course(4, "HIST", "History"),
  ];

  it("returns all courses for empty query", () => {
    const result = filterCoursesByFuzzyQuery(courses, "");
    expect(result.length).toBe(courses.length);
  });

  it("matches subsequences (telescope-style fuzzy)", () => {
    const result = filterCoursesByFuzzyQuery(courses, "prg");
    expect(result[0]?.shortname).toBe("PROG");
  });

  it("ranks strong prefix/boundary matches first", () => {
    const ranked = filterCoursesByFuzzyQuery(
      [
        course(10, "DSA", "Data Structures and Algorithms"),
        course(11, "DB", "Distributed Systems"),
      ],
      "db",
    );

    expect(ranked[0]?.fullname).toBe("Distributed Systems");
  });
});
