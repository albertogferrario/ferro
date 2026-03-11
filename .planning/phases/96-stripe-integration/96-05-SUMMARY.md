---
phase: 96-stripe-integration
plan: "05"
subsystem: cli-tooling
tags: [stripe, cli, mcp, scaffolding, introspection]
dependency_graph:
  requires: [96-01, 96-02, 96-03]
  provides: [ferro-make-stripe-cli, mcp-stripe-tools]
  affects: [ferro-cli, ferro-mcp]
tech_stack:
  added: []
  patterns:
    - write-if-not-exists file generation (same as make_auth)
    - regex source scanning for MCP tool introspection
    - SQL parsing from Rust migration source
key_files:
  created:
    - ferro-cli/src/commands/make_stripe.rs
    - ferro-mcp/src/tools/stripe.rs
  modified:
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
    - ferro-mcp/src/tools/mod.rs
    - ferro-mcp/src/service.rs
decisions:
  - Generated webhook handlers dispatch via ferro_queue::dispatch_job (not inline ferro-events) per Phase 96-03 locked decision
  - write_if_not_exists prevents overwriting user-modified scaffold files
  - MCP tools scan source files via regex (same pattern as list_projections, list_events)
  - stripe_subscription_info parses SQL from Rust migration source using execute_unprepared() regex extraction
  - Optional keys (STRIPE_CONNECT_WEBHOOK_SECRET, etc.) tracked as present but not as missing
metrics:
  duration: "13 minutes"
  completed_date: "2026-03-11"
  tasks_completed: 2
  files_changed: 6
---

# Phase 96 Plan 05: CLI Scaffolding and MCP Introspection for Stripe Summary

CLI `ferro make:stripe` command scaffolds full Stripe integration; 3 MCP tools provide configuration status, event listener discovery, and billing table introspection.

## Tasks Completed

### Task 1: ferro make:stripe CLI Command

Created `ferro-cli/src/commands/make_stripe.rs` with `execute(connect: bool)`:

- `src/stripe/mod.rs` — Stripe::init() bootstrap with StripeConfig::from_env()
- `src/stripe/webhook.rs` — inline sig verification + `dispatch_job(ProcessStripeWebhook::platform(...))` per locked decision
- `src/stripe/listeners.rs` — SyncSubscriptionPlan, HandleSubscriptionDeleted, HandleCheckoutCompleted stubs
- `src/stripe/connect_webhook.rs` — only with `--connect` flag, dispatches `ProcessStripeWebhook::connect(...)`
- Migration file in `src/migrations/` for `tenant_billing` table with correct schema and index
- Env var hints printed to stdout
- `write_if_not_exists` prevents overwriting existing files (idempotent)

Registered command in `ferro-cli/src/commands/mod.rs` and `ferro-cli/src/main.rs` as `make:stripe` with `--connect` flag.

13 unit tests covering: template content, dispatch_job usage, connect flag, file generation, idempotency.

### Task 2: MCP Stripe Introspection Tools

Created `ferro-mcp/src/tools/stripe.rs` with 3 functions:

**`stripe_config_status`**: Loads `.env`, checks STRIPE_SECRET_KEY, STRIPE_WEBHOOK_SECRET, STRIPE_PUBLISHABLE_KEY presence, scans `src/stripe/` for scaffold files. Returns `{ configured, keys_present, keys_missing, scaffold_exists, scaffold_files }`.

**`stripe_webhook_events`**: Scans `src/stripe/listeners.rs` via regex `impl Listener<EventType> for StructName`. Returns `{ events: [{ event_type, listener, file }] }`.

**`stripe_subscription_info`**: Scans migration files for `*tenant_billing*.rs`, extracts SQL from `execute_unprepared(...)`, parses CREATE TABLE columns and CREATE INDEX statements. Returns `{ table_exists, migration_file, columns, indexes }`.

All 3 registered in `ferro-mcp/src/service.rs` via `#[tool_router]` with descriptive `#[tool(name, description)]` attributes.

10 unit tests covering all three tools across scaffold-present/absent, listener parsing, migration SQL parsing, and serialization.

## Commits

- `e921003` — feat(96-05): add ferro make:stripe CLI scaffolding command
- `b47ebf8` — feat(96-05): add MCP Stripe introspection tools

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- ferro-cli/src/commands/make_stripe.rs: FOUND
- ferro-mcp/src/tools/stripe.rs: FOUND
- Commit e921003: FOUND
- Commit b47ebf8: FOUND
