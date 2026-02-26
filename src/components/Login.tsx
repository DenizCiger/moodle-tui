import React, { useState } from "react";
import { Box, Text, useInput, useStdout } from "ink";
import Spinner from "ink-spinner";
import TextInput from "ink-text-input";
import { COLORS } from "./colors.ts";
import { truncateText } from "./timetable/text.ts";
import {
  DEFAULT_MOODLE_SERVICE,
  type MoodleRuntimeConfig,
  type MoodleSavedConfig,
} from "../utils/config.ts";
import { normalizeBaseUrl, testCredentials } from "../utils/moodle.ts";

interface LoginProps {
  onLogin: (config: MoodleRuntimeConfig) => Promise<void> | void;
  initialConfig?: MoodleSavedConfig | null;
  error?: string;
  secureStorageNotice?: string;
}

type Field = "baseUrl" | "username" | "password" | "service";

const FIELDS: { key: Field; label: string; placeholder: string }[] = [
  {
    key: "baseUrl",
    label: "Base URL",
    placeholder: "https://moodle.example.org",
  },
  {
    key: "username",
    label: "Username",
    placeholder: "Your Moodle username",
  },
  {
    key: "password",
    label: "Password",
    placeholder: "Your Moodle password",
  },
  {
    key: "service",
    label: "Service",
    placeholder: DEFAULT_MOODLE_SERVICE,
  },
];

export default function Login({
  onLogin,
  initialConfig,
  error: appError,
  secureStorageNotice,
}: LoginProps) {
  const { stdout } = useStdout();
  const [values, setValues] = useState<Record<Field, string>>({
    baseUrl: initialConfig?.baseUrl || "",
    username: initialConfig?.username || "",
    password: "",
    service: initialConfig?.service || DEFAULT_MOODLE_SERVICE,
  });
  const [activeField, setActiveField] = useState(0);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const termWidth = stdout?.columns && stdout.columns > 0 ? stdout.columns : 120;
  const contentWidth = Math.max(1, termWidth - 2);
  const labelWidth = Math.max(
    1,
    Math.min(Math.max(4, Math.floor(contentWidth * 0.24)), Math.max(1, contentWidth - 3)),
  );
  const valueWidth = Math.max(0, contentWidth - labelWidth - 3);

  useInput(
    (input, key) => {
      if (loading) return;

      if (key.tab && key.shift) {
        setActiveField((prev) => Math.max(0, prev - 1));
        return;
      }
      if (key.tab) {
        setActiveField((prev) => Math.min(FIELDS.length - 1, prev + 1));
        return;
      }
      if (key.upArrow) {
        setActiveField((prev) => Math.max(0, prev - 1));
        return;
      }
      if (key.downArrow) {
        setActiveField((prev) => Math.min(FIELDS.length - 1, prev + 1));
        return;
      }
      if (input === "v") {
        setShowPassword((prev) => !prev);
      }
    },
    { isActive: Boolean(process.stdin.isTTY) },
  );

  const handleSubmit = async () => {
    const config: MoodleRuntimeConfig = {
      baseUrl: normalizeBaseUrl(values.baseUrl),
      username: values.username.trim(),
      password: values.password,
      service: values.service.trim() || DEFAULT_MOODLE_SERVICE,
    };

    if (!config.baseUrl || !config.username || !config.password) {
      setError("Base URL, username, and password are required.");
      return;
    }

    setLoading(true);
    setError("");
    const result = await testCredentials(config);
    if (!result.ok) {
      setError(`Login failed: ${result.message}`);
      setLoading(false);
      return;
    }

    await onLogin(config);
    setLoading(false);
  };

  return (
    <Box flexDirection="column" width={termWidth} padding={1}>
      <Box marginBottom={1}>
        <Text bold color={COLORS.brand}>
          {truncateText("Moodle TUI - Login", contentWidth)}
        </Text>
      </Box>

      <Box marginBottom={1}>
        <Text dimColor>
          {truncateText("Enter Moodle credentials. Use arrows or Tab to change focus.", contentWidth)}
        </Text>
      </Box>

      <Box marginBottom={1}>
        <Text dimColor>
          {truncateText("Password is stored securely via your OS credentials store.", contentWidth)}
        </Text>
      </Box>

      {FIELDS.map((field, index) => (
        <Box key={field.key} width={contentWidth}>
          <Box width={labelWidth}>
            <Text
              color={index === activeField ? COLORS.brand : COLORS.neutral.white}
              bold={index === activeField}
            >
              {index === activeField ? "> " : "  "}
              {truncateText(`${field.label}:`, Math.max(1, labelWidth - 2))}
            </Text>
          </Box>
          <Box marginLeft={1} width={valueWidth}>
            {index === activeField && !loading ? (
              <TextInput
                value={values[field.key]}
                onChange={(value) => setValues((prev) => ({ ...prev, [field.key]: value }))}
                onSubmit={() => {
                  if (activeField < FIELDS.length - 1) {
                    setActiveField(activeField + 1);
                  } else {
                    void handleSubmit();
                  }
                }}
                placeholder={field.placeholder}
                mask={field.key === "password" && !showPassword ? "*" : undefined}
                focus
              />
            ) : (
              <Text dimColor={index !== activeField}>
                {truncateText(
                  field.key === "password"
                    ? showPassword
                      ? values.password || field.placeholder
                      : "*".repeat(values.password.length) || field.placeholder
                    : values[field.key] || field.placeholder,
                  valueWidth,
                )}
              </Text>
            )}
          </Box>
        </Box>
      ))}

      {loading && (
        <Box marginTop={1}>
          <Text color={COLORS.warning}>
            <Spinner type="dots" />
          </Text>
          <Text color={COLORS.warning}> Authenticating...</Text>
        </Box>
      )}

      {(appError || error) && (
        <Box marginTop={1}>
          <Text color={COLORS.error}>{truncateText(appError || error, contentWidth)}</Text>
        </Box>
      )}

      {secureStorageNotice && (
        <Box marginTop={1}>
          <Text color={COLORS.warning}>{truncateText(secureStorageNotice, contentWidth)}</Text>
        </Box>
      )}

      {!loading && (
        <Box marginTop={1}>
          <Text dimColor>
            {truncateText(
              "Enter next/submit | Tab move focus | v toggle password visibility",
              contentWidth,
            )}
          </Text>
        </Box>
      )}
    </Box>
  );
}
