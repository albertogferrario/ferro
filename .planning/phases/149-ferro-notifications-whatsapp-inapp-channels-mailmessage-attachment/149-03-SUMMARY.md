---
phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment
plan: 03
subsystem: notifications
tags: [ferro-notifications, mail, attachments, mailattachment, fallible-builder, size-cap]

requires:
  - plan: 149-01
    provides: channels/mod.rs structure (MailMessage already re-exported; MailAttachment slot to add)
  - plan: 149-02
    provides: Error::AttachmentTooLarge { filename, size, limit } variant (the typed failure path)
provides:
  - MailAttachment struct (filename, content_type, content: Vec<u8>) with serde derives
  - MAX_ATTACHMENT_BYTES constant (25 MB inclusive cap)
  - MailMessage::attachments: Vec<MailAttachment> field with #[serde(default)] for backward-compat deserialization
  - Fallible builder MailMessage::attachment(filename, content_type, content) -> Result<Self, crate::Error>
  - channels::MailAttachment public re-export
  - top-level ferro_notifications::MailAttachment re-export (additive entry on the existing channels::{...} block)
affects:
  - 149-04 (SMTP/Resend dispatcher arm reads message.attachments and emits multipart/base64)
  - 149-07 (lib.rs sweep — MailAttachment is already in the top-level re-export block; nothing further needed)

tech-stack:
  added: []
  patterns:
    - "Fallible consuming builder: `pub fn attachment(self, ...) -> Result<Self, Error>` — call sites use `?` to propagate or `.unwrap()` for trusted-input call paths"
    - "Inclusive size cap (`> MAX_ATTACHMENT_BYTES` rejects, exact match accepts) — explicit boundary test (`test_mail_attachment_at_exact_limit_succeeds`) locks the contract against future drift to `>=`"
    - "`#[serde(default)]` on the new `attachments` field so JSON payloads serialized before this plan's landing still deserialize cleanly into `MailMessage` instances with an empty attachments vec"
    - "Per-attachment cap only — no cumulative cap (Resend's 40 MB total is the carrier's responsibility per CONTEXT.md D-11)"

key-files:
  created: []
  modified:
    - ferro-notifications/src/channels/mail.rs
    - ferro-notifications/src/channels/mod.rs
    - ferro-notifications/src/lib.rs

key-decisions:
  - 25 MB cap is inclusive (D-11): exactly `MAX_ATTACHMENT_BYTES` bytes succeeds; one byte over fails. Locked by `test_mail_attachment_at_exact_limit_succeeds` and `test_mail_attachment_over_limit_returns_typed_error`.
  - No cumulative cap enforced — Resend's 40 MB total is the carrier's responsibility per CONTEXT.md D-11. Documented in the rustdoc on `attachment()`.
  - `MAX_ATTACHMENT_BYTES` exposed as `pub const` so downstream call sites (and tests) can reference the framework cap without re-deriving the constant.
  - Rule 3 Blocking deviation: pulled the `lib.rs` top-level `MailAttachment` re-export forward — same pattern established in Plan 01, required to keep `cargo clippy --all --all-targets -- -D warnings` green.

patterns-established:
  - "Mail attachment builder pattern: `MailMessage::new().subject(...).attachment(filename, content_type, content)?.attachment(...)?.bcc(...)` — fallible attachments interleave fluently with infallible builder steps via `?`."
  - "Add a `#[serde(default)]` whenever extending an existing `Default + Serialize + Deserialize` struct with a new collection-typed field — preserves backward-compat for already-persisted JSON payloads."

requirements-completed:
  - ROADMAP-149-05

duration: 5m 1s
completed: 2026-04-28
---

# Phase 149 Plan 03: MailMessage Attachment Support Summary

**MailMessage gains `attachments: Vec<MailAttachment>` and a fallible `attachment()` builder enforcing the 25 MB per-attachment cap from CONTEXT.md D-11 — Plan 04's SMTP multipart and Resend base64 emitters now have the typed payload to consume.**

## Performance

- **Duration:** 5m 1s
- **Started:** 2026-04-28T22:33:00Z
- **Completed:** 2026-04-28T22:38:01Z
- **Tasks:** 2
- **Files modified:** 3
- **New unit tests:** 7 (1 builder test preserved, 7 new attachment-specific tests)
- **Lines added:** 137 net (134 to mail.rs, 3 to mod.rs + lib.rs combined)

## Accomplishments

- `MailAttachment { filename, content_type, content: Vec<u8> }` ships with `Debug + Clone + Serialize + Deserialize` derives — directly serializable to/from JSON for queue persistence and trace logging.
- `MAX_ATTACHMENT_BYTES = 25 * 1024 * 1024 = 26_214_400` exposed as `pub const` so the framework cap is queryable from downstream code without duplication.
- Fallible builder `MailMessage::attachment(filename, content_type, content) -> Result<Self, crate::Error>` returns `Error::AttachmentTooLarge { filename, size, limit }` on overflow; `Ok(Self)` on success; multiple `?`-chained calls accumulate.
- `attachments` field carries `#[serde(default)]` so older persisted `MailMessage` JSON (queue jobs, retry envelopes) still round-trips into the new struct shape with `attachments: vec![]`.
- 8 unit tests in `channels::mail::tests` (1 preserved, 7 new): under-limit, exact-limit boundary, over-limit (typed error fields asserted), accumulation across three attachments, serde round-trip with attachment, default-empty, and the constant value itself.
- Workspace builds clean under `cargo fmt --all -- --check`, `cargo clippy --all --all-targets -- -D warnings`, and `cargo test --all-features` (full workspace, 1700+ tests passing).

## Task Commits

Each task was committed atomically after passing fmt + clippy + test:

1. **Task 1: MailAttachment + 25MB-capped attachment() builder** — `415ea233` (feat)
2. **Task 2: Re-export MailAttachment from channels/mod.rs and crate root** — `7721495e` (feat)

## Files Created/Modified

- `ferro-notifications/src/channels/mail.rs` — modified. Adds `MAX_ATTACHMENT_BYTES`, `MailAttachment` struct, `attachments` field on `MailMessage` (with `#[serde(default)]`), and the fallible `attachment()` builder. Pre-existing 8 builder methods (`subject`, `body`, `html`, `from`, `reply_to`, `cc`, `bcc`, `header`) untouched. Test module extended from 1 to 8 tests.
- `ferro-notifications/src/channels/mod.rs` — modified. One-line change: `pub use mail::MailMessage;` → `pub use mail::{MailAttachment, MailMessage};`. All other Plan 01 re-exports preserved.
- `ferro-notifications/src/lib.rs` — modified. The existing `pub use channels::{...}` block (already pulled forward in Plan 01) gains `MailAttachment` between `InAppSeverity` and `MailMessage`. No other lib.rs changes.

## Decisions Made

- **Inclusive 25 MB cap.** Per CONTEXT.md D-11 the framework cap is `25 * 1024 * 1024 = 26_214_400` bytes inclusive — `> MAX_ATTACHMENT_BYTES` rejects, equality accepts. The `test_mail_attachment_at_exact_limit_succeeds` test locks this contract: any future refactor that drifts to `>=` fails this test before shipping.
- **No cumulative cap enforced.** Resend's 40 MB total per email is documented in CONTEXT.md as the carrier's responsibility — we surface only the per-attachment framework cap. The `attachment()` rustdoc spells this out so call sites know to budget themselves if their adapter targets Resend.
- **`MAX_ATTACHMENT_BYTES` is `pub const`.** Plan 04's SMTP/Resend adapter logic and any future test or doctor check can reference the constant directly rather than re-deriving `25 * 1024 * 1024`.
- **`#[serde(default)]` on the `attachments` field.** Without it, any pre-existing serialized `MailMessage` payload (e.g., queue job envelope persisted before this plan landed) would fail to deserialize with `missing field 'attachments'`. The default `Vec::new()` is the right contract: an old payload simply has no attachments.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Extended lib.rs top-level re-export block to include MailAttachment**
- **Found during:** Task 2 verification (cargo clippy --all --all-targets -- -D warnings)
- **Issue:** Adding `pub use mail::{MailAttachment, MailMessage};` to `channels/mod.rs` without a corresponding `MailAttachment` entry in the crate-level `pub use channels::{...}` block in `ferro-notifications/src/lib.rs` triggered `error: unused import: MailAttachment` under `-D warnings`. CI rejects warnings, so the build fails at the next pre-commit gate. Plan 03 only specified mod.rs as modified; lib.rs was scoped for Plan 07 (already partially pulled forward by Plan 01).
- **Fix:** Extended the existing `pub use channels::{ DatabaseMessage, InAppMessage, ..., MailMessage, ... };` line in `ferro-notifications/src/lib.rs` with `MailAttachment` placed alphabetically between `InAppSeverity` and `MailMessage`. Pure addition — no existing re-export removed or reordered semantically.
- **Files modified:** `ferro-notifications/src/lib.rs` (one line, additive)
- **Verification:** `cargo build -p ferro-notifications` clean; `cargo fmt --all -- --check` clean; `cargo clippy --all --all-targets -- -D warnings` clean; `cargo test --all-features` all passing.
- **Committed in:** `7721495e` (Task 2 commit, alongside the mod.rs change it was responding to)
- **Scope justification:** Identical pattern to Plan 01's Rule 3 deviation. Plan 01 had to pull `lib.rs` re-exports forward from Plan 07 to keep CI green; this plan is just adding one more entry to that already-pulled-forward block. Plan 07's remaining scope (publish.yml wave move, framework/src/lib.rs framework-level re-exports, integration tests) is unaffected.

---

**Total deviations:** 1 auto-fixed (Rule 3 blocking)
**Impact on plan:** Minimal. Plan 01 already established this pattern and the corresponding lib.rs block; Plan 03 just adds one alphabetically-placed entry. Plan 07's scope is reduced by exactly one re-export line (already accounted for in Plan 01's deviation note).

## Issues Encountered

None. The plan's full action snippet was directly applicable; the pre-commit clippy gate caught the lib.rs gap immediately and the fix was the established Plan 01 pattern.

## User Setup Required

None — adding an attachment field and builder is a pure type-surface change. Plan 04 will introduce the actual base64 encoding and SMTP multipart wiring; that plan owns the `Resend API key` user setup item.

## Next Phase Readiness

Plan 04 (SMTP multipart + Resend base64 dispatcher arm) can now reference:
- `ferro_notifications::MailAttachment` — the typed payload to read off `message.attachments` in the dispatcher.
- `MailMessage.attachments: Vec<MailAttachment>` — the iterable field carried through `to_mail()`.
- `ferro_notifications::Error::AttachmentTooLarge` — already produced by the builder; the dispatcher only needs to handle the lettre / Resend-side errors (filename header escaping, base64 encoding, multipart construction). Builder-side validation is a closed concern.
- `MAX_ATTACHMENT_BYTES` is `pub const` and queryable if Plan 04 wants a doctor check or pre-flight assertion.

Plans 04 and 07 are unblocked. Plan 04 can begin in this same Wave 1 in parallel with Plan 05 (no shared file with mail.rs).

## Threat Flags

None. The plan's threat model anticipated all surface changes:
- T-149-W1B-01 (Resource exhaustion via large attachment) — closed: 25 MB cap enforced before push to `Vec<MailAttachment>`. Both boundary tests (`test_mail_attachment_at_exact_limit_succeeds` and `test_mail_attachment_over_limit_returns_typed_error`) verify the contract.
- T-149-W1B-02 (Filename / content_type passed to lettre + Resend) — accepted; Plan 04 owns escaping via `lettre::Attachment::new(filename).body(content, ContentType::parse(content_type))`. No second sanitization layer added in Plan 03.
- T-149-W1B-03 (Vec<u8> attachment content lives in process memory) — accepted; same trust boundary as the existing `MailMessage::body: String`. No new exposure.

No new surface beyond the threat model.

## Self-Check: PASSED

Verification commands executed:

```
$ grep -q "pub struct MailAttachment" ferro-notifications/src/channels/mail.rs                # FOUND
$ grep -q "pub const MAX_ATTACHMENT_BYTES: usize = 25 \* 1024 \* 1024" ferro-notifications/src/channels/mail.rs   # FOUND
$ grep -q "pub attachments: Vec<MailAttachment>" ferro-notifications/src/channels/mail.rs     # FOUND
$ grep -q "pub fn attachment" ferro-notifications/src/channels/mail.rs                        # FOUND
$ grep -q "Result<Self, crate::Error>" ferro-notifications/src/channels/mail.rs               # FOUND
$ grep -q "pub use mail::{MailAttachment, MailMessage}" ferro-notifications/src/channels/mod.rs    # FOUND
$ grep -q "MailAttachment" ferro-notifications/src/lib.rs                                     # FOUND
$ git log --oneline | grep 415ea233                                                           # FOUND (Task 1)
$ git log --oneline | grep 7721495e                                                           # FOUND (Task 2)
$ cargo build -p ferro-notifications                                                          # exit 0
$ cargo fmt --all -- --check                                                                  # exit 0
$ cargo clippy --all --all-targets -- -D warnings                                             # exit 0
$ cargo test -p ferro-notifications channels::mail::tests                                     # 8/8 pass
$ cargo test --all-features                                                                   # all tests pass
```

---

*Phase: 149-ferro-notifications-whatsapp-inapp-channels-mailmessage-attachment*
*Completed: 2026-04-28*
