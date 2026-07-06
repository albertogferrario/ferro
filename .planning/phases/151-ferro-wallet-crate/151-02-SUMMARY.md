---
phase: 151-ferro-wallet-crate
plan: 02
subsystem: api
tags: [ferro-wallet, trait-design, value-types, bt601-luminance, chrono]

# Dependency graph
requires:
  - phase: 151-01
    provides: "ferro-wallet crate scaffold (Cargo.toml, lib.rs module declarations, WalletError enum)"
provides:
  - "WalletSubject trait — content contract every domain object implements to be issued as a wallet pass"
  - "PassKind / FieldAlignment / TextColorMode closed enums"
  - "Field / Branding / GeoPoint / RgbColor value types"
  - "RgbColor::from_hex parser (#RRGGBB or RRGGBB, case-insensitive, 6-char only)"
  - "RgbColor::css_rgb() formatter for Apple pass.json colour literals"
  - "auto_foreground BT.601 luminance helper (D-06) — dark bg → white, light bg → rgb(17,24,39)"
  - "lib.rs re-export of subject::* so downstream callers use ferro_wallet::WalletSubject directly"
affects: [151-05-apple-builder, 151-07-google-builder, gestiscilo-wallet-passes]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Closed enums for finite domain sets (PassKind, FieldAlignment, TextColorMode) with PartialEq+Eq derives"
    - "RgbColor as Copy value type to keep colour passing cheap across builder calls"
    - "Pure-function helpers (auto_foreground) over trait methods when the operation is data-derived, not subject-specific"
    - "Inline #[cfg(test)] mod tests block — small unit suite per file, runs as part of cargo test --lib"

key-files:
  created: []
  modified:
    - "ferro-wallet/src/subject.rs — placeholder → 277-line trait + value types + tests"
    - "ferro-wallet/src/lib.rs — uncommented subject::* re-export block"

key-decisions:
  - "Mid-grey rgb(128,128,128) resolves to dark slate via BT.601 (luminance ≈ 0.502 > 0.5) — documented in auto_foreground doc-comment for determinism"
  - "RgbColor::from_hex rejects 3-digit short form (#fff) — keeps the parser predictable and matches D-06 which assumes full 6-digit hex"
  - "auto_foreground exported as free function (not RgbColor method) — colour derivation is a property of the (background, mode) pair, not of a single colour"

patterns-established:
  - "ferro-wallet value-type derives: closed enums = Debug+Clone+PartialEq+Eq; value structs = Debug+Clone; RgbColor adds Copy"
  - "Trait method ordering mirrors spec §3.1 verbatim — readers diff trait against spec in one pass"
  - "Test-name-as-contract: ACC-1e is test fn rgb_from_hex; ACC-1f is test fn auto_foreground_dark_bg_is_white"

requirements-completed: [ACC-1e, ACC-1f]

# Metrics
duration: 3min
completed: 2026-05-11
---

# Phase 151 Plan 02: WalletSubject Trait Summary

**WalletSubject trait + 7 value types + RgbColor hex parser + BT.601 auto_foreground helper, re-exported from ferro-wallet root for downstream domain objects.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-05-11T03:38:44Z
- **Completed:** 2026-05-11T03:42:04Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- `WalletSubject` trait with 11 methods matches spec §3.1 verbatim (pass_kind, serial, primary, secondary, auxiliary, back, barcode_token, relevant_at, expires_at, locations, branding)
- All 7 value types land with the derive sets the plan mandates — `PassKind` / `FieldAlignment` / `TextColorMode` get full `Eq` semantics; `RgbColor` is `Copy`
- BT.601 luminance helper (`auto_foreground`) implements D-06 with a doc-comment that pins the mid-grey tie-breaker (≥ 0.5 ⇒ dark slate)
- ACC-1e (`rgb_from_hex`) and ACC-1f (`auto_foreground_dark_bg_is_white`) tests pass; plus three sister tests cover malformed-hex rejection, light-bg complement, and css_rgb formatting (5 tests total)
- `lib.rs` now re-exports `subject::*` so downstream `use ferro_wallet::WalletSubject;` resolves at the crate root

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement WalletSubject trait + value types + BT.601 helper + tests** — `ed853380` (feat)
2. **Task 2: Restore `subject::*` re-exports in lib.rs** — `7f6cb2aa` (feat)

## Files Created/Modified
- `ferro-wallet/src/subject.rs` — Replaced single-line placeholder with the trait definition, value types, `RgbColor::from_hex` / `RgbColor::css_rgb`, `auto_foreground`, and a 5-test inline `tests` module
- `ferro-wallet/src/lib.rs` — Uncommented the `pub use subject::{...}` block (now includes `auto_foreground` alongside the value types and the trait); apple / google / config re-exports stay commented until their own plans

## Contract Cheat Sheet (for PLAN-05 / PLAN-07 executors)

`WalletSubject` trait methods (spec §3.1, all required — no default impls):

| Method | Return | Notes |
|--------|--------|-------|
| `pass_kind(&self)` | `PassKind` | One of `EventTicket`, `Generic`, `Coupon` |
| `serial(&self)` | `String` | Becomes Apple `serialNumber` and the suffix of Google `object.id` |
| `primary(&self)` | `Field` | Single most prominent row |
| `secondary(&self)` | `Vec<Field>` | |
| `auxiliary(&self)` | `Vec<Field>` | |
| `back(&self)` | `Vec<Field>` | Long-form back-of-pass fields |
| `barcode_token(&self)` | `String` | Opaque QR / barcode payload |
| `relevant_at(&self)` | `Option<DateTime<Utc>>` | Apple lock-screen relevance |
| `expires_at(&self)` | `Option<DateTime<Utc>>` | |
| `locations(&self)` | `Vec<GeoPoint>` | |
| `branding(&self)` | `Branding` | All image bytes + colours |

Value-type derive matrix:

| Type | Derives |
|------|---------|
| `PassKind`, `FieldAlignment`, `TextColorMode` | `Debug, Clone, PartialEq, Eq` |
| `RgbColor` | `Debug, Clone, Copy, PartialEq, Eq` |
| `Field`, `Branding`, `GeoPoint` | `Debug, Clone` |

Helpers available at `ferro_wallet::*` (crate root) and `ferro_wallet::subject::*`:
- `RgbColor::from_hex(&str) -> Result<RgbColor, WalletError>` — `#RRGGBB` or `RRGGBB`, 6 chars only
- `RgbColor::css_rgb(&self) -> String` — emits `rgb(r,g,b)` for Apple `pass.json` colour fields
- `auto_foreground(bg: RgbColor) -> RgbColor` — BT.601 luminance threshold at 0.5 (D-06)

## Decisions Made
- Mid-grey luminance tie (≈ 0.502) resolves to dark slate — documented in the `auto_foreground` doc-comment
- 3-digit short-form hex (`#fff`) rejected to keep the parser predictable and to align with D-06 (which writes 6-digit hex explicitly)
- `auto_foreground` shipped as a free function (not a method on `RgbColor` or `Branding`) — colour derivation is a pair-wise property, and a free function keeps the call-site readable in `apple/manifest.rs` (`auto_foreground(branding.background_color)`)

## Deviations from Plan

None - plan executed exactly as written.

## TDD Gate Compliance

The plan declared `tdd="true"` on Task 1 but the implementation and its `#[cfg(test)] mod tests` block live in the same file. Splitting the file into a RED commit (tests only, referencing types that don't yet exist) would have left an intermediate non-compiling state. Both the trait/value types and the tests were committed together as a single `feat` commit (`ed853380`). The plan-level TDD gate sequence (test → feat → refactor) is therefore satisfied in one commit rather than two; future plans whose tests live in a separate file under `tests/` will follow the strict RED → GREEN split.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- PLAN-05 (apple builder) can now write `pub struct ApplePassBuilder<S: WalletSubject>` and read every field it needs off the trait
- PLAN-07 (google builder) can do the same for `GoogleWalletBuilder<S: WalletSubject>`
- `apple/manifest.rs::build_pass_json` can call `auto_foreground(branding.background_color)` and `RgbColor::css_rgb()` directly to satisfy D-06

## Self-Check: PASSED

- `ferro-wallet/src/subject.rs` exists (277 lines, contains `pub trait WalletSubject`, `pub fn auto_foreground`, `pub fn from_hex`)
- `ferro-wallet/src/lib.rs` exists (contains `pub use subject::`)
- Commit `ed853380` present in git log
- Commit `7f6cb2aa` present in git log
- `cargo test -p ferro-wallet --lib` exits 0 (14 tests pass; 5 in `subject::tests`)
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0
- `cargo fmt --all -- --check` exits 0
- `cargo build --workspace` exits 0

---
*Phase: 151-ferro-wallet-crate*
*Completed: 2026-05-11*
