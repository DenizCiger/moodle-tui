import React from "react";
import { Box, Text } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import { truncateText } from "./timetable/text.ts";
import type { MoodleCourse, MoodleCourseSection } from "../utils/moodle.ts";

const NAMED_HTML_ENTITIES: Record<string, string> = {
  amp: "&",
  lt: "<",
  gt: ">",
  quot: "\"",
  apos: "'",
  nbsp: " ",
};

function decodeHtmlEntities(value: string): string {
  return value.replace(/&(#x?[0-9a-fA-F]+|[a-zA-Z]+);/g, (full, entity: string) => {
    if (entity.startsWith("#x") || entity.startsWith("#X")) {
      const codePoint = Number.parseInt(entity.slice(2), 16);
      if (!Number.isFinite(codePoint) || codePoint <= 0) return full;
      return String.fromCodePoint(codePoint);
    }

    if (entity.startsWith("#")) {
      const codePoint = Number.parseInt(entity.slice(1), 10);
      if (!Number.isFinite(codePoint) || codePoint <= 0) return full;
      return String.fromCodePoint(codePoint);
    }

    return NAMED_HTML_ENTITIES[entity.toLowerCase()] ?? full;
  });
}

function stripHtml(value: string | undefined): string {
  if (!value) return "";
  return decodeHtmlEntities(value).replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

export type CourseTreeNodeKind =
  | "section"
  | "module"
  | "summary"
  | "module-description"
  | "module-url"
  | "content-item"
  | "label";

export interface CourseTreeRow {
  id: string;
  kind: CourseTreeNodeKind;
  depth: number;
  text: string;
  icon: string;
  collapsible: boolean;
  expanded: boolean;
  parentId?: string;
}

const MODULE_TYPE_ICONS: Record<string, string> = {
  forum: "💬",
  quiz: "📝",
  resource: "📄",
  assign: "✅",
  url: "🔗",
  page: "📃",
  book: "📚",
  folder: "📁",
  label: "🏷",
};

export function courseSectionNodeId(sectionId: number): string {
  return `section:${sectionId}`;
}

function courseModuleNodeId(sectionId: number, moduleId: number): string {
  return `module:${sectionId}:${moduleId}`;
}

function normalizeType(value: string | undefined): string {
  return (value || "").trim().toLowerCase();
}

function resolveModuleIcon(modname: string | undefined): string {
  return MODULE_TYPE_ICONS[normalizeType(modname)] || "📦";
}

function resolveContentIcon(contentType: string | undefined): string {
  const normalized = normalizeType(contentType);
  if (normalized === "folder") return "📁";
  if (normalized === "url") return "🔗";
  return "📄";
}

function toCollapsedSet(collapsedIds: ReadonlySet<string> | string[]): ReadonlySet<string> {
  if (collapsedIds instanceof Set) return collapsedIds;
  return new Set(collapsedIds);
}

export function buildCourseTreeRows(
  sections: MoodleCourseSection[],
  collapsedIds: ReadonlySet<string> | string[],
): CourseTreeRow[] {
  if (sections.length === 0) {
    return [
      {
        id: "empty",
        kind: "summary",
        depth: 0,
        text: "No visible course content returned by Moodle.",
        icon: "•",
        collapsible: false,
        expanded: false,
      },
    ];
  }

  const collapsedSet = toCollapsedSet(collapsedIds);
  const rows: CourseTreeRow[] = [];

  sections.forEach((section, sectionIndex) => {
    const sectionName =
      section.name?.trim() ||
      `Section ${section.section !== undefined ? section.section : sectionIndex + 1}`;
    const sectionId = courseSectionNodeId(section.id);
    const sectionCollapsed = collapsedSet.has(sectionId);

    rows.push({
      id: sectionId,
      kind: "section",
      depth: 0,
      text: sectionName,
      icon: "📁",
      collapsible: true,
      expanded: !sectionCollapsed,
    });

    if (sectionCollapsed) return;

    const summary = stripHtml(section.summary);
    if (summary) {
      rows.push({
        id: `summary:${section.id}`,
        kind: "summary",
        depth: 1,
        text: summary,
        icon: "🗒",
        collapsible: false,
        expanded: false,
        parentId: sectionId,
      });
    }

    if (section.modules.length === 0) {
      rows.push({
        id: `section-empty:${section.id}`,
        kind: "summary",
        depth: 1,
        text: "(No activities in this section)",
        icon: "•",
        collapsible: false,
        expanded: false,
        parentId: sectionId,
      });
      return;
    }

    section.modules.forEach((module) => {
      const modname = normalizeType(module.modname);
      if (modname === "label") {
        const labelText = stripHtml(module.description) || stripHtml(module.name) || "(Empty label)";
        rows.push({
          id: `label:${section.id}:${module.id}`,
          kind: "label",
          depth: 1,
          text: labelText,
          icon: "🏷",
          collapsible: false,
          expanded: false,
          parentId: sectionId,
        });
        return;
      }

      const moduleId = courseModuleNodeId(section.id, module.id);
      const moduleCollapsed = collapsedSet.has(moduleId);

      rows.push({
        id: moduleId,
        kind: "module",
        depth: 1,
        text: module.name,
        icon: resolveModuleIcon(module.modname),
        collapsible: true,
        expanded: !moduleCollapsed,
        parentId: sectionId,
      });

      if (moduleCollapsed) return;

      const description = stripHtml(module.description);
      if (description) {
        rows.push({
          id: `module-description:${section.id}:${module.id}`,
          kind: "module-description",
          depth: 2,
          text: description,
          icon: "•",
          collapsible: false,
          expanded: false,
          parentId: moduleId,
        });
      }

      if (module.url) {
        rows.push({
          id: `module-url:${section.id}:${module.id}`,
          kind: "module-url",
          depth: 2,
          text: module.url,
          icon: "🔗",
          collapsible: false,
          expanded: false,
          parentId: moduleId,
        });
      }

      module.contents.forEach((content, contentIndex) => {
        const itemLabel = content.filename || content.type || "content item";
        const itemUrl = content.fileurl || content.url;
        const text = itemUrl ? `${itemLabel}: ${itemUrl}` : itemLabel;

        rows.push({
          id: `content:${section.id}:${module.id}:${contentIndex}`,
          kind: "content-item",
          depth: 2,
          text,
          icon: resolveContentIcon(content.type),
          collapsible: false,
          expanded: false,
          parentId: moduleId,
        });
      });
    });
  });

  return rows;
}

interface CoursePageProps {
  termWidth: number;
  bodyHeight: number;
  course: MoodleCourse | null;
  sections: MoodleCourseSection[];
  rows: CourseTreeRow[];
  selectedIndex: number;
  scrollOffset: number;
  loading: boolean;
  error: string;
}

export default function CoursePage({
  termWidth,
  bodyHeight,
  course,
  sections,
  rows,
  selectedIndex,
  scrollOffset,
  loading,
  error,
}: CoursePageProps) {
  const contentRows = Math.max(4, bodyHeight - 2);
  const visibleRows = rows.slice(scrollOffset, scrollOffset + contentRows);
  const totalRows = rows.length;
  const visibleStart = totalRows > 0 ? Math.min(scrollOffset + 1, totalRows) : 0;
  const visibleEnd = totalRows > 0 ? Math.min(scrollOffset + visibleRows.length, totalRows) : 0;
  const maxLineWidth = Math.max(16, termWidth - 4);

  const renderTreePrefix = (row: CourseTreeRow): string => {
    const indent = "  ".repeat(Math.max(0, row.depth));
    const indicator = row.collapsible ? (row.expanded ? "▾" : "▸") : "•";
    return `${indent}${indicator}`;
  };

  const renderTreeLine = (row: CourseTreeRow): string => {
    const prefix = renderTreePrefix(row);
    if (row.kind === "label") {
      return `${prefix} ${row.text}`;
    }
    return `${prefix} ${row.icon} ${row.text}`;
  };

  return (
    <>
      <Box justifyContent="space-between">
        <Text dimColor>{truncateText(course?.fullname || "Course", Math.max(20, termWidth - 30))}</Text>
        <Text dimColor>{course ? `Sections: ${sections.length}` : ""}</Text>
      </Box>

      <Box
        flexDirection="column"
        marginTop={1}
        height={bodyHeight + 1}
        borderStyle="single"
        borderColor={COLORS.neutral.brightBlack}
      >
        <Box justifyContent="space-between" paddingX={1}>
          <Text bold>{course?.shortname || "Course"}</Text>
          <Text dimColor>
            {totalRows > 0 ? `${visibleStart}-${visibleEnd}/${totalRows}` : "0/0"}
          </Text>
        </Box>

        {loading ? (
          <Box justifyContent="center" alignItems="center" flexGrow={1}>
            <Text color={COLORS.warning}>
              <Spinner type="dots" /> Loading course content...
            </Text>
          </Box>
        ) : error ? (
          <Box justifyContent="center" alignItems="center" flexGrow={1} paddingX={1}>
            <Text color={COLORS.error}>
              {truncateText(error, Math.max(16, termWidth - 4))}
            </Text>
          </Box>
        ) : (
          <Box flexDirection="column" paddingX={1}>
            {visibleRows.map((row, index) => {
              const absoluteIndex = scrollOffset + index;
              const selected = absoluteIndex === selectedIndex;
              const isLabel = row.kind === "label";
              const secondary =
                row.kind === "summary" ||
                row.kind === "module-description" ||
                row.kind === "module-url";
              const color = secondary ? COLORS.neutral.gray : COLORS.neutral.white;

              return (
                <Text
                  key={row.id}
                  color={color}
                  bold={selected}
                  backgroundColor={selected ? COLORS.panel.selected : undefined}
                >
                  {isLabel ? (
                    <>
                      {truncateText(`${renderTreePrefix(row)} `, maxLineWidth)}
                      <Text underline>{truncateText(row.text, maxLineWidth)}</Text>
                    </>
                  ) : (
                    truncateText(renderTreeLine(row), maxLineWidth)
                  )}
                </Text>
              );
            })}
            {visibleRows.length === 0 && (
              <Text dimColor>
                {truncateText("  • No visible course content returned by Moodle.", maxLineWidth)}
              </Text>
            )}
          </Box>
        )}
      </Box>
    </>
  );
}
