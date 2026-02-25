import { describe, expect, it } from "bun:test";
import { buildSecretAccountKey } from "./secret.ts";

describe("secret account key", () => {
  it("uses baseUrl|username|service", () => {
    const key = buildSecretAccountKey({
      baseUrl: "https://moodle.school.tld",
      username: "student1",
      service: "moodle_mobile_app",
    });

    expect(key).toBe("https://moodle.school.tld|student1|moodle_mobile_app");
  });
});
