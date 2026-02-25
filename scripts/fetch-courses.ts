import { fetchCourses } from "../src/utils/moodle.ts";
import { loadConfig } from "../src/utils/config.ts";
import { loadPassword } from "../src/utils/secret.ts";

async function main() {
  const saved = loadConfig();
  if (!saved) {
    console.error("No saved Moodle config found. Log in via the TUI first.");
    process.exit(1);
  }

  const password = await loadPassword(saved);
  if (!password) {
    console.error("No saved Moodle password found. Log in via the TUI first.");
    process.exit(1);
  }

  const courses = await fetchCourses({
    ...saved,
    password,
  });

  console.log(`Fetched ${courses.length} enrolled course(s).\n`);
  for (const course of courses) {
    console.log(
      `- [${course.id}] ${course.shortname} :: ${course.fullname}${course.courseurl ? ` (${course.courseurl})` : ""}`,
    );
  }
}

void main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`Failed to fetch courses: ${message}`);
  process.exit(1);
});
