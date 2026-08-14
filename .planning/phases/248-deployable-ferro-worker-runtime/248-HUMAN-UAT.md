---
status: partial
phase: 248-deployable-ferro-worker-runtime
source: [248-VERIFICATION.md]
started: "2026-08-14T19:28:20Z"
updated: "2026-08-14T19:28:20Z"
---

## Current Test

[awaiting human testing]

## Tests

### 1. OFFLOAD-05 multi-process runtime: exactly-once + fault-domain isolation across replicas
expected: |
  With the app binary built (`cargo build -p app`), run three processes concurrently:
    Shell A:  ./target/debug/app serve --no-worker      # web replica, no in-process worker
    Shell B:  ./target/debug/app worker --queue default # worker replica 1
    Shell C:  ./target/debug/app worker --queue default # worker replica 2
  Enqueue a batch of jobs on the "default" queue (e.g. via a seed handler or an offloaded
  service call), then confirm:
    (a) every job is processed EXACTLY ONCE — no job is claimed by both worker replicas;
    (b) `serve --no-worker` (Shell A) processes NO jobs itself (the in-process worker is off);
    (c) fault-domain isolation: a worker scoped to `--queue reports` never drains "default"
        jobs, and a saturated "media" queue does not starve a "reports" worker.
  This is the one behaviour that cannot be automated in-process (it requires N real OS
  processes over a shared DB); the in-process SC#1–SC#3 proxies are already GREEN and the
  DB-level exclusive-claim mechanism is proven by ferro-queue/tests/race_claim_sqlite.rs.
result: [pending]

## Summary

total: 1
passed: 0
issues: 0
pending: 1
skipped: 0
blocked: 0

## Gaps
