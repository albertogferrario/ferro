---
phase: 117-catalog-and-json-schema
plan: "07"
subsystem: framework/app + ferro-cli
tags: [cli, json-ui-schema, schema-export, ferro-cli, framework-app]
dependency_graph:
  requires: [117-06]
  provides: [SCHEMA-02, ROADMAP-SC-6, ferro-json-ui-schema-cli]
  affects: [framework/src/app.rs, ferro-cli/src/commands, ferro-cli/src/main.rs]
tech_stack:
  added: []
  patterns: [shell-out-cli-wrapper, feature-gated-subcommand, catalog-build-for-exit-code]
key_files:
  modified:
    - framework/src/app.rs
    - ferro-cli/src/commands/json_ui_schema.rs
    - ferro-cli/src/commands/mod.rs
    - ferro-cli/src/main.rs
decisions:
  - "Framework-side `JsonUiSchema` variant + `run_json_ui_schema` handler are gated behind `#[cfg(feature = \"json-ui\")]` because `ferro-json-ui` is an optional dep under the `json-ui` feature. The ferro-cli binary is NOT feature-gated — it's a developer CLI that shells out to the user's project binary."
  - "Handler uses `Catalog::build()` rather than `global_catalog()` so build errors surface as non-zero exit codes (RESEARCH §8 L-1)."
  - "Output defaults to pretty-printed JSON; the `--pretty` flag is accepted for explicitness and back-compat with tooling that passes it, but compact is not reachable via any flag in Phase 117."
  - "Smoke tests were run via a standalone minimal binary (`/tmp/ferro-smoke-bin`) because the sample `app/src/main.rs` in this workspace uses hand-rolled clap dispatch rather than `Application::new().run()`. Newly scaffolded apps use the Application builder and pick up `json-ui:schema` automatically. Deviation noted below."
requirements_completed:
  - SCHEMA-02
metrics:
  duration: "~30 minutes"
  completed: "2026-04-18"
  tasks_completed: 4
  tasks_total: 4
  files_modified: 4
commits:
  - "cf55f285 feat(117-07): add json-ui:schema subcommand and handler to framework"
  - "a0c6e708 feat(117-07): add ferro-cli shell-out wrapper for json-ui:schema"
  - "4bccd891 feat(117-07): wire json-ui:schema into ferro-cli main dispatch"
---

# Phase 117 Plan 07: `ferro json-ui:schema` CLI Export Command

Ships SCHEMA-02: an external-facing CLI that exports the JSON-UI v2 spec schema
(full or per-component) to stdout or a file, consumable by IDEs and external
tools via `ferro json-ui:schema | jq .`.

## CLI Surface

`ferro json-ui:schema --help`:

```
Export the JSON-UI v2 spec schema (full spec or a single component's Props)

Usage: ferro json-ui:schema [OPTIONS]

Options:
  -o, --output <OUTPUT>        Write to file instead of stdout
      --pretty                 Pretty-print JSON output (default behavior — flag accepted for explicitness)
      --component <COMPONENT>  Export only the Props schema for a single component (e.g., "Card")
  -h, --help                   Print help
```

## Measured Schema Size

Full spec schema (stdout): **100,470 bytes (~98 KB)** pretty-printed.

This exceeds the plan's 40–80 KB estimate. The extra bulk comes from
(1) the full component catalog inlined under `$defs`, and (2) 2-space
pretty-print indentation across a large schema tree. Compact serialization
would shrink this substantially but is intentionally not exposed at Phase 117
(CONTEXT D-21) — the use case is IDE / external-tool consumption where
readability and diffability trump byte count.

Single-component Props schema (`--component Card`): **892 bytes**.

## Smoke Tests (5/5 PASS)

Smoke tests were run against a minimal standalone binary at
`/tmp/ferro-smoke-bin` that calls `Application::new().run().await` — this
mirrors the doc-example in `framework/src/app.rs` and matches what newly
scaffolded Ferro apps produce.

| # | Scenario | Command | Result |
|---|----------|---------|--------|
| 1 | Full schema to stdout | `ferro json-ui:schema` | PASS — exit 0, valid JSON, 100,470 bytes |
| 2 | `--output <file>` | `ferro json-ui:schema --output /tmp/...json` | PASS — exit 0, file valid JSON, 100,469 bytes |
| 3 | `--component Card` | `ferro json-ui:schema --component Card` | PASS — exit 0, 892 bytes valid JSON |
| 4 | Unknown component | `ferro json-ui:schema --component NotARealComponent` | PASS — exit 1, stderr contains `unknown component 'NotARealComponent'` |
| 5 | `--pretty` flag | `ferro json-ui:schema --pretty` | PASS — exit 0, valid JSON |

## Verification

- `cargo fmt --all -- --check`: clean
- `cargo clippy --all --all-targets --all-features -- -D warnings`: clean
- `cargo test --all-features`: all tests pass (no regressions)

## Deviations from Plan

1. **Smoke test harness.** The plan's smoke script assumed `cd app/ && cargo
   run -p ferro-cli -- json-ui:schema` would reach the framework handler. The
   sample app in this workspace (`app/src/main.rs`) uses a hand-rolled clap
   CLI that predates `Application::new().run()`; it doesn't forward unknown
   subcommands to the framework. Smoke-testing therefore used a dedicated
   minimal binary (`/tmp/ferro-smoke-bin`) whose `main` is literally
   `Application::new().run().await`. This is the canonical entry for
   newly-scaffolded apps, so `json-ui:schema` works out-of-the-box for any
   Ferro project generated via `ferro new`.

2. **Feature gate placement.** As specified in the task, the framework-side
   `JsonUiSchema` variant and `run_json_ui_schema` handler are gated on
   `#[cfg(feature = "json-ui")]`. The ferro-cli binary has no gate — it's a
   developer CLI that unconditionally shells out; gating is the framework's
   responsibility.

## Commits

- `cf55f285` feat(117-07): add json-ui:schema subcommand and handler to framework
- `a0c6e708` feat(117-07): add ferro-cli shell-out wrapper for json-ui:schema
- `4bccd891` feat(117-07): wire json-ui:schema into ferro-cli main dispatch
