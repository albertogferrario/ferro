# Phase 140: Core reshape - Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the product-axis module tree (`connect/`, `subscription/`) with the capability-axis tree and land three new API surfaces in one coherent release: `CheckoutBuilder`/`CheckoutIntent`, `ProcessedEventLog`/`MemoryProcessedLog`, and `Stripe::with(key)`. Remove all product-axis modules and the stubbed `is_processed` free fn. Version bump to `ferro-stripe 0.4.0`.

This is the structural reset phase — no dispatch changes, no event typing changes. Those are Phase 141.

</domain>

<decisions>
## Implementation Decisions

### Module tree
- **D-01:** Target layout matches design doc §3.1 exactly: `checkout.rs`, `refund.rs`, `account.rs`, `webhook/{verify,events,sync,queue}`, `idempotency.rs`, `client.rs`. `connect/` and `subscription/` directories deleted in full.
- **D-02:** `webhook/sync.rs` and `webhook/queue.rs` can be created as stubs (empty or `// Phase 141`) in this phase — the directory structure needs to exist but dispatch logic ships in Phase 141.
- **D-03:** `webhook/verify.rs` extracts the existing `verify_webhook` fn from `webhook/mod.rs`. `webhook/mod.rs` becomes a thin re-export shim or is removed.

### ProcessedEventLog trait
- **D-04:** `#[async_trait] pub trait ProcessedEventLog: Send + Sync { async fn try_mark_processed(&self, event_id: &str) -> Result<bool, Error>; }` — exactly as specified in ROADMAP.md §SC-2.
- **D-05:** `MemoryProcessedLog` backed by `DashMap<String, ()>`. `dashmap` is already in the workspace Cargo.lock (transitive dep) but not yet a direct dep of `ferro-stripe` — add it explicitly to `ferro-stripe/Cargo.toml`.
- **D-06:** Module doc comment on `idempotency.rs` ships the recommended SQL schema verbatim. Ferro does not ship the migration; consumers own the table.

### CheckoutBuilder / CheckoutIntent
- **D-07:** `idempotency_key` is required before `create()`. Calling `create()` without it returns `Err(Error::MissingIdempotencyKey)`. This is enforced at runtime, not at the type level (no typestate builder) — simplicity over typestate complexity.
- **D-08:** `CheckoutIntent` carries `session_id: String`, `url: String`, `expires_at: DateTime<Utc>`, `idempotency_key: String`.
- **D-09:** `Mode` enum: `pub enum Mode { Payment, Subscription }`.
- **D-10:** `LineItem` struct: `name: String`, `description: Option<String>`, `unit_amount_cents: i64`, `quantity: u32`, `currency: String`.

### Dispatch architecture (load-bearing decision)
- **D-11:** Stripe event structs do **not** implement `ferro_events::Event`. `SyncDispatcher` (Phase 141) will be the sole handler registry for both sync and queue paths. This decision is locked here so Phase 140's event stubs are shaped correctly and no `ferro_events::Event` impls are written that would need to be removed in 141.
- **D-12:** Existing event structs in `webhook/events.rs` keep their current shape for this phase (including `event_json` and `ferro_events::Event` impls) — they are not reshaped until Phase 141. Phase 140 does not touch `webhook/events.rs` substance beyond moving the file location if needed.

### Removal of is_processed
- **D-13:** `webhook::is_processed` free fn removed. The `lib.rs` re-export `pub use webhook::is_processed` is also removed. No callers remain in-workspace (verify before removing).

### Stripe::with(key)
- **D-14:** `Stripe::with(key: &str) -> stripe::Client` — returns a scoped client without touching the global static. The existing `Stripe::init` + global client pattern is unchanged.

### Versioning and CHANGELOG
- **D-15:** `ferro-stripe` version bumped to `0.4.0` in `Cargo.toml`. Workspace `version.workspace = true` means the bump goes in the workspace root `Cargo.toml`.
- **D-16:** CHANGELOG.md entry (create if absent) documents every removed symbol and its replacement.

### Claude's Discretion
- Internal error type for `MissingIdempotencyKey` — add to the existing `Error` enum in `error.rs`, name and message at implementer's discretion.
- `MemoryProcessedLog` concurrent test strategy — use `tokio::spawn` + `tokio::join!` or similar; exact structure left to implementer.
- Whether `webhook/mod.rs` becomes a re-export shim or is deleted and replaced by explicit `pub mod` declarations in `lib.rs`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary design
- `.planning/research/v11.6-FERRO-STRIPE-REFACTOR.md` — full capability-axis design: module layout §3.1, CheckoutBuilder API §3.2, ProcessedEventLog §3.5, breaking-change ledger §4, testing strategy §8

### Existing source (read before touching)
- `ferro-stripe/src/lib.rs` — current pub re-exports; all must be updated
- `ferro-stripe/src/webhook/mod.rs` — contains `is_processed` stub to remove and `verify_webhook` to preserve
- `ferro-stripe/src/webhook/events.rs` — current event structs; do not reshape in this phase
- `ferro-stripe/src/connect/checkout.rs` — code to delete
- `ferro-stripe/src/subscription/checkout.rs` — code to delete
- `ferro-stripe/src/subscription/sync.rs` — code to delete (or confirm no survivors)
- `ferro-stripe/Cargo.toml` — add `dashmap` dep

### Workspace
- `Cargo.toml` (root) — workspace version to bump for 0.4.0 release

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `testing.rs::signed_webhook_payload` — survives unchanged; pure HMAC helper, no module dependency
- `config.rs::StripeConfig` — survives unchanged
- `client.rs::Stripe` — extend with `with(key)`, keep `init` static

### Established Patterns
- Builder pattern in the codebase uses consuming `with_*` methods returning `Self` — `CheckoutBuilder` follows the same convention (`.line_item(self, …) -> Self`, `.create(self) -> Result<…>`)
- Error type uses `thiserror` derive — add `MissingIdempotencyKey` variant to existing enum
- `async-trait = "0.1"` already a dep — use for `ProcessedEventLog`

### Integration Points
- `lib.rs` pub re-exports are the public API surface — every deleted symbol needs its re-export removed; every new symbol needs one added
- `ferro-stripe` is published on crates.io — CHANGELOG entry is consumer-facing, not optional

</code_context>

<specifics>
## Specific Ideas

- `MemoryProcessedLog` concurrent test: spawn two `tokio::task`s that race on `try_mark_processed("evt_same_id")`, assert exactly one returns `Ok(true)` and one returns `Ok(false)`. This is the correctness proof for the idempotency primitive.
- `Stripe::with(key)` scoped override is specifically for per-tenant direct-charges scenarios where a different Stripe account key is needed per request. The implementer should add a doc comment making this use case explicit.

</specifics>

<deferred>
## Deferred Ideas

- Webhook secret rotation support (second-secret variant for `verify_webhook`) — deliberate non-goal for v11.6; revisit pre-1.0 only if a consumer needs it
- Typestate builder for `CheckoutBuilder` to enforce `idempotency_key` at compile time — adds complexity, runtime check is sufficient for now
- `stripe_subscription_info` MCP tool update — deferred to Phase 142

</deferred>

---

*Phase: 140-core-reshape*
*Context gathered: 2026-04-20*
