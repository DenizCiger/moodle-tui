import type { MoodleCourse } from "../utils/moodle.ts";

interface WeightedField {
  value: string;
  weight: number;
}

interface RankedCourse {
  course: MoodleCourse;
  score: number;
}

function isBoundaryChar(char: string): boolean {
  return char === " " || char === "-" || char === "_" || char === "/" || char === ".";
}

function fuzzyScore(queryRaw: string, candidateRaw: string): number | null {
  const query = queryRaw.toLowerCase();
  const candidate = candidateRaw.toLowerCase();
  if (!query) return 0;
  if (!candidate) return null;

  let queryIdx = 0;
  let previousMatchIdx = -1;
  let score = 0;

  for (let idx = 0; idx < candidate.length && queryIdx < query.length; idx += 1) {
    if (candidate[idx] !== query[queryIdx]) continue;

    score += 1;

    if (previousMatchIdx === idx - 1) {
      score += 6;
    }

    const previousChar = idx > 0 ? candidate[idx - 1] || "" : "";
    if (idx === 0 || isBoundaryChar(previousChar)) {
      score += 4;
    }

    if (idx < 6) {
      score += (6 - idx) * 0.25;
    }

    previousMatchIdx = idx;
    queryIdx += 1;
  }

  if (queryIdx !== query.length) {
    return null;
  }

  score -= candidate.length * 0.01;
  return score;
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
