# Phase 160: Remove v1 JSON-UI API — Research

**Researched:** 2026-05-17
**Domain:** Workspace-wide v1 surface deletion + neutral-voice doc rewriting
**Confidence:** HIGH

## Summary

The Rust v1 type surface (`JsonUiView`, `Component`, `ComponentNode`, `PluginProps`, `view.rs`) is already absent from production source — verified by the Phase 164 `V1-DELETION-AUDIT.md` and re-verified inline in this research (2026-05-17). What remains is **prose**, **doc comments**, and **MCP tool plumbing** that still narrates v1 as a contrast point — plus several surprises CONTEXT.md does not enumerate. Phase 160 is therefore a coordinated *prose-delete-and-reframe* phase, not a Rust type-deletion phase.

The work splits into four bands:

1. **Three verification gates** (Rust source absence, schema literal absence, workspace-wide grep gate per D-10) — confirm pre-conditions hold and prove deletion.
2. **Eleven concrete source/doc rewrite sites** spread across `ferro-json-ui/src/render/{containers,form,data,mod}.rs`, `ferro-mcp/src/tools/{code_templates,application_info,json_ui_inspect}.rs`, `ferro-json-ui/README.md`, `docs/src/reference/cli.md`, `docs/src/features/projections.md`, `docs/protocol/src/{terminology,architecture,rendering}.md`.
3. **One policy question CONTEXT.md does not resolve** — whether the `ferro json-ui:migrate-v1` codemod (subcommand, source file, fixtures, integration tests) stays or goes. The user-naming constraint says "no migration story in agent-readable surface"; the codemod is the embodiment of a migration story. **Recommendation:** keep for v12.0 (it has shipped, its presence does not contradict deletion of v1 *types*, and the migration guide it targeted has already been removed from `docs/src/json-ui/`), but the planner should explicitly close this question with the user during plan time.
4. **Cross-repo verification with one missing repo** — `ferro-code` directory is **empty** (`/Users/alberto/repositories/albertogferrario/ferro-code` exists but contains no files). CONTEXT.md D-09 mandates ferro-code verification; the planner must either descope ferro-code from this phase (recommended — it cannot be verified) or block on the user creating/populating the repo.

**Primary recommendation:** Execute Phase 160 as 10 plans, parallelizable in three waves: (W1) source cleanup + MCP plumbing deletion, (W2) public-doc reframes + verification grep gate, (W3) cross-repo build/test (ferro + gestiscilo only).

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Deepest cleanup scope. Verification + stale-reference cleanup + public-doc reframing — not verification-only.

- **D-02:** Delete v1-framing prose from `ferro-json-ui/src/render/containers.rs:631-635` and `ferro-json-ui/src/render/form.rs:33-39`. Replace with neutral description of what each function does today.

- **D-03:** Sweep `ferro-json-ui/src/` for any remaining `v1`, `legacy`, `removed`, `Port of`, `Differences from v1` framing introduced during the v2 cutover.

- **D-04:** Delete `fn migration_v1_to_v2_templates()` in `ferro-mcp/src/tools/code_templates.rs` AND the `templates.extend(migration_v1_to_v2_templates());` registration at line 79, AND the corresponding integration test asserting "at least 7 migration_v1_to_v2 templates" (`code_templates_returns_migration_patterns` at line 1819).

- **D-05:** Replace `ferro-mcp/src/tools/application_info.rs::scan_json_ui_specs`. Rewrite to scan v2-shaped surface (count `*.json` spec files under `src/views/`; optionally also count controller call sites of `JsonUi::render_file(...)`). Remove `Scans for legacy v1 patterns. TODO(Phase 120): ...` doc comment.

- **D-06:** Audit `ferro-mcp/src/tools/json_ui_inspect.rs:307` (`write_file(&views_dir, "old_view.rs", "// old v1 file");`). If the test exercises meaningful behavior, rename fixture to neutral name. If the test was specifically validating "we ignore old v1 view.rs files", delete the test.

- **D-07:** Reframe (not just substitute):
  - `docs/protocol/src/terminology.md:98` — `JsonUiRenderer produces ferro-json-ui/v1 component trees, but ...`
  - `docs/protocol/src/architecture.md:172` — `JsonUiRenderer` paragraph
  - `docs/protocol/src/rendering.md:136` — `ferro-json-ui/v1 schema with envelope ...`
  - `docs/src/features/projections.md:42` — inline code comment `// json["$schema"] == "ferro-json-ui/v1"`

- **D-08:** Sweep `docs/src/` and `docs/protocol/src/` for any remaining narrative `v1`, `legacy`, `Migrating from`, `was removed`, `in v2`, `since v2`. Treat each as a rewrite target.

- **D-09:** Verify all three repos in this phase (ferro + gestiscilo + ferro-code). Do not defer to Phase 161.

- **D-10:** Final gate must show zero matches for `\b(JsonUiView|ComponentNode|PluginProps)\b` across `ferro-json-ui/`, `framework/`, `ferro-mcp/` source trees, and zero matches for `ferro-json-ui/v1` workspace-wide except inside `.planning/`.

- **D-11:** No publish in Phase 160. Single end-of-loop publish at Phase 161.

### Claude's Discretion

- Exact rewording of each doc comment and prose passage — constraint is "neutral, present-tense, no v1 framing".
- Whether `application_info::scan_json_ui_specs` also walks controllers for `JsonUi::render_file` call sites or stays file-count-only — pick the simpler of the two at plan time.
- Test re-organization in `json_ui_inspect.rs` if D-06 forces a rename — keep coverage equivalent.

### Deferred Ideas (OUT OF SCOPE)

- `LoadError::Catalog` variant cleanup — deferred past Phase 160.
- Unified `$if` + `visible` directive — v12.1+ candidate.
- `Modal` chrome variant — intentional gap.
- Granular `Card` props (`padding`, `elevation`) — intentional gap.
- Codemod directory-recursive mode — intentionally rejected.
- v12.0 CHANGELOG drafting — Phase 161 owns.
- crates.io publish — Phase 161 owns.

## Project Constraints (from CLAUDE.md)

These directives are load-bearing for Phase 160. The planner must comply with each.

- **"Run fmt + clippy + tests before every commit"** — Phase 160 must run all three before each commit:
  ```bash
  cargo fmt --all -- --check
  cargo clippy --all --all-targets -- -D warnings
  cargo test --all-features
  ```
  CI enforces `-D warnings` so any warning is a build failure.

- **"Repository documents must read as neutral"** — every repo file (including `.planning/` files in principle, though those are out-of-scope per D-10) should read like neutral architectural documentation, not internal strategy notes. **Trigger phrases to flag during D-08 sweep**: "killer feature", "the bet", "betting on", "load-bearing weakness", "named weakness", "no stop-loss", "maximum-quality stance", "all-of-them", "forcing function", "co-dependent", "leverage source", "we accept that", "the risk we're taking". For Phase 160 specifically, the dominant trigger phrases are "v1", "legacy", "Port of v1", "Differences from v1", "removed in", "was removed", "migrating from", "since v2".

- **"This is always a feature branch — delete old code completely, no deprecation needed, no versioned names, no migration code unless explicitly requested, no 'removed code' comments — just delete it"** — directly aligns with the phase mandate. No `#[deprecated]` attributes, no feature flags, no compat shims, no "this was removed in Phase 115" framing.

- **"Project-agnostic crates"** — `ferro-*` crates must not hardcode application identity (app names, brand strings, URLs). Reviewers should reject hardcoded strings like `"gestiscilo"`, `"Ferro Application"`, `"https://example.com"` inside a `ferro-*` crate. **Relevant to Phase 160:** any reframed doc-comments or replacement scanner code in `ferro-mcp` / `ferro-json-ui` must not embed app-specific strings.

- **"No co-author lines in commits"** — git commit messages stay clean.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Rust source v1 surface absence (Rust types `JsonUiView`, `Component`, `ComponentNode`, `PluginProps`) | `ferro-json-ui` source crate | — | Owns the public type surface; deletion already complete (verified). |
| v1-framing doc-comments in renderer (`render/containers.rs`, `render/form.rs`, `render/data.rs`, `render/mod.rs`) | `ferro-json-ui` source crate | — | Doc comments live with code; rewrite is in-crate. |
| MCP catalog scanner (`application_info::scan_json_ui_specs`) | `ferro-mcp` introspection layer | `ferro-json-ui` (defines what is scanned) | MCP scanner is the agent-facing introspection surface; ferro-json-ui defines the scanned shape. |
| MCP code-template catalog (`code_templates::migration_v1_to_v2_templates`) | `ferro-mcp` introspection layer | — | Pure MCP-tool plumbing. |
| MCP component-inspect tool fixtures (`json_ui_inspect.rs:307`) | `ferro-mcp` test surface | — | Test fixture in `ferro-mcp`. |
| Public-doc reframing (mdbook) | `docs/src/` + `docs/protocol/src/` | — | Doc sources, not code. |
| README rewrite (`ferro-json-ui/README.md`) | `ferro-json-ui` crate root | — | Crate-level README; **NOT MENTIONED in CONTEXT.md but DEFINITELY in-scope** — contains a v1 code example as the only Usage block. |
| CLI reference v1 example (`docs/src/reference/cli.md:518-538`) | `docs/src/` | `ferro-cli` (output of `ferro make:json-view`) | Doc page; the example also drifted out of date with the actual `make:json-view` output (which now emits a JSON spec file + handler stub, not a `*.rs` view). |
| Cross-repo build/test gate | Build/test infrastructure across 3 repos | `[patch.crates-io]` mechanism in gestiscilo's `Cargo.toml` | Pure verification; no code lives here. |
| `ferro json-ui:migrate-v1` codemod (subcommand) | `ferro-cli` | `ferro-mcp` (catalog registration), `ferro-cli/tests/fixtures` | Migration tool; presence is in tension with naming constraint but not strictly forbidden — see Open Questions. |

## Standard Stack

### Core (already in place — Phase 160 introduces nothing new)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust toolchain | 1.88.0 | Workspace `rust-version` per root `Cargo.toml:35` | Locked workspace floor. [VERIFIED: `rustc --version` returned `rustc 1.88.0 (6b00bc388 2025-06-23)`; `Cargo.toml:35` confirms `rust-version = "1.88.0"`.] |
| `cargo` | 1.88.0 | Build/test driver | Standard. [VERIFIED: `cargo --version`.] |
| `rustfmt` (via `cargo fmt`) | toolchain-bundled | Format check | CLAUDE.md requires `cargo fmt --all -- --check` before commit. |
| `clippy` (via `cargo clippy`) | toolchain-bundled | Lint with `-D warnings` | CI enforces `-D warnings`; matches CLAUDE.md guidance. [VERIFIED: CLAUDE.md "Testing & Linting" section.] |
| `mdbook` | system-installed | Docs build (verification only, no new docs added) | Used in Phase 159 gate. [VERIFIED: `docs/book.toml` exists; Phase 159 ran `mdbook build docs/`.] |
| `grep` / ripgrep equivalents | system | D-10 grep gate execution | Standard. |

### Supporting (existing test/build harness — re-used as-is)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` 3 | workspace dep | Temp dir for `json_ui_inspect` test fixture rewrite | Already used by the test under D-06; no change. [VERIFIED: `ferro-mcp/src/tools/json_ui_inspect.rs:281`.] |
| `tracing` | workspace dep | Replacement for v1-framing log lines if any are found during D-03 sweep | Standard. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Plain `grep` for D-10 gate | `ripgrep` (`rg`) | `rg` is faster on large trees but `grep` is universally present and the gate runs once; pick `grep` for portability. The CONTEXT.md D-10 wording uses `\b...\b` regex anchors which both tools support. [CITED: GNU grep manual — POSIX ERE word boundaries; ripgrep --pcre2 for the same.] |
| Manual sed for prose substitution | Manual Edit-tool diffs | sed is fragile across multi-line passages — the D-07 reframes are full-paragraph rewrites that require human-quality wording. Use the Edit tool per file. |

**Installation (none — all already present):**

```bash
# Verification only — no install steps
cargo --version  # expect 1.88.0
rustc --version  # expect 1.88.0
which mdbook     # required for Phase 159 docs gate re-run (if needed)
```

**Version verification:**
- `rustc 1.88.0 (6b00bc388 2025-06-23)` [VERIFIED: Bash output 2026-05-17]
- `cargo 1.88.0 (873a06493 2025-05-10)` [VERIFIED: Bash output 2026-05-17]

## Architecture Patterns

### System Architecture Diagram (Phase 160 work flow)

```
                    ┌─────────────────────────────────────────────┐
                    │ CONTEXT.md decisions D-01..D-11             │
                    └────────────────┬────────────────────────────┘
                                     │
              ┌──────────────────────┼──────────────────────┐
              │                      │                      │
              ▼                      ▼                      ▼
    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
    │ Wave 1 (source) │    │ Wave 1 (mcp)    │    │ Wave 1 (readme) │
    │ render/*.rs     │    │ code_templates  │    │ ferro-json-ui/  │
    │ port-comment    │    │ migration fn +  │    │ README.md       │
    │ rewrites D-02/  │    │ test deletion   │    │ replace v1 code │
    │ D-03            │    │ D-04            │    │ example         │
    └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
             │                      │                      │
             └──────────────────────┼──────────────────────┘
                                    │
                                    ▼
                ┌─────────────────────────────────────┐
                │ Wave 1 (mcp scanner)                │
                │ application_info::scan_json_ui_specs │
                │ v2 rewrite per D-05                  │
                └────────────────────┬────────────────┘
                                     │
                                     ▼
                ┌─────────────────────────────────────┐
                │ Wave 1 (mcp test fixture)           │
                │ json_ui_inspect.rs:307 audit per    │
                │ D-06 (delete vs rename)             │
                └────────────────────┬────────────────┘
                                     │
                                     ▼
              ┌──────────────────────┼──────────────────────┐
              │                      │                      │
              ▼                      ▼                      ▼
    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
    │ Wave 2 (docs)   │    │ Wave 2 (cli ref)│    │ Wave 2 (docs    │
    │ docs/protocol/  │    │ docs/src/       │    │ sweep D-08)     │
    │ src/* — reframe │    │ reference/cli.md│    │ docs/src/       │
    │ per D-07        │    │ replace v1 view │    │ docs/protocol/  │
    │ (3 files)       │    │ example         │    │ src/            │
    └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
             │                      │                      │
             └──────────────────────┼──────────────────────┘
                                    │
                                    ▼
                ┌─────────────────────────────────────┐
                │ Wave 2 (docs/src/features/          │
                │ projections.md:42)                  │
                │ inline-code comment reframe D-07    │
                └────────────────────┬────────────────┘
                                     │
                                     ▼
                ┌─────────────────────────────────────┐
                │ Wave 3 (verification gate)          │
                │ D-10 grep sweep (zero matches)      │
                │ D-09 cross-repo build/test          │
                │  - ferro: fmt + clippy + test       │
                │  - gestiscilo: cargo test           │
                │  - ferro-code: SKIPPED (see Open Q) │
                └─────────────────────────────────────┘
```

Data flow: each Wave 1 task is independent (different files, no shared state); Wave 2 doc rewrites are likewise independent. Wave 3 is the gate — runs only after every Wave 1 and Wave 2 plan reports complete.

### Recommended Project Structure (already in place — no new structure)

```
ferro/
├── ferro-json-ui/
│   ├── README.md                  # ← Phase 160 rewrite (v1 example)
│   ├── src/
│   │   ├── lib.rs                 # ← already clean (verified)
│   │   ├── spec.rs                # ← LEAVE (SCHEMA_VERSION = "ferro-json-ui/v2")
│   │   ├── render/
│   │   │   ├── mod.rs             # ← D-03 sweep target (4 v1 comments)
│   │   │   ├── atoms.rs           # ← EXEMPLAR for neutral doc style
│   │   │   ├── containers.rs      # ← D-02 + D-03 (8 "Port of v1" comments)
│   │   │   ├── form.rs            # ← D-02 + D-03 (header + form fn)
│   │   │   └── data.rs            # ← D-03 (5+ "Port of v1" comments)
│   │   ├── layout.rs              # ← D-03 (test fixture "v1" literal — leave or rename)
│   │   └── projection/
│   │       ├── mod.rs             # ← already clean (uses "ferro-json-ui/v2")
│   │       └── builder.rs:42      # ← D-03 ("Silence ... until Plan 03 rewires the legacy renderer")
│   └── Cargo.toml
├── ferro-mcp/
│   └── src/tools/
│       ├── application_info.rs:244 # ← D-05 (rewrite scan_json_ui_specs)
│       ├── code_templates.rs:78    # ← D-04 (delete fn + registration + test)
│       └── json_ui_inspect.rs:307  # ← D-06 (audit test fixture)
├── ferro-cli/
│   ├── src/
│   │   ├── commands/
│   │   │   └── json_ui_migrate_v1.rs # ← OPEN QUESTION (keep or delete?)
│   │   ├── templates/make.rs:812     # ← D-03 (test asserts no v1 markers — leave; this is correct)
│   │   └── main.rs:166               # ← OPEN QUESTION (subcommand registration)
│   └── tests/
│       ├── json_ui_migrate_v1.rs     # ← OPEN QUESTION (codemod integration tests)
│       └── fixtures/migrate_v1/      # ← OPEN QUESTION (v1 source fixtures)
└── docs/
    ├── src/
    │   ├── reference/cli.md:518-538   # ← BIG GAP — v1 example in CLI ref (CONTEXT.md missed this)
    │   ├── features/projections.md:42 # ← D-07 (inline-code comment)
    │   └── json-ui/                   # ← already clean (verified — no v1/legacy framing)
    └── protocol/src/
        ├── terminology.md:94-100      # ← D-07 (JsonUiRenderer paragraph)
        ├── architecture.md:172-173    # ← D-07 (JsonUiRenderer bullet)
        └── rendering.md:132-136       # ← D-07 (Output Format section)
```

### Pattern 1: Neutral Doc-Comment Style (Exemplar)

**What:** Replace "Port of v1 X (render.rs L###-###). Differences from v1: ..." with a present-tense functional description.
**When to use:** Every D-02/D-03 rewrite in `ferro-json-ui/src/render/*.rs`.
**Exemplar source:** `ferro-json-ui/src/render/atoms.rs` (lines 1-55) — already neutral.

```rust
// Source: ferro-json-ui/src/render/atoms.rs:1-37 (VERIFIED 2026-05-17)
//! Phase 116: leaf renderers ported verbatim from v1 render.rs.       // ← Bad: provenance narrative
//!
//! Per CONTEXT D-21 the v1 HTML emission is the canonical contract;   // ← Bad: still references v1
//! this module changes only the function signature ...                // ← Bad: contrast narrative
```

The atoms.rs module-level comment is itself a Phase 160 D-03 target — it contains v1 framing. The *individual function* doc comments inside atoms.rs (which describe what each atom renders, in present tense, without "Port of v1 …") ARE the exemplar.

**Better example — the `decode_diagnostic` helper at `atoms.rs:26-37`:**

```rust
// Source: ferro-json-ui/src/render/atoms.rs:26-37 (VERIFIED 2026-05-17)
/// Emits a `<!-- ferro-json-ui: failed to decode TYPE props: MSG -->` comment.
/// Used as the fallback on serde_json::from_value decode errors across every
/// atom renderer.
fn decode_diagnostic(type_name: &str, err: impl std::fmt::Display) -> String { ... }
```

This is the target style: **what the function emits**, **what props it reads**, **what edge cases it handles** — no historical narrative, no version contrast.

**Pattern applied to `containers.rs:631-635` (current v1-framed):**

```rust
// CURRENT (Source: ferro-json-ui/src/render/containers.rs:631-635, VERIFIED 2026-05-17)
/// Port of v1 `render_button_group` (render.rs L758-765). Horizontal button row;
/// children come from `Element.children` per D-05.
///
/// Note: v1 iterated `props.buttons: Vec<ComponentNode>`; v2 takes children from
/// `Element.children` (generic `ButtonGroupProps` retains only the `gap` field).
pub(crate) fn render_button_group(...) -> String { ... }

// PROPOSED REWRITE (neutral, present-tense):
/// Horizontal button row. Renders each child ID in `el.children` and wraps the
/// resulting HTML in a `<div class="flex items-center gap-2 flex-wrap">` container.
/// `ButtonGroupProps.gap` is decoded for prop-shape diagnostics but does not
/// influence the emitted CSS (the gap is fixed at `gap-2`).
pub(crate) fn render_button_group(...) -> String { ... }
```

**Pattern applied to `form.rs:33-39` (current v1-framed):**

```rust
// CURRENT (Source: ferro-json-ui/src/render/form.rs:33-39, VERIFIED 2026-05-17)
/// Port of v1 `render_form` (render.rs:961–1015).
///
/// Differences from v1:
/// - Child fields come from `el.children` (list of IDs) instead of the
///   removed `FormProps.fields: Vec<ComponentNode>` per D-05.
/// - `action.url = None` falls back to `action="#"` AND emits the D-16
///   diagnostic comment (v1 silently used `"#"`).
pub(crate) fn render_form(...) -> String { ... }

// PROPOSED REWRITE:
/// Renders a `<form>` element. Child IDs in `el.children` become the form body;
/// `FormProps.action` controls submission target and method (with HTTP-method
/// spoofing for PUT/PATCH/DELETE → POST + hidden `_method` input). When
/// `action.url` is `None`, the form falls back to `action="#"` and emits a
/// diagnostic HTML comment.
pub(crate) fn render_form(...) -> String { ... }
```

### Pattern 2: MCP Scanner Replacement (D-05)

**What:** Replace the v1 `.rs`-file counter in `scan_json_ui_specs` with a v2 `.json`-file counter.
**When to use:** D-05 only — one rewrite in `application_info.rs`.

**Current (Source: `ferro-mcp/src/tools/application_info.rs:244-289`, VERIFIED 2026-05-17):**

```rust
/// Scans for legacy v1 patterns. TODO(Phase 120): add a parallel v2 scanner
/// that looks for `-> Spec` and counts flat-spec usages alongside v1 counts.
fn scan_json_ui_specs(project_root: &Path) -> JsonUiSpecsStatus {
    let views_dir = project_root.join("src").join("views");
    // ...
    // Count .rs files in src/views/ (excluding mod.rs)
    let view_count = std::fs::read_dir(&views_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.extension().map(|ext| ext == "rs").unwrap_or(false)
                        && path.file_name().map(|f| f != "mod.rs").unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0);
    // ...
}
```

**Proposed v2 rewrite (file-count only — the simpler option from CONTEXT.md "Claude's Discretion"):**

```rust
/// Counts JSON-UI spec files under `src/views/`. Each `.json` file corresponds
/// to a spec loaded at runtime by `JsonUi::render_file("views/{name}.json", ..)`.
/// The status surfaced here lets agents discover how many spec files a project
/// ships without enumerating individual filenames.
fn scan_json_ui_specs(project_root: &Path) -> JsonUiSpecsStatus {
    let views_dir = project_root.join("src").join("views");
    let views_dir_display = "src/views/".to_string();

    if !views_dir.exists() {
        return JsonUiSpecsStatus {
            available: false,
            view_count: 0,
            views_dir: views_dir_display,
            hint: Some(
                "No src/views/ directory found. Create JSON-UI spec files there \
                 and serve them with JsonUi::render_file(\"views/{name}.json\", data). \
                 Use the json_ui_generate MCP tool to scaffold a new spec."
                    .to_string(),
            ),
        };
    }

    let view_count = std::fs::read_dir(&views_dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                .count()
        })
        .unwrap_or(0);

    JsonUiSpecsStatus {
        available: true,
        view_count,
        views_dir: views_dir_display,
        hint: if view_count == 0 {
            Some(
                "Views directory exists but no JSON spec files found. \
                 Use json_ui_generate to create one."
                    .to_string(),
            )
        } else {
            None
        },
    }
}
```

**Field-shape preservation contract:** The struct `JsonUiSpecsStatus` (`application_info.rs:57-62`) has four fields: `available: bool`, `view_count: usize`, `views_dir: String`, `hint: Option<String>`. **Do not change the field names** — they are part of the MCP tool's serialized output and downstream agents may depend on them. The `view_count` *semantics* change (was "v1 .rs files"; becomes "v2 .json files") but the field name is stable, which is the correct trade-off: the v1 semantic was a stale `TODO(Phase 120)`-flagged artifact, so the contract change is intentional and aligned with the audit's "MCP output type — schema-breaking only for MCP consumers, which is acceptable per project norms" note in `.planning/phases/115-spec-v2-data-structures/115-RESEARCH.md:983`.

**Optional second-pass enhancement (CONTEXT.md "Claude's Discretion"):** also count controller call sites of `JsonUi::render_file(...)` via regex grep. **Recommendation: skip.** The file count alone is the simplest correct answer; adding a second metric increases test surface for marginal agent value. The planner can revisit if there's a clear consumer need.

**Existing test contract (no tests pin the field shape — VERIFIED via grep 2026-05-17):** no `#[cfg(test)] mod` exists in `application_info.rs` for `scan_json_ui_specs`; the only references to `JsonUiSpecsStatus` and `view_count` are inside that one file and inside `.planning/phases/115-*/` history docs (which are out-of-scope per D-10). The rewrite is therefore unconstrained by existing automated tests; the planner should add at least one happy-path test (`temp dir with 2 .json files → view_count == 2`) and one empty-dir test.

### Pattern 3: D-04 Test-Deletion Safety

**What:** Delete `migration_v1_to_v2_templates()` function, its registration, AND its test.
**When to use:** D-04 only.

**Verified deletion scope (Source: `ferro-mcp/src/tools/code_templates.rs`, VERIFIED 2026-05-17):**

1. Delete line 78-79 (registration):
   ```rust
   // v1 → v2 migration patterns
   templates.extend(migration_v1_to_v2_templates());
   ```
2. Delete lines 1504-1697 (entire `migration_v1_to_v2_templates()` function — 7 template constants).
3. Delete lines 1818-1830 (the `code_templates_returns_migration_patterns` test — asserts "at least 7 migration_v1_to_v2 templates").

**Test-grep audit for collateral damage (VERIFIED 2026-05-17):** the string `migration_v1_to_v2` appears only in `ferro-mcp/src/tools/code_templates.rs` and the planning files in `.planning/`. **No other test** references the category name or count. Safe to delete in one pass.

**Note on the comment at line 78:** `// v1 → v2 migration patterns` — delete the comment line too (otherwise it dangles as a stale header above the closing brace of `build_templates()`).

### Pattern 4: D-06 Test Fixture Audit

**What:** Resolve the v1 framing in `json_ui_inspect.rs:307` test fixture.
**When to use:** D-06 only.

**Test in question (Source: `ferro-mcp/src/tools/json_ui_inspect.rs:301-312`, VERIFIED 2026-05-17):**

```rust
#[test]
fn test_ignores_non_json_files() {
    let tmp = TempDir::new().unwrap();
    let views_dir = tmp.path().join("src/views");
    fs::create_dir_all(&views_dir).unwrap();

    write_file(&views_dir, "old_view.rs", "// old v1 file");
    write_file(&views_dir, "mod.rs", "pub mod old;");

    let result = execute(tmp.path(), None);
    assert_eq!(result.total, 0);
}
```

**Analysis:** The test name (`test_ignores_non_json_files`) and the second `write_file` call (`mod.rs`) make the test's *actual* purpose explicit — it verifies that the JSON-UI inspect scanner ignores stray non-`.json` files in the views directory regardless of what those files are. The fixture filenames (`old_view.rs`) and content (`"// old v1 file"`) are decorative — they are not load-bearing for the assertion.

**Recommendation: rename, don't delete.** The behavioral test ("don't crash on stray non-JSON files") is still meaningful in a v2-only world. Rewrite as:

```rust
#[test]
fn test_ignores_non_json_files() {
    let tmp = TempDir::new().unwrap();
    let views_dir = tmp.path().join("src/views");
    fs::create_dir_all(&views_dir).unwrap();

    write_file(&views_dir, "stale_artifact.rs", "// non-JSON artifact");
    write_file(&views_dir, "mod.rs", "pub mod old;");

    let result = execute(tmp.path(), None);
    assert_eq!(result.total, 0);
}
```

Keeps coverage equivalent; removes the v1 framing per the user-naming constraint.

### Pattern 5: Public-Doc Reframes (D-07)

Each rewrite below is the **complete current passage** followed by a **complete proposed replacement**. The planner can lift these directly or refine the wording — the constraint per CONTEXT.md is "neutral, present-tense, no v1 framing", not a specific phrasing.

**(a) `docs/protocol/src/terminology.md:94-100` (VERIFIED 2026-05-17):**

```markdown
CURRENT:
**Renderer**
: A component that transforms a Service Definition, its derived Intent
  Scores, and a Render Context into a UI component tree. Defined by the
  `Renderer` trait. The output format is implementation-specific:
  `JsonUiRenderer` produces ferro-json-ui/v1 component trees, but
  implementations MAY target A2UI, HTML, native components, or any other
  format.

PROPOSED:
**Renderer**
: A component that transforms a Service Definition, its derived Intent
  Scores, and a Render Context into a UI component tree. Defined by the
  `Renderer` trait. The output format is implementation-specific:
  `JsonUiRenderer` produces a `Spec` conforming to the
  `ferro-json-ui/v2` schema; implementations MAY target A2UI, HTML,
  native components, or any other format.
```

**(b) `docs/protocol/src/architecture.md:169-179` (VERIFIED 2026-05-17):**

```markdown
CURRENT:
The Renderer trait is the protocol's extension point for output formats. Any
target format MAY implement the trait:

- **JsonUiRenderer** — The reference implementation. Produces ferro-json-ui/v1
  component trees (Table, Card, Form, Badge, Progress, etc.).
- **A2UI** — A potential implementation targeting A2UI component catalogs.
- **HTML** — A potential implementation producing static or server-rendered
  HTML.
- **Native** — A potential implementation producing native mobile component
  trees.

PROPOSED:
The Renderer trait is the protocol's extension point for output formats. Any
target format MAY implement the trait:

- **JsonUiRenderer** — The reference implementation. Produces a `Spec`
  conforming to the `ferro-json-ui/v2` schema: a flat ID-keyed element map
  with components such as Table, Card, Form, Badge, and Progress.
- **A2UI** — A potential implementation targeting A2UI component catalogs.
- **HTML** — A potential implementation producing static or server-rendered
  HTML.
- **Native** — A potential implementation producing native mobile component
  trees.
```

**(c) `docs/protocol/src/rendering.md:132-136` (VERIFIED 2026-05-17):**

```markdown
CURRENT:
## Output Format

The protocol does not prescribe a specific JSON output schema for renderers. Each renderer implementation defines its own component vocabulary and envelope structure.

The reference `JsonUiRenderer` produces output conforming to the `ferro-json-ui/v1` schema, with a top-level envelope containing `schema`, `version`, `title`, and `body` fields. Other renderers (e.g., A2UI, HTML) MAY produce entirely different output structures while remaining conformant, provided they implement the `Renderer` trait.

PROPOSED:
## Output Format

The protocol does not prescribe a specific JSON output schema for renderers. Each renderer implementation defines its own component vocabulary and envelope structure.

The reference `JsonUiRenderer` produces output conforming to the `ferro-json-ui/v2` schema: a top-level `Spec` with a `$schema` tag, a `root` element ID, a flat `elements` map keyed by ID, and optional `title`, `layout`, and `data` fields. Children inside `elements` are referenced by ID rather than by nesting. Other renderers (e.g., A2UI, HTML) MAY produce entirely different output structures while remaining conformant, provided they implement the `Renderer` trait.
```

**Schema-shape verification (Source: `ferro-json-ui/src/spec.rs:64-89`, VERIFIED 2026-05-17):** the proposed rewording for (c) correctly describes the actual `Spec` struct — `schema` (renamed via `#[serde(rename = "$schema")]`), `root`, `elements: HashMap<String, Element>`, optional `title: Option<TitleBinding>`, optional `layout: Option<String>`, optional `data: Value`. The current docs are inaccurate (claim `version` and `body` fields exist; they do not).

**(d) `docs/src/features/projections.md:38-44` (VERIFIED 2026-05-17):**

```markdown
CURRENT:
let renderer = JsonUiRenderer;
let json = renderer
    .render(&product, &intents, &RenderContext::default())
    .expect("rendering a valid service definition should not fail");
// json["$schema"] == "ferro-json-ui/v1"
// json["components"] contains the generated component tree
```

The example is **doubly stale**: (1) it claims `ferro-json-ui/v1`; (2) it claims `json["components"]` — but the v2 `Spec` exposes `elements` (a map), not `components` (would be a list). (3) it uses `RenderContext::default()` but the v2 renderer takes `VisualContext::default()`.

**Cross-reference (Source: `ferro-json-ui/src/projection/mod.rs:82-97`, VERIFIED 2026-05-17):** the actual rustdoc example in the source is correct:

```rust
let renderer = JsonUiRenderer;
let result = renderer.render(&product, &intents, &VisualContext::default());
assert!(result.is_ok());

let spec = result.unwrap();
assert_eq!(spec.schema, "ferro-json-ui/v2");
assert!(spec.elements.contains_key(&spec.root));
```

The docs/src copy in `projections.md:30-44` is stale and out-of-sync with the source rustdoc.

**Proposed:** sync the docs/src code block to match the source rustdoc:

```markdown
PROPOSED (lines 36-44 of projections.md):
let intents = derive_intents(&product);
// intents[0] is the highest-confidence intent (Browse for a simple list-like service)

let renderer = JsonUiRenderer;
let result = renderer.render(&product, &intents, &VisualContext::default());
let spec = result.expect("rendering a valid service definition should not fail");
// spec.schema == "ferro-json-ui/v2"
// spec.elements is a flat ID-keyed map; spec.root names the root element
```

Also fix the import line `~30` above — `RenderContext` is not in the current public API; `VisualContext` is (Source: `ferro-json-ui/src/lib.rs:94`, VERIFIED 2026-05-17).

### Pattern 6: ferro-json-ui README rewrite (NEW — not in CONTEXT.md)

**Discovered during D-08 sweep (VERIFIED 2026-05-17):** `ferro-json-ui/README.md` is **completely stale**. It is the crate's only Usage block, and it ships a v1 `JsonUiView { layout: LayoutComponent::Stack { ... }, ... }` example that does not compile against the current `ferro-json-ui` API.

**Impact:** This README is the file rendered at the top of `crates.io/crates/ferro-json-ui` when the crate is published — first impression for every agent or human discovering the crate. Phase 161 publishes the crate; the README must be correct before publish.

**Proposed rewrite (Pattern 5 wire shape; uses the public API verified at `ferro-json-ui/src/lib.rs:49-87` and the rustdoc example at `ferro-json-ui/src/lib.rs:19-27`):**

```markdown
# ferro-json-ui

JSON-based server-driven UI schema types for the [Ferro](https://ferro-rs.dev) web framework.

Define UI as a JSON spec and have Ferro render it to HTML on the server — no
frontend build step required.

## Features

- 41 built-in components (Card, Form, DataTable, KanbanBoard, Modal, Tabs,
  Alert, Badge, Button, ...) and a plugin system for custom components
- Layout system (`dashboard`, `app`, `auth`) and ID-keyed element graph with
  parse-time structural validation
- Action system: navigate, submit form, call API, open modal
- Data binding via JSON Pointer (`{"$data": "/path"}`) and iteration directives
  (`$each`, `$if`)
- Compile-time schema validation via `schemars` on every typed `*Props`

## Usage

Serve a spec from a Rust handler:

```rust
use ferro::{handler, JsonUi, Request, Response};

#[handler]
pub async fn dashboard(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/dashboard.json", data)
}
```

Or construct a spec in Rust:

```rust
use ferro_json_ui::{Spec, Element};

let spec = Spec::builder()
    .title("Demo")
    .element("root", Element::new("Text").prop("content", "Hi"))
    .build()
    .unwrap();
```

## Documentation

Full documentation at [docs.ferro-rs.dev](https://docs.ferro-rs.dev).

## License

MIT
```

This is in-scope for Phase 160 per the user-naming constraint ("public docs describe JSON-UI as the only version that exists") even though CONTEXT.md did not list the README explicitly. **Recommend the planner add it as a dedicated rewrite plan.**

### Pattern 7: docs/src/reference/cli.md `make:json-view` example (NEW — not in CONTEXT.md)

**Discovered during D-08 sweep (Source: `docs/src/reference/cli.md:518-538`, VERIFIED 2026-05-17):** the CLI reference docs the `make:json-view` command and shows a "Generated file" example that is a v1 Rust `JsonUiView::new()` chain. The actual current `make:json-view` command emits a JSON spec file (`src/views/{name}.json`) and prints a v2-style handler snippet (Source: `ferro-cli/src/commands/make_json_view.rs:91-106`, VERIFIED).

**Proposed rewrite (lines 516-538 of cli.md):**

```markdown
PROPOSED:
**Generated file:** `src/views/user_index.json`

```json
{
  "$schema": "ferro-json-ui/v2",
  "title": "User Index",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": {
        "title": "User Index",
        "description": "Edit src/views/user_index.json to customize this view."
      },
      "children": ["heading"]
    },
    "heading": {
      "type": "Text",
      "props": { "content": "User Index", "element": "h1" }
    }
  }
}
```

**Generated handler usage:**

```rust
#[handler]
pub async fn user_index(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/user_index.json", data)
}
```
```

The template content comes verbatim from `ferro-cli/src/templates/make.rs:107-131` (VERIFIED 2026-05-17), so this rewrite is grounded in the actual CLI output.

**Also rewrite `cli.md:516` "Generated file: `src/views/user_index.rs`" → "Generated file: `src/views/user_index.json`"** — the `.rs` extension is wrong.

### Pattern 8: Doc-comment sweep targets in ferro-json-ui (D-03 catalog)

Full list of v1-framing comments to rewrite (VERIFIED via grep 2026-05-17). This goes beyond the two sites in CONTEXT.md D-02.

| File | Line range | Current framing |
|------|-----------|-----------------|
| `ferro-json-ui/src/render/mod.rs` | 1-12 (module doc) | "Phase 116 walker: flat-element renderer for v2 Specs. Replaces the Phase 115 placeholder." |
| `ferro-json-ui/src/render/mod.rs` | 41 (BUILTIN_TYPES doc) | "Single source of truth for distinguishing built-ins ... Per CONTEXT D-19 plugins cannot shadow built-ins" |
| `ferro-json-ui/src/render/mod.rs` | 88-94 (`render_spec_to_html` doc) | "Public render entry point. Walks `spec.root` ... wrapped in v1's flex-wrap container." |
| `ferro-json-ui/src/render/mod.rs` | 222-225 | "Flat pass over `spec.elements` collecting plugin type names. Replaces v1's …" |
| `ferro-json-ui/src/render/mod.rs` | 245-248 | "Asset. Ported from v1 render.rs lines 200–221." |
| `ferro-json-ui/src/render/mod.rs` | 270-272 | "then `<script>{init}</script>` per init script. Ported from v1 render.rs …" |
| `ferro-json-ui/src/render/atoms.rs` | 1-12 (module doc) | "Phase 116: leaf renderers ported verbatim from v1 render.rs. Per CONTEXT D-21 the v1 HTML emission is the canonical contract; …" |
| `ferro-json-ui/src/render/atoms.rs` | 57 | "── SVG icon constants (ported verbatim from v1 render.rs) ──" |
| `ferro-json-ui/src/render/containers.rs` | 1-10 (module doc) | "Phase 116 container renderers ported from v1 render.rs." |
| `ferro-json-ui/src/render/containers.rs` | 26-28 | "Port of v1 `render_card` (render.rs L769-813) ... Preserves v1's `max_width`…" |
| `ferro-json-ui/src/render/containers.rs` | 74-77 | "v1 gated the body wrapper on `!props.children.is_empty()`; v2 gates on Element.children …" |
| `ferro-json-ui/src/render/containers.rs` | 108-110 | "Port of v1 `render_modal` (render.rs L815-863)." |
| `ferro-json-ui/src/render/containers.rs` | 177-180 | "Port of v1 `render_tabs` (render.rs L865-959). Two preserved non-obvious behaviors from v1 …" |
| `ferro-json-ui/src/render/containers.rs` | 201 | "Single-tab auto-hide (v1 L867-877)." |
| `ferro-json-ui/src/render/containers.rs` | 293 | "Port of v1 `render_kanban_board` (render.rs L499-587)." |
| `ferro-json-ui/src/render/containers.rs` | 414 | "Port of v1 `render_page_header` (render.rs L708-756)." |
| `ferro-json-ui/src/render/containers.rs` | 484 | "Port of v1 `render_grid` (render.rs L2123-2155)." |
| `ferro-json-ui/src/render/containers.rs` | 529 | "── Collapsible SVG chevron (v1 render.rs L2159-2163) ──" |
| `ferro-json-ui/src/render/containers.rs` | 536 | "Port of v1 `render_collapsible` (render.rs L2165-2184)." |
| `ferro-json-ui/src/render/containers.rs` | 572 | "Port of v1 `render_form_section` (render.rs L2214-2259)." |
| `ferro-json-ui/src/render/containers.rs` | 631-635 (D-02) | "Port of v1 `render_button_group`... Note: v1 iterated `props.buttons: Vec<ComponentNode>`…" |
| `ferro-json-ui/src/render/containers.rs` | 638-640 | "Decode-check for D-12 diagnostic discipline; ... v1 hard-codes `gap-2`…" |
| `ferro-json-ui/src/render/form.rs` | 1-20 (module doc) | "Phase 116: form-control renderers ported from v1 `render.rs`. Per CONTEXT D-21 v1 HTML emission is the canonical contract." |
| `ferro-json-ui/src/render/form.rs` | 33-39 (D-02) | "Port of v1 `render_form` (render.rs:961–1015). Differences from v1: ..." |
| `ferro-json-ui/src/render/data.rs` | 1-15 (module doc) | "Phase 116: data-display renderers ported from v1 `render.rs`. Per CONTEXT D-21 v1 HTML emission is the canonical contract." |
| `ferro-json-ui/src/render/data.rs` | 24-28 | "Port of v1 `render_table` (render.rs:1017–1102) ... Azioni header label is preserved from v1 verbatim." |
| `ferro-json-ui/src/render/data.rs` | 119-129 | "Port of v1 `render_data_table` (render.rs:1104–1285). ... v1 wraps row actions in a `DropdownMenu`; this …" |
| `ferro-json-ui/src/render/data.rs` | 251 | "Render a single cell's value as a plain string. Matches v1 semantics …" |
| `ferro-json-ui/src/render/data.rs` | 265 | "Resolve the row key for a single row. Matches v1 (render.rs:1171–1181 …)" |
| `ferro-json-ui/src/render/data.rs` | 287 | "Legacy `{row_key}` — resolved against `row_key_value` (v1 verbatim)." |
| `ferro-json-ui/src/render/data.rs` | 307 | "Resolve URL from handler when url is None (v1 fallback)." |
| `ferro-json-ui/src/render/data.rs` | 516 | "v1 semantics: {row_key} substitutes against props.row_key's value" |
| `ferro-json-ui/src/projection/builder.rs` | 42 | `// Silence unused-import warnings until Plan 03 rewires the legacy renderer.` (the comment is also obsolete — Plan 03 has run; the `_plan_02_reserved` function is dead code) |
| `ferro-json-ui/src/layout.rs` | 727 | Test helper: `view_json: "{\"schema\":\"v1\"}"` — fixture literal in a test. **Recommendation: change `"v1"` → `"ferro-json-ui/v2"` for consistency** (the test does not assert on the literal value; it's only used to populate a layout context for HTML assembly tests). Cosmetic but matches the no-v1 framing. |

**Exceptions (LEAVE):**
- `ferro-json-ui/src/expression.rs:253-256` — `json!({ "a": "v1", "b": "v2" })` — generic string-substitution test; "v1"/"v2" are arbitrary fixture values, not version references.
- `ferro-json-ui/src/plugin.rs:381-401` — `struct PluginV1; ... fn render(...) -> String { "v1".to_string() }` — test plugin name for an override-precedence test; "v1"/"v2" are arbitrary, not version references.
- `ferro-cli/src/templates/make.rs:810-817` — `json_view_template_has_no_v1_markers` — this test ENFORCES the no-v1 invariant. **Leave; this is the test that confirms D-08 compliance for the json_view_template output.**

**Recommended scoping:** D-02 mentions only `containers.rs:631-635` and `form.rs:33-39`. D-03 expands the sweep — every entry in the table above is in-scope. The planner should treat each file as a separate plan unit to keep diffs reviewable.

### Anti-Patterns to Avoid

- **String-substitute v1 → v2 in prose** — D-07 explicitly requires "reframe, do not just substitute". A passage that read "JsonUiRenderer produces v1 trees, but ..." becomes structurally awkward if you swap "v1" for "v2" without rewording — and the "but ..." contrast loses meaning. Rewrite the whole sentence.
- **Add `#[deprecated(note = "...")]` attributes** — explicitly forbidden by the phase goal.
- **Add a `compat::` module shim** — explicitly forbidden by the phase goal.
- **Keep "TODO(Phase X):" markers that are now closed** — D-05 explicitly removes the `TODO(Phase 120)` marker on `scan_json_ui_specs`; D-03 sweep should similarly remove any `TODO(Phase N):` markers in `ferro-json-ui/src/render/*.rs` that reference phases now closed.
- **Edit `SCHEMA_VERSION = "ferro-json-ui/v2"`** — explicitly out of scope per CONTEXT.md `## Out of scope`. The wire literal stays everywhere it appears as a literal `String`.
- **Mass-rewrite ALL "Phase 116" / "Per CONTEXT D-21" framing in one pass** — these phase / decision references are commit-log noise, but rewriting them is a separate concern from the v1-framing rewrite. Scope creep; defer.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Sweeping the repo for v1 framing | A custom Rust binary or shell pipeline that does AST analysis | `grep -rnE 'pattern' <paths>` (the audit's existing pattern) | Phase 164 V1-DELETION-AUDIT.md already established the canonical grep commands (lines 80-105); reuse them. The audit's commands are simple enough that any wrapper would just add ceremony. |
| Validating that doc rewrites produce neutral prose | A linter that scans for "v1", "legacy", "Port of" etc. | The CONTEXT.md trigger-phrase list + `cargo fmt` (no v1 framing rules) + `cargo clippy` (catches unused imports if functions are deleted) | Linters for prose framing are an over-engineering trap. The D-10 grep gate is the structural check; manual review handles tone. |
| Coordinated cross-repo build runs | A custom orchestrator that runs ferro + gestiscilo + ferro-code | Sequential shell commands; each repo's existing `cargo` invocation | Each repo has its own build/test commands already. Trying to unify them produces a fragile aggregator. |
| Detecting field-shape regressions in `JsonUiSpecsStatus` | A schema-pinning JSON snapshot test | Standard Rust unit tests on `scan_json_ui_specs` return shape | The struct is small (4 fields). Standard tests are clearer than snapshot tooling for a 4-field struct. |

**Key insight:** Phase 160 is mostly *deletion* and *rewriting* work. The temptation to build "deletion-aiding tooling" is the trap. The audit's existing grep commands + the Edit tool are the right instruments.

## Runtime State Inventory

This is a string-rename / deletion phase. The runtime-state audit per `feedback_audit_report_fix_discrepancies.md` applies.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| **Stored data** | None — Phase 160 does not touch any datastore. The `SCHEMA_VERSION` wire literal stays `"ferro-json-ui/v2"`; specs at rest on disk are unaffected. JSON specs in `app/src/views/` continue to declare `"$schema": "ferro-json-ui/v2"`. No database stores the v1 strings. **Verified by:** grep for `ferro-json-ui/v1` workspace-wide returned only the four documentation sites listed under D-07. | none |
| **Live service config** | None — `ferro-json-ui` is a library, not a service. No external API or hosted service stores configuration tagged with v1/v2 identifiers. | none |
| **OS-registered state** | None — no daemons, scheduled tasks, or OS services register against ferro-json-ui APIs by name. | none |
| **Secrets and env vars** | None — no env var names contain `JSON_UI_V1`, `FERRO_V1`, or similar (verified via `env | grep -i ferro` and via the workspace grep for `v1`). | none |
| **Build artifacts / installed packages** | **Two relevant items.** (1) `docs/book/` — the mdbook-built HTML output cached on disk contains v1 fragments (`docs/book/print.html:15433`, `docs/book/reference/cli.html:652`). These regenerate on `mdbook build`; no manual action needed beyond running the build. (2) crates.io published `ferro-json-ui 0.2.35` — this is the **shipped** version on crates.io that consumers (gestiscilo, etc.) currently fetch by version. Phase 160 does NOT publish (D-11); the new README/doc rewrites become visible on crates.io only after Phase 161 publish. **Action:** none in Phase 160; Phase 161 publish makes the cleanup visible to crates.io users. | none in Phase 160 |

**Nothing dangerously deferred.** The phase is well-contained at the source / docs / MCP layer.

## Common Pitfalls

### Pitfall 1: Touching `SCHEMA_VERSION` by Accident

**What goes wrong:** A developer running a "rewrite all v1 to v2" pass touches `SCHEMA_VERSION = "ferro-json-ui/v2"` (line 31 of `spec.rs`) or any of the 15+ test fixtures that string-match `"ferro-json-ui/v2"` (Source: grep on `spec.rs` returned 18 matches at lines 31, 71, 422, 638, 964-965, 998, 1009, 1022, 1034, 1057, 1084, 1098, 1114, 1137, 1150, 1162, 1177, 1290).

**Why it happens:** "v2" appears in both wire literals (KEEP) and historical narrative (REMOVE). A naive sed sweep can corrupt the wire literal.

**How to avoid:** D-10 grep gate uses `\bferro-json-ui/v1\b` — that anchor explicitly excludes `ferro-json-ui/v2` from the gate. Inverse: never write a sed pattern that targets `ferro-json-ui/v[12]?` — always anchor to `/v1` specifically.

**Warning signs:** test failures in `spec.rs::tests::*` (the test fixtures string-match the wire literal); ferro-json-ui catalog tests fail; any spec round-trip test fails.

### Pitfall 2: Deleting `migration_v1_to_v2_templates` but Leaving the Comment

**What goes wrong:** Deleting the function body (lines 1504-1697) but forgetting the `// v1 → v2 migration patterns` comment at line 78 and the `templates.extend(...)` call at line 79.

**How to avoid:** D-04 lists all three sites. The Edit tool diff should touch line 78, line 79, lines 1504-1697, and lines 1818-1830 in a single coordinated change.

**Warning signs:** orphaned comment line above `templates` at end of `build_templates()`; `cargo clippy` warns on the dead comment? (no — clippy doesn't warn on comments, but human review catches it).

### Pitfall 3: gestiscilo `[patch.crates-io]` Already Pointing at Local ferro

**What goes wrong:** D-09 specifies "point `ferro = { path = "../ferro" }` for gestiscilo verification". The current gestiscilo `Cargo.toml` (VERIFIED 2026-05-17, lines 94-100) already has `[patch.crates-io]` pointing at `../../albertogferrario/ferro/framework` etc. — the verification is **already set up**. Trying to "set it up" again could break the existing patch.

**Path note:** the existing patch path is `../../albertogferrario/ferro/framework` (two levels up from `/Users/alberto/repositories/gestiscilo-it/app`) — confirming gestiscilo is at `/Users/alberto/repositories/gestiscilo-it/app` and ferro at `/Users/alberto/repositories/albertogferrario/ferro`. Both repos coexist on disk as expected.

**How to avoid:** verify the existing patch points at the v12.0/json-ui-v2 worktree before running `cargo test` from gestiscilo. The patch path is correct as-is; no edit needed.

**Warning signs:** gestiscilo build fetches `ferro-rs 0.2.35` from crates.io instead of the local source — manifests as the published v0.2.35 missing new APIs that v12.0/json-ui-v2 introduced.

### Pitfall 4: `cargo clippy --all` vs CI's `clippy --workspace`

**What goes wrong:** Local `cargo clippy --all` may include or exclude crates that CI's `cargo clippy --workspace` covers differently for edge cases. The CONTEXT.md D-09 command is `cargo clippy --all --all-targets -- -D warnings` — verify this matches what CI runs.

**How to avoid:** check `.github/workflows/` for the exact CI command before declaring the gate green. Per memory `feedback_ci_clippy_command_match.md`, the local pre-push command must match CI exactly.

**Warning signs:** local clippy passes, PR CI fails on a warning that didn't surface locally.

### Pitfall 5: Forgetting to Run `cargo test --all-features`

**What goes wrong:** Running `cargo test` without `--all-features` skips the `projections` feature-gated code (which is where `JsonUiRenderer`, `RenderMode`, `VisualContext`, and `projection::builder` live — Source: `ferro-json-ui/src/lib.rs:90-94`, VERIFIED 2026-05-17). A v1-removal regression in projection code would not surface.

**How to avoid:** always use `--all-features` per CONTEXT.md D-09 and CLAUDE.md.

**Warning signs:** a clean `cargo test` followed by a failing `cargo test --all-features`.

### Pitfall 6: Codemod and Its Fixtures Triggering the D-10 Grep Gate

**What goes wrong:** D-10 specifies "zero matches for `\b(JsonUiView|ComponentNode|PluginProps)\b` across `ferro-json-ui/`, `framework/`, `ferro-mcp/` source trees". The `ferro-cli` codemod and its fixtures (`ferro-cli/src/commands/json_ui_migrate_v1.rs`, `ferro-cli/tests/fixtures/migrate_v1/*.rs`) contain `JsonUiView` literals — but those paths are NOT in the D-10 gate's scope. D-10 is a *narrower* gate than the audit grep.

**Why this matters:** if the planner runs a workspace-wide grep, hits the codemod fixtures, and panics — that's a misreading of D-10. The codemod paths are explicitly **out** of the D-10 scope. The codemod is the only place in the workspace where `JsonUiView` legitimately appears as a string for the tool to recognize.

**How to avoid:** the D-10 gate command must restrict the paths exactly as CONTEXT.md says: `ferro-json-ui/`, `framework/`, `ferro-mcp/` only. Don't broaden it.

**Warning signs:** D-10 gate failing on `ferro-cli/...` paths — that's a false positive; narrow the scope.

### Pitfall 7: `cargo fmt --all -- --check` Treats Comment Rewrites as a Diff

**What goes wrong:** D-02 / D-03 are all comment-text rewrites. `cargo fmt` does not reformat comment text, BUT it does fix surrounding whitespace if the rewrite leaves trailing spaces. A doc-comment rewrite that ends with a trailing space breaks the format check.

**How to avoid:** trim trailing whitespace in every rewritten doc comment. The Edit tool does this naturally if the replacement string is clean.

**Warning signs:** `cargo fmt --check` fails on a file that only had a comment changed.

## Code Examples

Verified patterns from the current ferro-json-ui v2 public API. Use these as substitution targets in doc rewrites and as truth-source for example correctness.

### Loading a spec from a JSON file (the v2 happy path)

```rust
// Source: ferro-cli/src/templates/make.rs:107-131 (VERIFIED 2026-05-17)
// JSON spec file format (e.g. src/views/dashboard.json):
{
  "$schema": "ferro-json-ui/v2",
  "title": "Dashboard",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "Card",
      "props": { "title": "Dashboard", "description": "..." },
      "children": ["heading"]
    },
    "heading": { "type": "Text", "props": { "content": "Dashboard", "element": "h1" } }
  }
}

// Source: ferro-cli/src/templates/make.rs:134-143 (VERIFIED 2026-05-17)
// Rust handler that serves it:
#[handler]
pub async fn dashboard(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/dashboard.json", data)
}
```

### Building a spec in Rust

```rust
// Source: ferro-json-ui/src/lib.rs:19-27 (VERIFIED 2026-05-17 — already public API rustdoc example)
use ferro_json_ui::{Spec, Element};

let spec = Spec::builder()
    .title("Demo")
    .element("root", Element::new("Text").prop("content", "Hi"))
    .build()
    .unwrap();
```

### Using `JsonUiRenderer` (the projection path)

```rust
// Source: ferro-json-ui/src/projection/mod.rs:79-97 (VERIFIED 2026-05-17 — already public API rustdoc example)
use ferro_projections::{ServiceDef, DataType, FieldMeaning, derive_intents};
use ferro_json_ui::{JsonUiRenderer, VisualContext};
use ferro_projections::render::Renderer;

let product = ServiceDef::new("product")
    .display_name("Product")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("name", DataType::String, FieldMeaning::EntityName)
    .field("price", DataType::Float, FieldMeaning::Money);

let intents = derive_intents(&product);
let renderer = JsonUiRenderer;
let result = renderer.render(&product, &intents, &VisualContext::default());
assert!(result.is_ok());

let spec = result.unwrap();
assert_eq!(spec.schema, "ferro-json-ui/v2");
assert!(spec.elements.contains_key(&spec.root));
```

### The D-10 grep gate (verified pattern)

```bash
# Source: V1-DELETION-AUDIT.md:24-30 (audit's existing pattern) + CONTEXT.md D-10
# Run from workspace root:

# Gate 1: no v1 type names in core production trees
grep -rnE '\b(JsonUiView|ComponentNode|PluginProps)\b' ferro-json-ui/src framework/src ferro-mcp/src
# expect: zero matches (or only test fixture in ferro-cli, which is out of scope per D-10)

# Gate 2: no v1 schema literal anywhere outside .planning/
grep -rnE 'ferro-json-ui/v1' . 2>/dev/null | grep -v "\.planning/" | grep -v "target/"
# expect: zero matches

# Gate 3 (CLAUDE.md compliance):
cargo fmt --all -- --check
cargo clippy --all --all-targets -- -D warnings
cargo test --all-features
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `JsonUiView { layout, body }` builder | `Spec::builder().element(id, Element::new(...)).build()` | Phase 115 (commit `dbe5adaf`) | Pre-Phase-160 deletion already complete in source. |
| `Component::Card { children: Vec<Component> }` nested enum | `Element { type_name: "Card", children: Vec<String> }` flat map | Phase 115 | Type-erased dispatch via 41-entry `BUILTIN_TYPES` catalog. |
| `ComponentNode { component, action, visibility }` wrapper struct | `Element { props, action, visible, each, if_ }` | Phase 115; expanded by Phase 163 (`$each`, `$if`) | Wire format change; serialization shape consolidated. |
| `PluginProps { plugin_type, props }` generic plugin dispatch | First-class plugin type names via `JsonUiPlugin` + `RawHtml` for HTML islands | Phase 164 D-17a | `RawHtml` is the explicit HTML-island primitive. |
| `MAX_NESTING_DEPTH = 3` | `MAX_NESTING_DEPTH = 5` | Phase 164 D-14 | Real-world dashboards (root → grid → card → row → atom). |
| `JsonUiRenderer` accepted `RenderContext::default()` | `JsonUiRenderer` accepts `VisualContext::default()` | Phase 12.x projection-prep work | Stale rustdoc in `docs/src/features/projections.md:38-44` (in-scope for Phase 160). |

**Deprecated/outdated:**
- The `migration_v1_to_v2_templates()` MCP function (delete per D-04) — the migration story ends with Phase 160.
- `TODO(Phase 120):` markers — Phase 120 is the v2-cutover phase; Phase 160 closes its remaining items.
- The `// Silence unused-import warnings until Plan 03 rewires the legacy renderer.` comment at `ferro-json-ui/src/projection/builder.rs:42` — Plan 03 ran; the `_plan_02_reserved` placeholder is dead code that should be deleted (D-03 sweep finding).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The `ferro json-ui:migrate-v1` codemod, its source file, fixtures, and integration tests should remain in v12.0 (not deleted in Phase 160). | Summary § band 3; Open Questions Q1 | If wrong: an additional plan-unit is needed to delete the codemod + subcommand + fixtures + tests + CHANGELOG-entry-update. The codemod is real shipped behavior; deleting it is a user-facing change. **Mitigation:** the planner should explicitly close this with the user during plan time. |
| A2 | The `ferro-code` repo at `/Users/alberto/repositories/albertogferrario/ferro-code` is empty and ferro-code verification per D-09 cannot run in Phase 160. | Summary § band 4; Open Questions Q2 | If wrong: verification needs to expand to include ferro-code (extra plan-unit; new build/test commands). **Verified 2026-05-17:** `ls -la /Users/alberto/repositories/albertogferrario/ferro-code` returns an empty directory; no Cargo.toml, no source. The repo is structurally not present. |
| A3 | The `application_info::scan_json_ui_specs` field shape (`JsonUiSpecsStatus { available, view_count, views_dir, hint }`) has no downstream MCP-consumer test that pins the field names. | Pattern 2 | If wrong: MCP-consumer agents (e.g. claude-code itself running against ferro projects) may break when `view_count` semantics flip from "v1 .rs count" to "v2 .json count". **Verified 2026-05-17:** workspace grep for `view_count` returned only `application_info.rs` itself. **Mitigation:** Phase 115 Plan 04's SUMMARY already classified this as "MCP output type — schema-breaking only for MCP consumers, which is acceptable per project norms". The semantic flip is intentional. |
| A4 | The user-naming constraint (mirrored as D-02/D-03/D-07/D-08) means the `ferro-json-ui/README.md` v1 example is in-scope for Phase 160. | Pattern 6 | If wrong: the README rewrite gets descoped, and crates.io publish at Phase 161 ships a stale README. **Mitigation:** very low risk because the README is currently non-compiling against the public API; rewriting it is unambiguously a correctness fix, not a framing concern. |
| A5 | The `docs/src/reference/cli.md` `make:json-view` v1 example is in-scope for Phase 160. | Pattern 7 | Same risk profile as A4 — the example is currently inaccurate (wrong output path, wrong code shape). Rewriting it is a correctness fix. |
| A6 | The `ferro-cli/src/templates/make.rs:810-817` `json_view_template_has_no_v1_markers` test should be kept (it ENFORCES the no-v1 invariant for generated templates). | Pattern 8 § Exceptions | If wrong: the test would also be deleted, removing the existing regression guard. Recommendation is to KEEP. |
| A7 | The `nyquist_validation` config key is absent from `.planning/config.json`; orchestrator's "absent = enabled" rule applies, so this RESEARCH.md includes the Validation Architecture section. | Validation Architecture | If wrong: the section is harmless (planner ignores). |
| A8 | The Phase 159 gate (browser test of /pagamenti) was closed by commit `6601c015` per CONTEXT.md `canonical_refs`. This research does not re-run the browser test. | Inputs to this research | If wrong: Phase 160 may proceed on an unverified Phase 159 gate. **Mitigation:** the planner should add a one-line verification (read the Phase 159 `BROWSER-CHECK.md` for "Verdict: PASS") to the Wave 3 verification plan; cost is minimal. |

**Two assumptions (A1, A2) are load-bearing for plan scope.** The planner should resolve both with the user before locking the plan structure.

## Open Questions

1. **Keep or delete the `ferro json-ui:migrate-v1` codemod?**
   - What we know: the codemod is real shipped behavior (CHANGELOG entry under Unreleased, lines 53-54). Its source legitimately contains `JsonUiView` literals — required for the tool to function. The codemod was the migration path for consumers; gestiscilo and any other v1 codebase have already migrated. The user-naming constraint says "no migration story belongs in agent-readable surface".
   - What's unclear: does the existence of the subcommand `ferro json-ui:migrate-v1` and the help-text mention of "v1" violate the naming constraint? The subcommand name is itself a v1-framing artifact.
   - Recommendation: **keep for v12.0**. The codemod is a self-contained tool; its presence does not contaminate the renderer or the public type surface. Deleting it is correct under a strict reading of the user-naming constraint but is a separate scope-of-work concern. **Have the planner present this to the user with a yes/no decision before locking the plan.** If user picks "delete", add one plan unit covering: delete `ferro-cli/src/commands/json_ui_migrate_v1.rs`, delete `ferro-cli/src/commands/mod.rs:21` registration, delete `ferro-cli/src/main.rs:166` and `:592` registration, delete `ferro-cli/tests/json_ui_migrate_v1.rs`, delete `ferro-cli/tests/fixtures/migrate_v1/` directory, update CHANGELOG.

2. **How to satisfy D-09 ferro-code verification when the repo is empty?**
   - What we know: `/Users/alberto/repositories/albertogferrario/ferro-code` exists but is empty (no `Cargo.toml`, no source). The repo cannot be built or tested.
   - What's unclear: is ferro-code a planned future repo that does not yet exist, or is it expected to be cloned/created before Phase 160 runs?
   - Recommendation: **descope ferro-code from Phase 160's verification**. Add a note to the Phase 160 SUMMARY explaining ferro-code was not verified due to absent source. Carry the verification to whenever ferro-code first depends on local-path ferro. **Have the planner confirm with the user.**

3. **Does the `docs/src/json-ui/migration-v1-to-v2.md` file still need to exist?**
   - What we know: COMPLETED.md (162 D-20) says the file shipped. CHANGELOG.md still references it. **`ls docs/src/json-ui/migration-v1-to-v2.md` returns "No such file or directory"** (VERIFIED 2026-05-17). The file does not exist in the current tree. SUMMARY.md does not link to it (also verified). `docs/src/json-ui/components.md` and other json-ui docs do not link to it.
   - What's unclear: was the file deleted in a later phase under the naming constraint? Is the CHANGELOG entry stale? Or is the file under a different name?
   - Recommendation: **leave alone in Phase 160**. The file's absence is consistent with the user-naming constraint ("no migration page"). The CHANGELOG entry that mentions it is "Unreleased" status anyway — Phase 161 will draft the v12.0 CHANGELOG entry from COMPLETED.md fresh; it can omit the reference.

4. **Should we update the existing `Spec.title` / `TitleBinding` / `DataRef` doc-comments that mention "removed in v1" or similar?**
   - What we know: D-03 sweep of `ferro-json-ui/src/` returned no `removed` matches inside ferro-json-ui source (only in render comments listed in Pattern 8). The `TitleBinding` doc-comment at `spec.rs:43-53` is already neutral.
   - What's unclear: nothing — this is a probe that returned clean.
   - Resolution: no action; D-03 sweep table is complete as listed in Pattern 8.

## Environment Availability

Phase 160 is primarily source-edit work. Dependencies:

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` toolchain | All build/test steps | ✓ | 1.88.0 | — |
| `rustc` | All build/test steps | ✓ | 1.88.0 | — |
| `cargo fmt` | Format check | ✓ (toolchain-bundled) | 1.88.0 | — |
| `cargo clippy` | Lint check | ✓ (toolchain-bundled) | 1.88.0 | — |
| `mdbook` | Optional — only needed to re-run Phase 159 docs gate after doc rewrites | not verified inline; was present for Phase 159 | — | Skip docs gate re-run; trust Phase 159's gate close. |
| `grep` | D-10 verification gate | ✓ (system) | system | `rg` if installed |
| gestiscilo repo at `/Users/alberto/repositories/gestiscilo-it/app` | D-09 cross-repo verification | ✓ | local-path patched at `Cargo.toml:94-100` | — |
| ferro-code repo at `/Users/alberto/repositories/albertogferrario/ferro-code` | D-09 cross-repo verification | **✗ (empty dir)** | — | Descope ferro-code from D-09; record in SUMMARY. |

**Missing dependencies with no fallback:** None block execution. The single missing dep is ferro-code, and the fallback (descope) is acceptable per Open Question Q2.

**Missing dependencies with fallback:**
- `ferro-code` verification — fallback: descope; verify ferro-code consumes ferro cleanly whenever ferro-code first appears.

## Validation Architecture

> Included because `workflow.nyquist_validation` is absent from `.planning/config.json`; the orchestrator's "absent = enabled" rule applies.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust toolchain 1.88.0) + `cargo clippy` + `cargo fmt` |
| Config file | `Cargo.toml` (workspace), per-crate `Cargo.toml` (test deps) |
| Quick run command | `cargo test --all-features -p ferro-json-ui -p ferro-mcp` (targeted to the two crates Phase 160 modifies) |
| Full suite command | `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` |

### Phase Requirements → Test Map

CONTEXT.md does not enumerate REQ-IDs for Phase 160 (the phase predates the REQUIREMENTS.md AISDK ID scheme — see init context: `phase_req_ids is null`). The phase requirements come from ROADMAP.md ("cargo build green; cargo test green; cargo clippy clean; no reference to JsonUiView / ComponentNode / Component:: remains") and from CONTEXT.md D-01..D-11.

| Req | Behavior | Test Type | Automated Command | File Exists? |
|-----|----------|-----------|-------------------|-------------|
| ROADMAP-1 | `cargo build --all-features` exits 0 | smoke | `cargo build --all-features` | ✅ (no new test file needed) |
| ROADMAP-2 | `cargo test --all-features` exits 0 | full suite | `cargo test --all-features` | ✅ |
| ROADMAP-3 | `cargo clippy --all --all-targets -- -D warnings` exits 0 | lint | `cargo clippy --all --all-targets -- -D warnings` | ✅ |
| ROADMAP-4 | No reference to `JsonUiView` in production source (ferro-json-ui/, framework/, ferro-mcp/) | grep gate | `! grep -rnE '\bJsonUiView\b' ferro-json-ui/src framework/src ferro-mcp/src` | ✅ (shell one-liner) |
| ROADMAP-5 | No reference to `ComponentNode` in production source | grep gate | `! grep -rnE '\bComponentNode\b' ferro-json-ui/src framework/src ferro-mcp/src` | ✅ |
| ROADMAP-6 | No reference to `Component::` in production source | grep gate | `! grep -rnE '\bComponent::' ferro-json-ui/src framework/src ferro-mcp/src` | ✅ |
| D-10 | No `ferro-json-ui/v1` literal workspace-wide outside `.planning/` | grep gate | `! (grep -rnE 'ferro-json-ui/v1' . 2>/dev/null \| grep -v '\.planning/' \| grep -v 'target/' \| grep -v 'docs/book/')` (excluding `docs/book/` since it regenerates on `mdbook build`) | ✅ |
| D-05 | `scan_json_ui_specs` counts v2 JSON spec files | unit (new) | `cargo test -p ferro-mcp scan_json_ui_specs` | ❌ Wave 0 — new test file needed |
| D-06 | `test_ignores_non_json_files` exists with neutral fixture | unit (rename) | `cargo test -p ferro-mcp test_ignores_non_json_files` | ✅ (exists; rename only) |
| D-04 | `code_templates_returns_migration_patterns` test is gone (and the function with it) | regression | `! grep -n 'code_templates_returns_migration_patterns' ferro-mcp/src/tools/code_templates.rs` | ✅ (shell one-liner) |
| D-02/D-03 | No `Port of v1` or `Differences from v1` framing in `ferro-json-ui/src/render/` | grep gate | `! grep -rnE 'Port of v1\|Differences from v1' ferro-json-ui/src/render/` | ✅ |
| D-07 | The four reframed prose passages no longer contain `ferro-json-ui/v1` | grep gate | `! grep -nE 'ferro-json-ui/v1' docs/protocol/src/{terminology,architecture,rendering}.md docs/src/features/projections.md` | ✅ |
| D-09 | ferro builds + tests green; gestiscilo builds + tests green | smoke (cross-repo) | (ferro) `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`; (gestiscilo) `cd /Users/alberto/repositories/gestiscilo-it/app && cargo test` | ✅ (commands; ferro-code descoped per Open Q2) |

### Sampling Rate

- **Per task commit:** `cargo fmt --all -- --check && cargo clippy -p <crate> --all-targets -- -D warnings && cargo test -p <crate> --all-features` (the affected crate only)
- **Per wave merge:** Full suite — `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green + D-10 grep gate green + gestiscilo `cargo test` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-mcp/src/tools/application_info.rs` — add `#[cfg(test)] mod tests` covering `scan_json_ui_specs` happy path (`temp dir with 2 .json files → view_count == 2`) and empty-dir path (`no views dir → available == false, view_count == 0`).
- [ ] All other test infrastructure already exists.

## Security Domain

> Required when `security_enforcement` is enabled (absent from `.planning/config.json` → enabled by default).

### Applicable ASVS Categories

Phase 160 is a deletion + doc-rewrite phase. It does not introduce new authentication, session, access-control, or cryptographic surface. The only ASVS category materially relevant is V14 (Configuration) — making sure the deletion does not weaken any existing security stance.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Phase 160 does not touch auth code. |
| V3 Session Management | no | Phase 160 does not touch session code. |
| V4 Access Control | no | Phase 160 does not touch access-control code. |
| V5 Input Validation | no | Phase 160 does not touch input validators or spec validation. The two-stage validation pipeline (load-time vs render-time per Phase 164 D-16) is unaffected. |
| V6 Cryptography | no | Phase 160 does not touch crypto. |
| V14 Configuration | yes (advisory) | The MCP `scan_json_ui_specs` rewrite changes what the introspection tool reports. Verify the rewrite does not regress information-disclosure protections — specifically, the existing function does not enumerate filenames in the `hint` field, and the rewrite should preserve that (paths in `views_dir` are project-relative, not absolute). |

### Known Threat Patterns for {stack}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Stale grep gate exposes an undeleted `JsonUiView` import | Information disclosure (indirect) | D-10 grep gate enforced before merge; CI clippy `-D warnings` catches unused imports. |
| Rewritten `scan_json_ui_specs` leaks absolute paths in `hint` | Information disclosure | Preserve the existing pattern of `views_dir: "src/views/"` (project-relative, not absolute); reject any rewrite that uses `views_dir.display().to_string()` (which would emit the absolute path). |
| Doc rewrites inadvertently include user-system paths (e.g. an Edit-tool diff that pastes in `/Users/alberto/...`) | Information disclosure | Code review; the proposed prose in Pattern 5 contains no system paths. |

**No security regressions expected.** The phase removes code; it does not add executable surface.

## Sources

### Primary (HIGH confidence)

- **Phase 164 V1-DELETION-AUDIT.md** (`.planning/phases/164-json-ui-improvements-batch-3-documenti-field-test-findings-m/V1-DELETION-AUDIT.md`) — comprehensive v1→v2 surface audit; zero BLOCKERS; 2026-05-17 grep evidence.
- **Phase 164 COMPLETED.md** §5 — embedded migration table; downstream of V1-DELETION-AUDIT.md.
- **Phase 164 PLUGIN-SURFACE-AUDIT.md** — confirms plugin surface gaps closed (outcome B).
- **CONTEXT.md** (this phase's CONTEXT.md, dated 2026-05-17) — eleven locked decisions D-01..D-11.
- **`ferro-json-ui/src/spec.rs:31`** — `pub const SCHEMA_VERSION: &str = "ferro-json-ui/v2";` (the wire literal that stays).
- **`ferro-json-ui/src/lib.rs:47-87`** — current public API of the crate post-Phase-115 deletion.
- **`ferro-json-ui/src/render/atoms.rs:26-37`** — neutral doc-style exemplar.
- **`ferro-json-ui/src/projection/mod.rs:79-97`** — canonical `JsonUiRenderer` rustdoc example (v2-correct).
- **CLAUDE.md (project)** — testing / commit / project-agnostic-crate rules.
- **CLAUDE.md (user global)** — repo-document-as-neutral rules, trigger phrases.
- **`Cargo.toml:35`** — `rust-version = "1.88.0"`.
- **Bash inline verification (2026-05-17)** — `cargo --version`, `rustc --version`, `ls`, `grep -rn` commands run during this research; all results inlined above with VERIFIED tags.

### Secondary (MEDIUM confidence)

- **Phase 159 159-VERIFICATION.md** — confirms the Phase 159 gate state (gaps_found at time of report; CONTEXT.md says commit `6601c015` closed the gap, but this research did not re-run the browser test — Assumption A8).
- **Phase 115 Plan 04 SUMMARY.md** — historical record of the `JsonUiViewsStatus → JsonUiSpecsStatus` rename and the "v1 scanner by design" TODO marker; explains why the v1 semantics persisted into Phase 160.
- **CHANGELOG.md (Unreleased)** — references shipped artifacts (e.g. `migration-v1-to-v2.md`) that no longer exist; flagged in Open Question Q3.

### Tertiary (LOW confidence)

- None. All non-trivial claims were cross-verified against current source via inline Bash.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — toolchain versions, mdbook, grep all verified inline; no novel deps.
- Architecture (Phase 160 work flow): HIGH — derived from CONTEXT.md decisions + verified grep findings; no novel architecture introduced.
- Pitfalls: HIGH — pitfalls 1-7 cite specific line numbers and verified test paths.
- Open Questions: MEDIUM — Q1 (codemod) and Q2 (ferro-code) require user resolution; this research recommends but does not decide.
- Cross-repo verification: MEDIUM for gestiscilo (`[patch.crates-io]` verified; suite command inferred from convention but not inline-executed). LOW for ferro-code (verified empty; cannot be verified).

**Research date:** 2026-05-17
**Valid until:** 2026-06-17 (30 days; phase is well-bounded; consumers' local repos may evolve)
