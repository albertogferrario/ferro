---
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
plan: 02
subsystem: notifications
tags: [ferro-notifications, whatsapp, in-app, sms, push, channel-enum, error-variants, surface-only]

requires:
  - plan: 149-01
    provides: WhatsAppMessage, InAppMessage, InAppSeverity, SmsMessage, PushMessage skeletons re-exported from crate::channels
provides:
  - Channel::WhatsApp + Channel::InApp enum variants with explicit per-variant serde renames
  - Notification trait gains four default-None methods (to_whatsapp, to_in_app, to_sms, to_push)
  - Error::WhatsApp(#[from] ferro_whatsapp::Error), Error::Broadcast(String), Error::AttachmentTooLarge { filename, size, limit }
  - Error::broadcast(msg) helper for plan 06
affects:
  - 149-03 (MailMessage::attachment uses Error::AttachmentTooLarge)
  - 149-04 (WhatsApp dispatcher arm uses Error::WhatsApp via #[from])
  - 149-05 (InApp dispatcher arm uses Error::Broadcast + Error::broadcast helper)
  - 149-06 (Database channel fix unaffected — uses existing Error::Database)
  - 149-07 (lib.rs sweep + publish.yml wave move + integration tests)

tech-stack:
  added: []
  patterns:
    - Per-variant `#[serde(rename = "...")]` overrides on top of an enum-level `#[serde(rename_all = "lowercase")]` to handle multi-word variants whose lowercase form would lose the underscore (InApp → in_app, not inapp)
    - `#[from] ferro_whatsapp::Error` preserves the typed source chain via `Error::source()` (vs flattening to `WhatsApp(String)` which would lose source)
    - Default-None trait methods extend the Notification surface without breaking any existing impl
    - Exhaustive-match arms updated for new Channel variants — placeholder logging until real adapters land in plans 04/05

key-files:
  created: []
  modified:
    - ferro-notifications/src/channel.rs
    - ferro-notifications/src/notification.rs
    - ferro-notifications/src/error.rs
    - ferro-notifications/src/dispatcher.rs

key-decisions:
  - Added a regression-guard test rejecting the literal "inapp" wire form (closes the ARCH-FINDING-05 / RESEARCH.md serde-rename trap with a fail-fast assertion).
  - Pulled forward the dispatcher exhaustive-match arm fix from plans 04/05 (Rule 3 deviation) — adding new Channel variants forced a `match channel { ... }` exhaustiveness fix; plan 02 was scoped surface-only but a placeholder arm is the minimal change to keep the workspace compiling. Plans 04/05 will replace these arms with real dispatch logic.
  - Pulled forward Error::Broadcast(String) + helper from plan 06's scope (Rule 2 deviation) — the variant is a load-bearing primitive for InApp adapter error mapping; adding it now keeps the Error enum in one logical commit and avoids re-touching error.rs in plan 06.

patterns-established:
  - "Surface-additive plan pattern: extend enum / trait / error type in one wave; downstream waves wire dispatcher arms against locked surfaces."
  - "Placeholder match arms for not-yet-wired channels: a single `Channel::WhatsApp | Channel::InApp | Channel::Sms | Channel::Push => { info!(\"not implemented\"); }` arm absorbs all unimplemented variants until each gets its real arm."

requirements-completed:
  - ROADMAP-149-01
  - ROADMAP-149-02

duration: 4m 12s
completed: 2026-04-28
---

# Phase 149 Plan 02: Channel + Notification + Error Surface Extensions Summary

**Public surface of `ferro-notifications` extended with `Channel::WhatsApp` + `Channel::InApp` (with explicit serde renames closing the lowercase-rule trap), four default-None `Notification` trait methods (D-02 + ARCH-FINDING-03), and three new `Error` variants (`WhatsApp(#[from])`, `Broadcast(String)`, `AttachmentTooLarge {..}`) — plans 03-06 now compile their dispatcher and adapter logic against locked types.**

## Performance

- **Duration:** 4m 12s
- **Started:** 2026-04-28T22:25:04Z
- **Completed:** 2026-04-28T22:29:16Z
- **Tasks:** 3
- **Files modified:** 4
- **New unit tests:** 12 (4 channel + 4 notification + 4 error)

## Accomplishments

- `Channel` enum gains `WhatsApp` + `InApp` variants. Per-variant `#[serde(rename)]` attributes override the enum-level `lowercase` rule where it would corrupt the wire form (`InApp` → `inapp` ❌; with override → `in_app` ✓).
- A regression-guard test asserts the literal `"inapp"` is rejected by the deserializer — any future refactor that drops the `#[serde(rename = "in_app")]` override fails this test before it can ship.
- `Notification` trait now has 7 default-None converter methods: 3 existing (`to_mail`, `to_database`, `to_slack`) + 4 new (`to_whatsapp`, `to_in_app`, `to_sms`, `to_push`). All existing `Notification` impls compile unchanged (forward-compat verified by the unmodified `TestNotification` impl in the test module).
- `Error` enum extended with `WhatsApp(#[from] ferro_whatsapp::Error)`, `Broadcast(String)`, and `AttachmentTooLarge { filename, size, limit }`. `#[from]` preserves the underlying typed error chain via `Error::source()`.
- `Error::broadcast(msg)` helper added now so plan 06's InApp adapter has the constructor ready (matches the existing `mail` / `slack` / `database` helper pattern).
- Workspace builds clean under `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test --all-features` (full workspace, all tests passing).
- 4 new channel tests, 4 new notification tests, 4 new error tests — all green.

## Task Commits

Each task was committed atomically after passing fmt + clippy + test:

1. **Task 1: Channel::WhatsApp + Channel::InApp variants** — `cab3f37a` (feat)
2. **Task 2: Notification trait default-None methods** — `dbc00d3f` (feat)
3. **Task 3: Error::WhatsApp + Error::Broadcast + Error::AttachmentTooLarge variants** — `0b7bf7a1` (feat)

## Files Created/Modified

- `ferro-notifications/src/channel.rs` — modified. Added `WhatsApp` + `InApp` variants with `#[serde(rename = "whatsapp")]` and `#[serde(rename = "in_app")]` respectively. `as_str()` and `Display` extended. 4 new unit tests including the `"inapp"` regression guard.
- `ferro-notifications/src/notification.rs` — modified. Imports extended to pull `WhatsAppMessage`, `InAppMessage`, `SmsMessage`, `PushMessage` from `crate::channels`. Four new default-None trait methods added between `to_slack` and `notification_type`. The existing rustdoc example is untouched (it documents `to_mail` and `to_database` only). 4 new unit tests.
- `ferro-notifications/src/error.rs` — modified. Three new variants and one new helper. Existing `mail`, `slack`, `database` helpers and variants preserved unchanged. 4 new unit tests including a `#[from]` chain assertion.
- `ferro-notifications/src/dispatcher.rs` — modified. The `match channel { ... }` arm at line 322 was extended from `Channel::Sms | Channel::Push` to `Channel::WhatsApp | Channel::InApp | Channel::Sms | Channel::Push` so the dispatcher compiles against the new variants. The arm body still emits the `"Channel not implemented"` info log; real adapter logic lands in plans 04/05. (Rule 3 deviation — see below.)

## Decisions Made

- **Regression-guard test for the serde-rename trap.** RESEARCH.md called out that `#[serde(rename_all = "lowercase")]` would silently produce `"inapp"` for `InApp`. Beyond fixing it with an explicit `rename = "in_app"`, added `assert!(serde_json::from_str::<Channel>("\"inapp\"").is_err())` as a regression guard — a future refactor that drops the override now fails this test before it can ship a bad wire form to downstream `gestiscilo-it` (closes T-149-W1A-01 tampering threat).
- **`Error::Broadcast` + helper added in plan 02 rather than plan 06.** Plan 06 will need this variant when it wires the InApp adapter. Adding it now keeps `error.rs` cohesive (one logical commit per error-surface change rather than two) and lets plan 06 focus on adapter logic. Pure addition — no existing variant or helper changed.
- **Placeholder match arm extends to absorb all 4 unimplemented variants in one match-pattern.** Cheaper than four separate `_ => { ... }` arms and self-documents what's still pending.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extended dispatcher's exhaustive Channel match for new variants**
- **Found during:** Task 1 verification (cargo test channel::tests)
- **Issue:** The plan's task 1 only modifies `channel.rs`, but adding `WhatsApp` + `InApp` variants to a Rust enum used in an existing exhaustive `match channel { ... }` produces `error[E0004]: non-exhaustive patterns: Channel::WhatsApp and Channel::InApp not covered` in `ferro-notifications/src/dispatcher.rs:306`. The crate fails to compile, blocking the test verification step.
- **Fix:** Extended the existing `Channel::Sms | Channel::Push => { info!("Channel not implemented"); }` arm to `Channel::WhatsApp | Channel::InApp | Channel::Sms | Channel::Push => { ... }`. The arm body is unchanged (still emits an info-level "not implemented" log). No adapter logic added — that's plans 04 (WhatsApp) and 05 (InApp).
- **Files modified:** `ferro-notifications/src/dispatcher.rs` (one match arm edit, no new code)
- **Verification:** `cargo build -p ferro-notifications` clean; `cargo clippy --all --all-targets -- -D warnings` clean; `cargo test -p ferro-notifications channel::tests` passes (4/4); full workspace `cargo test --all-features` passes (no regression).
- **Committed in:** `cab3f37a` (Task 1 commit)
- **Scope justification:** Surface-only by spirit (no behavior added, no new function, no new field). The plan's verification commands (`cargo test channel::tests`) cannot run without this fix. Stays within plan 02's "no dispatcher work" rule because the arm body is unchanged — only the pattern was widened.

**2. [Rule 2 - Critical functionality] Added Error::Broadcast + Error::broadcast helper in plan 02**
- **Found during:** Task 3 (writing error.rs)
- **Issue:** RESEARCH.md §"InApp Adapter" calls out that `ferro_broadcast::Error` is not auto-convertible to `ferro_notifications::Error` and recommends adding `Error::Broadcast(String)`. Plan 02 lists the new variants for D-05 (WhatsApp) and D-11 (AttachmentTooLarge) but does not explicitly list Broadcast. The plan's `<must_haves>` block does not mandate Broadcast, but its template at line 348 of the action shows `Broadcast(String)` and the variant is referenced in plan 06's expected mapping logic.
- **Fix:** Added `Error::Broadcast(String)` and `Error::broadcast(msg)` helper alongside the WhatsApp + AttachmentTooLarge variants in the same commit. Pure addition — no existing variant changed.
- **Files modified:** `ferro-notifications/src/error.rs`
- **Verification:** `test_error_broadcast_helper` passes; full workspace tests green.
- **Committed in:** `0b7bf7a1` (Task 3 commit)
- **Scope justification:** Plan 02's task 3 action snippet at line 348 already includes `Broadcast(String)` — the plan author included it in the code template but the `<must_haves>.truths` block doesn't enumerate it explicitly. Adding it now matches the action's literal code and keeps `error.rs` in one logical commit.

---

**Total deviations:** 2 auto-fixed (1 Rule 3 blocking, 1 Rule 2 critical)
**Impact on plan:** Minimal. Both fixes were already implied by the plan's action snippets and verification commands; this summary records them explicitly so plan 06 (InApp adapter) starts from a clean Error surface and plans 04/05 don't have to rediscover the dispatcher exhaustiveness issue.

## Issues Encountered

None. The Rule 3 dispatcher fix was identified within seconds (compile error pointed straight to the line), and the workspace test sweep confirmed no regressions across 485 + 30 + dozens of crate-level test results.

## User Setup Required

None — no external service configuration required for surface-only changes. The `whatsapp_enabled` flag and `WhatsAppMessage::send`-path consumer requirements arrive in plan 04.

## Next Phase Readiness

Plan 03 (MailMessage::attachment) can now reference:
- `ferro_notifications::Error::AttachmentTooLarge { filename, size, limit }` — the typed error to return from the 25 MB cap check.

Plan 04 (WhatsApp dispatcher arm) can now reference:
- `ferro_notifications::Channel::WhatsApp` — the enum variant the dispatcher matches on.
- `ferro_notifications::Error::WhatsApp(ferro_whatsapp::Error)` — the typed error variant for `?`-propagation.
- The placeholder match arm in `dispatcher.rs:322` is the exact insertion point — split it back into a real `Channel::WhatsApp => Self::send_whatsapp(...).await?` arm.

Plan 05 (InApp dispatcher arm) can now reference:
- `ferro_notifications::Channel::InApp` and the same placeholder match arm.
- `ferro_notifications::Error::Broadcast(String)` + `Error::broadcast(msg)` helper for `ferro_broadcast::Error` mapping.

Plan 06 (database channel fix) is unaffected — `Error::Database` is unchanged.

Plan 07 (full lib.rs sweep + publish.yml wave move + integration tests) — `lib.rs` re-exports for the new types were already pulled forward in plan 01.

## Threat Flags

None. The plan's threat model anticipated all surface changes:
- T-149-W1A-01 (channel serde wire-form tampering) — closed by the `"inapp"` rejection regression test.
- T-149-W1A-02 (filename + size leak in AttachmentTooLarge) — accepted; trusted-server-log diagnostic only.
- T-149-W1A-03 (error chain flattening) — closed by `#[from]` choice and the `test_error_whatsapp_from_impl` source-chain assertion.

No new surface introduced beyond the threat model.

## Self-Check: PASSED

Verification commands executed:

```
$ grep -q "Channel::WhatsApp" ferro-notifications/src/channel.rs                # FOUND
$ grep -q "Channel::InApp" ferro-notifications/src/channel.rs                   # FOUND
$ grep -q 'rename = "in_app"' ferro-notifications/src/channel.rs                # FOUND
$ grep -q 'rename = "whatsapp"' ferro-notifications/src/channel.rs              # FOUND
$ grep -q "fn to_whatsapp" ferro-notifications/src/notification.rs              # FOUND
$ grep -q "fn to_in_app" ferro-notifications/src/notification.rs                # FOUND
$ grep -q "fn to_sms" ferro-notifications/src/notification.rs                   # FOUND
$ grep -q "fn to_push" ferro-notifications/src/notification.rs                  # FOUND
$ grep -q "WhatsApp(#\[from\] ferro_whatsapp::Error)" ferro-notifications/src/error.rs   # FOUND
$ grep -q "AttachmentTooLarge" ferro-notifications/src/error.rs                  # FOUND
$ grep -q "Broadcast(String)" ferro-notifications/src/error.rs                   # FOUND
$ grep -q "pub fn broadcast" ferro-notifications/src/error.rs                    # FOUND
$ git log --oneline | grep cab3f37a                                              # FOUND (Task 1)
$ git log --oneline | grep dbc00d3f                                              # FOUND (Task 2)
$ git log --oneline | grep 0b7bf7a1                                              # FOUND (Task 3)
$ cargo build -p ferro-notifications                                             # exit 0
$ cargo fmt --all -- --check                                                     # exit 0
$ cargo clippy --all --all-targets -- -D warnings                                # exit 0
$ cargo test --all-features                                                      # all 485+ tests pass
```

---

*Phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment*
*Completed: 2026-04-28*
