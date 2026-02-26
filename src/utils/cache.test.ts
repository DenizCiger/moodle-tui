import { afterEach, describe, expect, it } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";
import {
  getCachedCourseSections,
  getCachedCourses,
  getCachedDashboard,
  saveCourseSectionsToCache,
  saveDashboardToCache,
} from "./cache.ts";
import { getAppConfigDir } from "./config.ts";

const CACHE_TTL_MS = 1000 * 60 * 60 * 24 * 21;
const tempDirs: string[] = [];

function withTempConfigDir(): string {
  const dir = mkdtempSync(join(tmpdir(), "tui-moodle-cache-"));
  tempDirs.push(dir);
  process.env.TUI_MOODLE_CONFIG_DIR = dir;
  return dir;
}

function getCacheFilePath(): string {
  return join(getAppConfigDir(), "cache.json");
}

afterEach(() => {
  delete process.env.TUI_MOODLE_CONFIG_DIR;
  while (tempDirs.length > 0) {
    const dir = tempDirs.pop();
    if (dir) rmSync(dir, { recursive: true, force: true });
  }
});

describe("cache storage", () => {
  it("saves and loads dashboard cache", () => {
    withTempConfigDir();

    saveDashboardToCache(
      [{ id: 7, shortname: "MATH", fullname: "Mathematics" }],
      [{ id: 5, name: "Homework 1", dueDate: 2_000_000_000, courseId: 7 }],
    );

    const dashboard = getCachedDashboard();
    expect(dashboard).not.toBeNull();
    expect(dashboard?.courses.map((course) => course.id)).toEqual([7]);
    expect(dashboard?.upcomingAssignments.map((assignment) => assignment.id)).toEqual([5]);
    expect(getCachedCourses()?.map((course) => course.id)).toEqual([7]);
  });

  it("reads legacy course cache layout", () => {
    withTempConfigDir();

    writeFileSync(
      getCacheFilePath(),
      JSON.stringify({
        timestamp: Date.now(),
        courses: [{ id: 3, shortname: "ENG", fullname: "English" }],
      }),
    );

    const dashboard = getCachedDashboard();
    expect(dashboard).not.toBeNull();
    expect(dashboard?.courses.map((course) => course.id)).toEqual([3]);
    expect(dashboard?.upcomingAssignments).toEqual([]);
  });

  it("saves and loads cached course sections by course id", () => {
    withTempConfigDir();

    saveCourseSectionsToCache(42, [
      {
        id: 11,
        name: "Week 1",
        modules: [{ id: 91, name: "Assignment", modname: "assign", contents: [] }],
      },
    ]);

    const sections = getCachedCourseSections(42);
    expect(sections).not.toBeNull();
    expect(sections?.[0]?.id).toBe(11);
    expect(sections?.[0]?.modules[0]?.id).toBe(91);
  });

  it("ignores expired dashboard and course page cache entries", () => {
    withTempConfigDir();
    const expiredTimestamp = Date.now() - CACHE_TTL_MS - 1;

    writeFileSync(
      getCacheFilePath(),
      JSON.stringify({
        dashboard: {
          timestamp: expiredTimestamp,
          data: {
            courses: [{ id: 1, shortname: "X", fullname: "Expired Course" }],
            upcomingAssignments: [{ id: 1, name: "Expired", dueDate: 100, courseId: 1 }],
          },
        },
        coursePages: {
          "1": {
            timestamp: expiredTimestamp,
            data: [{ id: 55, name: "Expired Section", modules: [] }],
          },
        },
      }),
    );

    expect(getCachedDashboard()).toBeNull();
    expect(getCachedCourseSections(1)).toBeNull();
  });
});
