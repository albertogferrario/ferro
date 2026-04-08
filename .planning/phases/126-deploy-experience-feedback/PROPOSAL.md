# Phase 126 — Deploy Experience Feedback: PROPOSAL

**Date:** 2026-04-08
**Source:** REPORT.md (gestiscilo end-to-end deploy field notes)
**Status:** Awaiting user review — no phases created, no code touched

## Summary

Eighteen REPORT items triaged: 2 already shipped (commit `70ad9ed4` / 0.2.1),
1 deferred to gsd-tools (item 11), 1 already in scope of Phase 123/124 with
clarification (item 12), and 14 routed into **three proposed new phases**
(127, 128, 129). Top recommendation: ship Phase 127 first — it contains
item 18 (Dockerfile has no `ENTRYPOINT`/`CMD`), which is a hard blocker for
any actual deploy and should land before any other deploy-experience work.

## Triage Table

| #  | Summary                                                              | Classification    | Target                                 |
|----|----------------------------------------------------------------------|-------------------|----------------------------------------|
| 1  | `rewrite_cargo_docker_toml` dropped dep metadata                     | shipped           | commit 70ad9ed4 / 0.2.1                |
| 2  | `rust:stable-slim-bookworm` is not a real Docker tag                 | shipped           | commit 70ad9ed4 / 0.2.1                |
| 3  | Silent conflict between `copy_dirs` metadata and `.dockerignore`     | new-phase         | Phase 128 (Deploy preflight)           |
| 4  | No version-skew detection between local path deps and registry      | new-phase         | Phase 128 (Deploy preflight)           |
| 5  | `docker:init` reorders dep tables when re-serializing                | new-phase         | Phase 127 (Generated artifact polish)  |
| 6  | Generated Dockerfile runs `cargo build --release` three times        | new-phase         | Phase 127 (Generated artifact polish)  |
| 7  | `docker:init`/`do:init` print success but no "Next steps" footer     | new-phase         | Phase 127 (Generated artifact polish)  |
| 8  | Publish workflow auto-bumps patch on docs/CI-only commits            | new-phase         | Phase 129 (Publish workflow refinement)|
| 9  | No `--dry-run` on `docker:init` / `do:init`                          | new-phase         | Phase 127 (Generated artifact polish)  |
| 10 | `.dockerignore` excludes `*.md` and `LICENSE` (cargo warning noise)  | new-phase         | Phase 127 (Generated artifact polish)  |
| 11 | gsd-tools `phase add` collision + JSON-UI v2 number clash            | deferred-external | gsd-tools repo (manual filing)         |
| 12 | `ferro deploy:check` as a real CLI command (not just MCP)            | already-in-scope  | Phase 124 `ferro doctor` (see D-07)    |
| 13 | Better feedback when ferro is pulled from crates.io vs path          | new-phase         | Phase 128 (Deploy preflight)           |
| 14 | `ferro_version` is global; ferro is many crates                      | new-phase         | Phase 129 (Publish workflow refinement)|
| 15 | `[package.metadata.ferro.deploy]` is a mouthful — interactive init   | new-phase         | Phase 128 (Deploy preflight)           |
| 16 | Generated `.do/app.yaml` `envs:` section is comments-only            | new-phase         | Phase 127 (Generated artifact polish)  |
| 17 | `Cargo.docker.toml` can drift from `Cargo.toml`                      | new-phase         | Phase 128 (Deploy preflight)           |
| 18 | Generated Dockerfile has no `ENTRYPOINT` or `CMD` (deploy blocker)   | new-phase         | Phase 127 (Generated artifact polish)  |

## Cross-Reference Notes

### Already-in-scope citations (D-06)

- **Item 12 — `ferro deploy:check` as a CLI command** is already partially
  covered by **Phase 123** (`deploy_check` MCP tool, defined in
  `123-deploy-mcp-tools/SCOPE.md` under "`deploy_check`") and by **Phase 124**
  (`ferro doctor`, defined in `124-doctor-introspection-and-ci-scaffold/SCOPE.md`
  under "`ferro doctor`"). Phase 122.2 explicitly states the deploy
  simplification "fold[s] surviving checks into `ferro doctor`." See D-07
  resolution below — item 12 is **not** a separate proposal; the new preflight
  checks proposed under Phase 128 are added as Phase 124 `ferro doctor`
  subchecks (or direct extensions to Phase 124's surface).

### D-07 resolution: where does `deploy_check` live?

The user previously made this call in **Phase 122.2** ("Delete 3 MCP deploy
tools and fold surviving checks into `ferro doctor`"). Honoring that:

- **Read-only diagnostics** (env diff, runtime requirements scan, path-dep
  detection, dirty-tree check, copy_dirs vs .dockerignore conflict, version
  skew, `Cargo.docker.toml` staleness) → **`ferro doctor`** (Phase 124 surface).
  This is the natural home per CONTEXT code_context: "The `ferro doctor` /
  introspection pattern (Phase 124) is the natural home for read-only
  diagnostic checks."
- **Mutating / scaffolding** (interactive `deploy:init`, `--dry-run` flags on
  `docker:init`/`do:init`, real `envs:` generation, `Cargo.docker.toml`
  regeneration) → **`ferro deploy:*`** subcommands (122.2 surface).
- **MCP tool exposure** for the diagnostic checks remains via Phase 123's
  `deploy_check` MCP tool, which becomes a thin wrapper over the same
  `ferro doctor` check registry — no double-implementation, no double-booking.

Net effect: there is **one** check implementation, exposed two ways (CLI via
`ferro doctor`, MCP via `deploy_check`). Phase 128 below adds new checks to
that single registry rather than spawning a parallel `ferro deploy:check`
command.

## Proposed New Phases

### Phase 127 — Generated artifact polish (deploy blocker fix + template hygiene)

**Goal:** Make the artifacts that `docker:init` and `do:init` emit actually
runnable end-to-end. Today, even after a successful `docker build`, the
resulting image silently exits because the Dockerfile has no `ENTRYPOINT` or
`CMD` (item 18) — the same gap will break DigitalOcean App Platform deploys
because the generated `app.yaml` `web` service has no `run_command`. Alongside
this critical fix, sweep the small template-quality issues that surfaced in
the same session: stop running `cargo build --release` three times, stop
reordering dep tables on re-serialization, generate real `envs:` entries
instead of comment scaffolds, add a "Next steps" footer to both init commands,
ship `--dry-run` for both init commands, and stop generating cargo warnings
from `.dockerignore`-excluded README files.

**Absorbs REPORT items:** 5, 6, 7, 9, 10, 16, 18

**Depends on:** Phase 122.2 (deploy scaffold surface this builds on). No new
dependencies.

**App applicability:** Both. Item 18 in particular blocks any deploy on either
shape. `gestiscilo-it/app` (multi-bin) needs the `web_bin`-aware ENTRYPOINT
detection; `gestiscilo-it/mkmenu` (single-bin) gets the simpler
`ENTRYPOINT ["/usr/local/bin/<package_name>"]` form.

---

### Phase 128 — Deploy preflight (`ferro doctor` deploy checks + drift detection)

**Goal:** Catch deploy failures *before* a 1–10 minute Docker round-trip.
Extend `ferro doctor` (Phase 124 surface) with the deploy-specific checks the
gestiscilo session discovered one painful build at a time: `copy_dirs`
entries that collide with `.dockerignore` (item 3), version skew between
local path deps and the rewritten `Cargo.docker.toml` (items 4, 13), and
`Cargo.docker.toml` staleness vs `Cargo.toml` (item 17). Also ship the
interactive `ferro deploy:init` scaffolder for the
`[package.metadata.ferro.deploy]` block (item 15) so users do not have to
hand-type the table from docs. The same check registry is exposed via the
existing Phase 123 `deploy_check` MCP tool — one implementation, two surfaces
(see D-07).

**Absorbs REPORT items:** 3, 4, 13, 15, 17

**Depends on:** Phase 124 (`ferro doctor` check registry must exist) and
Phase 123 (`deploy_check` MCP tool to wrap the new checks). Phase 122.2
provides the `Cargo.docker.toml` rewrite surface that items 4 and 17 hook
into.

**App applicability:** Both. `gestiscilo-it/app` is the canonical victim of
all five items in this phase. `gestiscilo-it/mkmenu` benefits from the same
checks (drift detection and the interactive scaffolder are shape-agnostic).

---

### Phase 129 — Publish workflow refinement (gated bumps, per-crate version notes)

**Goal:** Stop releasing every workspace member on docs-only or CI-only
commits. Gate the auto-patch-bump on whether any *library* crate actually
changed (item 8) — `ferro-cli/`-only or `docs/`-only pushes should not churn
versions on every other crate. Document in `PUBLISHING.md` that `ferro_version`
is currently a single global field (item 14) and add a per-crate override
hook for the day a crate desyncs from the lockstep release; do not implement
the desync support until a real desync forces it.

**Absorbs REPORT items:** 8, 14

**Depends on:** None (touches `.github/workflows/publish.yml` and
`PUBLISHING.md` only). Independent of 127 and 128.

**App applicability:** Both — but the benefit is to the *ferro maintainer*,
not to either deployed app directly. Reduces version churn that consumers
(both apps) would otherwise have to update.

## Sequencing Recommendation

Ordered by user pain / real deploy friction (D-05). Hard dependencies are
called out only where they force reordering — none do, here.

1. **Phase 127 — Generated artifact polish** — Ship first. Item 18 is a
   blocker for *any* actual deploy: a successful `docker build` produces a
   non-functional image, and the same root cause will silently break
   DigitalOcean App Platform `web` services. Until this lands, the deploy
   scaffold from Phase 122.2 produces images that exit 0 on first run with no
   logs. The other items in this phase (template hygiene, `--dry-run`, real
   `envs:` generation) are cheap to ship in the same focused session and
   round out the artifact-quality story.
2. **Phase 128 — Deploy preflight** — Ship second. Now that the artifacts
   themselves are correct, prevent the *next* deploy session from rediscovering
   the same five issues one Docker build at a time. Depends on Phase 124's
   `ferro doctor` check registry existing, which is independent of 127 — if
   124 has not landed yet, it should land before 128, but does not block 127.
3. **Phase 129 — Publish workflow refinement** — Ship last. Pure maintainer
   ergonomics; no end-user blocking. Can land any time but is the lowest pain
   of the three for the user actually trying to deploy an app.

## Deferred / External

- **REPORT item 11 (gsd-tools collision bug):** Belongs to the **gsd-tools
  repo**, not ferro. The user must file this manually. The bug has two parts:
  (a) `gsd-tools phase add` assigned `115` four times in a row in one batch
  because it does not see its own previous insertions when computing the next
  integer, and (b) the CLI also collided with an unrelated active milestone's
  phase numbers (JSON-UI v2 was already at 115–121), forcing a manual
  renumber to 122–125. Both should be filed as gsd-tools issues; the second
  is what motivated the 127–129 numbering choice in this proposal.

## Notes

- All three proposed phases preserve the **`Cargo.docker.toml` indirection
  pattern** introduced in Phase 122.2 — none of them mutate `Cargo.toml`
  directly. This is load-bearing for keeping local dev untouched and should
  not be revisited.
- The `runtime_apt` knob from Phase 122.2 is explicitly working well per
  REPORT "What worked well" — none of these proposals change it.
- The `themes/` auto-detection from on-disk presence is also working well —
  no proposal touches it.
- Phase 127 is intentionally larger than the other two because item 18 is
  load-bearing for deploy correctness and the surrounding template fixes are
  cheap to bundle in the same session per D-04 (concrete clustering over
  speculative roadmaps).
- Per D-08, proposed phase numbers (127, 128, 129) sit in the post-126 range
  and do not collide with the JSON-UI v2 milestone (115–121) or the current
  deploy/doctor block (122–126).
