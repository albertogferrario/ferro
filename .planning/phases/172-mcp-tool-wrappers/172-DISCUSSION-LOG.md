# Phase 172: MCP Tool Wrappers - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 172-mcp-tool-wrappers
**Mode:** `--auto` (all gray areas auto-selected; recommended option chosen per area)
**Areas discussed:** Shared-core location, ai_scaffold write semantics, ai_explain output contract, Error handling & runtime, Tool naming & registration, Versioning & gate

---

## Shared-core location (D-01)

| Option | Description | Selected |
|--------|-------------|----------|
| Extract core into `ferro-mcp`, CLI becomes thin wrapper | Single definition site in the crate the CLI already depends on; SC#3 becomes a compile-time guarantee | ✓ |
| New shared crate `ferro-ai-scaffold` | Extra publish/maintenance surface; `ferro-mcp` already has every needed dep | |
| Duplicate the logic in both | Violates SC#3 ("no duplicate implementation") | |

**Auto-selected:** Extract core into `ferro-mcp`.
**Notes:** Dep arrow is `ferro-cli → ferro-mcp` (`ferro-cli/Cargo.toml:45`); reverse would cycle. `ferro-mcp` already depends on `ferro-ai` (:23) and `ferro-projections` (:25). Relevance filter + prompt assembly relocate into `ferro-mcp`; the `ServiceDef`→Rust-source emitter and file write stay CLI-only.

---

## ai_scaffold write semantics (D-02)

| Option | Description | Selected |
|--------|-------------|----------|
| Return-only, no disk write | Tool returns `ServiceDef` JSON; agent decides what to persist | ✓ |
| Write the file like the CLI and also return it | MCP tool would silently mutate the project filesystem | |

**Auto-selected:** Return-only.

---

## ai_explain output contract (D-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Structured JSON from resolved `ServiceDef` (no LLM); prose fallback via shared CLI path | Deterministic, zero-token structured branch; LLM only when no `ServiceDef` | ✓ |
| Always call the LLM and ask it to emit JSON | Spends tokens to re-derive data that is already typed introspection output | |

**Auto-selected:** Structured-from-`ServiceDef`, prose fallback.
**Notes:** CLI `ai:explain` is prose-only today; shared path is `resolve_target` + prose branch. Structured branch is the MCP rendering of the already-resolved `ServiceDef`.

---

## Error handling & runtime (D-04)

| Option | Description | Selected |
|--------|-------------|----------|
| `Result`-returning async core; structured tool-error serialization; tokio bridge stays CLI-side | Library core never exits/prints; mirrors `test_classifier` tool shape | ✓ |
| Core keeps `process::exit`/`eprintln` | Would terminate the MCP server process / pollute its stderr | |

**Auto-selected:** `Result`-returning async core.

---

## Tool naming & registration (D-05)

| Option | Description | Selected |
|--------|-------------|----------|
| `ai_scaffold` / `ai_explain` (requirement-specified names) | Matches AICLI-05 and roadmap; deliberate divergence from CLI `ai:make` | ✓ |
| Rename to match CLI `ai_make` | Contradicts the requirement's spelling | |

**Auto-selected:** `ai_scaffold` / `ai_explain`.
**Notes:** Registered via `#[tool(name, description)]` in `service.rs` next to `test_classifier`. Descriptions self-sufficient per SC#4.

---

## Versioning & gate (D-06)

**Auto-selected:** Bump workspace version (root `Cargo.toml:36`, `ferro-mcp` uses `version.workspace = true`); run full fmt/clippy/test gate; add core unit tests in `ferro-mcp`.

---

## Claude's Discretion

- Module layout inside `ferro-mcp/src/tools/` (one file per tool vs shared submodule).
- Whether `ai_explain` ServiceDef-detection reuses `list_projections` matching or a small resolver.
- Result wrapper struct names / serde shape (must carry `success`/`error` + payloads).

## Deferred Ideas

- CLI `ai:explain` adopting the structured JSON branch (MCP gets it first).
- Embedding-based relevance reranking (already deferred in 171-CONTEXT D-02).
