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
    expect(isShortcutPressed("courses-up", "", key({ upArrow: true }))).toBe(true);
    expect(isShortcutPressed("courses-down", "", key({ downArrow: true }))).toBe(true);
  });

  it("matches edge navigation shortcuts", () => {
    expect(isShortcutPressed("courses-home", "", key({ home: true }))).toBe(true);
    expect(isShortcutPressed("courses-end", "", key({ end: true }))).toBe(true);
  });

  it("matches course open and back shortcuts", () => {
    expect(isShortcutPressed("courses-open", "", key({ return: true }))).toBe(true);
    expect(isShortcutPressed("course-back", "", key({ escape: true }))).toBe(true);
  });

  it("matches search shortcuts", () => {
    expect(isShortcutPressed("courses-search", "/", key())).toBe(true);
    expect(isShortcutPressed("courses-search-clear", "c", key())).toBe(true);
    expect(isShortcutPressed("courses-search-submit", "", key({ return: true }))).toBe(
      true,
    );
    expect(isShortcutPressed("courses-search-cancel", "", key({ escape: true }))).toBe(
      true,
    );
  });

  it("returns course-specific sections", () => {
    const sections = getShortcutSections("courses");
    expect(sections.some((section) => section.title === "Global")).toBe(true);
    expect(sections.some((section) => section.title === "Courses")).toBe(true);
    expect(sections.some((section) => section.title === "Course Search Input")).toBe(
      true,
    );
  });
});
