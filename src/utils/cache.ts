import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { join } from "path";
import { getAppConfigDir } from "./config.ts";
import {
  normalizeCourseSection,
  type MoodleCourse,
  type MoodleCourseSection,
  type MoodleUpcomingAssignment,
} from "./moodle.ts";

interface CoursesCacheData {
  timestamp: number;
  courses: MoodleCourse[];
}

interface CacheEntry<TData> {
  timestamp: number;
  data: TData;
}

interface DashboardCacheData {
  courses: MoodleCourse[];
  upcomingAssignments: MoodleUpcomingAssignment[];
}

interface CacheData {
  // Legacy layout
  timestamp?: number;
  courses?: unknown;
  // Current layout
  dashboard?: CacheEntry<unknown>;
  coursePages?: Record<string, CacheEntry<unknown>>;
}

const CACHE_TTL_MS = 1000 * 60 * 60 * 24 * 21;
const MAX_CACHED_COURSE_PAGES = 48;

function getCacheFilePath(): string {
  return join(getAppConfigDir(), "cache.json");
}

function ensureCacheDir(): void {
  mkdirSync(getAppConfigDir(), { recursive: true });
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  return value as Record<string, unknown>;
}

function asNumber(value: unknown): number | undefined {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  return undefined;
}

function asString(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  return value;
}

function isExpired(timestamp: number): boolean {
  return Date.now() - timestamp > CACHE_TTL_MS;
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

function normalizeUpcomingAssignments(value: unknown): MoodleUpcomingAssignment[] {
  if (!Array.isArray(value)) return [];

  const assignments: MoodleUpcomingAssignment[] = [];

  for (const entry of value) {
    const row = asRecord(entry);
    if (!row) continue;

    const id = asNumber(row.id);
    const dueDate = asNumber(row.dueDate);
    const courseId = asNumber(row.courseId);
    const name = asString(row.name);
    if (id === undefined || dueDate === undefined || courseId === undefined || !name) continue;

    const assignment: MoodleUpcomingAssignment = {
      id,
      name,
      dueDate,
      courseId,
    };

    const courseShortName = asString(row.courseShortName);
    const courseFullName = asString(row.courseFullName);
    if (courseShortName) assignment.courseShortName = courseShortName;
    if (courseFullName) assignment.courseFullName = courseFullName;

    assignments.push(assignment);
  }

  return assignments;
}

function normalizeCourseSections(value: unknown): MoodleCourseSection[] {
  if (!Array.isArray(value)) return [];

  const sections: MoodleCourseSection[] = [];
  for (const entry of value) {
    const normalized = normalizeCourseSection(entry);
    if (!normalized) continue;
    sections.push(normalized);
  }

  return sections;
}

function readCache(): CacheData {
  try {
    const file = getCacheFilePath();
    if (!existsSync(file)) return {};
    const raw = readFileSync(file, "utf-8");
    const parsed = JSON.parse(raw) as unknown;
    const record = asRecord(parsed);
    if (!record) return {};
    return record as CacheData;
  } catch {
    return {};
  }
}

function writeCache(cache: CacheData): void {
  ensureCacheDir();
  writeFileSync(getCacheFilePath(), JSON.stringify(cache, null, 2), { mode: 0o600 });
}

function pruneCoursePages(
  pages: Record<string, CacheEntry<unknown>> | undefined,
): Record<string, CacheEntry<unknown>> {
  if (!pages) return {};

  const entries = Object.entries(pages)
    .filter(([, entry]) => !isExpired(entry.timestamp))
    .sort((left, right) => right[1].timestamp - left[1].timestamp)
    .slice(0, MAX_CACHED_COURSE_PAGES);

  return Object.fromEntries(entries);
}

function getDashboardFromCacheValue(
  rawDashboard: CacheEntry<unknown> | undefined,
): DashboardCacheData | null {
  if (!rawDashboard) return null;
  if (isExpired(rawDashboard.timestamp)) return null;

  const dashboardRecord = asRecord(rawDashboard.data);
  if (!dashboardRecord) return null;

  const courses = normalizeCourses(dashboardRecord.courses);
  const upcomingAssignments = normalizeUpcomingAssignments(
    dashboardRecord.upcomingAssignments,
  );

  return { courses, upcomingAssignments };
}

function getLegacyCoursesFromCache(cache: CacheData): MoodleCourse[] | null {
  const legacyTimestamp = asNumber(cache.timestamp);
  if (legacyTimestamp !== undefined && isExpired(legacyTimestamp)) {
    return null;
  }

  const courses = normalizeCourses(cache.courses);
  if (courses.length > 0) return courses;
  if (Array.isArray(cache.courses)) return [];
  return null;
}

function saveCache(cache: CacheData): void {
  const normalizedCache: CacheData = {
    ...cache,
    coursePages: pruneCoursePages(cache.coursePages),
  };
  writeCache(normalizedCache);
}

export function getCachedDashboard():
  | {
      courses: MoodleCourse[];
      upcomingAssignments: MoodleUpcomingAssignment[];
    }
  | null {
  try {
    const cache = readCache();
    const dashboardEntry = getDashboardFromCacheValue(cache.dashboard);
    if (dashboardEntry) {
      return dashboardEntry;
    }

    const legacyCourses = getLegacyCoursesFromCache(cache);
    if (!legacyCourses) return null;
    return { courses: legacyCourses, upcomingAssignments: [] };
  } catch {
    return null;
  }
}

export function getCachedCourses(): MoodleCourse[] | null {
  try {
    const dashboard = getCachedDashboard();
    return dashboard ? dashboard.courses : null;
  } catch {
    return null;
  }
}

export function saveDashboardToCache(
  courses: MoodleCourse[],
  upcomingAssignments: MoodleUpcomingAssignment[],
): void {
  try {
    const cache = readCache();
    const now = Date.now();
    cache.timestamp = now;
    cache.courses = courses;
    cache.dashboard = {
      timestamp: now,
      data: {
        courses,
        upcomingAssignments,
      },
    };

    saveCache(cache);
  } catch {
    // ignore cache write failures
  }
}

export function saveCoursesToCache(courses: MoodleCourse[]): void {
  try {
    const cache = readCache();
    const cachedDashboard = getDashboardFromCacheValue(cache.dashboard);
    const upcomingAssignments = cachedDashboard?.upcomingAssignments ?? [];
    saveDashboardToCache(courses, upcomingAssignments);
  } catch {
    // ignore cache write failures
  }
}

export function getCachedCourseSections(courseId: number): MoodleCourseSection[] | null {
  try {
    const cache = readCache();
    const key = String(courseId);
    const entry = cache.coursePages?.[key];
    if (!entry) return null;
    if (isExpired(entry.timestamp)) {
      if (cache.coursePages) {
        delete cache.coursePages[key];
        saveCache(cache);
      }
      return null;
    }

    const sections = normalizeCourseSections(entry.data);
    if (sections.length > 0) return sections;
    if (Array.isArray(entry.data)) return [];
    return null;
  } catch {
    return null;
  }
}

export function saveCourseSectionsToCache(
  courseId: number,
  sections: MoodleCourseSection[],
): void {
  try {
    const cache = readCache();
    const key = String(courseId);

    cache.coursePages = {
      ...(cache.coursePages ?? {}),
      [key]: {
        timestamp: Date.now(),
        data: sections,
      },
    };

    saveCache(cache);
  } catch {
    // ignore cache write failures
  }
}

export function clearCache(): void {
  try {
    const now = Date.now();
    const payload: CacheData & CoursesCacheData = {
      timestamp: now,
      courses: [],
      dashboard: {
        timestamp: now,
        data: {
          courses: [],
          upcomingAssignments: [],
        },
      },
      coursePages: {},
    };
    writeCache(payload);
  } catch {
    // ignore
  }
}
