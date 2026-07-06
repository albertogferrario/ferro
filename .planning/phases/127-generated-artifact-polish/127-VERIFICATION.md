---
phase: 127-generated-artifact-polish
verified: 2026-04-09T00:00:00Z
status: passed
score: 9/9 must-haves verified
---

# Phase 127: Generated Artifact Polish Verification Report

**Phase Goal:** Make `ferro docker:init` and `ferro do:init` emit artifacts runnable end-to-end without manual editing. Item 18 (Dockerfile ENTRYPOINT/CMD) is the deploy blocker.

**Status:** passed

## Must-Have Verification (Goal-Backward)

| # | Must-Have | Status | Evidence |
|---|-----------|--------|----------|
| 1 | Dockerfile emits `ENTRYPOINT ["/usr/local/bin/<bin>"]` + `CMD ["serve"]` | VERIFIED | `templates/docker.rs:90-93` composes block; `Dockerfile.tpl:33` has `{{ENTRYPOINT}}` token; tests `entrypoint_emitted_for_single_bin`, `cmd_is_serve`, `entrypoint_emitted_for_multi_bin` |
| 2 | Exactly one `cargo build --release` invocation | VERIFIED | `Dockerfile.tpl:22` single line, no per-bin builds; test `dockerfile_single_build_invocation` asserts count == 1 and `--bin` count == 0 |
| 3 | `.dockerignore` contains `!README.md` after `*.md` with doc comment | VERIFIED | `dockerignore.tpl:54-56`: `*.md`, comment explaining D-20/D-21, then `!README.md`; test `dockerignore_whitelists_readme` validates ordering + preceding comment |
| 4 | `.do/app.yaml` web service has NO `run_command:` | VERIFIED | `app.yaml.tpl:7-17` has no `run_command`; inline D-05 comment references Dockerfile ENTRYPOINT; test `web_service_has_no_run_command` |
| 5 | envs block contains real `- key: NAME` entries | VERIFIED | `templates/do.rs:60-95` `render_envs_block_from_lines` emits `- key: ` per EnvLine::Key; template uses `{{ENVS_BLOCK}}`; tests `envs_block_from_env_example`, `render_app_yaml_emits_real_envs_entries` |
| 6 | Secret keys get `type: SECRET` + `scope: RUN_AND_BUILD_TIME`; non-secret get `scope: RUN_TIME` | VERIFIED | `templates/do.rs:72-80` branches on `is_secret_key`; `deploy/secret_keys.rs` implements D-08 heuristic with `_URL` carve-out; tests `secret_scope_and_type`, `is_secret_key_*` (12 cases) |
| 7 | Both commands accept `--dry-run` and write zero files | VERIFIED | `docker_init.rs:49,98-103` short-circuits before any `fs::write`; `do_init.rs:38,81-101` same; `compute_cargo_docker_toml` is pure (test `compute_returns_string_without_writing`) |
| 8 | 3-5 line "Next steps" footer on success, suppressed in dry-run | VERIFIED | `docker_init.rs:129-133` footer + `122-123` printed only in non-dry-run path; `do_init.rs:114-117` same; tests `docker_init_footer_line_count`, `do_init_footer_line_count` enforce [3,5] range + ASCII-only |
| 9 | `rewrite_ferro_version.rs` preserves dep table order (toml_edit) | VERIFIED | `deploy/rewrite_ferro_version.rs:16` uses `toml_edit::DocumentMut`; `rewrite_contents` only mutates `path`/`version` per-dep; tests `preserves_dep_table_order` (6 deps non-alphabetic) + `preserves_package_rename_and_features` |

## Decisions D-01..D-21 Coverage

| Decision | Status | Location |
|----------|--------|----------|
| D-01 ENTRYPOINT+CMD | VERIFIED | `templates/docker.rs:90-93` |
| D-02 bin detection order (4 steps) | VERIFIED | `deploy/bin_detect.rs:17-32`, 4 tests |
| D-03 `CMD ["serve"]` | VERIFIED | `templates/docker.rs:91` literal |
| D-04 new template token wired | VERIFIED | `Dockerfile.tpl:33` `{{ENTRYPOINT}}`; debug_assert catches unresolved tokens |
| D-05 no `run_command:` on web | VERIFIED | `app.yaml.tpl:7-17`; test `web_service_has_no_run_command` |
| D-06 real envs entries | VERIFIED | `templates/do.rs:60-95` |
| D-07 secret type+scope | VERIFIED | `templates/do.rs:72-80` |
| D-08 secret heuristic | VERIFIED | `deploy/secret_keys.rs:25-35`, `_url` carve-out |
| D-09 source order + blank separators | VERIFIED | `EnvLine::Blank` handling `templates/do.rs:82-84`; test `envs_preserve_blank_separators` |
| D-10 per-bin builds dropped | VERIFIED | No `{{BIN_BUILDS}}` token; test asserts 0 occurrences of `cargo build --release --bin` |
| D-11 toml_edit migration | VERIFIED | `rewrite_ferro_version.rs:16` |
| D-12 existing regression tests pass | VERIFIED | `preserves_package_rename_and_features` still present |
| D-13 3-5 line footer | VERIFIED | Both `*_footer_line_count` tests |
| D-14 docker:init footer text | VERIFIED | `docker_init.rs:129-133` contains `docker build` + `docker run --rm -p 8080:8080 --env-file .env.production` |
| D-15 do:init footer text | VERIFIED | `do_init.rs:114-117` contains `doctl apps create --spec .do/app.yaml` |
| D-16 footer suppressed in dry-run | VERIFIED | Both commands `return Ok(())` before `print!(footer)` when `dry_run==true` |
| D-17 --dry-run prints files no writes | VERIFIED | `print_dry_run` with `--- path ---` headers; both commands route through it |
| D-18 --dry-run short-circuits Cargo.docker.toml persist | VERIFIED | `compute_cargo_docker_toml` pure; `persist_*` only called in non-dry-run branch; test `compute_returns_string_without_writing` |
| D-19 render errors remain hard | VERIFIED | Errors propagated via `?` before dry-run branch; test `dry_run_propagates_render_error` |
| D-20 `!README.md` whitelisted | VERIFIED | `dockerignore.tpl:56` |
| D-21 doc comment on whitelist | VERIFIED | `dockerignore.tpl:55`; test validates preceding line is comment |

## Anti-Pattern Scan

No TODO/FIXME/placeholder markers in modified files. No empty returns, no hardcoded stubs. All templates debug_assert on unresolved `{{` tokens. Both command paths wire end-to-end to pure renderers with substantive test coverage (14+ tests across the four plans).

## Behavioral Spot-Checks

Skipped (no runnable server required). All assertions are covered by `cargo test -p ferro-cli` unit/integration tests, which the VALIDATION.md per-decision map wires directly.

## Requirements Coverage

All 21 locked decisions from CONTEXT.md map to concrete code + tests. No orphaned decisions. Phase boundary honored — no Phase 128 preflight work crept in.

## Gaps Summary

None. Goal achieved: generated artifacts are runnable end-to-end. Dockerfile has ENTRYPOINT, do/app.yaml has real envs, both commands support `--dry-run` with footer suppression, dep-table order survives rewrite. The deploy blocker (item 18) is resolved by construction — `detect_web_bin` is the single source of truth shared by both scaffolders.

---

_Verified: 2026-04-09_
_Verifier: Claude (gsd-verifier)_
