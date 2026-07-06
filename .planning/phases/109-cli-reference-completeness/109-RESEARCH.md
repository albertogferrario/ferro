# Phase 109: CLI Reference Completeness - Research

**Researched:** 2026-03-26
**Domain:** Documentation — ferro-cli reference (docs/src/reference/cli.md)
**Confidence:** HIGH

## Summary

Phase 109 adds reference entries for 13 CLI commands that exist in ferro-cli but have no corresponding documentation entry in `docs/src/reference/cli.md`. All 13 commands are fully implemented and wired in `ferro-cli/src/main.rs` (except `generate-routes`, which is a module with a `run()` fn but not exposed as a standalone subcommand — the requirement still mandates its documentation). The existing documentation format is consistent and well-understood: each command gets a section with synopsis, options table, description, and a code example.

The work is entirely mechanical documentation writing: read source, extract flags, write the section in the established format. No code changes are required. The only judgment call per command is accurate prose describing what the command does and a sensible example invocation. The `projection:check` command is feature-gated (`#[cfg(feature = "projections")]`) which should be noted in its entry. The `generate-routes` command is a library module used internally by `generate-types` but its `run()` fn is marked `#[allow(dead_code)]` — document it as an available command with the caveat that it is called automatically by `ferro serve` and `ferro generate-types`.

**Primary recommendation:** Edit `docs/src/reference/cli.md` to add one section per undocumented command, following the existing format exactly. Also add each new command to the Command Summary table at the bottom of the file.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLIMCP-01 | All 13 undocumented CLI commands added to reference/cli.md | Full source code for all 13 commands read and understood. Format pattern established from existing 37 documented commands. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| mdBook | — | Docs site renderer | Ferro docs site uses mdBook; cli.md is a plain Markdown source file |

### Supporting
No additional libraries needed. This phase edits a Markdown file only.

**Installation:** No new dependencies.

## Architecture Patterns

### Recommended Project Structure
The file to edit is:
```
docs/src/reference/cli.md
```

No new files. One edit to one existing file.

### Pattern 1: Existing Section Format

**What:** Every documented command follows an identical four-part template.
**When to use:** For every new entry.

**Template (based on existing entries):**

```markdown
### `ferro <command>`

One-sentence description of what the command does.

```bash
# Common usage
ferro <command> [args] [flags]

# Variant
ferro <command> --flag
```

**Options:**

| Option | Default | Description |
|--------|---------|-------------|
| `--flag` | `value` | What it does |

**What it does:**

1. Step one
2. Step two

**Generated file:** (if applicable) `path/to/file`

```rust
// generated content example
```
```

### Pattern 2: Command Summary Table

**What:** After the last full entry, a Command Summary table lists all commands with one-line descriptions.
**When to use:** Must be updated to include each new entry.

### Anti-Patterns to Avoid
- **Adding flags not present in main.rs:** Every flag must be verified against the `#[arg(...)]` definitions in `ferro-cli/src/main.rs`.
- **Duplicating existing make:policy mention:** `make:policy` already appears in the Command Summary table — it needs a full body section added, not a second table row.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Discovering flag names | Guessing from command module | Read main.rs `#[arg]` definitions | main.rs is the single source of truth for the CLI interface |
| Writing command behavior | Summarizing from vibes | Read the command's `run()`/`execute()` function | Source code is authoritative |

**Key insight:** Every flag, default value, and behavior detail must be sourced from `main.rs` and the command's implementation file. Docs that diverge from main.rs will cause user confusion.

## Common Pitfalls

### Pitfall 1: generate-routes Is Not a Registered Subcommand
**What goes wrong:** `generate_routes` has a `pub fn run()` in its module and is declared in `mod.rs`, but it is NOT registered as a subcommand in `main.rs`. Running `ferro generate-routes` would fail.
**Why it happens:** The module exists as a library used by `generate_types` and `ai.rs`. The `run()` fn is `#[allow(dead_code)]`.
**How to avoid:** Document `generate-routes` as a command available via `ferro generate-types` (which calls it internally), or document it accurately as an internal utility exposed for direct use. The requirement lists it — add the entry but note accurately that it is called internally by `generate-types`. Check whether a `GenerateRoutes` subcommand actually exists in main.rs before claiming it is independently invocable.
**Current status confirmed:** No `GenerateRoutes` variant exists in the `Commands` enum in `main.rs`. The command is NOT independently invocable. Documentation should reflect this — either document it as a sub-behavior of `generate-types` or as a standalone tool invoked indirectly.

### Pitfall 2: projection:check Is Feature-Gated
**What goes wrong:** Adding the entry without noting the `projections` feature flag.
**Why it happens:** The command is only compiled with `#[cfg(feature = "projections")]`.
**How to avoid:** Note in the documentation that this command requires the `projections` feature. Example: "Requires the `projections` feature. See `Cargo.toml`."

### Pitfall 3: make:policy Appears in Summary But Not in Body
**What goes wrong:** The Command Summary table already lists `make:policy` (line 1021 of cli.md). Adding it to the table again would duplicate it.
**Why it happens:** Someone added it to the summary but never wrote the full section.
**How to avoid:** Only add the full body section. Do not add another table row.

### Pitfall 4: make:api --exclude Takes Comma-Delimited Values
**What goes wrong:** Documenting `--exclude` as space-separated when it is comma-separated.
**Why it happens:** The `value_delimiter = ','` in the clap definition is non-obvious.
**How to avoid:** Check `main.rs` arg definition: `#[arg(long, value_delimiter = ',')]`.

## Code Examples

Verified patterns from `ferro-cli/src/main.rs` and command source files:

### api:check
```bash
# Check local API server for MCP integration
ferro api:check

# Custom URL and API key
ferro api:check --url http://localhost:8080 --api-key fe_live_xxx

# Custom spec path
ferro api:check --spec-path /api/docs/openapi.json
```

Flags from main.rs:
- `--url` (default: `http://localhost:8080`)
- `--api-key` (optional)
- `--spec-path` (default: `/api/openapi.json`)

### clean
```bash
# Remove all build artifacts
ferro clean

# Remove only artifacts older than 7 days (requires cargo-sweep)
ferro clean --sweep 7
```

Flags from main.rs:
- `--sweep <days>` (optional; requires `cargo install cargo-sweep`)

### generate-routes
```bash
# NOTE: generate-routes is NOT a standalone subcommand in ferro
# Route TypeScript generation runs automatically during `ferro generate-types`
ferro generate-types
```

### make:api
```bash
# Generate API for specific models
ferro make:api User Post

# Generate API for all detected models
ferro make:api --all

# Skip confirmation and exclude sensitive fields
ferro make:api User --yes --exclude password_hash,secret_token

# Include all fields (disable auto-exclusion)
ferro make:api User --include-all
```

Flags from main.rs:
- `models` (positional, Vec<String>)
- `--all`
- `--yes` / `-y`
- `--exclude` (comma-delimited)
- `--include-all`

### make:api-key
```bash
# Generate a live API key
ferro make:api-key "Production Bot"

# Generate a test key
ferro make:api-key "CI Testing" --env test
```

Flags from main.rs:
- `name` (positional, required)
- `--env` (default: `live`; accepts `live` or `test`)

### make:lang
```bash
ferro make:lang fr
ferro make:lang pt-br
```

Flags from main.rs:
- `name` (positional, required; e.g., `en`, `fr`, `pt-br`, `zh-hans`)

Generated files:
- `lang/<locale>/validation.json`
- `lang/<locale>/app.json`

### make:policy
```bash
ferro make:policy Post
ferro make:policy PostPolicy --model Post
```

Flags from main.rs:
- `name` (positional, required)
- `--model` / `-m` (optional; defaults to name without "Policy" suffix)

Generated file: `src/policies/<name>_policy.rs`

### make:projection
```bash
ferro make:projection user
ferro make:projection order --from-model
```

Flags from main.rs:
- `name` (positional, required)
- `--from-model` (populate fields from matching SeaORM model in `src/models/`)

Generated file: `src/projections/<name>.rs`

### make:stripe
```bash
ferro make:stripe

# Include Stripe Connect scaffolding
ferro make:stripe --connect
```

Flags from main.rs:
- `--connect` (include Connect webhook and connect account ID field)

Generated files:
- `src/stripe/mod.rs`
- `src/stripe/webhook.rs`
- `src/stripe/listeners.rs`
- `src/stripe/connect_webhook.rs` (with `--connect`)
- `src/migrations/m<timestamp>_create_tenant_billing_table.rs`

### make:theme
```bash
ferro make:theme ocean
ferro make:theme corporate
```

Flags from main.rs:
- `name` (positional, required)

Generated files:
- `themes/<name>/tokens.css` (Tailwind v4 `@theme` with 23 semantic token slots)
- `themes/<name>/theme.json` (empty JSON object for intent template overrides)

### make:whatsapp
```bash
ferro make:whatsapp
```

No flags. Generates:
- `src/whatsapp/mod.rs`
- `src/whatsapp/webhook.rs`
- `src/whatsapp/listeners.rs`

### projection:check
```bash
# Check all projections
ferro projection:check

# Check one projection by function name
ferro projection:check --name user_service
```

Flags from main.rs (feature-gated: `projections`):
- `--name` (optional; check a single projection function)

### validate:contracts
```bash
ferro validate:contracts

# Filter by route
ferro validate:contracts --filter /users

# JSON output for CI
ferro validate:contracts --json
```

Flags from main.rs:
- `--filter` / `-f` (optional)
- `--json`

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| generate-types only | generate-types now calls generate-routes internally | Phase 22.x | generate-routes is a library module, not standalone |

**Deprecated/outdated:**
- None. All 13 commands are current and active.

## Validation Architecture

`workflow.nyquist_validation` is not set to `false` in `.planning/config.json`, but this phase only edits Markdown documentation. There is no executable test for documentation content accuracy. Validation is manual: count entries in the docs after the task, verify each of the 13 commands has a section and a table row.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None — documentation-only phase |
| Config file | N/A |
| Quick run command | `grep -c "^### " docs/src/reference/cli.md` (count sections) |
| Full suite command | Manual review: verify 13 new sections present, each with synopsis/flags/description/example |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLIMCP-01 | 13 new sections in cli.md, each with correct format | manual | `grep -c "^### " docs/src/reference/cli.md` (should be 50+ after) | N/A |

### Sampling Rate
- **Per task commit:** `grep -c "^### \`ferro" docs/src/reference/cli.md`
- **Per wave merge:** Same
- **Phase gate:** Count == 50 (37 existing + 13 new) before verify-work

### Wave 0 Gaps
None — existing infrastructure covers all phase requirements.

## Open Questions

1. **generate-routes standalone invocability**
   - What we know: The module exists, `run()` is defined, but no `Commands` variant in main.rs registers it
   - What's unclear: Was this intentional (it's just a library) or an oversight?
   - Recommendation: Document `generate-routes` as an internal module invoked by `generate-types`. Do NOT claim it is directly invocable via `ferro generate-routes`. If in doubt, note it as a utility function used by `generate-types`.

2. **projection:check feature flag phrasing**
   - What we know: `#[cfg(feature = "projections")]` gates this command
   - What's unclear: What feature flag name to use in docs
   - Recommendation: Use `projections` as the feature name (exact string from main.rs) and note it requires `cargo build --features projections`

## Sources

### Primary (HIGH confidence)
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/main.rs` — complete CLI argument definitions, canonical flag names and defaults
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/api_check.rs` — api:check behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/clean.rs` — clean behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/generate_routes.rs` — generate-routes behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_api.rs` — make:api behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_api_key.rs` — make:api-key behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_lang.rs` — make:lang behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_policy.rs` — make:policy behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_projection.rs` — make:projection behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_stripe.rs` — make:stripe behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_theme.rs` — make:theme behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/make_whatsapp.rs` — make:whatsapp behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/projection_check.rs` — projection:check behavior
- `/Users/alberto/repositories/albertogferrario/ferro/ferro-cli/src/commands/validate_contracts.rs` — validate:contracts behavior
- `/Users/alberto/repositories/albertogferrario/ferro/docs/src/reference/cli.md` — existing documentation format (37 documented commands)

### Secondary (MEDIUM confidence)
None needed — all findings sourced from the authoritative codebase.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — single Markdown file, known format
- Architecture: HIGH — existing 37 entries establish the exact template
- Pitfalls: HIGH — sourced from direct source code inspection of main.rs

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable documentation audit milestone)
