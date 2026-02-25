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
