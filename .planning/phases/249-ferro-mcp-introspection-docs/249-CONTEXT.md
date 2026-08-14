# Phase 249: ferro-mcp introspection + docs - Context

**Gathered:** 2026-08-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Close the single-source loop for `#[offload]` (OFFLOAD-06): make offloadable methods
introspectable through `ferro-mcp` so an agent reads the same trait as the in-process
contract, the wire payload, and the offload spec — and write the documentation covering the
authoring surface, the result-handle/streaming pattern, the deployable-worker scaling model,
and the deferred elastic non-goals.

Two deliverables, no new offload capability:
- **A — MCP introspection (SC#1):** `list_services` marks offloadable methods and exposes
  their derived payload schema.
- **B — Docs (SC#2, SC#3):** authoring an `#[offload]` method, the result handle + streaming
  pattern, the deployable worker / scaling model, the "many-user" capacity story, and the
  deferred elastic direction.

This phase touches **ferro-mcp + docs only**. It does not modify `ferro-macros`,
`ferro-queue`, or the offload runtime. Phase 249.1 (convergence sweep) runs immediately after.

</domain>

<decisions>
## Implementation Decisions

### MCP introspection surface
- **D-01:** Extend `list_services` **in place** — do not add a separate offload tool. SC#1
  names `list_services`; the agent reads services and their offload facts in one call. The
  existing runtime-first (`/_ferro/services`) → static-fallback dual path is preserved.
- **D-02:** Each offloadable method surfaces as a per-method entry carrying: the method name,
  the **declared queue** (`#[offload(queue = "…")]`, defaulting to `default`), and the typed
  payload param list (D-04). Non-offload services keep their current `{name, binding_type}`
  shape — offload data is additive, never removed from existing output.
- **D-03:** The `list_services` MCP tool **description** (`ferro-mcp/src/service.rs`) is updated
  to state that it marks offloadable methods and their payload — held to the same accuracy bar
  as the Rust API (per CLAUDE.md: MCP tool descriptions are part of the framework surface).

### Introspection data source
- **D-04:** Offload facts are derived by **static source parsing inside ferro-mcp** — parse
  `#[offload]` methods from the app's source in the existing static-analysis path, exactly as
  `list_services` already static-parses `#[service(...)]`. This keeps Phase 249 confined to
  ferro-mcp + docs (no `ferro-macros` / `ferro-queue` change), and works **before the app is
  running** — the agent-authoring case the introspection surface exists for.
- **D-05:** The runtime `/_ferro/services` endpoint carrying offload metadata is **out of scope**
  (Deferred). Offload facts come from the static parse; the runtime path continues to return
  services only. If the app is running, `list_services` still returns runtime services plus (if
  the planner chooses) statically-parsed offload facts — the planner decides whether offload
  parsing runs in both modes or only the static branch; either satisfies SC#1.

### Payload schema representation
- **D-06:** The "derived payload schema" (SC#1) is a **typed parameter list** — an ordered
  `[{ name, rust_type }]` per offloadable method, recovered from the parsed method signature.
  No full JSON Schema, and **no new trait bound**: the offload contract stays
  `Serialize + DeserializeOwned` (adding `schemars::JsonSchema` to every offloaded payload is
  explicitly rejected). This mirrors how `list_models` / route introspection already describe
  types as Rust type strings. Full JSON Schema is a Deferred idea if a consumer needs it.

### Documentation home & structure
- **D-07:** A **dedicated page** `docs/src/features/offload.md` is the canonical home for the
  work-distribution story: authoring an `#[offload]` method, the result handle + subscribe/await
  streaming pattern, the deployable worker + scaling model, and the non-goals.
- **D-08:** `docs/src/features/queues.md` keeps its existing `#[offload]` material but is
  reduced to / cross-linked as a **pointer** into `offload.md` (avoid a duplicated,
  drift-prone second copy). `docs/src/features/deployments.md` **cross-links** the scaling
  recipe. The docs nav (SUMMARY / mdBook nav) gains the new page.
  - Naming note (Claude's discretion): `offload.md` is chosen over `work-distribution.md` to
    match the feature-attribute naming of sibling pages (`queues.md`, `caching.md`).

### Scaling / capacity docs depth
- **D-09:** SC#3's capacity story is documented as a **concrete deploy recipe + an honest
  limitations subsection**, not a conceptual narrative alone. The recipe describes the decided
  Phase 248 surface: web replicas on `serve --no-worker` + dedicated `<app-bin> worker --queue
  <class>` replicas + cache + the shared queue/broadcast transport — i.e. the "stateless tier +
  replicable workers + cache + queue" answer stated as a real deployment shape.
- **D-10:** An **Honest Limitations** subsection documents Phase 248's deferred operational
  gaps as known constraints (not silently omitted): `DB_MAX_CONNECTIONS` × replicas vs the
  Postgres connection ceiling (PgBouncer guidance), no built-in metrics/OpenTelemetry export in
  generated manifests, and result latency being worker-scheduling-bound (unsuited to sub-second
  interactive compute). Source: 248-CONTEXT Deferred Ideas + spec §Honest limitations.
- **D-11:** The docs also cover the **deferred elastic direction** (scale-to-zero / KEDA,
  warm-start, non-K8s actuation, WASM isolates) as explicit 2.0 non-goals — framed as future
  work, not commitments, per the neutral-repository-voice discipline. Source: spec §Future
  direction.

### Claude's Discretion
- Exact serde shape of the extended `list_services` output (e.g. a `methods: [...]` array on
  each `ServiceItem` vs a top-level `offloadable_methods` block) — either satisfies D-01/D-02.
- Whether static offload parsing runs in both the runtime and static branches or only the
  static branch (D-05).
- Whether a light mention of offloadable methods is also added to `generation_context` (a read
  surface) — acceptable if cheap; NOT an authoring `code_templates` snippet (that was declined —
  see Deferred).
- Internal section ordering within `offload.md`; how much authoring content is relocated from
  `queues.md` vs left as a pointer.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Offload milestone spec & phase definition
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` — anchor spec.
  Read §Introspection (single-source-of-truth rationale), §Scaling model (worker shape,
  stateless-tier capacity answer, deploy split), §Honest limitations, §Future direction (2.0
  non-goals), §Non-Goals. This spec is the source for docs deliverable B.
- `.planning/ROADMAP.md` §"Phase 249: `ferro-mcp` introspection + docs" — Goal and Success
  Criteria SC#1–3.
- `.planning/REQUIREMENTS.md` — OFFLOAD-06 (introspection + docs scope) and Out-of-Scope (v16.4).

### Prior-phase surface the docs must describe accurately
- `.planning/phases/248-deployable-ferro-worker-runtime/248-CONTEXT.md` — the **decided worker
  surface** the scaling docs describe: `<app-bin> worker --queue <name>` repeatable (D-02),
  no-`--queue` = all queues (D-03), `serve --no-worker` (D-05), `#[offload(queue = "name")]`
  default `default` (D-04). Its Deferred Ideas are the source for the Honest Limitations
  subsection (D-10).

### ferro-mcp surface to extend
- `ferro-mcp/src/tools/list_services.rs` — the tool to extend; `ServiceItem { name,
  binding_type }` and `scan_services_from_files()` static parser are the extension points.
- `ferro-mcp/src/service.rs` §`list_services` (~L602) — tool registration + description to
  update (D-03).
- `framework/src/debug/mod.rs` §`handle_services` (~L115) and `framework/src/server.rs`
  (`/_ferro/services` route, ~L220) — the runtime endpoint, referenced only for the
  deferred runtime-path note (D-05).

### Existing offload docs to relocate / relink
- `docs/src/features/queues.md` — existing offload material: §"Offloading Service Methods"
  (~L188), serializable-contract section (~L257), "Subscribe and await an offloaded result"
  (~L291). `offload.md` becomes canonical; this page becomes a pointer (D-08).

### Compile-time offload shape (reference for the static parser)
- `ferro-macros/src/offload.rs` — `OffloadMethodInfo` (declared queue, method name, params) is
  the compile-time metadata the static parser must recover the equivalent of (read-only
  reference; this phase does NOT modify the macro).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-mcp/src/tools/list_services.rs::scan_services_from_files` — the line-based static
  parser for `#[service(...)]`; extend the same walk to detect `#[offload]` methods (and their
  `queue = "…"` argument + parameter list). `ServiceItem` is the struct to extend with per-method
  offload data.
- `list_services` dual-path design (runtime `/_ferro/services` then static fallback) — the
  existing pattern to preserve; offload facts slot into the static branch (D-04/D-05).
- `docs/src/features/queues.md` offload sections — content to relocate/point from; already
  covers authoring, serializable contract, and subscribe-and-await (phases 245/247), so
  deliverable B is largely *worker/scaling docs + consolidation*, not net-new authoring prose.

### Established Patterns
- Docs are per-feature Markdown under `docs/src/features/*.md`, registered in the mdBook nav;
  neutral public voice (repository docs are treated as public artifacts).
- MCP tool descriptions live in `ferro-mcp/src/service.rs` alongside the tool method and are
  held to the Rust-API accuracy bar.

### Integration Points
- `list_services` tool output is consumed by agents introspecting the project; its description
  in `service.rs` advertises the offload capability.
- mdBook nav (SUMMARY / book config) must register `offload.md`.
- `queues.md` `#[offload]` section → pointer into `offload.md`; `deployments.md` → cross-link
  the scaling recipe.

</code_context>

<specifics>
## Specific Ideas

- The introspection framing to preserve: **one trait, three readers** — the in-process
  contract, the wire payload, and the agent-readable spec are the same declaration
  (spec §Introspection). `list_services` surfacing the declared queue + typed payload is that
  principle made visible.
- The scaling docs answer one concrete question — "how does a Ferro app serve many users?" —
  with a deployable shape, not an abstraction: `serve --no-worker` web replicas + `worker
  --queue` replicas + cache + shared queue/broadcast transport. Capacity is "run more workers,"
  and the docs say so plainly, including where it stops (honest limitations).

</specifics>

<deferred>
## Deferred Ideas

- **Runtime `/_ferro/services` offload awareness** — surfacing offloadable methods from the
  running app's registry (rather than static source parse). Chose static parse (D-04); the
  runtime path stays services-only (D-05). Would require the job registry to carry richer
  metadata (service+method name, param types), i.e. macro enrichment.
- **Full JSON Schema for offload payloads** — chose the typed param list (D-06). Full JSON
  Schema would add a `schemars::JsonSchema` bound to the offload contract; revisit only if a
  consumer needs machine-consumable schema.
- **`#[offload]` authoring snippet in `code_templates` / `generation_context`** — the "extend
  `list_services` + code_templates" option was declined in favour of the read-surface-only
  extension. A light read-only mention in `generation_context` remains at Claude's discretion
  (D, discretion); an authoring template is out.
- **Macro-emitted richer registry metadata** — the rejected data-source option (enrich the
  `#[offload]` macro to emit service/method/param metadata into the job registry). Not this
  phase; would be the substrate if runtime offload introspection is ever wanted.
- **Deploy `workers:` scaffolder emission** — 248 D-08; extend `[package.metadata.ferro.deploy]`
  with a `workers` array and emit one worker component per class from `do:init` / `docker:init`.
  Deploy-scaffolder line of work, not 249.

</deferred>

---

*Phase: 249-ferro-mcp-introspection-docs*
*Context gathered: 2026-08-14*
