# Question Contract Examples (Phase B)

Reference payloads for b.7–b.8. Wyvern keeps `type: "question"` envelope; inner fields follow public Claude AskUserQuestion semantics (ADR-0007, NFR-0009).

## Minimal single-select

**Input:**

```json
{
  "type": "question",
  "questions": [
    {
      "question": "Output format?",
      "header": "Format",
      "options": [
        { "label": "JSON", "description": "Structured" },
        { "label": "Plain", "description": "Text only" }
      ],
      "multiSelect": false
    }
  ]
}
```

**Stdout (normal completion):**

```json
{
  "questions": [
    {
      "question": "Output format?",
      "header": "Format",
      "options": [
        { "label": "JSON", "description": "Structured" },
        { "label": "Plain", "description": "Text only" }
      ],
      "multiSelect": false
    }
  ],
  "answers": { "Output format?": "JSON" },
  "response": ""
}
```

## Multi-select (comma-joined labels per REQ-0062)

**Stdout:**

```json
{
  "answers": { "Pick tools": "JSON, Plain" },
  "questions": [ "...verbatim input..." ],
  "response": ""
}
```

If a label contains a comma, use the label text exactly as provided in `options[].label`; comma-join is only used between labels, not within labels.

## Force close (Wyvern extension — REQ-0068, NFR-0009)

**Stdout:**

```json
{
  "button": "dismissed",
  "questions": [ "...verbatim input..." ],
  "answers": {},
  "response": ""
}
```

This `button` field is **not** present on normal completion. Document in b.8 acceptance tests.

## With preview (b.8)

**Input option:**

```json
{
  "label": "JSON",
  "description": "Structured output",
  "preview": "<pre>{\"ok\":true}</pre>"
}
```

Preview renders as HTML fragment beside the option; markdown fragments are converted to HTML at render time.

## Validation failures (examples)

| Case | Field | Message shape |
|------|-------|---------------|
| 0 questions | `questions` | empty array not allowed |
| 5 questions | `questions` | max 4 per REQ-0062 |
| 1 option | `questions[0].options` | min 2 options |
| header 13 chars | `questions[0].header` | max 12 characters |
