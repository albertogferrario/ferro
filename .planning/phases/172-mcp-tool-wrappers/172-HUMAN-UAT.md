---
status: partial
phase: 172-mcp-tool-wrappers
source: [172-VERIFICATION.md]
started: 2026-06-08T00:00:00Z
updated: 2026-06-08T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Live `ai_scaffold` via MCP
expected: Invoking the `ai_scaffold` MCP tool with a natural-language `description` against a running project returns a coherent `ferro_projections::ServiceDef` JSON object (referencing real models/fields where relevant) and writes NO files to disk.
result: [pending]

### 2. Live `ai_explain` structured branch
expected: Invoking `ai_explain` with a `target` that resolves to an existing `ServiceDef` returns structured projection JSON (`Intent`, `FieldMeaning`, `ActionDef`/`GuardDef`, `StateMachine`) with zero LLM token spend.
result: [pending]

### 3. Live `ai_explain` prose fallback
expected: Invoking `ai_explain` with a `target` (route/model) that has no backing `ServiceDef` returns a `{ "prose": "..." }` projection-framed explanation via the LLM.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
