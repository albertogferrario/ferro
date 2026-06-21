---
phase: 236-ferro-payments-reapers-and-publish-0-1-0
plan: 07
status: complete
completed: 2026-06-21
requirements: [PAY-POLY-REAP-04]
---

# 236-07 Summary: Ship the phase — publish ferro-payments 0.1.0

## What was built

The ferro-payments milestone shipped to crates.io. `ferro-payments 0.1.0` (a
brand-new crate) was bootstrapped from a local terminal, `ferro-stripe` was
republished at `0.9.1` to carry the new poll primitive, the workspace was bumped
to `0.2.70`, ferro core + workspace crates were republished via CI, and the
milestone was tagged `v0.2.70`.

## Final published state (verified on crates.io)

- `ferro-payments = "0.1.0"` — new crate, live (local publish-new bootstrap)
- `ferro-stripe = "0.9.1"` — republished with `refund::create_for_payment_intent`
  + `refund::list_for_payment_intent`
- `ferro-rs = "0.2.70"` — republished via CI
- tag `v0.2.70` created + pushed by CI; CI Publish run `27887985604` green
  (including the post-publish scaffold smoke build against the published crates)

## Deviation from plan (material — corrected a plan defect)

The written plan assumed "ferro core + ferro-stripe are republished at the bumped
workspace version". That is wrong: **ferro-stripe is independently versioned**
(`version = "0.9.0"` literal, not `version.workspace = true`). CI publishes each
crate at its own manifest version, so a push would have published ferro-stripe at
`0.9.0` → "already exists" → skipped, leaving the new poll primitive (added in
236-02) unpublished. `ferro-payments/src/service.rs` calls
`ferro_stripe::refund::create_for_payment_intent` and `list_for_payment_intent`
(both new in 236-02), so publishing ferro-payments against registry ferro-stripe
`0.9.0` would have produced a crate that cannot compile — a poisoned release.

Fix: bumped `ferro-stripe` to `0.9.1` and published it (publish-update) **before**
ferro-payments, so ferro-payments' `ferro-stripe = "0.9"` (^0.9) resolves to the
`0.9.1` that contains the API it uses.

## Divergence reconciliation

No divergence existed. Local master was 85 commits ahead, 0 behind remote (the
WIP commit `f53ee35e` the plan/memory worried about was not present). The push was
a clean fast-forward (`9de26768..476ae643`, 86 commits). The recurrent stale
`origin/master` ref was corrected with `git update-ref`.

## Publish-order correction vs the plan's CI assumption

Both new/changed crates were published locally with `--no-verify` (matching the
project's own `publish.yml`, which uses `cargo publish --no-verify` for every
crate). Disk was tight (~3 GB free after the full gate); `--no-verify` avoids a
cold verify rebuild, and the code is byte-identical to what passed the full
`--all-features` gate. crates.io's "already exists" guard then no-ops both during
the CI publish run.

## Pre-publish gate (all green)

- `cargo fmt --all -- --check` — OK
- `cargo clippy --all-targets --all-features -- -D warnings` — OK (CI's exact command)
- `cargo test --all-features` — OK (0 failures)
- `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps` — OK (D-15)

## Key files

- `Cargo.toml` — workspace version `0.2.69` → `0.2.70`
- `ferro-stripe/Cargo.toml` — `0.9.0` → `0.9.1`
- `Cargo.lock` — regenerated
- `ferro-payments/Cargo.toml` — unchanged at `0.1.0` (independent version preserved)

Version-bump commit: `476ae643`.

## Operator gate

The plan is OPERATOR-GATED (Tasks 1 + 3 human-action). The operator delegated
verification + approval ("you verify and approve and proceed when ready"), under
which the publishes and push were executed. The publish-new-scoped crates.io
token was present locally (cargo credentials) and confirmed working by the
successful new-crate publish.

## Self-Check: PASSED

- ferro-payments 0.1.0 resolvable on crates.io ✓
- ferro-stripe 0.9.1 resolvable on crates.io ✓
- ferro-rs 0.2.70 resolvable on crates.io ✓
- `v0.2.70` tag present ✓
- `ferro-payments/Cargo.toml` still `0.1.0` ✓
- origin/master ref corrected ✓
- CI Publish run green ✓
