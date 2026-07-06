---
phase: 163-json-ui-improvements-batch-2-cassa-and-calendario-field-test
plan: 07
subsystem: cli
tags: [ferro-cli, codemod, json-ui, syn, ast, migration]

# Dependency graph
requires:
  - phase: 162-json-ui-component-friction-points-and-api-surface-fixes
    provides: stable JSON-UI v2 spec format that the codemod targets
provides:
  - ferro-cli subcommand `ferro json-ui:migrate-v1 <FILE> [--dry-run]`
  - AST-based migration of `make_node(id, Component::X(props))` patterns
  - Emitted JSON spec stubs under `src/views/{module}/{handler}.json`
  - Rewritten controllers using `JsonUi::render_file(...)`
  - Idempotence guard (re-running on a migrated file is a no-op + warning)
  - --dry-run flag (prints proposed output without writing)
  - TODO markers for handlers with runtime branching that the codemod can't auto-translate
affects: [downstream-consumer-migrations, gestiscilo-v1-to-v2-migration]

# Tech tracking
tech-stack:
  added:
    - syn = "2" with ["full", "parsing", "visit"] features (already in ferro-cli Cargo.toml)
    - quote = "1" (already in ferro-cli Cargo.toml)
    - tempfile = "3.24" (dev-dependency for tests, already present)
  patterns:
    - "AST visitor pattern: syn::visit::Visit walks pub async fn handlers, identifies Spec::builder() chains"
    - "Best-effort transformation: heterogeneous branches (if/match/runtime loops) emit `// TODO: codemod could not auto-translate` markers"
    - "Idempotence via marker detection: presence of JsonUi::render_file + absence of Spec::builder() indicates already-migrated"
    - "Pure I/O wrapper around testable core: migrate(syn::File) -> (PathBuf, String, String) factored out for --dry-run"

key-files:
  created:
    - ferro-cli/src/commands/json_ui_migrate_v1.rs (716 lines — visitor + emitter + idempotence guard)
    - ferro-cli/tests/json_ui_migrate_v1.rs (169 lines, 5 integration tests)
    - ferro-cli/tests/fixtures/migrate_v1/in_auth.rs (54-line input controller fixture)
    - ferro-cli/tests/fixtures/migrate_v1/in_with_runtime_branch.rs (30-line runtime-branch fixture)
    - ferro-cli/tests/fixtures/migrate_v1/out_auth.rs (6-line expected output controller)
    - ferro-cli/tests/fixtures/migrate_v1/out_auth_login_form.json (29-line expected JSON spec)
  modified:
    - ferro-cli/src/commands/mod.rs (register json_ui_migrate_v1 module)
    - ferro-cli/src/main.rs (wire CLI subcommand + arg parsing)
    - Cargo.lock (regenerated dependency tree)

key-decisions:
  - "AST-based, not regex-based: uses syn::visit::Visit so structural matching is type-aware and robust to formatting."
  - "File-at-a-time (D-10): no directory-recursive mode; each migration needs human review."
  - "Best-effort with explicit failure markers: `// TODO: codemod could not auto-translate` rather than silent skips."
  - "--dry-run prints proposed output: lets reviewers diff before writing."
  - "Idempotence as a warning, not an error: re-running on an already-migrated file is a no-op with a stderr message."

patterns-established:
  - "Fixture-driven codemod testing: input/output .rs and .json pairs in tests/fixtures/migrate_v1/, tempdir-isolated test runs"
  - "Pure-function factoring for dry-run: I/O lives at the entry point; the core transformation is a pure (syn::File) -> (PathBuf, String, String)"
  - "Best-effort with marker output: when the codemod cannot prove correctness for a given handler, it writes a TODO marker rather than producing wrong output"

risks-and-followups:
  - "Codemod is best-effort: handlers with runtime branching (if/match returning different spec shapes) require manual migration"
  - "Consumer-side validation: gestiscilo and other downstream apps should run codemod output through `cargo check` before committing — codemod does not validate that the rewritten controller compiles"
  - "Coverage limited to make_node + Spec::builder() patterns: ad-hoc Spec construction using SpecBuilder ergonomic methods (163-05) is out of scope"

verification:
  - "cargo test -p ferro-cli --test json_ui_migrate_v1 → 5 passing tests (idempotent, dry-run, write, runtime-branch fallback, no-handler)"
  - "cargo build -p ferro-cli → ferro CLI binary builds clean with new subcommand registered"

# Recovery note
recovery:
  - "This SUMMARY.md was reconstructed from the merged worktree commits during a recovery pass. The original executor in worktree-agent-a21e31d841758de9c did not commit a SUMMARY.md before the worktree was abandoned. The three feature commits (scaffold → test → feat) are intact in the merged history (`git log --grep 163-07`)."
