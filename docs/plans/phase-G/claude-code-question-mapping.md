# Claude Code AskUserQuestion → Wyvern mapping

Phase B pointer for the `question` envelope. **g.5 scope, fixtures, and `--invoke` I/O live only in** [g5-askuserquestion-claude-code.md](g5-askuserquestion-claude-code.md). This file is not a second checklist.

## Envelope

Wyvern wraps Claude's question payload in the standard command envelope:

```json
{
  "type": "question",
  "questions": [ ]
}
```

Inner field names and semantics match the public Claude **AskUserQuestion** tool (NFR-0009).

## Response (stdout)

Normal completion — same shape agents expect from AskUserQuestion:

```json
{
  "questions": [ ],
  "answers": { "<question text>": "<selected label(s)>" },
  "response": ""
}
```

Multi-select: comma-joined labels per REQ-0062.

## Wyvern-only extensions

| Case | Wyvern behavior |
|------|-----------------|
| Force close / OS dismiss | `button: "dismissed"` in result path (REQ-0068) |
| Headless CI | `--viewer none`; still blocking until HTTP dismiss |

## Agent integration (Claude Code)

**Pattern A — hook install (g.5 wizard chain):**

Configure **Globally** / **Repo (local)** enable switches in the askuserquestion-hook wizard. The g.4 CLI **post** runner applies hooks (REQ-0125). Do not require a manual `python3 apply-askuserquestion-hook.py` pipe.

See [claude-code-hook-workflow.md](claude-code-hook-workflow.md).

**Pattern B — direct invoke (agents/scripts):**

```bash
RESULT=$(wyvern '{"type":"question","questions":[...]}')
# parse .answers from RESULT
```

**Not in g.5 scope:** MCP wrapper (Phase E), patching Claude Code tool routing.

## Fixtures

| File | Scenario |
|------|----------|
| `share/wyvern/examples/question/minimal.json` | Single-select |
| `share/wyvern/examples/question/multi-select.json` | Multi-select |
| `share/wyvern/examples/question/with-preview.json` | Preview field |

See also [question-contract-examples.md](../phase-B/question-contract-examples.md).
