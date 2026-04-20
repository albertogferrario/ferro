# Phase 142: ferro-mcp Parity - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Update the three ferro-mcp Stripe introspection tools (`stripe_webhook_events`, `stripe_config_status`, `stripe_subscription_info`) to reflect Phase 141's SyncDispatcher architecture. The old listener model (`impl Listener<EventType> for StructName` in `src/stripe/listeners.rs`) is gone; handlers are now anonymous closures registered via `SyncDispatcher::on(|event: EventType| ...)`. Scaffold detection must match the capability-axis module tree (`checkout.rs`, `refund.rs`, `account.rs`, `webhook/`). Bump workspace version.

This phase does NOT change ferro-stripe or any framework crate — only ferro-mcp.

</domain>

<decisions>
## Implementation Decisions

### stripe_webhook_events — Scan Pattern

- **D-01:** Drop the hard-coded `src/stripe/listeners.rs` path. Instead, walk all `.rs` files under `src/` (project root).
- **D-02:** Use regex `\.on\(\s*\|[a-zA-Z_]+:\s*(\w+)\s*\|` to match the canonical closure form `.on(|event: EventType| ...)`. Also match turbofish `.on::<(\w+)` as a secondary pattern for callers that use that form explicitly.
- **D-03:** `WebhookEventInfo` struct changes:
  - **Remove** `listener: String` — there is no named struct in the closure-based API.
  - **Keep** `event_type: String` and `file: String`.
  - **Add** `line: u32` — the 1-based line number of the `.on(...)` call in the source file.
- **D-04:** The `StripeWebhookEvents` wrapper struct (`pub events: Vec<WebhookEventInfo>`) is unchanged.
- **D-05:** If no matches are found across any file, return empty `events` vec (same behavior as before when listeners.rs was absent).

### stripe_config_status — Scaffold Detection

- **D-06:** Continue scanning `src/stripe/` (project-root-relative). The user's app stripe module remains at this path by convention.
- **D-07:** Add four boolean fields to `StripeConfigStatus`:
  - `checkout_exists: bool` — `src/stripe/checkout.rs` present
  - `refund_exists: bool` — `src/stripe/refund.rs` present
  - `account_exists: bool` — `src/stripe/account.rs` present
  - `webhook_dir_exists: bool` — `src/stripe/webhook/` directory present
- **D-08:** Existing `scaffold_exists: bool` and `scaffold_files: Vec<String>` remain — `scaffold_exists` stays `src/stripe/` directory check, `scaffold_files` continues to list all `.rs` files in that directory (non-recursive). The four new booleans give structured capability-axis coverage on top.
- **D-09:** Update the `stripe_config_status` MCP tool description to mention capability-axis layout detection.

### stripe_subscription_info — Keep As-Is (behavior only)

- **D-10:** Retire no logic. The `tenant_billing` migration scanner remains valid — apps that use subscriptions still create this table; the tool gives them schema introspection.
- **D-11:** Update the MCP tool description to clarify the tool scans for app-level billing table migrations, not the ferro-stripe framework subscription module (which no longer exists as a named axis after Phase 141).
- **D-12:** No struct changes to `StripeSubscriptionInfo`, `ColumnInfo`, or `IndexInfo`.

### MCP Tool Descriptions

- **D-13:** `stripe_webhook_events` description updated from "discovered in src/stripe/listeners.rs" to describe SyncDispatcher handler scanning. New description example:
  > "Scan project source for SyncDispatcher webhook handler registrations. Returns event types and file locations for all `.on(|event: EventType| ...)` calls found in `src/`."
- **D-14:** `stripe_config_status` description updated to mention capability-axis layout fields.
- **D-15:** `stripe_subscription_info` description updated to clarify app-level billing table, not framework module.

### Tests

- **D-16:** Update `test_webhook_events_parses_listeners` fixture to use SyncDispatcher closure syntax instead of `impl Listener<EventType> for StructName`. The test should now write a file with `.on(|event: StripeSubscriptionUpdated| ...)` and assert the correct `event_type` and `line` values.
- **D-17:** Add a test for the turbofish pattern `.on::<StripeCheckoutCompleted, _, _>(handler)`.
- **D-18:** Add `test_config_status_capability_axis_fields` — create `src/stripe/checkout.rs` + `src/stripe/webhook/` dir, assert the four boolean fields are set correctly.
- **D-19:** Remove or update the old `test_webhook_events_serializes` test to drop the `listener` field reference.

### Version Bump

- **D-20:** Workspace version 0.2.2 → 0.2.3 (patch — existing crates updated, no new crates).

### Claude's Discretion

- Whether to keep a `listener: Option<String>` for backward compatibility or hard-remove it. Recommended: hard-remove (feature branch, no backward compat needed).
- Exact file-walk implementation (walkdir crate already in workspace or glob-style `fs::read_dir` recursion) — planner picks.
- Whether `stripe_config_status` `scaffold_files` listing becomes recursive into `webhook/` or stays flat. Recommended: keep flat (existing behavior), use `webhook_dir_exists` for the directory signal.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Doc (authoritative for Phase 141 output, which this phase must match)
- `.planning/research/v11.6-FERRO-STRIPE-REFACTOR.md` — §3.3 event struct field sets, §3.4 SyncDispatcher API, §3.6 queue path. Phase 142 introspects the patterns this doc defines.

### Phase 141 Output
- `.planning/phases/141-protocol-uplift/141-CONTEXT.md` — locked decisions for Phase 141. Phase 142 MCP tools must reflect these decisions, not any earlier architecture.
- `ferro-stripe/src/webhook/sync.rs` — SyncDispatcher implementation; the `.on(|event: EventType| ...)` closure form in its doc comment is the canonical registration pattern to match.
- `ferro-stripe/src/webhook/events.rs` — all ten StripeEvent types (event_type strings); these are the values `stripe_webhook_events` should surface.

### Current ferro-mcp Source (read before touching)
- `ferro-mcp/src/tools/stripe.rs` — all three tool implementations + tests to update
- `ferro-mcp/src/service.rs` lines 1542-1595 — MCP tool registrations and descriptions to update

### Roadmap
- `.planning/ROADMAP.md` §"Phase 142: ferro-mcp parity" — six success criteria; use as checklist for verification.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `stripe_config_status` env-var scanning logic — unchanged; only scaffold detection extends.
- `dotenvy::from_path` pattern in `stripe_config_status` — keep as-is.
- `tempfile::TempDir` test pattern — all existing tests use it; new tests follow same pattern.
- `regex::Regex` — already a dep in ferro-mcp; reuse for SyncDispatcher pattern matching.

### Established Patterns
- File walker: current `stripe_webhook_events` uses `fs::read_dir` on a single directory. For `src/` subtree walk, the planner should use `walkdir` (check if already in workspace) or recursive `fs::read_dir`.
- Regex capture groups: `Regex::new(...).captures_iter(&content).map(|cap| ...)` — follow existing pattern in `stripe_webhook_events`.
- Line number from regex: use `content[..cap.get(0).unwrap().start()].lines().count() + 1` to compute 1-based line.

### Integration Points
- `ferro-mcp/src/service.rs` `#[tool(...)]` attribute macros — update `description` strings in the three registrations.
- JSON schema is regenerated automatically when the rmcp proc-macro compiles the updated struct/method signatures. No manual regen step.
- Workspace `Cargo.toml` version field — single bump covers all crates including ferro-mcp.

</code_context>

<specifics>
## Specific Ideas

- The `WebhookEventInfo` struct after removing `listener` should still serialize cleanly via serde; no need for `#[serde(skip)]` since the field is removed entirely.
- When scanning for SyncDispatcher registrations, limit results to deduplicated (event_type, file, line) tuples — same event type could appear in multiple files, all should be reported.
- `stripe_config_status` response JSON gains four new booleans — downstream agent tools reading this output will naturally discover them; no breaking change to existing fields.

</specifics>

<deferred>
## Deferred Ideas

- Scanning for `Arc<SyncDispatcher>` provider registration (e.g., in `src/providers/`) to confirm the dispatcher is wired to the webhook endpoint — out of scope for this phase (MCP tool, not a validator).
- Per-file handler count or coverage metrics — roadmap backlog.
- `stripe_subscription_info` retirement once subscription billing tables are no longer a common pattern — revisit at v1.0 audit.

</deferred>

---

*Phase: 142-ferro-mcp-parity*
*Context gathered: 2026-04-20*
