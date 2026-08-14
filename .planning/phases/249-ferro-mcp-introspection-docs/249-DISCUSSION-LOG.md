# Phase 249: ferro-mcp introspection + docs - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-08-14
**Phase:** 249-ferro-mcp-introspection-docs
**Areas discussed:** MCP surface, Payload schema, Docs home, Scaling depth, Introspection data source

---

## Gray Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| MCP surface | How list_services marks offloadable methods; in-place vs companion tool; data source | ✓ |
| Payload schema | What "derived payload schema" means (JSON Schema vs typed list vs normalizer) | ✓ |
| Docs home | Dedicated offload page vs extend queues.md vs split | ✓ |
| Scaling depth | Conceptual vs concrete recipe; whether to document honest limits | ✓ |

**User's choice:** All four areas selected.

---

## MCP surface

| Option | Description | Selected |
|--------|-------------|----------|
| Extend list_services in place | Per-method offloadable entries added to existing tool; runtime-first + static fallback preserved | ✓ |
| Dedicated offload tool | New list_offload_methods alongside list_services | |
| list_services + code_templates | Extend read surface AND add #[offload] authoring snippet | |

**User's choice:** Extend list_services (Recommended).
**Notes:** SC#1 names list_services; single-call read. The declined "+ code_templates" option
signals: read surface yes, authoring template no → recorded as Deferred.

---

## Payload schema

| Option | Description | Selected |
|--------|-------------|----------|
| Typed param list | [{name, rust_type}] per method; no new trait bound | ✓ |
| Full JSON Schema | schemars::JsonSchema on the derived payload | |
| Typed list now, JSON Schema later | Ship typed list; defer JSON Schema | |

**User's choice:** Typed param list (Recommended).
**Notes:** Keeps the offload contract at Serialize + DeserializeOwned; mirrors existing type
description in list_models / routes. Full JSON Schema recorded as Deferred.

---

## Docs home

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated offload page | New docs/src/features/offload.md; queues.md → pointer; deployments.md cross-links | ✓ |
| Extend queues.md in place | Everything stays on the queues page | |
| Split queues.md + deployments.md | Authoring in queues.md, scaling in deployments.md | |

**User's choice:** Dedicated offload page (Recommended).
**Notes:** Gives the milestone one canonical home; avoids the worker/scaling story living inside
a page framed around the queue primitive. Page name offload.md is Claude's discretion.

---

## Scaling depth

| Option | Description | Selected |
|--------|-------------|----------|
| Recipe + honest limits | Concrete deploy recipe PLUS deferred-gaps subsection | ✓ |
| Recipe, no limits subsection | Recipe only, omit 248's deferred ops gaps | |
| Conceptual narrative only | Describe the model without a recipe or limits list | |

**User's choice:** Recipe + honest limits (Recommended).
**Notes:** Documents the decided Phase 248 surface (serve --no-worker web + worker --queue
replicas + cache + queue) and Phase 248's deferred operational gaps (PgBouncer / connection
math, no metrics export, latency bounds) as honest limitations.

---

## Introspection data source (follow-up fork)

| Option | Description | Selected |
|--------|-------------|----------|
| Static source parse in ferro-mcp | Parse #[offload] from source, like list_services parses #[service]; no macro/queue change | ✓ |
| Enrich macro-emitted registry metadata | Extend #[offload] macro to emit service/method/param metadata into the job registry | |
| Runtime endpoint extension | Extend /_ferro/services (or add /_ferro/offload) to expose offload from the running registry | |

**User's choice:** Static source parse in ferro-mcp (Recommended).
**Notes:** Confines Phase 249 to ferro-mcp + docs; works before the app runs (agent-authoring
case). Runtime offload metadata and macro enrichment recorded as Deferred.

## Claude's Discretion

- Exact serde shape of the extended list_services output.
- Whether static offload parsing runs in both runtime + static branches or only static.
- Light read-only offload mention in generation_context (optional; authoring template excluded).
- offload.md internal section ordering and how much is relocated from queues.md.

## Deferred Ideas

- Runtime /_ferro/services offload awareness.
- Full JSON Schema for offload payloads.
- #[offload] authoring snippet in code_templates.
- Macro-emitted richer registry metadata.
- Deploy workers: scaffolder emission (248 D-08).
