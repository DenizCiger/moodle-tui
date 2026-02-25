import { describe, expect, it } from "bun:test";
import { getShortcutSections, isShortcutPressed, type InputKey } from "./shortcuts.ts";

function key(overrides: Partial<InputKey> = {}): InputKey {
  return {
    upArrow: false,
    downArrow: false,
    leftArrow: false,
    rightArrow: false,
    pageDown: false,
    pageUp: false,
    home: false,
    end: false,
    return: false,
    escape: false,
    ctrl: false,
    shift: false,
    tab: false,
    backspace: false,
    delete: false,
    meta: false,
    ...overrides,
  };
}

describe("shortcut registry", () => {
  it("matches global settings open", () => {
    expect(isShortcutPressed("settings-open", "?", key())).toBe(true);
    expect(isShortcutPressed("settings-open", "x", key())).toBe(false);
  });

  it("matches list navigation keys", () => {
    expect(isShortcutPressed("dashboard-up", "", key({ upArrow: true }))).toBe(true);
    expect(isShortcutPressed("dashboard-down", "", key({ downArrow: true }))).toBe(true);
    expect(isShortcutPressed("dashboard-expand", "", key({ rightArrow: true }))).toBe(true);
    expect(isShortcutPressed("dashboard-collapse", "", key({ leftArrow: true }))).toBe(true);
  });

  it("matches edge navigation shortcuts", () => {
    expect(isShortcutPressed("dashboard-home", "", key({ home: true }))).toBe(true);
    expect(isShortcutPressed("dashboard-end", "", key({ end: true }))).toBe(true);
  });

  it("matches course open and back shortcuts", () => {
    expect(isShortcutPressed("dashboard-open-finder", "/", key())).toBe(true);
    expect(isShortcutPressed("dashboard-open-content-finder", "f", key())).toBe(true);
    expect(isShortcutPressed("dashboard-open-assignment-modal", "", key({ return: true }))).toBe(
      true,
    );
    expect(isShortcutPressed("dashboard-back", "", key({ escape: true }))).toBe(true);
  });

  it("matches course finder shortcuts", () => {
    expect(isShortcutPressed("course-finder-submit", "", key({ return: true }))).toBe(true);
    expect(isShortcutPressed("course-finder-cancel", "", key({ escape: true }))).toBe(
      true,
    );
    expect(isShortcutPressed("course-content-finder-submit", "", key({ return: true }))).toBe(
      true,
    );
    expect(isShortcutPressed("course-content-finder-cancel", "", key({ escape: true }))).toBe(
      true,
    );
    expect(isShortcutPressed("assignment-modal-close", "", key({ escape: true }))).toBe(true);
  });

  it("returns course-specific sections", () => {
    const sections = getShortcutSections("dashboard");
    const dashboardSection = sections.find(
      (section) => section.title === "Dashboard / Course Page",
    );

    expect(sections.some((section) => section.title === "Global")).toBe(true);
    expect(sections.some((section) => section.title === "Dashboard / Course Page")).toBe(true);
    expect(sections.some((section) => section.title === "Course Finder")).toBe(
      true,
    );
    expect(sections.some((section) => section.title === "Course Content Finder")).toBe(
      true,
    );
    expect(sections.some((section) => section.title === "Assignment Modal")).toBe(true);
    expect(dashboardSection?.items.some((item) => item.id === "dashboard-expand")).toBe(true);
    expect(dashboardSection?.items.some((item) => item.id === "dashboard-collapse")).toBe(true);
    expect(
      dashboardSection?.items.some((item) => item.id === "dashboard-open-content-finder"),
    ).toBe(true);
    expect(
      dashboardSection?.items.some((item) => item.id === "dashboard-open-assignment-modal"),
    ).toBe(true);
  });
});
