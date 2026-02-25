import type { MoodleCourse } from "../utils/moodle.ts";
import { fuzzyScore } from "./fuzzySearch.ts";

interface WeightedField {
  value: string;
  weight: number;
}

interface RankedCourse {
  course: MoodleCourse;
  score: number;
}

function buildWeightedFields(course: MoodleCourse): WeightedField[] {
  return [
    { value: course.shortname, weight: 1.2 },
    { value: course.fullname, weight: 1.0 },
    { value: course.displayname || "", weight: 0.95 },
    { value: course.categoryname || "", weight: 0.7 },
    { value: course.summary || "", weight: 0.35 },
  ];
}

function rankCourse(course: MoodleCourse, query: string): RankedCourse | null {
  const fields = buildWeightedFields(course);
  let bestScore: number | null = null;

  for (const field of fields) {
    const score = fuzzyScore(query, field.value);
    if (score === null) continue;
    const weighted = score * field.weight;
    if (bestScore === null || weighted > bestScore) {
      bestScore = weighted;
    }
  }

  if (bestScore === null) return null;
  return { course, score: bestScore };
}

export function filterCoursesByFuzzyQuery(
  courses: MoodleCourse[],
  queryRaw: string,
): MoodleCourse[] {
  const query = queryRaw.trim();
  if (!query) return courses;

  return courses
    .map((course) => rankCourse(course, query))
    .filter((result): result is RankedCourse => Boolean(result))
    .sort((left, right) => {
      if (right.score !== left.score) return right.score - left.score;
      return left.course.fullname.localeCompare(right.course.fullname, undefined, {
        sensitivity: "base",
      });
    })
    .map((result) => result.course);
}
