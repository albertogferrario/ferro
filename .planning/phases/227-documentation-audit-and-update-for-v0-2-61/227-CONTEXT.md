# Phase 227: Documentation Audit and Update for v0.2.61 - Context

**Gathered:** 2026-06-15
**Status:** Ready for planning
**Mode:** `--auto` (gray areas auto-selected, recommended defaults chosen)

<domain>
## Phase Boundary

A factual-accuracy audit of every page under `docs/src/` after the v0.2.59 → v0.2.61
changes (brew install added in Phase 226; MSRV 1.88 + rustls/toolchain-free clarified
on 2026-06-14). The phase verifies command/code examples against the live CLI and the
scaffold template source, then fixes discrepancies.

**In scope:** correcting stale facts on existing `docs/src/` pages — TLS/runtime config
(now rustls), install-flow consistency (brew-first), stale version pins, getting-started
walkthrough, generator (`make:*`) and `ferro serve` flow accuracy.

**Out of scope:** rewriting prose for style, restructuring the docs tree, READMEs and the
scaffold's generated README (Phase 228), and introducing new doc infrastructure (e.g. a
CHANGELOG system — see Deferred).
</domain>

<decisions>
## Implementation Decisions

### Audit breadth & fix threshold
- **D-01:** Audit **every** page in `docs/src/`, not only the flagged candidates. The
  install page (`getting-started/installation.md`) is already verified known-good — confirm,
  don't re-touch.
- **D-02:** Fix threshold is **factual accuracy only**. Correct wrong commands, stale config
  snippets, dead version pins, and outdated flow descriptions. Do **not** rewrite prose,
  reorganize pages, or add new sections beyond what a fix requires. "Audit and update," not a
  rewrite (per ROADMAP scope notes).

### Verification method
- **D-03:** Verify every command/code example against ground truth, not by reading alone:
  - CLI commands → check against the real subcommands in `ferro-cli/src/commands/` (e.g.
    `make_auth.rs`, `make_scaffold.rs`, `make_job.rs`, `serve.rs`, `new.rs`).
  - Scaffold config claims → check against `ferro-cli/src/templates/files/backend/Cargo.toml.tpl`
    (already confirmed: uses `runtime-tokio-rustls`, sqlite+postgres, no native-tls/OpenSSL).
- **D-04:** TLS/OpenSSL sweep: confirm no `native-tls` / `runtime-tokio-native-tls` / OpenSSL
  appears in any config or example snippet. Current state (scouted): docs are already clean —
  the only match is the correct "no OpenSSL needed; the scaffold uses rustls" line on the
  install page. Treat this as a verification checkpoint, not a known-large fix.

### Install-flow consistency
- **D-05:** Make **brew the lead install method consistently** across pages, matching
  `getting-started/installation.md`. Specifically `docs/src/reference/cli.md` currently leads
  with `cargo install ferro-cli` (lines 8/16) — reorder so brew is primary and cargo/curl are
  alternates, with the toolchain-free-CLI vs Rust-needed-to-build-app distinction stated.
- **D-06:** Replace stale hard version pins in examples with version-neutral phrasing or a
  `<pinned>` placeholder. Known instance: `docs/src/cli/frontend-types.md:97`
  (`--ferro-version 0.2.33`). Do not chase a "current version" string that will rot again;
  prefer neutral/placeholder phrasing where the exact version is illustrative.

### Generator + serve flow
- **D-07:** Verify the `make:auth` / `make:scaffold` / `make:job` generator docs and the
  `ferro serve` walkthrough against the live CLI behavior. Pages referencing these:
  `getting-started/quickstart.md`, `getting-started/working-with-agents.md`,
  `features/authentication.md`, `features/queues.md`, `reference/cli.md`, and others surfaced
  by grep. Fix any command name / flag / output drift found.

### Claude's Discretion
- Exact per-page edits are determined during execution from the verification results.
- Ordering of the audit sweep (alphabetical, by-staleness-risk, or getting-started-first) is
  Claude's call — getting-started + reference/cli are the highest-signal starting points.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase definition
- `.planning/ROADMAP.md` §"Phase 227" — goal, depends-on (Phase 226), and scope notes
  (known-good = install section; likely-stale = TLS/sea-orm config, version numbers,
  getting-started walkthrough; CHANGELOG question).

### Ground-truth sources for verification
- `ferro-cli/src/commands/` — the authoritative list of real CLI subcommands. Any doc command
  must map to a file here (`make_*.rs`, `serve.rs`, `new.rs`, `db_*.rs`, etc.).
- `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` — authoritative scaffold dependency
  set (confirms `runtime-tokio-rustls`, sqlite+postgres, no native-tls).
- `docs/src/getting-started/installation.md` — the known-good install page; the consistency
  target other pages' install snippets must align to.

### Pages flagged during scout (non-exhaustive — D-01 still requires auditing all)
- `docs/src/reference/cli.md` — leads with `cargo install`; needs brew-first reorder (D-05).
- `docs/src/cli/frontend-types.md` — stale `--ferro-version 0.2.33` pin + `cargo install
  ferro-cli --version <pinned>` (D-06).
- `docs/src/getting-started/quickstart.md`, `docs/src/getting-started/working-with-agents.md`
  — getting-started walkthrough, verify `ferro new` → `ferro serve` flow (D-07).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-cli/src/commands/` directory listing IS the verification oracle for D-03 — no need to
  run the binary for command-existence checks; the file set is authoritative.
- `Cargo.toml.tpl` is the single source of truth for what a scaffolded app actually depends on.

### Established Patterns
- Docs live in `docs/src/` as an mdBook (`SUMMARY.md` is the TOC). Any new page would need a
  `SUMMARY.md` entry — relevant only if D-08 (CHANGELOG) were ever pulled in, which it is not.
- CI runs `cargo doc -Dwarnings` (per project memory) — this audit touches user docs in
  `docs/src/`, not rustdoc, so the doc-build gate is not directly exercised, but don't break
  intra-doc links in `SUMMARY.md`.

### Integration Points
- `getting-started/installation.md` is the canonical install copy; treat it as the source other
  pages reconcile to, not a page to re-edit.
</code_context>

<specifics>
## Specific Ideas

- Scout already confirmed the TLS/rustls migration is **mostly already reflected** in docs — the
  big risk the roadmap worried about (`runtime-tokio-native-tls` scattered in examples) did not
  materialize. The real, found discrepancies are narrower: install-method ordering on
  `reference/cli.md` and the `0.2.33` version pin in `frontend-types.md`. Plan should treat the
  sweep as confirmation-heavy with a small set of concrete fixes, not a large rewrite.
</specifics>

<deferred>
## Deferred Ideas

- **CHANGELOG for 0.2.60/0.2.61** — The ROADMAP scope note asks "consider whether a CHANGELOG
  entry belongs here." Decision: **defer.** No CHANGELOG file/system exists today; introducing
  one is doc *infrastructure*, not a factual-accuracy fix, and would expand this phase past its
  "not a rewrite" boundary. If wanted, it is a small dedicated follow-up (or folds into Phase
  228's README sweep). Not done in Phase 227 unless the user overrides.
- **README / scaffold-generated README / tap README / install-script messaging** — explicitly
  Phase 228. Any README drift found while auditing `docs/src/` is noted for 228, not fixed here.

### Reviewed Todos (not folded)
None — no pending todos matched this phase.
</deferred>

---

*Phase: 227-documentation-audit-and-update-for-v0-2-61*
*Context gathered: 2026-06-15*
