---
phase: 261
slug: asset-ergonomics
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-07-26
---

# Phase 261 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) + `trybuild 1.x` (proc-macro UI tests) |
| **Config file** | `ferro-macros/Cargo.toml` `[dev-dependencies]` already includes `trybuild = "1"` |
| **Quick run command** | `cargo test -p ferro-bundle -p ferro-macros -p ferro-cli -- --test-threads=1` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~60–120 seconds (workspace clippy dominates) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-bundle -p ferro-macros -p ferro-cli -- --test-threads=1`
- **After every plan wave:** full CI-exact gate (`fmt --check` + `clippy --all --all-targets -D warnings` + `test --all-features`)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~120 seconds

*Note (project rule): serialize CPU-heavy cargo runs — one at a time, never parallel/chained; reuse a wave's test evidence rather than re-running.*

---

## Per-Task Verification Map

| Task Group | Wave | Requirement / SC | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|-----------|------|------------------|-----------------|-----------|-------------------|-------------|--------|
| Decouple ferro-bundle from `ferro-rs` (D-06) | 0 | LIVE-03 enabler | N/A | build | `cargo build -p ferro-bundle` (no `ferro-rs` dep) | ❌ W0 | ⬜ pending |
| `mime_from_ext` helper in ferro-bundle | 1 | SC-2 | Unknown ext → `application/octet-stream` (no arbitrary CT injection) | unit | `cargo test -p ferro-bundle test_mime_from_ext` | ❌ W0 | ⬜ pending |
| Unknown-ext passthrough | 1 | SC-2 | bytes byte-identical | unit | `cargo test -p ferro-bundle mime_from_ext_unknown_is_octet_stream` | ❌ W0 | ⬜ pending |
| Hash determinism (existing guard) | 1 | SC-1 | stable content-addressed URL | unit | `cargo test -p ferro-bundle hash_is_deterministic` | ✅ exists | ⬜ pending |
| Duplicate-name panic guard (existing) | 1 | SC-1 | single registration per callsite | unit | `cargo test -p ferro-bundle duplicate_name_panics` | ✅ exists | ⬜ pending |
| `ferro::bundle` re-export + `Request→HttpResponse` adapter in framework | 1 | SC-1 (D-06) | N/A | build/unit | `cargo test -p ferro-rs bundle` | ❌ W0 | ⬜ pending |
| `asset!()` proc-macro (`OnceLock` + `include_bytes!` + `&'static str`) | 2 | SC-1, LIVE-03 | path is a compile-time literal (no runtime path traversal) | trybuild | `cargo test -p ferro-macros --test asset_macro` | ❌ W0 | ⬜ pending |
| `asset!()` hashed-URL stability across calls | 2 | SC-1 | idempotent registration | unit/integration | `cargo test -p ferro-macros` (or app-level render test) | ❌ W0 | ⬜ pending |
| `ferro assets fetch iconify <set>` → `.svg` in `assets/` | 2 | SC-3 | writes only under `--out`/`assets/`; HTTPS via rustls | integration (tempdir) | `cargo test -p ferro-cli assets_fetch_iconify` | ❌ W0 | ⬜ pending |
| `ferro assets fetch fontsource <family>` → `.woff2` in `assets/` | 2 | SC-3 | same | integration (tempdir) | `cargo test -p ferro-cli assets_fetch_fontsource` | ❌ W0 | ⬜ pending |
| Fetch runs on Rust toolchain alone | 2 | SC-3 | no nasm/node/OpenSSL (rustls-tls) | structural | `cargo build -p ferro-cli` on CI + `cargo tree` shows no native TLS | ✅ CI enforces | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] **Decouple ferro-bundle:** remove `ferro-rs` from `ferro-bundle/Cargo.toml`; lift `serve_inner` to a framework-agnostic public API; move ferro-bundle Wave 3 → Wave 1a in `.github/workflows/publish.yml`. (Unblocks the `ferro::bundle` re-export the macro emits.)
- [ ] `ferro-bundle/src/lib.rs` — add `mime_from_ext` (invert the 13-entry `ext_from_content_type` table) + unit tests `test_mime_from_ext`, `mime_from_ext_unknown_is_octet_stream`.
- [ ] `ferro-macros/src/asset.rs` — new file with `asset_impl()`; register `#[proc_macro] pub fn asset` in `ferro-macros/src/lib.rs`.
- [ ] `ferro-macros/tests/asset_macro.rs` — trybuild harness (`t.pass("tests/ui/asset/pass/*.rs")`).
- [ ] `ferro-macros/tests/ui/asset/pass/minimal.rs` + `ferro-macros/tests/ui/asset/pass/fixture.js` — a REAL committed asset file the fixture embeds (trybuild compiles it, so the file must exist).
- [ ] `framework` — `pub mod bundle { pub use ferro_bundle::Bundle; }` + `Request→HttpResponse` adapter; add `ferro-bundle` dep.
- [ ] `ferro-cli/src/commands/assets.rs` — new `assets fetch` command module; register in `commands/mod.rs` + the clap `Commands` enum in `main.rs`; tempdir integration tests.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live Iconify/Fontsource download reaches the real network | SC-3 | Author-time network call; CI/unit tests use tempdir + should not hit the network on every run | Run `ferro assets fetch iconify lucide/home` and `ferro assets fetch fontsource inter` in a scratch dir; confirm a `.svg` and a `.woff2` land under `assets/`. Automated tests may mock or gate the network. |

*The three success criteria otherwise have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have an `<automated>` verify or a Wave 0 dependency
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING (❌ W0) references
- [ ] No watch-mode flags
- [ ] Feedback latency < 120s
- [ ] `nyquist_compliant: true` set in frontmatter (by planner/executor once Wave 0 lands)

**Approval:** pending
