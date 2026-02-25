import React from "react";
import { Box, Text } from "ink";
import Spinner from "ink-spinner";
import { COLORS } from "./colors.ts";
import { truncateText } from "./timetable/text.ts";
import type { MoodleCourse, MoodleCourseSection } from "../utils/moodle.ts";

function stripHtml(value: string | undefined): string {
  if (!value) return "";
  return value.replace(/<[^>]*>/g, " ").replace(/\s+/g, " ").trim();
}

export function buildCourseContentLines(sections: MoodleCourseSection[]): string[] {
  if (sections.length === 0) {
    return ["No visible course content returned by Moodle."];
  }

  const lines: string[] = [];

  sections.forEach((section, sectionIndex) => {
    const sectionName =
      section.name?.trim() ||
      `Section ${section.section !== undefined ? section.section : sectionIndex + 1}`;
    lines.push(`== ${sectionName} ==`);

    const summary = stripHtml(section.summary);
    if (summary) {
      lines.push(summary);
    }

    if (section.modules.length === 0) {
      lines.push("(No activities in this section)");
      lines.push("");
      return;
    }

    section.modules.forEach((module) => {
      lines.push(`- [${module.modname || "module"}] ${module.name}`);

      const description = stripHtml(module.description);
      if (description) {
        lines.push(`  ${description}`);
      }

      if (module.url) {
        lines.push(`  URL: ${module.url}`);
      }

      module.contents.forEach((content) => {
        const itemLabel = content.filename || content.type || "content item";
        const itemUrl = content.fileurl || content.url;
        if (itemUrl) {
          lines.push(`  * ${itemLabel}: ${itemUrl}`);
        } else {
          lines.push(`  * ${itemLabel}`);
        }
      });
    });

    lines.push("");
  });

  return lines;
}

interface CoursePageProps {
  termWidth: number;
  bodyHeight: number;
  course: MoodleCourse | null;
  sections: MoodleCourseSection[];
  contentLines: string[];
  scrollOffset: number;
  loading: boolean;
  error: string;
}

export default function CoursePage({
  termWidth,
  bodyHeight,
  course,
  sections,
  contentLines,
  scrollOffset,
  loading,
  error,
}: CoursePageProps) {
  const contentRows = Math.max(4, bodyHeight - 2);
  const visibleContentLines = contentLines.slice(scrollOffset, scrollOffset + contentRows);
  const totalLines = contentLines.length;
  const visibleStart = totalLines > 0 ? Math.min(scrollOffset + 1, totalLines) : 0;
  const visibleEnd = totalLines > 0 ? Math.min(scrollOffset + contentRows, totalLines) : 0;

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
            {totalLines > 0 ? `${visibleStart}-${visibleEnd}/${totalLines}` : "0/0"}
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
            {visibleContentLines.map((line, index) => (
              <Text key={`${scrollOffset}-${index}`}>
                {truncateText(line, Math.max(16, termWidth - 4))}
              </Text>
            ))}
          </Box>
        )}
      </Box>
    </>
  );
}
