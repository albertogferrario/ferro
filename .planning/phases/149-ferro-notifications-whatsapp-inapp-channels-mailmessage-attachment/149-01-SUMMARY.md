---
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
plan: 01
subsystem: notifications
tags: [ferro-notifications, whatsapp, in-app, sms, push, skeleton, type-contracts]

requires:
  - phase: 149-research
    provides: ARCH-FINDING-03 (symmetric Sms/Push placeholders), Crate Surface Map, locked signatures
provides:
  - WhatsAppMessage skeleton type wrapping ferro_whatsapp::Message (text / template builders)
  - InAppMessage + InAppSeverity skeletons (Info/Success/Warning/Error)
  - SmsMessage + PushMessage placeholder types per ARCH-FINDING-03
  - channels/mod.rs re-exports for the new public types
  - ferro-notifications/src/lib.rs top-level re-exports for the new types (pulled forward from plan 07)
  - ferro-whatsapp path dep added to ferro-notifications/Cargo.toml
affects:
  - 149-02 (Channel enum variants + Notification trait methods, depends on these types)
  - 149-03 (MailMessage attachment)
  - 149-04 (WhatsApp dispatcher arm — calls into WhatsAppMessage.message)
  - 149-05 (InApp dispatcher arm — calls into InAppMessage shape)
  - 149-06 (database channel fix)
  - 149-07 (full lib.rs sweep + publish.yml wave move + integration tests)

tech-stack:
  added:
    - ferro-whatsapp (path dep) for the WhatsAppMessage type wrapper
  patterns:
    - Wave-0 skeleton-only delivery: locked signatures, no behavior, downstream waves can compile against them
    - Inline #[cfg(test)] tests next to the type definition
    - Consuming builders for fluent in-app message construction (.data().severity())

key-files:
  created:
    - ferro-notifications/src/channels/whatsapp.rs
    - ferro-notifications/src/channels/in_app.rs
    - ferro-notifications/src/channels/future.rs
  modified:
    - ferro-notifications/src/channels/mod.rs
    - ferro-notifications/src/lib.rs
    - ferro-notifications/Cargo.toml
    - Cargo.lock (workspace version sync + new ferro-whatsapp dep edge)

key-decisions:
  - Pulled lib.rs top-level re-exports forward from plan 07 to satisfy CLAUDE.md zero-warning rule (Rule 3 deviation)
  - WhatsAppMessage wraps ferro_whatsapp::Message rather than re-implementing the shape (per CONTEXT.md D-03)
  - SmsMessage / PushMessage live in channels/future.rs (one shared file rather than two single-type files) so the trait surface stays tidy

patterns-established:
  - Wave-0 skeleton plan pattern: type contracts only, builders compile, dispatcher untouched
  - channels/future.rs as the parking lot for unimplemented but signature-stable channels

requirements-completed:
  - ROADMAP-149-01
  - ROADMAP-149-02

duration: 9m 7s
completed: 2026-04-28
---

# Phase 149 Plan 01: ferro-notifications Wave-0 Skeletons Summary

**Locked-signature skeleton types for WhatsApp / InApp / Sms / Push channels in ferro-notifications, wired through channels/mod.rs and the crate's top-level re-exports — downstream plans 02-07 now compile their tests and adapter code against fixed contracts.**

## Performance

- **Duration:** 9m 7s
- **Started:** 2026-04-28T22:13:11Z
- **Completed:** 2026-04-28T22:22:18Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Three new public types ship as skeletons: `WhatsAppMessage`, `InAppMessage` (+ `InAppSeverity`), and the placeholder pair `SmsMessage` / `PushMessage`.
- All four message types are constructible via their plan-locked builders; round-trip serde behaves as specified (lowercase severity wire form).
- Six new unit tests pass; existing 200+ ferro-notifications tests unchanged.
- Workspace builds clean under `cargo fmt`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test --all-features`.
- ferro-whatsapp path dependency added to ferro-notifications without disturbing other crates (Cargo.lock confirms only one new edge).

## Task Commits

Each task was committed atomically after passing fmt + clippy + test:

1. **Task 1: WhatsAppMessage skeleton** — `898814cd` (feat)
2. **Task 2: InAppMessage / InAppSeverity + Sms/PushMessage skeletons** — `feeb84fe` (feat)
3. **Task 3: Wire mod.rs + lib.rs re-exports** — `0b388781` (feat)

## Files Created/Modified

- `ferro-notifications/src/channels/whatsapp.rs` — created. `WhatsAppMessage { message: ferro_whatsapp::Message }` with `text()` / `template(name, language, parameters)` builders + 2 inline tests.
- `ferro-notifications/src/channels/in_app.rs` — created. `InAppMessage { notification_type, data, severity }` with `new() / .data() / .severity()` builders. `InAppSeverity` enum (Info/Success/Warning/Error) with `#[serde(rename_all = "lowercase")]`. 2 inline tests.
- `ferro-notifications/src/channels/future.rs` — created. `SmsMessage { body }` + `PushMessage { title, body }`, both with Default/Debug/Clone/Serialize/Deserialize. 2 inline tests.
- `ferro-notifications/src/channels/mod.rs` — modified. Adds `mod whatsapp; mod in_app; mod future;` declarations and `pub use` re-exports for `WhatsAppMessage`, `InAppMessage`, `InAppSeverity`, `SmsMessage`, `PushMessage`. Existing `DatabaseMessage`, `MailMessage`, `Slack*` re-exports preserved.
- `ferro-notifications/src/lib.rs` — modified. Top-level `pub use channels::{...}` block extended to re-export the five new types. (Pulled forward from plan 07; see Deviations.)
- `ferro-notifications/Cargo.toml` — modified. Adds `ferro-whatsapp = { path = "../ferro-whatsapp", version = "0.2" }` to `[dependencies]`. Plan 07 will add `ferro-broadcast`, `base64`, and the publish-wave move.
- `Cargo.lock` — modified. Workspace pre-existing version bump 0.2.17 → 0.2.18 propagated. Single new dep edge: `ferro-notifications` → `ferro-whatsapp`.

## Decisions Made

- **Pulled lib.rs re-exports forward from plan 07.** The plan as written explicitly defers the `lib.rs` top-level re-export to plan 07, but adding `pub use channels::...` lines in `channels/mod.rs` without a corresponding top-level re-export produces unused-import warnings. CLAUDE.md mandates `cargo clippy -- -D warnings` clean before every commit, and CI enforces the same. To honor both the plan's intent (locked signatures, no dispatcher touch, no behavior) and the project rule (no warnings), the minimal extra edit is the lib.rs re-export block — purely additive, breaks nothing, satisfies CI. This is recorded as a Rule 3 deviation below.
- **future.rs as a shared placeholder file.** The plan suggests "single file vs sub-folder per channel" is at executor discretion. Sms and Push share the same lifecycle (signature-only, no adapter, no config) so housing them in one `channels/future.rs` keeps the channels directory legible.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pulled `lib.rs` top-level re-exports forward from plan 07**
- **Found during:** Task 3 (wire mod.rs)
- **Issue:** Adding `pub use future::{PushMessage, SmsMessage};` etc. in `channels/mod.rs` without a corresponding `pub use channels::...` in `ferro-notifications/src/lib.rs` produced six unused-import / dead-code warnings. CI enforces `cargo clippy --all --all-targets -- -D warnings`, so the build would fail at the next step. Plan 01's verification only required `cargo build` to exit 0 (warnings allowed) and explicitly deferred the lib.rs re-export to plan 07.
- **Fix:** Extended the existing `pub use channels::{DatabaseMessage, MailMessage, SlackAttachment, SlackField, SlackMessage};` line in `ferro-notifications/src/lib.rs` to also re-export `InAppMessage`, `InAppSeverity`, `PushMessage`, `SmsMessage`, `WhatsAppMessage`. Purely additive — no existing re-export removed.
- **Files modified:** `ferro-notifications/src/lib.rs`
- **Verification:** `cargo build -p ferro-notifications` clean, `cargo clippy --all --all-targets -- -D warnings` clean, `cargo test --all-features` all passing.
- **Committed in:** `0b388781` (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 blocking)
**Impact on plan:** Minimal — pulled forward a small slice of plan 07 to keep CI green. Plan 07 still owns the publish-wave move, the `ferro-broadcast` + `base64` deps, and the framework/src/lib.rs framework-level re-exports. No scope creep beyond the unavoidable warning fix.

## Issues Encountered

None. The Cargo.lock diff initially showed workspace version bumps (0.2.17 → 0.2.18) that looked unrelated; inspection confirmed the workspace `[workspace.package]` already declared 0.2.18 and the lock file was simply re-syncing — pre-existing drift, not introduced by this plan.

## User Setup Required

None — no external service configuration required for skeleton types.

## Next Phase Readiness

Plan 02 (Channel enum variants + Notification trait methods) can now reference:
- `ferro_notifications::WhatsAppMessage` (fully constructed)
- `ferro_notifications::InAppMessage` and `InAppSeverity`
- `ferro_notifications::SmsMessage` and `PushMessage`

All five types are stable contracts. Plan 02 will add `Channel::WhatsApp` / `Channel::InApp` to the enum and `to_whatsapp` / `to_in_app` / `to_sms` / `to_push` default-`None` methods to the `Notification` trait — those signatures will compile on top of these skeletons without further type churn.

Plans 04-07 (dispatcher arms, error variants, attachment builders, publish.yml wave move) remain untouched.

## Self-Check: PASSED

Verification commands executed:

```
$ test -f ferro-notifications/src/channels/whatsapp.rs            # FOUND
$ test -f ferro-notifications/src/channels/in_app.rs              # FOUND
$ test -f ferro-notifications/src/channels/future.rs              # FOUND
$ grep -q "pub struct WhatsAppMessage" .../whatsapp.rs            # FOUND
$ grep -q "pub struct InAppMessage" .../in_app.rs                 # FOUND
$ grep -q "pub enum InAppSeverity" .../in_app.rs                  # FOUND
$ grep -q "pub struct SmsMessage" .../future.rs                   # FOUND
$ grep -q "pub struct PushMessage" .../future.rs                  # FOUND
$ grep -q "^ferro-whatsapp" ferro-notifications/Cargo.toml        # FOUND
$ git log --oneline | grep 898814cd                               # FOUND (Task 1)
$ git log --oneline | grep feeb84fe                               # FOUND (Task 2)
$ git log --oneline | grep 0b388781                               # FOUND (Task 3)
$ cargo build -p ferro-notifications                              # exit 0
$ cargo fmt --all -- --check                                      # exit 0
$ cargo clippy --all --all-targets -- -D warnings                 # exit 0
$ cargo test --all-features                                       # all pass
```

---

*Phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment*
*Completed: 2026-04-28*
