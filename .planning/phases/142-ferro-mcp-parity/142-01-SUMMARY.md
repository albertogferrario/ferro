---
phase: 142-ferro-mcp-parity
plan: "01"
subsystem: ferro-mcp
tags: [introspection, stripe, webhook, regex, walkdir]
dependency_graph:
  requires: []
  provides: [updated-stripe-mcp-tools]
  affects: [ferro-mcp]
tech_stack:
  added: []
  patterns: [WalkDir recursive scan, dual-regex capture, Path::is_file capability checks]
key_files:
  modified:
    - ferro-mcp/src/tools/stripe.rs
decisions:
  - "Hard-remove WebhookEventInfo.listener field — no Option wrapper, feature branch"
  - "Use WalkDir over recursive fs::read_dir — idiomatic in codebase, already a dep"
  - "Keep scaffold_files listing flat (non-recursive) — use webhook_dir_exists for dir signal"
  - "Compile regexes once per function call — hot path is I/O, not regex compilation"
metrics:
  duration: "298s (~5 min)"
  completed: "2026-04-20"
  tasks: 3
  files: 1
---

# Phase 142 Plan 01: ferro-mcp Stripe Introspection Parity Summary

One-liner: Updated `stripe_webhook_events` to walk `src/` via WalkDir with closure + turbofish regexes, and extended `StripeConfigStatus` with four capability-axis boolean fields matching Phase 141's module layout.

## What Was Built

### WebhookEventInfo — updated struct shape

Old shape had `event_type: String`, `listener: String`, `file: String`. The `listener` field referred to a named struct (`impl Listener<EventType> for StructName`) which no longer exists in the SyncDispatcher closure-based API.

New shape:
```rust
pub struct WebhookEventInfo {
    pub event_type: String,
    pub file: String,
    pub line: u32,   // 1-based line number of the .on(...) call
}
```

### stripe_webhook_events — rewritten function body

Old implementation hard-coded `src/stripe/listeners.rs` and matched `impl Listener<EventType> for StructName`. After Phase 141 replaced listener structs with anonymous closures, this scanned the wrong file for the wrong pattern.

New implementation:
- Walks all `.rs` files under `src/` recursively via `WalkDir`
- Primary regex: `\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|` — matches closure form `.on(|event: EventType| ...)`
- Secondary regex: `\.on::<(\w+)` — matches turbofish form `.on::<EventType, _, _>(handler)`
- Line number formula: `(content[..byte_offset].lines().count() + 1) as u32` (1-based)
- Returns empty `events` vec when `src/` directory is absent

### StripeConfigStatus — four new capability-axis fields

```rust
pub checkout_exists: bool,    // src/stripe/checkout.rs
pub refund_exists: bool,      // src/stripe/refund.rs
pub account_exists: bool,     // src/stripe/account.rs
pub webhook_dir_exists: bool, // src/stripe/webhook/ directory
```

The `stripe_config_status` function adds four `Path::is_file()` / `Path::is_dir()` checks against `src/stripe/` after the existing `scaffold_files` computation. The `scaffold_files` listing remains flat (non-recursive `fs::read_dir` on `src/stripe/` only) — directories have no `.rs` extension so `webhook/` is naturally excluded.

## Tests

| Test | Status | Notes |
|------|--------|-------|
| test_config_status_scaffold_exists | pass | unchanged |
| test_config_status_scaffold_missing | pass | unchanged |
| test_config_status_serializes | pass | updated — added 4 new boolean fields to struct literal |
| test_config_status_keys_missing_when_not_configured | pass | unchanged |
| test_config_status_capability_axis_fields | pass | NEW (D-18) |
| test_webhook_events_not_found_returns_empty | pass | unchanged |
| test_webhook_events_parses_listeners | pass | updated (D-16) — closure fixture, asserts line > 0 |
| test_webhook_events_turbofish | pass | NEW (D-17) |
| test_webhook_events_serializes | pass | updated (D-19) — dropped listener field |
| test_subscription_info_no_migration | pass | unchanged |
| test_subscription_info_parses_migration | pass | unchanged |
| test_subscription_info_serializes | pass | unchanged |

**Test count: 10 → 12** (added test_webhook_events_turbofish, test_config_status_capability_axis_fields)

## Deviations from Plan

None — plan executed exactly as written. Tasks 1 and 2 were committed together as one atomic change since both operate on `ferro-mcp/src/tools/stripe.rs` and are structurally coupled (struct field removal immediately breaks existing test literals, requiring both struct and test updates in the same pass). Task 3 (quality gate) required no code changes.

## Quality Gate Results

- `cargo fmt --all -- --check`: clean
- `cargo clippy -p ferro-mcp --all-targets -- -D warnings`: clean (no warnings)
- `cargo test --manifest-path ferro-mcp/Cargo.toml`: 205 passed, 0 failed

## Known Stubs

None. All four capability-axis fields are wired to live `Path::is_file()` / `Path::is_dir()` checks. The webhook scanner returns real matches from source files.

## Threat Flags

None. This plan performs read-only static source analysis within `project_root`. No new network endpoints, auth paths, or trust boundary crossings introduced.

## Self-Check: PASSED

- `ferro-mcp/src/tools/stripe.rs` exists and is modified: FOUND
- Commit `3e0720d9` exists: FOUND
- `grep "pub line: u32" ferro-mcp/src/tools/stripe.rs`: FOUND (line 132)
- `grep "pub listener:" ferro-mcp/src/tools/stripe.rs`: 0 matches — CORRECT (field removed)
- `grep "pub checkout_exists: bool" ferro-mcp/src/tools/stripe.rs`: FOUND (line 32)
- `grep "WalkDir::new" ferro-mcp/src/tools/stripe.rs`: FOUND (line 155)
- 12 stripe tests pass: CONFIRMED
