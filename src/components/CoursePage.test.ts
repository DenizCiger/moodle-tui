import { describe, expect, it } from "bun:test";
import { buildCourseTreeRows, courseSectionNodeId } from "./CoursePage.tsx";
import type { MoodleCourseSection } from "../utils/moodle.ts";

const BASE_SECTIONS: MoodleCourseSection[] = [
  {
    id: 10,
    name: "Week 1",
    section: 1,
    summary: "<p>Intro summary</p>",
    visible: 1,
    modules: [
      {
        id: 55,
        name: "Forum discussion",
        modname: "forum",
        description: "<p>Say hello</p>",
        url: "https://moodle.school.tld/mod/forum/view.php?id=55",
        visible: 1,
        contents: [
          {
            type: "file",
            filename: "guide.pdf",
            fileurl: "https://moodle.school.tld/pluginfile.php/guide.pdf",
          },
        ],
      },
      {
        id: 56,
        name: "Banner",
        modname: "label",
        description: "<h3>Read this first</h3>",
        visible: 1,
        contents: [],
      },
    ],
  },
];

describe("course tree rows", () => {
  it("uses stable ids and collapses sections", () => {
    const rows = buildCourseTreeRows(BASE_SECTIONS, [courseSectionNodeId(10)]);

    expect(rows.length).toBe(1);
    expect(rows[0]?.id).toBe("section:10");
    expect(rows[0]?.expanded).toBe(false);
  });

  it("shows module and label rows when section is expanded", () => {
    const rows = buildCourseTreeRows(BASE_SECTIONS, []);

    expect(rows.some((row) => row.id === "module:10:55")).toBe(true);
    expect(rows.some((row) => row.id === "label:10:56")).toBe(true);
  });

  it("hides module children when module is collapsed", () => {
    const rows = buildCourseTreeRows(BASE_SECTIONS, ["module:10:55"]);

    expect(rows.some((row) => row.id === "module:10:55")).toBe(true);
    expect(rows.some((row) => row.id === "module-description:10:55")).toBe(false);
    expect(rows.some((row) => row.id === "module-url:10:55")).toBe(false);
    expect(rows.some((row) => row.id === "content:10:55:0")).toBe(false);
  });

  it("includes full children for expanded modules", () => {
    const rows = buildCourseTreeRows(BASE_SECTIONS, []);

    expect(rows.some((row) => row.id === "module-description:10:55")).toBe(true);
    expect(rows.some((row) => row.id === "module-url:10:55")).toBe(true);
    expect(rows.some((row) => row.id === "content:10:55:0")).toBe(true);

    expect(rows.find((row) => row.id === "module:10:55")?.linkUrl).toBe(
      "https://moodle.school.tld/mod/forum/view.php?id=55",
    );
    expect(rows.find((row) => row.id === "module-url:10:55")?.linkUrl).toBe(
      "https://moodle.school.tld/mod/forum/view.php?id=55",
    );
    expect(rows.find((row) => row.id === "content:10:55:0")?.linkUrl).toBe(
      "https://moodle.school.tld/pluginfile.php/guide.pdf",
    );
  });

  it("renders label modules as label leaf rows with plain text", () => {
    const rows = buildCourseTreeRows(BASE_SECTIONS, []);
    const label = rows.find((row) => row.id === "label:10:56");

    expect(label?.kind).toBe("label");
    expect(label?.text).toBe("Read this first");
    expect(label?.icon).toBe("🏷");
  });

  it("decodes HTML entities in section summaries and label text", () => {
    const rows = buildCourseTreeRows(
      [
        {
          id: 12,
          name: "Week 3",
          section: 3,
          summary: "<p>A &amp; B and &#67;</p>",
          modules: [
            {
              id: 200,
              name: "Label",
              modname: "label",
              description: "Tom &amp; Jerry",
              contents: [],
            },
          ],
        },
      ],
      [],
    );

    expect(rows.find((row) => row.id === "summary:12")?.text).toBe("A & B and C");
    expect(rows.find((row) => row.id === "label:12:200")?.text).toBe("Tom & Jerry");
  });

  it("maps module icons by module type with fallback", () => {
    const iconSections: MoodleCourseSection[] = [
      {
        id: 11,
        name: "Week 2",
        section: 2,
        summary: "",
        modules: [
          { id: 101, name: "Forum", modname: "forum", contents: [] },
          { id: 102, name: "Quiz", modname: "quiz", contents: [] },
          { id: 103, name: "Resource", modname: "resource", contents: [] },
          { id: 104, name: "Unknown", modname: "custommod", contents: [] },
          { id: 105, name: "Label", modname: "label", description: "Heads up", contents: [] },
        ],
      },
    ];

    const rows = buildCourseTreeRows(iconSections, []);

    expect(rows.find((row) => row.id === "module:11:101")?.icon).toBe("💬");
    expect(rows.find((row) => row.id === "module:11:102")?.icon).toBe("📝");
    expect(rows.find((row) => row.id === "module:11:103")?.icon).toBe("📄");
    expect(rows.find((row) => row.id === "module:11:104")?.icon).toBe("📦");
    expect(rows.find((row) => row.id === "label:11:105")?.icon).toBe("🏷");
    expect(rows.find((row) => row.id === "module:11:101")?.linkUrl).toBeUndefined();
  });
});
