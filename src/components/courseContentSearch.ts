import type { CourseTreeNodeKind, CourseTreeRow } from "./CoursePage.tsx";
import { fuzzyScore } from "./fuzzySearch.ts";

interface RankedCourseRow {
  row: CourseTreeRow;
  score: number;
}

const ROW_KIND_WEIGHTS: Record<CourseTreeNodeKind, number> = {
  section: 1.15,
  module: 1.1,
  label: 1.05,
  "content-item": 1.0,
  "module-description": 0.85,
  "module-url": 0.8,
  summary: 0.75,
};

function isSearchableRow(row: CourseTreeRow): boolean {
  return row.id !== "empty";
}

function rankRow(row: CourseTreeRow, query: string): RankedCourseRow | null {
  const score = fuzzyScore(query, row.text);
  if (score === null) return null;
  const weight = ROW_KIND_WEIGHTS[row.kind] ?? 1;
  return { row, score: score * weight };
}

export function filterCourseContentByFuzzyQuery(
  rows: CourseTreeRow[],
  queryRaw: string,
): CourseTreeRow[] {
  const searchableRows = rows.filter(isSearchableRow);
  const query = queryRaw.trim();
  if (!query) return searchableRows;

  return searchableRows
    .map((row) => rankRow(row, query))
    .filter((result): result is RankedCourseRow => Boolean(result))
    .sort((left, right) => {
      if (right.score !== left.score) return right.score - left.score;
      const byText = left.row.text.localeCompare(right.row.text, undefined, {
        sensitivity: "base",
      });
      if (byText !== 0) return byText;
      return left.row.id.localeCompare(right.row.id, undefined, { sensitivity: "base" });
    })
    .map((result) => result.row);
}
