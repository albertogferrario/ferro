# Phase 261: `asset!()` ergonomics - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-26
**Phase:** 261-asset-ergonomics
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** Macro embedding & path resolution, Registration lifetime & caching, Content-type inference, Crate wiring, `ferro assets fetch` surface

---

## Macro embedding & path resolution

| Option | Description | Selected |
|--------|-------------|----------|
| Emit `include_bytes!` in expansion | Call-site-source-relative path resolution (Rust native) | ✓ |
| Read file at proc-macro time | Resolves against CWD/`CARGO_MANIFEST_DIR` — fragile | |

**Selected (auto):** Emit `include_bytes!` — spec-literal (§3), least-surprise path semantics.

## Registration lifetime & caching

| Option | Description | Selected |
|--------|-------------|----------|
| Lazy `static OnceLock<String>` at use site | Register once on first eval, cache URL; safe in hot path | ✓ |
| Eager boot-time registration | Would require a separate boot call — defeats one-liner intent | |
| Naive re-register per call | Panics on 2nd request (duplicate name) | |

**Selected (auto):** Lazy OnceLock; return `&'static str`; bundle name from sanitized path.
**Notes:** Duplicate-path collision surfaces on first evaluation, not boot — documented trade-off.

## Content-type inference

| Option | Description | Selected |
|--------|-------------|----------|
| ext→MIME helper in `ferro-bundle` | Single source of truth; bundle already owns ct→ext | ✓ |
| Reuse `ferro-assets::infer_content_type` | Returns transform enum, not MIME string — wrong surface | |
| Inline map in macro | Duplicates logic, not introspectable | |

**Selected (auto):** New `pub fn` in ferro-bundle; unknown → `application/octet-stream` passthrough.

## Crate wiring / re-export

| Option | Description | Selected |
|--------|-------------|----------|
| Re-export `ferro::bundle` from framework | Macro emits `::ferro::bundle::Bundle`, works downstream | ✓ |
| Expansion references `::ferro_bundle` directly | Forces consumer to add ferro-bundle dep explicitly | |

**Selected (auto):** Re-export from framework; macro reuses `crate::utils::ferro` root helper (memoize precedent).

## `ferro assets fetch` surface

| Option | Description | Selected |
|--------|-------------|----------|
| `assets fetch iconify/fontsource <name>` + reuse reqwest, out to `assets/` | Typed sources, existing pure-Rust HTTP dep, new consumer convention | ✓ |
| New HTTP crate / feature-gated network | Redundant — reqwest blocking+rustls already present | |
| Fetch bundles into a single JSON blob | Doesn't flow through per-file `asset!()` | |

**Selected (auto):** clap-derive subcommand group; reuse existing `reqwest` blocking+rustls-tls;
write individual servable files (`.svg`/`.woff2`) to `assets/` (`--out` override).

## Claude's Discretion

- Exact Iconify / Fontsource endpoint URLs + response formats (research to pin).
- `OnceLock` vs `LazyLock`; name-sanitization function; optional alias arg (recommend defer).

## Deferred Ideas

- Macro-emitted stable alias argument.
- Auto-wiring fetched assets into `asset!()` call sites / route generation.
- Delta-granular / list-diffing (live-fragment track, not assets).
