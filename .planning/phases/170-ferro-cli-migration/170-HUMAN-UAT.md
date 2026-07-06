---
status: partial
phase: 170-ferro-cli-migration
source: [170-VERIFICATION.md]
started: 2026-06-08
updated: 2026-06-08
---

## Current Test

[awaiting human testing — requires a live LLM provider API key]

## Tests

### 1. Live two-pass AI generation against a real provider (SC#3 end-to-end)
expected: With a provider configured (e.g. `export FERRO_AI_PROVIDER=anthropic FERRO_AI_API_KEY=sk-ant-...`), running `ferro make:json-view test_view --description "A simple product listing view"` completes both passes and writes `src/views/test_view.json` containing a catalog-valid JSON-UI spec, with NO static-template fallback triggered.
result: [pending]

### 2. Provider-agnosticism (SC#4)
expected: Repeating test 1 with `FERRO_AI_PROVIDER=openai` (and the matching `FERRO_AI_API_KEY`) also produces a catalog-valid spec — proving the migration removed the Anthropic-only constraint.
result: [pending]

### 3. Static fallback when unconfigured (SC#3 fallback path)
expected: With all `FERRO_AI_*` and `ANTHROPIC_API_KEY` unset, `ferro make:json-view foo --description "x"` prints the informational message and writes a valid static-template spec (no crash).
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps

(none — all automated criteria passed; these are live-provider smoke tests that require a human-supplied API key and cannot be run in CI)
