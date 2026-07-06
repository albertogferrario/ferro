---
phase: 141-protocol-uplift
fixed_at: 2026-04-20T00:00:00Z
review_path: .planning/phases/141-protocol-uplift/141-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 141: Code Review Fix Report

**Fixed at:** 2026-04-20
**Source review:** .planning/phases/141-protocol-uplift/141-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: Raw icon HTML injected without escaping in `render_action_card`

**Files modified:** `ferro-json-ui/src/render.rs`
**Commit:** 13cbedb5
**Applied fix:** Added two-line comment above the icon format block documenting the trust boundary: raw HTML passthrough is intentional, the field is set at schema-authoring time and is not reachable from user data. This matches Option B from the review (document the developer-controlled trust boundary rather than escaping SVG that would break rendering). The WR-02 fix to the confirm dialog was included in this same commit since it modifies the same file.

### WR-01: Panic instead of recoverable error in `ProcessStripeWebhook::handle`

**Files modified:** `ferro-stripe/src/webhook/queue.rs`
**Commit:** a0a435ab
**Applied fix:** Replaced `.expect("ProcessStripeWebhook requires dispatcher …")` with `.ok_or_else(|| ferro_queue::Error::JobFailed { job: …, message: … })?` so a missing dispatcher returns a `JobFailed` error instead of panicking the worker task. Also updated the module-level doc comment to reflect the new behavior (returns `JobFailed`, does not panic).

### WR-02: Confirm dialog silently drops `message` — only `confirmTitle` ever shown

**Files modified:** `ferro-json-ui/src/render.rs`
**Commit:** 13cbedb5 (same commit as CR-01 — same file)
**Applied fix:** Changed `confirm(this.dataset.confirmTitle || this.dataset.confirmMessage)` to `confirm(this.dataset.confirmMessage || this.dataset.confirmTitle)` so the more detailed message field takes precedence when present, with title as fallback.

### WR-03: `amount_total_cents` defaults to 0 silently for absent checkout amounts

**Files modified:** `ferro-stripe/src/webhook/events.rs`
**Commit:** 26618a1f
**Applied fix:** Added a doc comment on the `amount_total_cents` field documenting the zero-means-absent contract: `0` when `amount_total` is absent from the Stripe event (free or setup-mode sessions), and callers must not use this field alone to assert payment was received. The `unwrap_or(0)` mapping is retained per Option B from the review.

---

_Fixed: 2026-04-20_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
