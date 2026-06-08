---
status: partial
phase: 171-ferro-ai-make-ferro-ai-explain-cli-commands
source: [171-VERIFICATION.md]
started: 2026-06-08
updated: 2026-06-08
---

## Current Test

[awaiting human testing — requires a real LLM provider + a sample ferro project]

## Tests

### 1. Live ai:make quality (SC#2, SC#6)
expected: With `FERRO_AI_*` set, in a sample ferro app, `ferro ai:make "track customer orders with pending/paid/shipped states" --dry-run` prints a single ServiceDef whose fields use real `FieldMeaning` values (e.g. Status/Money/EntityName, not invented strings), includes a `state_machine` with pending/paid/shipped states, references models/fields that actually exist in the project (not generic templates), and writes no file (dry-run).
command: `ferro ai:make "track customer orders with pending/paid/shipped states" --dry-run`
result: [pending]

### 2. Live ai:explain quality (SC#4)
expected: `ferro ai:explain <existing-service>` returns projection-framed prose that names the service's Intents, identifies which fields' FieldMeanings drive rendering, describes the ActionDefs exposed under which GuardDefs, and covers any StateMachine transitions — referencing only what the service actually defines.
command: `ferro ai:explain <service-name>`
result: [pending]

### 3. (Optional) Cost guard
expected: `FERRO_AI_MAX_TOKENS_PER_COMMAND=256 ferro ai:explain <service-name>` produces a visibly shorter/truncated response than the default.
command: `FERRO_AI_MAX_TOKENS_PER_COMMAND=256 ferro ai:explain <service-name>`
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
