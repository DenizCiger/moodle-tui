import {
  DEFAULT_MOODLE_SERVICE,
  type MoodleRuntimeConfig,
} from "./config.ts";

export interface MoodleTokenResponse {
  token?: string;
  error?: string;
  errorcode?: string;
  debuginfo?: string;
}

export interface MoodleCourse {
  id: number;
  shortname: string;
  fullname: string;
  displayname?: string;
  categoryid?: number;
  categoryname?: string;
  summary?: string;
  visible?: number;
  progress?: number | null;
  courseurl?: string;
}

export interface MoodleCourseModuleContentItem {
  type?: string;
  filename?: string;
  filepath?: string;
  filesize?: number;
  fileurl?: string;
  mimetype?: string;
  timemodified?: number;
  url?: string;
}

export interface MoodleCourseModule {
  id: number;
  name: string;
  modname?: string;
  description?: string;
  url?: string;
  visible?: number;
  contents: MoodleCourseModuleContentItem[];
}

export interface MoodleCourseSection {
  id: number;
  name?: string;
  section?: number;
  summary?: string;
  visible?: number;
  modules: MoodleCourseModule[];
}

export interface MoodleUpcomingAssignment {
  id: number;
  name: string;
  dueDate: number;
  courseId: number;
  courseShortName?: string;
  courseFullName?: string;
}

type JsonRecord = Record<string, unknown>;

function asRecord(payload: unknown): JsonRecord | null {
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) return null;
  return payload as JsonRecord;
}

function asString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function asNumber(value: unknown): number | undefined {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string" && value.trim().length > 0) {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) return parsed;
  }
  return undefined;
}

function toVisibleNumber(value: unknown): number | undefined {
  if (typeof value === "boolean") return value ? 1 : 0;
  return asNumber(value);
}

function toProgress(value: unknown): number | null | undefined {
  if (value === null) return null;
  return asNumber(value);
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

export function normalizeBaseUrl(rawBaseUrl: string): string {
  return rawBaseUrl.trim().replace(/\/+$/, "");
}

function buildTokenEndpoint(baseUrl: string): string {
  return `${normalizeBaseUrl(baseUrl)}/login/token.php`;
}

function buildRestEndpoint(baseUrl: string): string {
  return `${normalizeBaseUrl(baseUrl)}/webservice/rest/server.php`;
}

async function postForm(url: string, params: Record<string, string>): Promise<unknown> {
  const body = new URLSearchParams(params);
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body,
  });

  const rawText = await response.text();
  let parsed: unknown = null;
  try {
    parsed = JSON.parse(rawText);
  } catch {
    // JSON parse failure is handled below.
  }

  if (!response.ok) {
    const detail = typeof rawText === "string" && rawText.trim().length > 0 ? rawText : "no body";
    throw new Error(`HTTP ${response.status} while calling Moodle endpoint: ${detail}`);
  }

  if (parsed === null) {
    throw new Error("Moodle endpoint returned non-JSON response");
  }

  return parsed;
}

export function normalizeTokenResponse(payload: unknown): MoodleTokenResponse {
  const record = asRecord(payload);
  if (!record) return {};

  return {
    token: asString(record.token),
    error: asString(record.error),
    errorcode: asString(record.errorcode),
    debuginfo: asString(record.debuginfo),
  };
}

function extractMoodleException(payload: unknown): string | null {
  const record = asRecord(payload);
  if (!record) return null;

  const message = asString(record.message);
  const errorcode = asString(record.errorcode);
  const exception = asString(record.exception);
  const debuginfo = asString(record.debuginfo);

  const hasErrorSignal = Boolean(message || errorcode || exception);
  if (!hasErrorSignal) return null;

  const fragments = [
    message ? `message=${message}` : "",
    errorcode ? `errorcode=${errorcode}` : "",
    exception ? `exception=${exception}` : "",
    debuginfo ? `debuginfo=${debuginfo}` : "",
  ].filter(Boolean);

  return fragments.length > 0 ? fragments.join(" | ") : "Moodle returned an unknown error";
}

async function callMoodleWebservice(
  config: MoodleRuntimeConfig,
  token: string,
  wsfunction: string,
  params: Record<string, string>,
): Promise<unknown> {
  const payload = await postForm(buildRestEndpoint(config.baseUrl), {
    wstoken: token,
    wsfunction,
    moodlewsrestformat: "json",
    ...params,
  });

  const exception = extractMoodleException(payload);
  if (exception) {
    throw new Error(exception);
  }

  return payload;
}

export async function requestToken(config: MoodleRuntimeConfig): Promise<string> {
  const service =
    config.service.trim().length > 0 ? config.service.trim() : DEFAULT_MOODLE_SERVICE;

  const payload = await postForm(buildTokenEndpoint(config.baseUrl), {
    username: config.username,
    password: config.password,
    service,
  });

  const tokenResponse = normalizeTokenResponse(payload);
  if (tokenResponse.token) return tokenResponse.token;

  const reason = [tokenResponse.error, tokenResponse.errorcode, tokenResponse.debuginfo]
    .filter(Boolean)
    .join(" | ");
  throw new Error(reason || "Token request failed");
}

export async function testCredentials(
  config: MoodleRuntimeConfig,
): Promise<{ ok: true } | { ok: false; message: string }> {
  try {
    await requestToken(config);
    return { ok: true };
  } catch (error) {
    return { ok: false, message: getErrorMessage(error) };
  }
}

function normalizeCourseRecord(record: JsonRecord): MoodleCourse | null {
  const id = asNumber(record.id);
  const shortname = asString(record.shortname);
  const fullname = asString(record.fullname);
  if (id === undefined || !shortname || !fullname) return null;

  return {
    id,
    shortname,
    fullname,
    displayname: asString(record.displayname),
    categoryid: asNumber(record.categoryid),
    categoryname: asString(record.categoryname),
    summary: asString(record.summary),
    visible: toVisibleNumber(record.visible),
    progress: toProgress(record.progress),
    courseurl: asString(record.courseurl) ?? asString(record.viewurl),
  };
}

export function normalizeCourse(payload: unknown): MoodleCourse | null {
  const record = asRecord(payload);
  if (!record) return null;
  return normalizeCourseRecord(record);
}

function normalizeCourseModuleContentItem(
  payload: unknown,
): MoodleCourseModuleContentItem | null {
  const record = asRecord(payload);
  if (!record) return null;

  return {
    type: asString(record.type),
    filename: asString(record.filename),
    filepath: asString(record.filepath),
    filesize: asNumber(record.filesize),
    fileurl: asString(record.fileurl),
    mimetype: asString(record.mimetype),
    timemodified: asNumber(record.timemodified),
    url: asString(record.url),
  };
}

function normalizeCourseModuleRecord(record: JsonRecord): MoodleCourseModule | null {
  const id = asNumber(record.id);
  const name = asString(record.name);
  if (id === undefined || !name) return null;

  const contents = Array.isArray(record.contents)
    ? record.contents
        .map((item) => normalizeCourseModuleContentItem(item))
        .filter((item): item is MoodleCourseModuleContentItem => Boolean(item))
    : [];

  return {
    id,
    name,
    modname: asString(record.modname),
    description: asString(record.description),
    url: asString(record.url),
    visible: toVisibleNumber(record.visible),
    contents,
  };
}

function normalizeCourseSectionRecord(record: JsonRecord): MoodleCourseSection | null {
  const id = asNumber(record.id);
  if (id === undefined) return null;

  const modules = Array.isArray(record.modules)
    ? record.modules
        .map((item) => asRecord(item))
        .filter((item): item is JsonRecord => Boolean(item))
        .map((item) => normalizeCourseModuleRecord(item))
        .filter((item): item is MoodleCourseModule => Boolean(item))
    : [];

  return {
    id,
    name: asString(record.name),
    section: asNumber(record.section),
    summary: asString(record.summary),
    visible: toVisibleNumber(record.visible),
    modules,
  };
}

export function normalizeCourseSection(payload: unknown): MoodleCourseSection | null {
  const record = asRecord(payload);
  if (!record) return null;
  return normalizeCourseSectionRecord(record);
}

export function normalizeUpcomingAssignments(
  payload: unknown,
  nowTimestamp: number,
): MoodleUpcomingAssignment[] {
  const record = asRecord(payload);
  if (!record) return [];

  const courses = Array.isArray(record.courses) ? record.courses : [];
  const upcoming: MoodleUpcomingAssignment[] = [];

  for (const rawCourse of courses) {
    const courseRecord = asRecord(rawCourse);
    if (!courseRecord) continue;

    const courseId = asNumber(courseRecord.id);
    if (courseId === undefined) continue;

    const courseShortName = asString(courseRecord.shortname);
    const courseFullName = asString(courseRecord.fullname);
    const assignments = Array.isArray(courseRecord.assignments) ? courseRecord.assignments : [];

    for (const rawAssignment of assignments) {
      const assignmentRecord = asRecord(rawAssignment);
      if (!assignmentRecord) continue;

      const id = asNumber(assignmentRecord.id);
      const name = asString(assignmentRecord.name);
      const dueDate = asNumber(assignmentRecord.duedate);

      if (id === undefined || !name || dueDate === undefined || dueDate <= 0) {
        continue;
      }

      if (dueDate < nowTimestamp) {
        continue;
      }

      upcoming.push({
        id,
        name,
        dueDate,
        courseId,
        courseShortName,
        courseFullName,
      });
    }
  }

  upcoming.sort((left, right) => {
    if (left.dueDate !== right.dueDate) return left.dueDate - right.dueDate;

    const byCourse = (left.courseFullName || left.courseShortName || "").localeCompare(
      right.courseFullName || right.courseShortName || "",
      undefined,
      { sensitivity: "base" },
    );
    if (byCourse !== 0) return byCourse;

    const byName = left.name.localeCompare(right.name, undefined, { sensitivity: "base" });
    if (byName !== 0) return byName;

    return left.id - right.id;
  });

  return upcoming;
}

function extractUserId(payload: unknown): number | null {
  const record = asRecord(payload);
  if (!record) return null;
  const userId = asNumber(record.userid);
  return userId ?? null;
}

export async function fetchCourses(config: MoodleRuntimeConfig): Promise<MoodleCourse[]> {
  const token = await requestToken(config);

  const siteInfo = await callMoodleWebservice(
    config,
    token,
    "core_webservice_get_site_info",
    {},
  );
  const userId = extractUserId(siteInfo);
  if (userId === null) {
    throw new Error("Could not resolve current user id from Moodle site info");
  }

  const rawCourses = await callMoodleWebservice(
    config,
    token,
    "core_enrol_get_users_courses",
    { userid: String(userId) },
  );

  if (!Array.isArray(rawCourses)) {
    throw new Error("Unexpected Moodle response for enrolled courses");
  }

  const courses = rawCourses
    .map((entry) => asRecord(entry))
    .filter((entry): entry is JsonRecord => Boolean(entry))
    .map((entry) => normalizeCourseRecord(entry))
    .filter((entry): entry is MoodleCourse => Boolean(entry))
    .sort((left, right) => left.fullname.localeCompare(right.fullname, undefined, { sensitivity: "base" }));

  return courses;
}

export async function fetchCourseContents(
  config: MoodleRuntimeConfig,
  courseId: number,
): Promise<MoodleCourseSection[]> {
  const token = await requestToken(config);

  const rawSections = await callMoodleWebservice(
    config,
    token,
    "core_course_get_contents",
    { courseid: String(courseId) },
  );

  if (!Array.isArray(rawSections)) {
    throw new Error("Unexpected Moodle response for course contents");
  }

  return rawSections
    .map((entry) => asRecord(entry))
    .filter((entry): entry is JsonRecord => Boolean(entry))
    .map((entry) => normalizeCourseSectionRecord(entry))
    .filter((entry): entry is MoodleCourseSection => Boolean(entry));
}

export async function fetchUpcomingAssignments(
  config: MoodleRuntimeConfig,
  nowTimestamp = Math.floor(Date.now() / 1000),
): Promise<MoodleUpcomingAssignment[]> {
  const token = await requestToken(config);

  const payload = await callMoodleWebservice(
    config,
    token,
    "mod_assign_get_assignments",
    {},
  );

  return normalizeUpcomingAssignments(payload, nowTimestamp);
}
