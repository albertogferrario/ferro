# Phase 228: README and Scaffold Doc Sweep — Research

**Researched:** 2026-06-15
**Domain:** Documentation consistency — READMEs, scaffold template, user-facing shell scripts
**Confidence:** HIGH (all findings verified by direct file inspection; no external lookups needed)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Fix threshold is factual accuracy + install-method consistency only — no prose rewrites.
- **D-02:** Homebrew is the lead install method everywhere. Concrete known fixes:
  - `ferro-cli/src/templates/files/root/README.md.tpl:10` — `cargo install ferro-cli` → brew-first
  - `ferro-cli/src/templates/files/root/README.md.tpl:82` — troubleshooting line → brew-first
  - `scripts/create-app.sh:142` — `cargo install ferro-cli` → brew-first (keep cargo as alternate)
  - Verify `scripts/install.sh` messaging (audit during research)
- **D-03:** State the toolchain-free distinction consistently: CLI toolchain-free via Homebrew; Rust 1.88+ required to build/run a scaffolded app.
- **D-04:** Fix `README.md:185` — stale `v0.2.0 / v12.0 spec-driven rendering`. Replace with low-churn phrasing (e.g. "Pre-1.0; breaking changes allowed between minor versions until 1.0.").
- **D-05:** Produce a ready-to-paste tap README draft as an artifact inside the ferro repo at `.planning/phases/228-readme-and-scaffold-doc-sweep/tap-README-draft.md`. Do NOT push/commit to the separate `albertogferrario/homebrew-ferro` repo.

### Claude's Discretion

- Exact wording of each README/script edit, determined during execution from the verified flow.
- Whether `scripts/install.sh` needs any edit (depends on current messaging — researched below).

### Deferred Ideas (OUT OF SCOPE)

- Tap-repo README commit — draft only in this phase; cross-repo commit is a separate user action.
- CHANGELOG — still deferred from Phase 227.
</user_constraints>

---

## Summary

This phase corrects factual drift between four files and the `docs/src/getting-started/installation.md`
consistency target. No feature code changes. All findings come from direct file reads; confidence is HIGH.

The drift falls into three categories:

1. **Install-method ordering** — `cargo install ferro-cli` appears as the primary install method in
   the scaffold README template (lines 10 and 82) and in `scripts/create-app.sh` (line 142). Homebrew
   should lead.

2. **Stale version / milestone copy** — `README.md:185` names `v0.2.0` (current: 0.2.61) and
   `v12.0 spec-driven rendering` (shipped; v15.0 has since shipped). Needs neutral, low-churn
   replacement.

3. **Stale or wrong commands** — `scripts/install.sh:184` emits `ferro migrate` (not a valid command;
   correct is `ferro db:migrate`). `scripts/create-app.sh:138-139` emits `cargo run -- migrate` /
   `cargo run -- serve` (should be `ferro db:migrate` / `ferro serve` after the binary is in PATH, or
   at minimum aligned with the real flow). The scaffold README template line 85 tells users to `cargo run`
   once to generate types — factually correct but worded without the `ferro serve` framing.

4. **Minor version inconsistency** — scaffold README `README.md.tpl:9` says "Rust (stable, 1.75+)".
   `docs/src/getting-started/installation.md` and the published install notes (d7837df6) say Rust 1.88+.
   The scaffold README must match the canonical minimum.

`scripts/install.sh` itself has no `cargo install` messaging (it downloads a pre-built binary), but does
emit `ferro migrate` (line 184) instead of `ferro db:migrate` — a command that does not exist in the CLI.

The tap repo (`albertogferrario/homebrew-ferro`) is confirmed absent from the local working tree; the
deliverable is a draft file only.

**Primary recommendation:** Four targeted edits (tpl:9, tpl:10, tpl:82, tpl:85) + one README.md edit
(line 185) + two script fixes (install.sh:184, create-app.sh:138-139 and 142) + tap draft file. Run
`cargo test -p ferro-cli -- test_readme_substitution` to gate the template edit.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Root README | Repository root doc | — | Static text; no code path |
| Scaffold README template | `ferro-cli` crate (template renderer) | Rust test gate | Rendered at `ferro new` via `templates::readme()` |
| install.sh / create-app.sh | Scripts (shell) | — | User-facing; no Rust build |
| Tap README draft | Planning artifact | — | Draft only; not committed to tap repo |

---

## Consistency Target (Source of Truth)

Exact phrasing from `docs/src/getting-started/installation.md` that all files must reconcile to:

**Toolchain-free CLI distinction (line 1-4):**
> The Ferro CLI installs toolchain-free via Homebrew (below). The following are only needed to
> **build and run** a scaffolded app:
> - Rust 1.88+ (with Cargo) — to build the app (no OpenSSL needed; the scaffold uses rustls)
> - Node.js 18+ (for the frontend dev server)
> - PostgreSQL, SQLite, or MySQL — SQLite is the default, no setup required

**Brew-first install block:**
```
brew install albertogferrario/ferro/ferro
```

**Correct command flow:**
```
ferro new my-app
cd my-app
ferro serve
```

**Correct migrate command:**
```
ferro db:migrate
```

---

## File Inventory and Discrepancy List

### 1. Root `README.md`

**File:** `README.md` (190 lines)

**Install block (lines 17-25):** Already brew-first. No change needed.

```markdown
# macOS / Linux — no Rust required
brew install albertogferrario/ferro/ferro

# or, with Rust:  cargo install ferro-cli
```

The secondary `cargo install` at line 21 is inline as an alternate — this is correct per D-02
(keep cargo as alternate).

**Discrepancies:**

| Line | Current text | Issue | Fix |
|------|-------------|-------|-----|
| 185 | `v0.2.0 — pre-1.0. Breaking changes are allowed between minor versions until 1.0. Current milestone work targets v12.0 spec-driven rendering.` | Stale version (0.2.0 → 0.2.61) and milestone (v12.0 → v15.0 shipped) | Replace with neutral, low-churn phrasing per D-04 |

**No other stale version strings found.** Grep for `v0\.\|v12\.\|v13\.\|v14\.\|v15\.\|native-tls\|openssl\|cargo install` confirmed:
- Only `cargo install` appears at line 21 as an acknowledged alternate (acceptable)
- No `native-tls` or OpenSSL anywhere in root README
- No other version pins besides line 185

**Line 185 proposed replacement (exact wording is Claude's discretion per D-01/D-04):**

> Pre-1.0. Breaking changes are allowed between minor versions until 1.0.

(Drops the specific version number and milestone name — both rot.)

---

### 2. Scaffold README Template `ferro-cli/src/templates/files/root/README.md.tpl`

**File:** 88 lines. Substitution variables: `{project_name}`, `{project_title}`, `{description}`.

**Rendering path:** `ferro-cli/src/templates/project.rs:221` — `readme()` function applies three
`.replace()` calls. No other transformation. The `test_readme_substitution` test (mod.rs:582-589)
asserts: `# My App`, `A test description`, `cd my-app`, `ferro serve`, `ferro db:migrate` all
present — it tests substitution and the presence of those strings, not their surrounding prose.
Edits to prose around those strings are safe as long as those five strings remain.

**Discrepancies:**

| Line | Current text | Issue | Fix |
|------|-------------|-------|-----|
| 9 | `- **Rust** (stable, 1.75+) — install via [rustup](https://rustup.rs)` | MSRV is 1.88+ (per commit d7837df6 and installation.md). 1.75 is stale. | Change to `1.88+` |
| 10 | `- **Ferro CLI** — \`cargo install ferro-cli\` (or build from source)` | `cargo install` is the primary method shown; brew should lead | `brew install albertogferrario/ferro/ferro` as primary; `cargo install ferro-cli` and build-from-source as alternates |
| 11 | `- **Node.js** 20+ and **npm**` | Inconsistent with installation.md (18+). Vite 5 (the scaffolded app's bundler) requires Node 18+, not 20+. | Change to `18+` |
| 82 | `- **\`ferro: command not found\`** — install with \`cargo install ferro-cli\`.` | Troubleshooting leads with cargo. Should lead with brew, keep cargo as fallback. | `brew install albertogferrario/ferro/ferro` first |
| 85 | `- ... run \`cargo run\` once to generate types before running \`npm run dev\`.` | Not incorrect (works before CLI is installed) but inconsistent with `ferro serve` flow which auto-generates types on start. The sentence implies bare `cargo run` is the canonical path. | Align to `ferro serve` (which auto-generates types on each server start). Existing wording is factually correct but the last clause already says "Types are regenerated automatically on each server start" — the `cargo run` instruction is redundant and confusing when `ferro serve` is available. Propose replacing `cargo run` with `ferro serve`. |

**Template variable safety:** Lines to edit contain no `{...}` substitution tokens. Safe to edit prose
around them without breaking the render.

**Cargo.toml.tpl oracle — scaffold dependency claims verified:**
- `sea-orm-migration` features: `["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls"]` — confirmed rustls, no native-tls.
- `sea-orm` features: `["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-rustls", "macros"]` — SQLite is included (default in practice).
- The scaffold README line 12 ("SQLite is used by default; no extra install needed") is **correct**.
- No `native-tls` or OpenSSL anywhere in `Cargo.toml.tpl` — **no change needed here**.

---

### 3. `scripts/install.sh`

**File:** 207 lines. This script downloads a pre-built binary from GitHub releases and optionally
creates a project. It does NOT invoke `cargo install`.

**What it does (context for planner):**
- Detects platform → fetches latest GitHub release tag → downloads the `ferro-{VERSION}-{PLATFORM}.tar.gz`
  binary archive → extracts to `$HOME/.ferro/bin` → optionally runs `$INSTALL_DIR/ferro new <name>`.
- No Rust toolchain invocations at any point.

**Discrepancies:**

| Line | Current text | Issue | Fix |
|------|-------------|-------|-----|
| 184 | `printf "  ${CYAN}ferro migrate${NC}\n"` | `ferro migrate` is not a valid CLI command. The CLI has `ferro db:migrate` (registered as `db:migrate` in `main.rs`). | Change to `ferro db:migrate` |

**No `cargo install` messaging anywhere in install.sh** — D-02 scout finding for this script is confirmed
correct (no messaging to change). The script's install method (pre-built binary download) is already
consistent with toolchain-free CLI.

**PATH setup messaging (lines 156-164):** Correctly tells users to add `$HOME/.ferro/bin` to PATH.
No inconsistency.

---

### 4. `scripts/create-app.sh`

**File:** 146 lines. This script downloads the CLI to a temp dir, runs `ferro new`, inits git, then
discards the temp binary.

**What it does (context for planner):**
- Downloads pre-built binary to temp dir (same download mechanism as install.sh).
- Runs `$TMP_DIR/ferro new $PROJECT_NAME --no-interaction --no-git`.
- Inits git.
- Prints "Next steps" to stdout, then discards the temp binary.
- Tells the user to install the CLI permanently via `cargo install ferro-cli` (line 142).

**Discrepancies:**

| Line | Current text | Issue | Fix |
|------|-------------|-------|-----|
| 138 | `printf "  ${CYAN}cargo run -- migrate${NC}\n"` | `cargo run -- migrate` is non-standard; canonical command is `ferro db:migrate`. This appears in "Next steps" where the CLI may not yet be in PATH — but since the script already downloaded the binary, the right message is `ferro db:migrate` (after installing the CLI). | Change to `ferro db:migrate` |
| 139 | `printf "  ${CYAN}cargo run -- serve${NC}\n"` | Same: canonical command after CLI install is `ferro serve`. | Change to `ferro serve` |
| 142 | `printf "  ${CYAN}cargo install ferro-cli${NC}\n"` | `cargo install` shown as the only permanent-install option; brew should lead per D-02. | Change to brew-first, keep `cargo install ferro-cli` as alternate per D-02. |

**Note on line 137 context:** The "Next steps" block shows `cargo run -- migrate` and `cargo run -- serve`
as if the user doesn't have `ferro` in PATH. This is technically true for the temp-download flow.
However, the block is immediately followed by "Or install Ferro CLI globally: cargo install ferro-cli",
so the intent is clearly to guide toward a permanent install. The fix is: show `ferro db:migrate` and
`ferro serve` (the proper commands after installing), and update line 141-142 to brew-first + cargo as
alternate. The planner should reframe lines 135-143 as "Install the CLI, then:".

---

## Verification Mechanism (per Fix)

| Fix | Verification |
|-----|-------------|
| README.md:185 stale version/milestone | `grep "v0.2.0\|v12.0" README.md` returns no matches |
| tpl:9 Rust 1.75 → 1.88 | `grep "1.75" ferro-cli/src/templates/files/root/README.md.tpl` returns no matches |
| tpl:10 brew-first | `grep -n "cargo install ferro-cli" ferro-cli/src/templates/files/root/README.md.tpl` — line 10 no longer primary |
| tpl:11 Node 18+ | `grep "20+" ferro-cli/src/templates/files/root/README.md.tpl` returns no matches |
| tpl:82 troubleshooting brew-first | `grep -n "cargo install ferro-cli" README.md.tpl:82` — no longer present as only option |
| tpl render test | `cargo test -p ferro-cli -- test_readme_substitution` must pass green |
| install.sh:184 ferro db:migrate | `grep "ferro migrate" scripts/install.sh` returns no matches |
| create-app.sh lines 138-139 | `grep "cargo run --" scripts/create-app.sh` returns no matches |
| create-app.sh:142 brew-first | `grep "cargo install ferro-cli" scripts/create-app.sh` — only appears as alternate, not sole option |

**Cargo scope:** The only Rust-compiled artifact touched is `README.md.tpl` (included via `include_str!`
in `project.rs`). Run `cargo test -p ferro-cli -- test_readme_substitution` (scoped) — NOT the full
workspace suite — to confirm the template still renders correctly. All other changes are plain-text
files with no Rust test gate.

---

## Tap-repo README Draft (D-05)

**Confirmed:** `albertogferrario/homebrew-ferro` is not present anywhere in the local filesystem.
The deliverable is a draft file created inside the ferro repo; the planner should create it at:

```
.planning/phases/228-readme-and-scaffold-doc-sweep/tap-README-draft.md
```

**Key facts the draft must state:**
1. One-liner: `brew install albertogferrario/ferro/ferro`
2. What it installs: the Ferro CLI binary — toolchain-free (no Rust required)
3. How the tap works: the formula is auto-bumped on each ferro GitHub release (token-free via
   `workflow_run` CI, per Phase 226); manual updates are not needed.
4. What users can do after install: `ferro new my-app`, `ferro serve`, `ferro --help`
5. Requirement for building an app: Rust 1.88+ is needed to build the scaffolded app (not to use the CLI).
6. Links: main ferro repo, full docs.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Template substitution | Custom regex engine | Existing `str::replace()` chain in `templates/project.rs:221` |
| Test gating | New test | Existing `test_readme_substitution` in `templates/mod.rs:582` |

---

## Common Pitfalls

### Pitfall 1: Breaking the Template Substitution Test
**What goes wrong:** Editing `README.md.tpl` removes one of the five strings the test asserts: `# My App` (comes from `{project_title}`), `A test description` (from `{description}`), `cd my-app` (from `{project_name}`), `ferro serve`, `ferro db:migrate`.
**Why it happens:** The test checks for these literal strings after substitution. If you rename a command or restructure sections, you might move one of these strings.
**How to avoid:** Run `cargo test -p ferro-cli -- test_readme_substitution` immediately after editing the template. The test is fast (no I/O).
**Warning signs:** Test output says `assertion failed` with a `contains` message.

### Pitfall 2: Editing Generated Output Instead of the Template
**What goes wrong:** Editing `README.md` inside a scaffolded app instead of `ferro-cli/src/templates/files/root/README.md.tpl`.
**Why it happens:** The template path is non-obvious.
**How to avoid:** Always edit the `.tpl` file. Any generated-app README is throw-away output.

### Pitfall 3: Stale Command Names in Scripts
**What goes wrong:** Leaving `ferro migrate` (invalid) while fixing only the `cargo install` issue.
**Why it happens:** Multiple unrelated stale items in the same file.
**How to avoid:** Address all items in the discrepancy table per file in a single pass.

### Pitfall 4: Leaving `cargo run -- serve` as the Serve Instruction
**What goes wrong:** User follows `create-app.sh` "Next steps" and types `cargo run -- serve`, which compiles the entire app from source instead of using the installed CLI binary. Misleads users into thinking a Rust toolchain is needed just to run the server.
**Why it happens:** The script was written before `ferro serve` existed (or before brew distribution made the CLI the primary path).
**How to avoid:** Replace with `ferro serve` in lines 138-139 of `create-app.sh`.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Node.js 18+ is the correct minimum (installation.md says 18+; Vite 5 in package.json template requires 18+) | Scaffold README discrepancy | If project actually requires 20+, fixing 20→18 would be wrong. Low risk: Vite 5 explicitly supports Node 18+. [VERIFIED: Cargo.toml.tpl + installation.md] |
| A2 | `ferro migrate` (without `db:`) is not a valid command | Scripts discrepancy | If a `migrate` alias was added, install.sh fix would be unnecessary. Verified by grep on `ferro-cli/src/main.rs` — no `name = "migrate"` registration exists. [VERIFIED: main.rs grep] |

**All other claims verified by direct file inspection.** No assumed knowledge used for the discrepancy list.

---

## Open Questions (RESOLVED)

> Both questions are resolved and adopted in 228-01-PLAN.md.
> **Q1 resolution:** replace `cargo run` with `ferro serve` and drop the "once" qualifier, keeping the "types are regenerated automatically" clarification (Task 1, tpl:85).
> **Q2 resolution:** keep the `npm install` step (create-app.sh line ~137) — it is required for the Inertia frontend and is NOT a discrepancy; only the command lines 138/139/142 change (Task 3).

1. **`cargo run` reference in tpl:85 — remove or keep?**
   - What we know: The line says "run `cargo run` once to generate types before running `npm run dev`". The surrounding sentence immediately clarifies "Types are regenerated automatically on each server start." This is factually true but redundant if the user is using `ferro serve`.
   - What's unclear: Should the troubleshooting line simply replace `cargo run` with `ferro serve`, or should the entire sentence be dropped (since `ferro serve` auto-regenerates)?
   - Recommendation (Claude's discretion per D-01): Replace `cargo run` with `ferro serve` and drop the "once" qualifier, keeping the "types are regenerated automatically" clarification. Result: "run `ferro serve` to start the server — types are regenerated automatically on each start."

2. **create-app.sh "Next steps" framing (lines 135-143)**
   - What we know: After fixing lines 138-139 and 142, the block will show brew-first install + `ferro db:migrate` / `ferro serve`.
   - What's unclear: Line 137 currently says `cd frontend && npm install && cd ..` — is this step still needed before `ferro serve`? The scaffold README (tpl:27) includes it. Should create-app.sh match?
   - Recommendation: Keep the `npm install` step (it is required for Inertia frontend). This is not a discrepancy — it is correct.

---

## Environment Availability

Step 2.6: SKIPPED — this phase consists of plain-text edits to `.md`, `.tpl`, and `.sh` files with
no external tools, services, or runtimes beyond `cargo test -p ferro-cli`.

The one Rust invocation (`cargo test -p ferro-cli -- test_readme_substitution`) requires a local
Rust toolchain, which is present (this is the ferro dev environment).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` via cargo |
| Config file | None (workspace uses cargo defaults) |
| Quick run command | `cargo test -p ferro-cli -- test_readme_substitution` |
| Full suite command | `cargo test -p ferro-cli` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| — | Scaffold README renders with substitutions intact after tpl edit | unit | `cargo test -p ferro-cli -- test_readme_substitution` | ✅ `ferro-cli/src/templates/mod.rs:582` |

### Sampling Rate

- **Per tpl edit:** `cargo test -p ferro-cli -- test_readme_substitution`
- **After all edits:** `cargo test -p ferro-cli` (scoped to ferro-cli, not full workspace)
- **Phase gate:** All edits committed; template test green; grep verifications pass

### Wave 0 Gaps

None — existing test infrastructure covers the only Rust-tested artifact.

---

## Sources

### Primary (HIGH confidence — direct file inspection)

- `README.md` — full read, line 185 confirmed stale
- `ferro-cli/src/templates/files/root/README.md.tpl` — full read, lines 9/10/11/82/85 confirmed
- `ferro-cli/src/templates/files/backend/Cargo.toml.tpl` — confirmed `runtime-tokio-rustls`, no native-tls
- `scripts/install.sh` — full read, line 184 confirmed `ferro migrate` (invalid command)
- `scripts/create-app.sh` — full read, lines 138/139/142 confirmed stale
- `ferro-cli/src/templates/mod.rs` — full read, `test_readme_substitution` at line 582 confirmed
- `ferro-cli/src/templates/project.rs:221` — `readme()` function confirmed, three `.replace()` calls
- `ferro-cli/src/main.rs` — grep confirmed `db:migrate` is the registered command; no bare `migrate` alias
- `docs/src/getting-started/installation.md` — full read; Rust 1.88+, Node 18+, brew-first confirmed as canonical

### Secondary (MEDIUM confidence)

- Vite 5 Node.js requirement (Node 18+) — inferred from `package.json.tpl` specifying `"vite": "^5.0.0"` + Vite 5 upstream docs (public knowledge; not re-verified via web search in this session) [ASSUMED]

---

## Metadata

**Confidence breakdown:**
- Discrepancy list: HIGH — every line number verified by file read
- Command correctness: HIGH — verified against `ferro-cli/src/main.rs`
- Template test safety: HIGH — test source read and assertions mapped
- Node.js version: MEDIUM — canonical source is installation.md (18+); Vite 5 independently requires 18+ [one claim ASSUMED, see Assumptions Log A1]

**Research date:** 2026-06-15
**Valid until:** Until a CLI command rename or new install method is added (stable; no expiry pressure)
