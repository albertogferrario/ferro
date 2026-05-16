# Phase 156: frontend/src/types/ — Generator-Owned Convention Cleanup — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-14
**Phase:** 156-frontend-types-directory-generator-owned-convention
**Mode:** --auto (all gray areas auto-resolved with recommended defaults)
**Areas discussed:** Generator header fix, Doctor check heuristic, FERRO_VERSION source, ferro setup existence, Dockerfile renderer

---

## Generator Header Fix

| Option | Description | Selected |
|--------|-------------|----------|
| Fix to `frontend/src/lib/types/` | Corrects the comment to match the D-02/D-03 convention | ✓ |
| Leave as `frontend/src/types/` | Keeps the contradiction in the codebase | |

**Auto-selected:** Fix to `frontend/src/lib/types/`
**Notes:** Codebase scan confirmed `generate_types.rs` lines 710-711 direct users to `frontend/src/types/` for custom types — directly contradicts the convention. In-scope per §7. Added as D-18.

---

## Doctor Check Heuristic

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit allowlist (`inertia-props.ts`, `routes.ts`) | Files NOT in this list = hand-written. Simple, no false positives. | ✓ |
| Any file in directory | Flag everything in `frontend/src/types/` if directory exists | |

**Auto-selected:** Explicit allowlist
**Notes:** Generator confirmed to emit exactly `inertia-props.ts` and `routes.ts` from codebase scan of `generate_types.rs` `run()` function. Explicit list avoids false positives if directory is populated by other tools in future. Added as D-20.

---

## FERRO_VERSION Source

| Option | Description | Selected |
|--------|-------------|----------|
| Parse from `Cargo.lock` | Look up `ferro-rs` package; most accurate, matches what project actually compiles against | ✓ |
| Parse from `Cargo.toml` dep specifier | Simpler but may be a version range, not a resolved version | |
| Use current binary's own version | `env!("CARGO_PKG_VERSION")` — mismatch risk when binary version ≠ project dep version | |

**Auto-selected:** Parse from Cargo.lock
**Notes:** D-16 already preferred this approach; codebase scan confirmed `DockerContext` has no `ferro_version` field yet. Fallback to `env!("CARGO_PKG_VERSION")` if Cargo.lock parse fails. Added as D-21.

---

## ferro setup Command

| Option | Description | Selected |
|--------|-------------|----------|
| Does not exist — docs say "run `cargo run` once" | Accurate; no `setup.rs` in `ferro-cli/src/commands/` | ✓ |
| ferro setup exists | N/A | |

**Auto-selected:** Does not exist
**Notes:** Confirmed from codebase scan — no `setup.rs` in commands directory. D-11 and D-08 docs updated accordingly. Added as D-19.

---

## ferro doctor Existence

| Option | Description | Selected |
|--------|-------------|----------|
| Already exists — extend registry | Confirmed: `ferro-cli/src/commands/doctor.rs` + `ferro-cli/src/doctor/registry.rs` with 10 checks | ✓ |
| Does not exist — create skeleton | N/A | |

**Auto-selected:** Already exists
**Notes:** Confirmed from codebase scan. New `FrontendTypesConventionCheck` follows existing pattern (one file per check, struct + DoctorCheck impl, check_impl() fn, TempDir tests).

---

## Claude's Discretion

- Exact docs page path (`docs/dx/frontend-types.md` vs another location)
- Whether `debug_assert!` in `render_dockerfile` needs updating for new `{{FERRO_VERSION}}` token
- CI Docker build verification approach

## Deferred Ideas

- Making type generator output deterministic
- CI drift-check
- Migration scripts for consumer apps
- `ferro setup` bootstrap command
- `ferro doctor` check for outdated rendered Dockerfiles
- Cheaper alternatives to `cargo install` (cargo-binstall, prebuilt binaries)
