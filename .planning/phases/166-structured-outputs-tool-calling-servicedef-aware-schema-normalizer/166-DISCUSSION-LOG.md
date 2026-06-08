# Phase 166: Structured Outputs, Tool Calling & ServiceDef-aware Schema Normalizer - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-08
**Phase:** 166-structured-outputs-tool-calling-servicedef-aware-schema-normalizer
**Mode:** `--auto` (recommended defaults selected without interactive prompts)
**Areas discussed:** complete::<T>() API, Generic normalizer, ServiceDef-aware closing, ToolRegistry & tool calling, Client-layer tool extension

---

## complete::<T>() API

| Option | Description | Selected |
|--------|-------------|----------|
| Free fn `complete::<T>(client, prompt)` | Matches SC#1 literal form; ergonomic small-core surface | ✓ |
| Method on a trait / builder | More configurable but larger mental surface | |

**Auto choice:** Free function (D-01). Request-taking escape hatch left to Claude's discretion (D-02).

---

## Generic Schema Normalizer

| Option | Description | Selected |
|--------|-------------|----------|
| Target schemars 1.x → Anthropic-canonical, single normalized output | Resolve `$defs`, add `additionalProperties:false`, strip Anthropic-rejected keywords; preserve `enum` | ✓ |
| Per-provider normalizers | One normalizer per provider's constraint set | |

**Auto choice:** Single Anthropic-canonical normalizer; `enum` preserved as the locking mechanism (D-03, D-04, D-05). Exact reject-list confirmed in research against Anthropic docs.

---

## ServiceDef-aware Path (structural guarantee)

| Option | Description | Selected |
|--------|-------------|----------|
| Close the projection enums (drop `Custom` untagged branch) in LLM-facing schema | Makes SC#3 achievable — LLM cannot emit invalid FieldMeaning/Intent | ✓ |
| Keep enums open (rely on generic normalization) | Any string valid → SC#3 test cannot assert "invalid rejected" | |

**Key finding:** `FieldMeaning` and `Intent` carry `#[serde(untagged)] Custom(String)`, so raw schemars output accepts any string. Closing them is the entire reason the SD-aware path exists (D-06).

**Detection mechanism:**

| Option | Description | Selected |
|--------|-------------|----------|
| Runtime `$defs` inspection, single `complete::<T>()` entry | Stable Rust, one entry point, explicit | ✓ |
| Marker trait (blanket + specific impl) | Requires specialization — conflicts on stable Rust | |
| Separate `complete_service_def()` entry | Parallel surface; violates single-entry intent | |

**Auto choice:** Runtime detection (D-07); valid-value set sourced from ferro-projections, not duplicated (D-08); SC#3 test via `jsonschema` validator (D-09); closing trade-off acknowledged (D-10).

---

## ToolRegistry & Tool Calling

| Option | Description | Selected |
|--------|-------------|----------|
| Async tool handler; `max_iterations` required at construction; `ToolError{message}` | IO-capable tools; no unbounded path; model-legible errors | ✓ |
| Sync handler; `Default` max_iterations | Simpler signature but blocks IO tools / risks implicit unbounded loop | |

**Auto choice:** Async handler (D-11); required `max_iterations` with warn@5 / hard cap (D-12); `ToolError{message}` (D-13).

---

## Client-layer Tool Extension

| Option | Description | Selected |
|--------|-------------|----------|
| Extend `LlmClient` path; dispatch reuses 165 clients; planner picks exact shape | Single HTTP source per provider; max_iterations enforced in ferro-ai | ✓ |
| Separate HTTP path in ToolRegistry | Duplicate provider logic; rejected | |

**Auto choice:** Tool-use goes through `LlmClient` (D-14); exact request/response shape is a research/planning decision constrained by cross-provider support.

## Claude's Discretion

- Request-taking `complete` variant signature (D-02)
- Normalizer input parameter type (D-05)
- Internal module layout
- JSON-Schema validator crate choice (`jsonschema` recommended)
- Client-layer tool-extension shape (D-14)
- Projection valid-value extraction technique (D-08)

## Deferred Ideas

- Renderer-as-tool adapter (Phase 171+)
- Tool calling in streaming context (v12.1 future-requirement)
- Conversation memory / multi-session history (out of scope)
