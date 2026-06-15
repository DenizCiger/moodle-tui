# Quiz AI Extension

Sample Moodle TUI plugin scaffold for quiz study help.

This plugin is intentionally limited to hints and explanations. It must not
choose, fill, save, or submit quiz answers.

## Gemini Structured Output Shape

Future Gemini calls should request `application/json` with this response schema:

```json
{
  "type": "object",
  "properties": {
    "summary": { "type": "string" },
    "hints": {
      "type": "array",
      "items": { "type": "string" }
    },
    "confidence": {
      "type": "string",
      "enum": ["low", "medium", "high"]
    },
    "limitations": {
      "type": "array",
      "items": { "type": "string" }
    }
  },
  "required": ["summary", "hints", "confidence", "limitations"]
}
```

The system instruction should state that the model provides study guidance only
and must not provide a selected answer, field value, or submission payload.
