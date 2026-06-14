# Phase 228: README and Scaffold Doc Sweep - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning
**Mode:** `--auto` (gray areas auto-selected, recommended defaults chosen)

<domain>
## Phase Boundary

A factual-accuracy + consistency sweep of every README and user-facing CLI script, aligning
them with the audited `docs/src/` (Phase 227) and the real v0.2.61 install/build flow
(Homebrew-first, rustls/SQLite-default scaffold, `ferro new` → `ferro serve`).

**In scope (all live in THIS repo):**
- Root `README.md` — verify brew-first install + quickstart match docs and the real flow; fix
  stale version/milestone strings.
- Scaffold-generated README template — `ferro-cli/src/templates/files/root/README.md.tpl`
  (rendered by `new.rs` via `templates::readme()`). Align install guidance + reflect the
  rustls/SQLite-default app and `ferro serve` flow.
- `scripts/install.sh` and `scripts/create-app.sh` — user-facing echo/messaging (brew-first;
  toolchain-free-CLI vs Rust-needed-to-build distinction).
- Consistency of the **toolchain-free CLI vs Rust-needed-to-build-the-app** distinction across
  all of the above.

**Out of scope:**
- The `albertogferrario/homebrew-ferro` tap repo README — it is a **separate GitHub repo not
  checked out in this tree**. Editing/committing another repo's tree from this session violates
  the cross-repo boundary. This phase produces a ready-to-paste draft inside the ferro repo; the
  actual tap-repo commit is a separate, user-performed (or explicitly-confirmed `gh`) action.
- `docs/src/` factual content — already audited in Phase 227. Don't re-edit; reconcile to it.
- Prose rewrites / restructuring. Factual accuracy and install-method consistency only.
</domain>

<decisions>
## Implementation Decisions

### Audit scope & fix threshold
- **D-01:** Fix threshold is factual accuracy + install-method consistency only — no prose
  rewrites. Mirror Phase 227's discipline.

### Install-method consistency (brew-first everywhere)
- **D-02:** Make Homebrew the lead install method consistently, matching
  `docs/src/getting-started/installation.md`. Concrete known fixes (scout-confirmed):
  - `ferro-cli/src/templates/files/root/README.md.tpl:10` — `Ferro CLI — `cargo install ferro-cli`
    (or build from source)` → lead with `brew install albertogferrario/ferro/ferro`, keep cargo/
    source as alternates.
  - `ferro-cli/src/templates/files/root/README.md.tpl:82` — troubleshooting `ferro: command not
    found — install with `cargo install ferro-cli`` → brew-first.
  - `scripts/create-app.sh:142` — `Or install Ferro CLI globally: cargo install ferro-cli` →
    brew-first (keep cargo as alternate).
  - Verify `scripts/install.sh` messaging (the curl installer for the CLI binary) does not
    contradict the brew-first recommendation.

### Toolchain-free distinction
- **D-03:** State the distinction consistently wherever install is mentioned: the **Ferro CLI is
  toolchain-free via Homebrew** (no Rust needed to install the CLI), but **Rust 1.88+ is required
  to build/run a scaffolded app**. This phrasing already exists in
  `docs/src/getting-started/installation.md` — reconcile READMEs/scripts to it.

### Stale version / milestone references
- **D-04:** Fix `README.md:185` — currently `v0.2.0 — pre-1.0 ... Current milestone work targets
  v12.0 spec-driven rendering.` This is stale (workspace is 0.2.61; v15.0 has shipped). Replace
  with neutral, low-churn phrasing (e.g. "Pre-1.0; breaking changes allowed between minor
  versions until 1.0.") — avoid pinning a specific milestone number that rots. Sweep the rest of
  `README.md` for other stale version strings.

### Tap-repo README (cross-repo deliverable)
- **D-05:** Produce a ready-to-paste tap README **as an artifact inside the ferro repo** (e.g.
  `.planning/phases/228-readme-and-scaffold-doc-sweep/tap-README-draft.md` or a clearly-marked
  location) describing `brew install albertogferrario/ferro/ferro`, the token-free self-bumping
  tap, and how it tracks ferro releases. Do **NOT** auto-push or commit to the separate
  `albertogferrario/homebrew-ferro` repo from this session — that is an outward-facing, cross-repo
  action the user performs (or explicitly authorizes via `gh`).

### Claude's Discretion
- Exact wording of each README/script edit, determined during execution from the verified flow.
- Whether `scripts/install.sh` needs any edit (depends on what its current messaging says — audit
  during execution).
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition
- `.planning/ROADMAP.md` §"Phase 228" — goal + depends-on (Phase 227).

### The consistency target (source of truth for install copy + the toolchain-free distinction)
- `docs/src/getting-started/installation.md` — the audited, known-good install page; READMEs and
  scripts reconcile their install guidance and the CLI-vs-build distinction to this.

### Files to audit/fix (all in this repo)
- `README.md` — root README (install §19–25 already brew-first; stale §185).
- `ferro-cli/src/templates/files/root/README.md.tpl` — scaffold-generated README (88 lines;
  rendered by `ferro-cli/src/commands/new.rs:163` → `templates::readme()` in
  `ferro-cli/src/templates/project.rs:221`).
- `scripts/install.sh`, `scripts/create-app.sh` — user-facing CLI scripts.

### Ground-truth oracle for scaffold claims
- `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` — confirms rustls + SQLite/Postgres
  drivers (the scaffold README's "SQLite by default / rustls" claims must match this).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The scaffold README is a single template file (`files/root/README.md.tpl`) substituted via
  `templates::readme(project_name, project_title, description)` — edit the `.tpl`, not generated
  output. There is a substitution test at `ferro-cli/src/templates/mod.rs` (`test_readme_substitution`).

### Established Patterns
- `installation.md` already encodes the exact brew-first ordering + toolchain-free phrasing to
  copy. Reconcile, don't reinvent.

### Integration Points
- `new.rs` writes the README at project root on `ferro new`; the `.dockerignore` template
  whitelists `!README.md` (see `templates/docker.rs`) — keep the README present in scaffolds.
- A `.tpl` edit changes generated-app output; the `test_readme_substitution` test must still pass
  (it asserts substitution, not content) — a Rust test, so if the executor edits the `.tpl` it
  should run `cargo test -p ferro-cli readme` (scoped, not the full suite) to confirm the template
  still renders.

</code_context>

<specifics>
## Specific Ideas

- The root README install block (`brew install` → `ferro new` → `ferro serve`) is already
  correct; the main root-README work is the stale §185 milestone line, plus a full-file sweep.
- The scaffold README and `create-app.sh` are where the `cargo install`-first drift lives — those
  are the concrete edits.
</specifics>

<deferred>
## Deferred Ideas

- **Tap-repo README commit** — drafting is in scope (D-05); committing to the separate
  `albertogferrario/homebrew-ferro` repo is a cross-repo action handled outside this session.
- **CHANGELOG** — still deferred (carried from Phase 227); not introduced here.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.
</deferred>

---

*Phase: 228-readme-and-scaffold-doc-sweep*
*Context gathered: 2026-06-15*
