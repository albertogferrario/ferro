---
phase: 235-ferro-payments-webhook-sync-dispatcher-integration-and-auto
plan: "01"
subsystem: ferro-payments
tags: [billable-kind, cow, type-migration, webhook-foundation]
dependency_graph:
  requires: [234-03]
  provides: [BillableKind::from_string for Wave 3 handle_* methods]
  affects: [ferro-payments/src/lib.rs]
tech_stack:
  added: [std::borrow::Cow]
  patterns: [Cow<'static, str> for open-set string discriminators accepting both literals and runtime values]
key_files:
  modified: [ferro-payments/src/lib.rs]
decisions:
  - "as_str() return narrowed from &'static str to &str — zero caller breakage confirmed (all callers accept &str)"
  - "const fn new() retained unchanged — compile-time literal construction unchanged"
  - "from_string() is net-new — not yet called by any existing code (Wave 3 will use it)"
metrics:
  duration_seconds: 104
  completed_date: "2026-06-17"
  tasks_completed: 1
  tasks_total: 1
  files_modified: 1
---

# Phase 235 Plan 01: BillableKind Cow Migration Summary

`BillableKind` migrated from `&'static str` to `Cow<'static, str>` so it can be constructed from either compile-time literals (`new`) or runtime DB-read `String` values (`from_string`).

## What Was Built

`BillableKind(Cow<'static, str>)` in `ferro-payments/src/lib.rs` with three constructors/accessors:

- `pub const fn new(s: &'static str) -> Self` — unchanged caller contract, now wraps `Cow::Borrowed`
- `pub fn from_string(s: String) -> Self` — net-new; wraps `Cow::Owned`; used by Wave 3 `handle_*` methods to build kinds from `intent.billable_kind: String`
- `pub fn as_str(&self) -> &str` — narrowed return lifetime from `&'static str` to `&str`; all existing callers accept `&str`

## Verification

| Check | Result |
|-------|--------|
| `cargo check -p ferro-payments` | exit 0 |
| `cargo clippy -p ferro-payments --all-targets -- -D warnings` | exit 0, 0 warnings |
| `cargo test -p ferro-payments` | 23/23 passed |

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 — BillableKind Cow migration | `5eade44c` | feat(235-01): migrate BillableKind to Cow<'static, str> |

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `ferro-payments/src/lib.rs` contains `Cow<'static, str>` ✓
- `ferro-payments/src/lib.rs` contains `pub fn from_string(s: String) -> Self` ✓
- `ferro-payments/src/lib.rs` contains `pub const fn new(s: &'static str) -> Self` ✓
- `ferro-payments/src/lib.rs` contains `pub fn as_str(&self) -> &str` (no `'static`) ✓
- Commit `5eade44c` exists ✓
