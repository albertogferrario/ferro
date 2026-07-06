---
phase: 127
slug: generated-artifact-polish
status: draft
nyquist_compliant: false
wave_0_complete: true
created: 2026-04-08
---

# Phase 127 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust, workspace) |
| **Config file** | `ferro-cli/Cargo.toml` (existing) |
| **Quick run command** | `cargo test -p ferro-cli` |
| **Full suite command** | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |
| **Estimated runtime** | ~90 seconds (quick), ~5 minutes (full) |

---

## Sampling Rate

- **After every task commit:** `cargo test -p ferro-cli`
- **After every plan wave:** `cargo test -p ferro-cli` (fast) + `cargo clippy -p ferro-cli --all-targets -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~90 seconds

---

## Per-Task Verification Map

Maps every locked decision in CONTEXT.md (D-01..D-21) to a verifiable assertion. The planner will assign each decision to a task and mirror these into `<acceptance_criteria>` blocks.

| Decision | Plan | Wave | Test Type | Automated Command / Assertion | Status |
|----------|------|------|-----------|-------------------------------|--------|
| D-01 (ENTRYPOINT + CMD present) | 02 | 2 | unit | `cargo test -p ferro-cli entrypoint_emitted_for_single_bin` — rendered Dockerfile contains `ENTRYPOINT ["/usr/local/bin/` and `CMD ["serve"]` | ⬜ pending |
| D-02 (bin detection order) | 01 | 1 | unit | `cargo test -p ferro-cli bin_detect_*` — 4 tests: explicit override, package-match, first-bin, package-name fallback | ⬜ pending |
| D-03 (CMD is `["serve"]`) | 02 | 2 | unit | `cargo test -p ferro-cli cmd_is_serve` — asserts `CMD ["serve"]` | ⬜ pending |
| D-04 (new token wired in docker.rs) | 02 | 2 | unit | Template renders without leaving `{{ENTRYPOINT}}` in output (grep assertion) | ⬜ pending |
| D-05 (no `run_command:` on web) | 03 | 2 | unit | `cargo test -p ferro-cli web_service_has_no_run_command` — `app.yaml` web block lacks `run_command:` | ⬜ pending |
| D-06 (real envs entries, not comments) | 03 | 2 | unit | `cargo test -p ferro-cli envs_block_from_env_example` — rendered `app.yaml` has `- key: ` lines matching each `.env.example` key | ⬜ pending |
| D-07 (secret type + scope) | 03 | 2 | unit | `cargo test -p ferro-cli secret_scope_and_type` — `STRIPE_SECRET_KEY` → `type: SECRET\n    scope: RUN_AND_BUILD_TIME`, `DATABASE_URL` → no `type:`, `scope: RUN_TIME` | ⬜ pending |
| D-08 (secret heuristic) | 01 | 1 | unit | `cargo test -p ferro-cli is_secret_key_*` — ≥6 parameterized cases including `_URL` carve-out and all substring hits | ⬜ pending |
| D-09 (source order + blank-line separators preserved) | 01 | 1 | unit | `cargo test -p ferro-cli env_example_parser_preserves_order` — parser returns `Vec<EnvLine>` with Blank variants | ⬜ pending |
| D-10 (per-bin builds dropped) | 02 | 2 | unit | `cargo test -p ferro-cli dockerfile_single_build_invocation` — rendered Dockerfile contains `cargo build --release` exactly once | ⬜ pending |
| D-11 (toml_edit preserves order) | 01 | 1 | unit | `cargo test -p ferro-cli preserves_dep_table_order` — round-trip of fixture Cargo.toml with 6 deps in non-alphabetic order | ⬜ pending |
| D-12 (existing rewriter tests still pass) | 01 | 1 | regression | `cargo test -p ferro-cli preserves_package_rename_and_features` continues to pass | ⬜ pending |
| D-13 (3-5 line cargo-style footer) | 04 | 3 | unit | `cargo test -p ferro-cli footer_line_count` — line count in [3,5], no emoji | ⬜ pending |
| D-14 (docker:init footer text) | 04 | 3 | unit | `cargo test -p ferro-cli docker_init_footer_contents` — footer contains `docker build` and `docker run` | ⬜ pending |
| D-15 (do:init footer text) | 04 | 3 | unit | `cargo test -p ferro-cli do_init_footer_contents` — footer contains `doctl apps create --spec` | ⬜ pending |
| D-16 (footer suppressed in --dry-run) | 04 | 3 | unit | `cargo test -p ferro-cli footer_suppressed_in_dry_run` | ⬜ pending |
| D-17 (--dry-run writes nothing) | 04 | 3 | integration | `cargo test -p ferro-cli dry_run_no_filesystem_writes` — tempdir snapshot before/after is identical | ⬜ pending |
| D-18 (--dry-run short-circuits Cargo.docker.toml rewrite) | 04 | 3 | integration | `cargo test -p ferro-cli dry_run_no_cargo_docker_toml_persisted` — file absent from tempdir | ⬜ pending |
| D-19 (render errors remain hard) | 04 | 3 | unit | `cargo test -p ferro-cli dry_run_propagates_render_error` — missing metadata still returns `Err` in dry-run | ⬜ pending |
| D-20 (`!README.md` whitelisted) | 02 | 2 | unit | `cargo test -p ferro-cli dockerignore_whitelists_readme` — rendered `.dockerignore` contains `!README.md` after `*.md` | ⬜ pending |
| D-21 (documentation comment on whitelist) | 02 | 2 | unit | Rendered `.dockerignore` contains a one-line comment near `!README.md` explaining why | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `cargo test` framework — already installed (Rust workspace)
- [x] Existing test files in `ferro-cli/src/templates/docker.rs`, `ferro-cli/src/templates/do.rs`, `ferro-cli/src/deploy/rewrite_ferro_version.rs` provide test harness patterns to follow
- [ ] New helper module `ferro-cli/src/deploy/bin_detect.rs` (Wave 1, plan 01)
- [ ] New helper module `ferro-cli/src/deploy/secret_keys.rs` (or inline) (Wave 1, plan 01)

Existing infrastructure covers framework-level needs. No new frameworks required.

---

## Manual-Only Verifications

| Behavior | Decision | Why Manual | Test Instructions |
|----------|----------|------------|-------------------|
| `gestiscilo` smoke test reaches app code | D-01..D-04 | Requires the external gestiscilo app repo and a real Docker daemon; cannot run in `cargo test` | After phase: `cd ../gestiscilo-it/app && ferro docker:init --force && docker build -t gestiscilo:test . && docker run --rm -p 8080:8080 --env-file .env.production gestiscilo:test` — container should reach app code (fail on DB connect or run), NOT exit 0 silently |
| `doctl apps create --spec .do/app.yaml` passes DO platform validation | D-05..D-09 | Requires a DO account and `doctl` CLI | After phase: `ferro do:init --force && doctl apps create --spec .do/app.yaml --dry-run` or equivalent validation |

---

## Validation Sign-Off

- [ ] All 21 decisions map to at least one automated verification
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 helpers created before Wave 2 consumers
- [ ] No watch-mode flags
- [ ] Feedback latency < 90s
- [ ] `nyquist_compliant: true` set in frontmatter after planner wires tests into plan `<acceptance_criteria>`

**Approval:** pending
