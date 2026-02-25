import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { join } from "path";
import { getAppConfigDir } from "./config.ts";
import type { MoodleCourse } from "./moodle.ts";

interface CoursesCacheData {
  timestamp: number;
  courses: MoodleCourse[];
}

function getCacheFilePath(): string {
  return join(getAppConfigDir(), "cache.json");
}

function ensureCacheDir(): void {
  mkdirSync(getAppConfigDir(), { recursive: true });
}

function normalizeCourses(value: unknown): MoodleCourse[] {
  if (!Array.isArray(value)) return [];

  const courses: MoodleCourse[] = [];

  for (const entry of value) {
    if (!entry || typeof entry !== "object") continue;
    const row = entry as Partial<MoodleCourse>;
    if (typeof row.id !== "number") continue;
    if (typeof row.shortname !== "string") continue;
    if (typeof row.fullname !== "string") continue;

    const course: MoodleCourse = {
      id: row.id,
      shortname: row.shortname,
      fullname: row.fullname,
    };

    if (typeof row.displayname === "string") course.displayname = row.displayname;
    if (typeof row.categoryid === "number") course.categoryid = row.categoryid;
    if (typeof row.categoryname === "string") course.categoryname = row.categoryname;
    if (typeof row.summary === "string") course.summary = row.summary;
    if (typeof row.visible === "number") course.visible = row.visible;
    if (typeof row.progress === "number" || row.progress === null) {
      course.progress = row.progress;
    }
    if (typeof row.courseurl === "string") course.courseurl = row.courseurl;

    courses.push(course);
  }

  return courses;
}

export function getCachedCourses(): MoodleCourse[] | null {
  try {
    const file = getCacheFilePath();
    if (!existsSync(file)) return null;

    const raw = readFileSync(file, "utf-8");
    const parsed = JSON.parse(raw) as Partial<CoursesCacheData> | null;
    if (!parsed || typeof parsed !== "object") return null;

    const courses = normalizeCourses(parsed.courses);
    return courses.length > 0 ? courses : [];
  } catch {
    return null;
  }
}

export function saveCoursesToCache(courses: MoodleCourse[]): void {
  try {
    ensureCacheDir();
    const payload: CoursesCacheData = {
      timestamp: Date.now(),
      courses,
    };
    writeFileSync(getCacheFilePath(), JSON.stringify(payload, null, 2), { mode: 0o600 });
  } catch {
    // ignore cache write failures
  }
}

export function clearCache(): void {
  try {
    ensureCacheDir();
    const payload: CoursesCacheData = { timestamp: Date.now(), courses: [] };
    writeFileSync(getCacheFilePath(), JSON.stringify(payload, null, 2), { mode: 0o600 });
  } catch {
    // ignore
  }
}
