import React, { useCallback, useEffect, useMemo, useState } from "react";
import { Box, Text } from "ink";
import Spinner from "ink-spinner";
import type { MoodleCourse } from "../utils/moodle.ts";
import { COLORS } from "./colors.ts";
import { filterCoursesByFuzzyQuery } from "./courseSearch.ts";
import TextInput from "./TextInput.tsx";
import { truncateText } from "./timetable/text.ts";

interface CourseFinderOverlayProps {
  termWidth: number;
  termHeight: number;
  courses: MoodleCourse[];
  loading: boolean;
  onClose: () => void;
  onApplyCourse: (course: MoodleCourse) => void;
}

export default function CourseFinderOverlay({
  termWidth,
  termHeight,
  courses,
  loading,
  onClose,
  onApplyCourse,
}: CourseFinderOverlayProps) {
  const [draft, setDraft] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [scrollOffset, setScrollOffset] = useState(0);

  const searchResults = useMemo(
    () => filterCoursesByFuzzyQuery(courses, draft),
    [courses, draft],
  );

  const maxModalWidth = Math.max(1, termWidth - 2);
  const maxModalHeight = Math.max(1, termHeight - 2);
  const modalWidth = Math.min(maxModalWidth, Math.max(28, Math.min(112, termWidth - 8)));
  const modalHeight = Math.min(maxModalHeight, Math.max(10, Math.min(30, termHeight - 4)));
  const rows = Math.max(1, modalHeight - 7);
  const resultLineWidth = Math.max(1, modalWidth - 4);

  const visibleResults = useMemo(
    () => searchResults.slice(scrollOffset, scrollOffset + rows),
    [searchResults, scrollOffset, rows],
  );

  useEffect(() => {
    setSelectedIdx((previous) => Math.min(previous, Math.max(searchResults.length - 1, 0)));
  }, [searchResults.length]);

  useEffect(() => {
    const maxScroll = Math.max(searchResults.length - rows, 0);
    setScrollOffset((previous) => Math.min(previous, maxScroll));
  }, [searchResults.length, rows]);

  useEffect(() => {
    if (selectedIdx < scrollOffset) {
      setScrollOffset(selectedIdx);
      return;
    }

    if (selectedIdx >= scrollOffset + rows) {
      setScrollOffset(selectedIdx - rows + 1);
    }
  }, [rows, scrollOffset, selectedIdx]);

  const moveSelection = useCallback(
    (delta: number) => {
      setSelectedIdx((previous) =>
        Math.max(0, Math.min(previous + delta, Math.max(searchResults.length - 1, 0))),
      );
    },
    [searchResults.length],
  );

  const applySelection = useCallback(
    (query: string) => {
      const instantResults = filterCoursesByFuzzyQuery(courses, query);
      const boundedIndex = Math.max(0, Math.min(selectedIdx, Math.max(instantResults.length - 1, 0)));
      const selected = instantResults[boundedIndex];

      if (!selected) {
        onClose();
        return;
      }

      onApplyCourse(selected);
    },
    [courses, onApplyCourse, onClose, selectedIdx],
  );

  const visibleStart = searchResults.length > 0 ? scrollOffset + 1 : 0;
  const visibleEnd = Math.min(searchResults.length, scrollOffset + visibleResults.length);

  return (
    <Box
      position="absolute"
      width={termWidth}
      height={termHeight}
      justifyContent="center"
      alignItems="center"
    >
      <Box
        flexDirection="column"
        width={modalWidth}
        height={modalHeight}
        borderStyle="round"
        borderColor={COLORS.brand}
        backgroundColor={COLORS.neutral.black}
        paddingX={1}
      >
        <Box justifyContent="space-between">
          <Text bold color={COLORS.brand}>
            Course Finder
          </Text>
          <Text dimColor>
            {searchResults.length > 0
              ? `${Math.min(selectedIdx + 1, searchResults.length)}/${searchResults.length}`
              : "0/0"}
          </Text>
        </Box>

        <Box>
          <Text color={COLORS.brand}>{"> "}</Text>
          <TextInput
            value={draft}
            onChange={(value) => {
              setDraft(value);
              setSelectedIdx(0);
              setScrollOffset(0);
            }}
            onSubmit={(value) => {
              applySelection(value);
            }}
            onKey={(_input, key) => {
              if (key.escape) {
                onClose();
                return true;
              }

              if (key.upArrow) {
                moveSelection(-1);
                return true;
              }

              if (key.downArrow) {
                moveSelection(1);
                return true;
              }

              if (key.pageUp) {
                moveSelection(-rows);
                return true;
              }

              if (key.pageDown) {
                moveSelection(rows);
                return true;
              }

              if (key.home) {
                setSelectedIdx(0);
                return true;
              }

              if (key.end) {
                setSelectedIdx(Math.max(searchResults.length - 1, 0));
                return true;
              }

              return false;
            }}
            placeholder="shortname, name, category, summary"
            focus
          />
        </Box>

        <Box minHeight={1}>
          {loading ? (
            <Text color={COLORS.warning}>
              <Spinner type="dots" /> Loading courses...
            </Text>
          ) : (
            <Text dimColor>
              {truncateText("Use ↑/↓, PgUp/PgDn, Home/End, Enter apply, Esc cancel.", Math.max(1, modalWidth - 4))}
            </Text>
          )}
        </Box>

        <Box flexDirection="column" flexGrow={1} overflow="hidden">
          {!loading && searchResults.length === 0 && (
            <Text dimColor>No courses found for this query.</Text>
          )}

          {!loading &&
            visibleResults.map((course, idx) => {
              const absoluteIdx = scrollOffset + idx;
              const selected = absoluteIdx === selectedIdx;
              return (
                <Text
                  key={course.id}
                  color={COLORS.neutral.white}
                  bold={selected}
                  backgroundColor={selected ? COLORS.panel.selected : undefined}
                >
                  {truncateText(course.fullname, resultLineWidth)}
                </Text>
              );
            })}
        </Box>

        <Box justifyContent="space-between">
          <Text dimColor>
            {searchResults.length > 0 ? `Showing ${visibleStart}-${visibleEnd}` : "Showing 0-0"}
          </Text>
          <Text dimColor>
            {searchResults.length > rows
              ? `Scroll ${scrollOffset}/${Math.max(searchResults.length - rows, 0)}`
              : " "}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}
