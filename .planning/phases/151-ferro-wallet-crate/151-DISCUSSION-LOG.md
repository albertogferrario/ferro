# Phase 151: ferro-wallet — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 151-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 151-ferro-wallet-crate
**Mode:** `--auto` (single-pass augmentation of prior context)
**Areas discussed:** Architecture split · Config loading · Image pipeline · Error shape · Apple cryptography · Apple colour derivation · Google ID format · Google JWT shape · Test strategy · Workspace + release · Scaffold ordering

---

## Mode and Provenance

This phase entered `/gsd-discuss-phase 151 --auto` with a pre-existing `151-CONTEXT.md` containing decisions D-01 through D-11 already captured (likely from a manual session on 2026-05-11 03:46, alongside the spec at `docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md`). Per workflow `check_existing` rules under `--auto`: auto-selected "Update it" and carried forward existing decisions verbatim. Per the auto-mode pass cap, this is a single-pass augmentation — no re-decisions invented, no spec re-derivation.

The augmentation added the four mandatory template sections that were missing:
- `<canonical_refs>` (mandatory per workflow success criteria)
- `<code_context>` (reusable assets + integration points)
- `<specifics>` (named-and-rejected alternatives)
- `<deferred>` (explicit out-of-scope items, hoisted from `<domain>`)

---

## Architecture split — D-01

| Option | Description | Selected |
|--------|-------------|----------|
| Two builders, separate | `ApplePassBuilder` and `GoogleWalletBuilder` as independent types; only `WalletSubject` shared | ✓ |
| Unified `WalletBuilder` enum | One type, dispatch internally on platform target | |
| Unified `WalletBuilder` trait | Shared trait, two impls | |

**Rationale:** Apple's wire format is PKCS#7-signed ZIP; Google's is RS256 JWT pointing at JSON. They share zero bytes. A unified surface would obscure format-specific failure modes (Apple WWDR chain, Google service-account auth) and gain no shared code.

---

## Config loading — D-02

| Option | Description | Selected |
|--------|-------------|----------|
| Permissive (missing cluster ⇒ None) | `apple: Option<AppleConfig>`, `google: Option<GoogleConfig>`, never errors on absent env vars | ✓ |
| Strict (require both clusters) | Fail-fast at boot if either Apple or Google config incomplete | |
| Single-platform-required (Apple OR Google) | Require at least one cluster present | |

**Rationale:** Many ferro apps will deploy Apple-only or Google-only. Callers gate behaviour on `WalletConfig.apple.is_some()` / `.google.is_some()`. Matches `ferro-stripe`'s optional-cluster pattern.

---

## Image pipeline — D-03

| Option | Description | Selected |
|--------|-------------|----------|
| Fit + centre-pad on transparent canvas | Resize preserving aspect, pad to target dims with alpha=0 | ✓ |
| Stretch to fit | Anisotropic resize | |
| Crop-to-fit (centre crop) | Cover-style crop | |

**Rationale:** Apple/Google pass artwork is rendered against the pass background; transparent padding preserves brand artwork shape across logo aspect ratios. `apple_icon_set` falls back to centre-square-crop when caller supplies no explicit icon (different concern — icons are square-by-spec).

---

## Error type — D-04

| Option | Description | Selected |
|--------|-------------|----------|
| Name-prefixed `Display` per variant | `"config: …"`, `"apple sign: …"`, etc. — production log greps stay surgical | ✓ |
| Generic `Display` (just inner message) | Less ceremonious | |
| Single opaque `WalletError(String)` | Loses categorisation | |

**Rationale:** Matches `ferro-stripe::error` style; ops-friendly when greping CloudWatch / journald for failure categories.

---

## Apple cryptography — D-05

| Option | Description | Selected |
|--------|-------------|----------|
| openssl PKCS#7 detached | `Pkcs7Flags::DETACHED \| Pkcs7Flags::BINARY`, DER-encoded, WWDR pushed onto single-element `Stack<X509>` | ✓ |
| `cms` crate (pure Rust) | Avoid openssl dep | |
| Shell out to `openssl smime` | External binary | |

**Rationale:** openssl is already a transitive dep across the workspace (TLS, hashing). The `cms` crate is less battle-tested for Apple WWDR chains specifically. Shell-out is hostile in containers without openssl-cli.

---

## Apple foreground / label colour derivation — D-06

| Option | Description | Selected |
|--------|-------------|----------|
| ITU-R BT.601 luminance threshold | `< 0.5` ⇒ white `rgb(255,255,255)`; `>= 0.5` ⇒ dark slate `rgb(17,24,39)` | ✓ |
| WCAG contrast solver | Pick foreground that maximises contrast ratio against background | |
| Always white | Universal default | |

**Rationale:** BT.601 matches what most "light vs dark" detection libraries use; cheap, deterministic, no contrast-ratio library dep. `labelColor` always tracks `foregroundColor` in v1 (spec §3 simplification).

---

## Google class + object ID format — D-07

| Option | Description | Selected |
|--------|-------------|----------|
| `{issuer_id}.{pass_type_id with dots→underscores}` for class, `{issuer_id}.{subject.serial()}` for object; fixed `"booking"` pass-type suffix | ✓ |
| Match Apple's `passTypeIdentifier` byte-for-byte | Google rejects dots in segments | |
| Caller-supplied class ID | Pushes complexity outward | |

**Rationale:** Google has no equivalent to Apple's pass-type-id. v1 uses a fixed `"booking"` suffix via `pass_type_id_default()` const; downstream callers needing a different logical class can fork in a future phase. Replacing dots with underscores satisfies Google's class-ID validation.

---

## Google JWT shape — D-08

| Option | Description | Selected |
|--------|-------------|----------|
| RS256 with `iss/aud=google/typ=savetowallet/iat/origins/payload.eventTicketObjects[]` | Standard Google save-link shape | ✓ |
| ES256 | Smaller signatures | |
| Skip JWT and use API-direct flow | Requires server-to-server OAuth, much heavier | |

**Rationale:** Save-link JWT is the canonical "Google Pay" UX. ES256 not supported by `pay.google.com/gp/v/save/`. API-direct adds a service-account OAuth flow that's out of scope for v1.

---

## Test strategy without real credentials — D-09

| Option | Description | Selected |
|--------|-------------|----------|
| Self-signed crypto at test runtime | Apple: openssl-mint X.509 used as both signing cert and WWDR (well-formed for openssl). Google: in-test RSA keypair + decode roundtrip | ✓ |
| Fixture certificates checked into repo | Risk: cert expiry; key material in git | |
| Skip integration; unit-test components only | Loses ZIP structure + JWT roundtrip coverage | |

**Rationale:** Zero CI dependency on real Apple WWDR / Google service-account secrets. Tests assert the nine expected `.pkpass` files, key `pass.json` fields, and exact JWT claim shape.

---

## Workspace + release — D-10

| Option | Description | Selected |
|--------|-------------|----------|
| Workspace member + workspace patch-bump + auto-publish via existing GitHub Action | ✓ |
| Standalone crate outside the workspace | Loses shared lints / version pinning | |
| Manual `cargo publish` from local | Loses CI provenance | |

**Rationale:** Matches every other `ferro-*` crate's release flow. Bumps workspace `[workspace.package] version` (currently 0.2.23) on verification. Wave 1 placement in `.github/workflows/publish.yml` because the crate is a leaf (no internal workspace deps).

---

## Scaffold ordering — D-11

| Option | Description | Selected |
|--------|-------------|----------|
| Placeholder module files in Task 01; strip + restore re-exports per landed builder | Keeps `cargo check` green across plans | ✓ |
| All files empty until builders land | Breaks `cargo check` mid-phase | |
| All files complete in Task 01 | Forces monolithic plan, defeats the purpose of waves | |

**Rationale:** Task 01 stubs `// placeholder` lines so every subsequent plan compiles. `lib.rs` temporarily strips `pub use apple::ApplePassBuilder;` / `pub use google::GoogleWalletBuilder;` re-exports, restored in the plan that lands the corresponding builder body. Atomic-commit-friendly.

---

## Claude's Discretion

None for Phase 151. All 11 decisions are locked. The planner has no remaining "you decide" surface — task body code is reproduced in the downstream `wallet-passes.md` Phase A reference, and shape is fixed by the spec.

## Deferred Ideas

See `<deferred>` in `151-CONTEXT.md`. Summary:
- Apple Web Service Protocol (live updates / Express Mode)
- Google `objects.patch`
- Locale negotiation beyond raw passthrough
- Additional pass kinds (`Generic`, `Coupon`, `Boarding`, `StoreCard`) — declared but un-tested in v1
