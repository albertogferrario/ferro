---
status: partial
phase: 185-ferro-queue-db-backed-job-queue
source: [185-VERIFICATION.md]
started: 2026-06-07T00:00:00Z
updated: 2026-06-07T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. WorkerLoop auto-start inside the app server

expected: Register a job type (`Queue::register::<J>()` in bootstrap), run the app binary (`cargo run` in `app/` or a consumer app), dispatch a job, observe it executing with no separate worker process. Startup log shows the WorkerLoop spawning.
result: [pending]

### 2. SIGTERM graceful drain

expected: With a job in-flight, send SIGTERM to the app process. The loop stops claiming, the in-flight job drains to completion (or its `failed()` hook), claimed-but-unstarted rows reset to `pending`, process exits cleanly.
result: [pending]

### 3. Worker-death reap after visibility timeout

expected: Kill a worker mid-job (SIGKILL), wait past the visibility timeout (default 300s), start a new worker — the stuck job is re-queued by the reaper with `attempts` incremented and executes.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
