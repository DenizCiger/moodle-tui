#!/usr/bin/env node

const readline = require("node:readline");

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

function send(message) {
  process.stdout.write(`${JSON.stringify(message)}\n`);
}

function studyHelpPayload(context) {
  return {
    summary:
      "Review the question text and identify the concept being tested before choosing an answer.",
    hints: [
      "Look for keywords that map to a course concept.",
      "Eliminate options that describe a different role than the one asked about.",
    ],
    confidence: "low",
    limitations: [
      "This scaffold does not choose, fill, save, or submit quiz answers.",
      "Gemini calls are intentionally left for a future plugin implementation.",
    ],
    received_question: context?.question_text ?? "",
  };
}

rl.on("line", (line) => {
  let message;
  try {
    message = JSON.parse(line);
  } catch (error) {
    send({ type: "error", id: null, message: `invalid JSON: ${error.message}` });
    return;
  }

  if (message.type === "initialize") {
    send({
      type: "ok",
      id: null,
      payload: {
        protocol_version: 1,
        name: "Quiz AI Extension",
        capabilities: ["quiz_study_help"],
      },
    });
    return;
  }

  if (message.type === "invoke" && message.action === "study_help") {
    send({
      type: "ok",
      id: message.id ?? null,
      payload: studyHelpPayload(message.payload),
    });
    return;
  }

  send({
    type: "error",
    id: message.id ?? null,
    message: `unsupported message or action: ${message.type ?? "unknown"}`,
  });
});
