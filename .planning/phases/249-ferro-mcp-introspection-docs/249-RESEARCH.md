# Phase 249: `ferro-mcp` Introspection + Docs — Research

**Researched:** 2026-08-15
**Domain:** ferro-mcp static parser extension + mdBook documentation authoring
**Confidence:** HIGH — all findings verified against actual source files

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**MCP introspection surface**
- D-01: Extend `list_services` in place — no separate offload tool.
- D-02: Each offloadable method carries `{ name, queue, params: [{name, rust_type}] }`. Non-offload services keep `{name, binding_type}` unchanged — additive only.
- D-03: The `list_services` MCP tool description is updated to state it marks offloadable methods and their payload.

**Introspection data source**
- D-04: Offload facts derived by static source parsing inside ferro-mcp, extending the existing `scan_services_from_files` walk.
- D-05: Runtime `/_ferro/services` offload-metadata path deferred. Runtime path stays services-only.

**Payload schema representation**
- D-06: Typed parameter list `[{ name, rust_type }]`, not full JSON Schema. No new trait bound (`schemars::JsonSchema` rejected). Contract stays `Serialize + DeserializeOwned`.

**Documentation home & structure**
- D-07: Dedicated page `docs/src/features/offload.md` is the canonical home.
- D-08: `queues.md` reduces to a pointer; `deployments.md` cross-links the scaling recipe.

**Scaling / capacity docs depth**
- D-09: Concrete deploy recipe + Honest Limitations subsection.
- D-10: Honest Limitations documents PgBouncer guidance, no built-in metrics/OTel, and latency being worker-scheduling-bound.
- D-11: Deferred elastic direction documented as 2.0 non-goals (neutral public voice).

### Claude's Discretion

- Exact serde shape of extended `list_services` output (`methods` array on `ServiceItem` vs top-level `offloadable_methods` block).
- Whether static offload parsing runs in both the runtime and static branches or only the static branch (D-05).
- Whether a light mention of offloadable methods is added to `generation_context` (read surface only — no authoring template).
- Internal section ordering within `offload.md`; relocation vs. pointer extent from `queues.md`.

### Deferred Ideas (OUT OF SCOPE)

- Runtime `/_ferro/services` offload metadata path.
- Full JSON Schema for offload payloads (`schemars::JsonSchema`).
- `#[offload]` authoring snippet in `code_templates` / `generation_context` (template side).
- Macro-emitted richer registry metadata.
- Deploy `workers:` scaffolder emission.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OFFLOAD-06 | Offloadable methods introspectable through `ferro-mcp` (`list_services`, derived payload schema); docs cover authoring surface, result path, scaling model, and the non-goals / deferred elastic direction. | §Current Parser Shape documents the extension point; §Macro Reference Shape documents the parse target; §Output Serde Shape gives the recommended struct extension; §Documentation Mechanics confirms the nav integration path; §Scaling Model Source Facts confirms what is safe to document. |
</phase_requirements>

---

## Summary

Phase 249 has two independent deliverables on top of a well-understood substrate. The static parser in `ferro-mcp/src/tools/list_services.rs` is a compact line-by-line scanner (184 lines total); it is already the model for how `ServiceItem` is discovered, and extending it to recognize `#[offload]` attributes and parse method signatures is the smallest possible change consistent with D-04. The compile-time reference for what the parser must reproduce is `ferro-macros/src/offload.rs::OffloadMethodInfo` and the adjacent `service.rs` strip loop, both of which are well-factored and commented.

The documentation deliverable is primarily a consolidation and an addition: the authoring and result-path material exists in `queues.md` (§"Offloading Service Methods" through §"Subscribe and await"), and the scaling-model text is in the anchor spec `2026-06-24-offload-work-distribution-design.md`. The work is extracting those into `offload.md`, adding the Phase 248 deployment recipe, the Honest Limitations subsection, and the 2.0 non-goals — then reducing `queues.md` to a pointer.

The one genuinely novel implementation challenge is the method-signature parsing inside a line-based scanner. The analysis below gives a concrete recommendation: extend `scan_services_from_files` with a state machine that tracks `#[offload]` detection, queue arg extraction, and multi-line `async fn` parameter collection — all at the line/character level, without `syn`. This is viable within the narrow grammar that `#[offload]` signatures must conform to (owned or simple-borrow params, ident-only patterns, no closures or macro invocations), and the correctness boundary is safe: the parser is a best-effort read surface for agents, not a compiler gate.

**Primary recommendation:** Extend `scan_services_from_files` with a two-state machine (offload-pending → fn-collecting) that produces `OffloadableMethod` entries appended to `ServiceItem` as a `Vec`. Use `skip_serializing_if = "Vec::is_empty"` so non-offload service output is byte-for-byte unchanged.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Offload fact discovery | ferro-mcp (static parse) | — | D-04: facts come from source analysis inside ferro-mcp; no runtime dependency |
| ServiceItem output shape | ferro-mcp tools layer | ferro-mcp service.rs (tool desc) | Struct lives in list_services.rs; description registered in service.rs |
| Docs registration / nav | docs/src/SUMMARY.md | book.toml | mdBook nav wired via SUMMARY.md; book.toml unchanged if no new preprocessors |
| Scaling model source of truth | docs/superpowers/specs (spec) | 248-CONTEXT.md (decisions) | Documentation must derive from spec + decided Phase 248 surface, not invention |

---

## Finding 1: Current Parser Shape [VERIFIED: source read]

**File:** `ferro-mcp/src/tools/list_services.rs` (184 lines total)

`scan_services_from_files` walks `{project_root}/src/**/*.rs` using `walkdir`, reads each file as a `String`, then iterates `content.lines()` twice — once for `#[service(...)]` and `#[injectable]`, once for `singleton!(` and `bind!(` macro calls. All parsing is line-by-line string matching; `syn` is not used.

`ServiceItem` today (lines 31–37):

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceItem {
    pub name: String,
    pub binding_type: String,
}
```

`ServicesInfo` (lines 15–20):

```rust
pub struct ServicesInfo {
    pub services: Vec<ServiceItem>,
    pub source: ServiceSource,  // Runtime | StaticAnalysis
}
```

The runtime path (`fetch_runtime_services`) hits `/_ferro/services` with a 2-second timeout; its response is deserialized as `Vec<RuntimeServiceInfo>` and mapped into `Vec<ServiceItem>`. If the HTTP call fails for any reason the function returns `None` and `execute()` falls through to `scan_services_from_files`.

**Dual-path detail relevant to D-05:**
The runtime path returns only `{name, binding_type}` rows (it reads from `DebugResponse { data: Vec<RuntimeServiceInfo> }` — `RuntimeServiceInfo` carries exactly `name` and `binding_type`). The runtime `/_ferro/services` endpoint does not carry any offload metadata. Therefore offload facts can only come from the static path. The choice between "static parse always" vs "static parse only in static branch" is purely about whether agents that call `list_services` while the app is running see offload data. Running offload parsing in both branches is strictly more informative and costs nothing in the static case.

**Recommendation for D-05 (Claude's discretion):** Run offload parsing in both branches — after the runtime services are collected, augment the `ServicesInfo` with offload data from the static parse regardless of `source`. The runtime path stays `services: Vec<ServiceItem>` (unchanged); a separate `offloadable_methods` field (or augmented `methods` on `ServiceItem`) carries the static-parse offload facts. This means agents always have offload introspection whether or not the app is running.

---

## Finding 2: Compile-Time Offload Reference Shape [VERIFIED: source read]

**File:** `ferro-macros/src/offload.rs` (OffloadMethodInfo lines 48–68) and `ferro-macros/src/service.rs` (strip loop lines 183–219)

`OffloadMethodInfo` carries:

| Field | Type | Meaning |
|-------|------|---------|
| `job_ident` | `Ident` | Derived Job struct name (PascalCase) |
| `method_ident` | `Ident` | Original snake_case method name |
| `field_names` | `Vec<Ident>` | Non-`self` parameter names |
| `field_types` | `Vec<TokenStream2>` | Owned versions after `owned_type()` substitution |
| `field_forwards` | `Vec<FieldForward>` | How each param is forwarded (not needed for MCP) |
| `is_async` | `bool` | Whether `async fn` |
| `returns_result` | `bool` | Whether return type wraps `Result<_, _>` |
| `output_type` | `TokenStream2` | Success type (T of `Result<T,E>` or bare T) |
| `declared_queue` | `Option<String>` | Queue from `#[offload(queue = "name")]`; `None` = "default" |

**Queue argument parsing (service.rs lines 196–211):**

The macro checks `attr.meta.require_path_only()`. If the attribute has arguments (i.e., `#[offload(queue = "name")]` vs bare `#[offload]`), it calls `attr.parse_nested_meta()` looking for `queue = <LitStr>`. Any other key is a compile error. The parsed value is stored as `Option<String>` — `None` for the bare attribute, `Some("name")` for the keyed form.

**Parameter collection (`collect_info`, lines 147–163):**

Iterates `method.sig.inputs`. For each `FnArg`:
- `Receiver` (`&self` / `&mut self`): skipped.
- `Typed(PatType)`: requires `Pat::Ident` pattern (simple identifiers only — destructured patterns like `(a, b)` are rejected with a compile error). Records `ident` as `field_name` and the owned version of `ty` (via `owned_type`) as `field_type`.

**`owned_type` substitution rules:**
- `&str` → `String`
- `&[T]` → `Vec<T>`
- `&T` → `T` (any other reference, not `&mut`)
- `&mut T` → compile error
- `T` (owned) → `T` unchanged

**Edge cases the static parser must handle:**

| Edge Case | Macro Behaviour | Static Parser Action |
|-----------|-----------------|----------------------|
| `#[offload]` (no args) | `declared_queue = None` → runtime default | Emit `queue: "default"` |
| `#[offload(queue = "x")]` | `declared_queue = Some("x")` | Parse literal; strip outer quotes |
| `&self` / `&mut self` | Skipped from field collection | Skip lines starting with `&self` / `self` |
| `non-offload` methods interleaved | Not processed | State machine: only enter fn-collect on `#[offload]` trigger |
| `async fn` | `is_async = true` | Detect `async fn` keyword; MCP doesn't use this, skip |
| No params beyond `&self` | `field_names: []` → empty payload | `params: []` is valid |
| `Result<T, E>` return | `output_type = T` | MCP emits Rust type string verbatim from signature; no unwrapping needed |
| Generic params (`fn foo<T>`) | Not rejected by macro (proc-macro sees them) | Emit `rust_type` verbatim; do not try to expand |
| Doc comments / other attrs between `#[offload]` and `fn` | Stripped cleanly by proc-macro | State machine: allow `///`, `//`, `#[...]` lines between trigger and `fn`; only `fn` terminates collection |
| Multi-line param list | Handled token-by-token by syn | Static parser must handle split across lines (see §Line-Based Robustness below) |

---

## Finding 3: Line-Based Robustness Assessment [VERIFIED: source read + reasoning]

The existing parser is entirely line-based. The question for D-04 is whether the same approach is robust enough for `#[offload]` detection.

**What is simple:**
- Detecting `#[offload]` or `#[offload(queue = "...")]` on a single trimmed line — the same pattern as `#[service(...)]` detection.
- Extracting the `queue` string literal from `#[offload(queue = "name")]` with a small regex or manual substring scan (find `queue = "`, find closing `"`).
- Recognizing the `fn` keyword on the subsequent line to start parameter collection.

**What is harder:**
- Method signatures span multiple lines in practice:
  ```rust
  async fn build_monthly(
      &self,
      tenant_id: i64,
      month: Month,
  ) -> Report;
  ```
  A purely line-by-line approach can handle this with a state machine: after detecting the `fn` keyword, accumulate lines until encountering `)` or `-> ...;` that closes the signature, then parse param segments from the accumulated text.

- Distinguishing `&self` / `self` / `mut self` from ordinary params: the first `FnArg` is a receiver. In text form this is the segment that starts with `&self` or `self` (or `&mut self`). Easy to filter.

- The param segment format is `name: Type` separated by commas, possibly with trailing commas and generics that contain `<`, `>`, `,`. Simple splitting on `,` will fail on `HashMap<K, V>` parameters. A bracket-aware split (count `<`/`>` depth, only split on `,` at depth 0) handles this correctly in ~10 lines.

- The `owned_type` substitution is optional for the MCP surface: the tool description says "typed param list with Rust type strings". Emitting the raw type as written in source (`&str` → `&str`, not `String`) is acceptable for an agent read surface; the ownership substitution is only load-bearing for the generated Job struct. However, matching what the macro does (emitting owned types) is cleaner because it matches what the Job struct carries. Recommendation: do the substitution at text level (`&str` → `String`, `&[T]` → `Vec<T>`, `&T` → `T`) — it is straightforward string matching.

**Recommendation: line-based with a state machine, no `syn`.**

The grammar of `#[offload]`-eligible signatures is narrow enough (ident params, owned or simple-borrow types, no closures or macro invocations in param position). Line-based with accumulated state is sufficient. Adding `syn` would require adding it as a dependency of `ferro-mcp` (currently only in `ferro-macros`), turning a best-effort read surface into a full compiler-grade parse that still cannot handle macros-in-params anyway. The correctness boundary is appropriate: a best-effort static scanner for agent read-back, not a re-implementation of the proc-macro.

The state machine has three states:
1. **Idle** — scanning for `#[offload]` or `#[offload(queue = "...")]`.
2. **OffloadPending(queue)** — saw the attribute; waiting for `fn name(` or `async fn name(`.
3. **FnCollecting(method_name, queue, accumulated_params)** — saw `fn`; accumulating lines until the parameter list closes at `)` or `);` or `) ->`.

---

## Finding 4: Recommended Output Serde Shape [VERIFIED: source read + design reasoning]

The CONTEXT.md leaves this to Claude's discretion. The choice: `methods` array on each `ServiceItem` vs a top-level `offloadable_methods` block on `ServicesInfo`.

**Recommendation: `methods` array on `ServiceItem`, empty by default.**

Rationale: an agent reading `list_services` receives services and their offload facts in one traversal. Placing offload data on the `ServiceItem` that owns the trait keeps the data co-located with the service declaration — the agent can inspect a service and its offloadable methods together without cross-referencing a parallel array. A top-level `offloadable_methods` block would force the agent to correlate service names across two separate arrays.

The additive-only constraint (D-02) is satisfied by `#[serde(skip_serializing_if = "Vec::is_empty")]` on the `methods` field — services with no `#[offload]` methods serialize as today: `{"name": "...", "binding_type": "..."}`. No existing consumer of `list_services` output is broken.

**Concrete struct extension:**

```rust
// New struct added to list_services.rs
#[derive(Debug, Serialize, Clone)]
pub struct OffloadableMethod {
    pub name: String,
    /// Declared queue from #[offload(queue = "...")] or "default" when omitted.
    pub queue: String,
    /// Ordered parameter list (non-self params), types as Rust strings.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<OffloadParam>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OffloadParam {
    pub name: String,
    pub rust_type: String,
}

// ServiceItem extended
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceItem {
    pub name: String,
    pub binding_type: String,
    /// Offloadable methods on this service trait. Empty for non-offload services.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<OffloadableMethod>,
}
```

Serde conventions match the codebase: `#[serde(rename_all = "snake_case")]` on enums, `skip_serializing_if` for optional Vec fields. `ServiceItem` does not currently derive `rename_all` (it is a struct, not an enum), so no rename change needed.

**JSON output shape for a service with two offloadable methods:**

```json
{
  "name": "ReportBuilder",
  "binding_type": "trait_binding",
  "methods": [
    {
      "name": "build_monthly",
      "queue": "default",
      "params": [
        { "name": "tenant_id", "rust_type": "i64" },
        { "name": "month", "rust_type": "Month" }
      ]
    },
    {
      "name": "export_csv",
      "queue": "reports",
      "params": [
        { "name": "tenant_id", "rust_type": "i64" }
      ]
    }
  ]
}
```

**JSON output for a plain service (unchanged):**

```json
{ "name": "MailerService", "binding_type": "trait_binding" }
```

---

## Finding 5: Tool Description and Dual-Path [VERIFIED: source read]

**Current description** (service.rs lines 601–608):

```
List all registered dependency injection container services.

**When to use:** Understanding available services, checking DI bindings,
planning new service dependencies, debugging resolution errors.

**Returns:** Singleton registrations, trait-to-concrete bindings, scopes.

**Combine with:** `get_handler` to see service usage, `application_info` for service overview.
```

**Required change (D-03):** The `**Returns:**` line must state that service entries carrying `#[offload]` methods include those methods with their declared queue and typed parameter list.

Suggested replacement for the `**Returns:**` and `**When to use:**` lines only (minimal diff):

```
**When to use:** Understanding available services, checking DI bindings,
planning new service dependencies, debugging resolution errors, or
discovering which service methods are offloadable.

**Returns:** Singleton registrations, trait-to-concrete bindings.
Service entries with `#[offload]`-marked methods include a `methods` array
listing each method's name, declared queue, and typed parameter list
(`[{ name, rust_type }]`). Plain services omit the `methods` field.
```

**Dual-path note (D-05):** The planner should wire offload parsing to run in both the runtime and static branches (see §Finding 1 recommendation). In `execute()`, after either the runtime fetch or the static scan produces `Vec<ServiceItem>`, a second pass scans source files for `#[offload]` entries and augments matching `ServiceItem` entries (matched by service name). The static scan for offload data always runs because offload facts are not in the runtime endpoint. This satisfies D-05 (runtime path stays services-only, i.e. the `/_ferro/services` endpoint is not modified) while giving agents complete information in both modes.

---

## Finding 6: `generation_context` Mention (Discretion) [VERIFIED: source read]

`generation_context` is assembled in `ferro-mcp/src/tools/generation_context.rs` in a synchronous `execute()` function that returns a `GenerationContext` struct. The struct already has `live_projection: LiveProjectionGuidance` with four string fields added as a unit for v17.0.

Adding a light offload mention is straightforward: add a new `offload: OffloadGuidance` field (or a `&'static str` to the existing `common_patterns` section) that names `#[offload]` as an available authoring primitive, points to `docs/src/features/offload.md`, and states the queue defaulting rule. This costs approximately 10–15 lines: one new struct or string field, one entry in `execute()`, and an update to the `GenerationContext` derive.

**Recommendation:** Add a compact `offload` field to `GenerationContext` with a `&'static str` summary — same pattern as `live_projection.memoize`. No new struct needed. Content: one sentence on what `#[offload]` does, one sentence on queue defaulting, one pointer to the docs page. This is a read-only mention with no authoring template (per deferred decision).

**Effort:** low (< 20 lines). No new dep, no new drift guard needed unless a test already covers `GenerationContext` field count.

---

## Finding 7: Tests — Current Pattern [VERIFIED: source read]

`list_services.rs` has **zero unit tests today** (confirmed: no `#[test]` in that file). The parser is used through the `execute()` async function or the inner `scan_services_from_files` sync function, but neither has a test harness in the file.

Other tools in ferro-mcp do have inline unit tests:

- `route_dependencies.rs` (lines 220–313): inline `#[test]` functions in a `mod tests` block, each testing a helper function (`extract_model_usage`, `extract_services`) against inline string snippets. Pattern:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_name() {
          let source = r#"...rust source snippet..."#;
          let result = parse_fn(source);
          assert!(result.contains(...));
      }
  }
  ```
  These are plain `#[test]` functions (not `#[tokio::test]`), because the helpers they test are synchronous.

**Appropriate test strategy for the offload parser extension:**

The helper functions to extract (and test directly) are:
1. `detect_offload_attr(line: &str) -> Option<String>` — returns `Some(queue)` or `Some("default")` for a bare `#[offload]`; `None` for non-matching lines.
2. `extract_method_params(fn_signature_text: &str) -> Vec<OffloadParam>` — takes the accumulated param list text between `(` and `)` and returns the parsed `Vec<OffloadParam>`.
3. `augment_service_items_with_offload(items: &mut Vec<ServiceItem>, source: &str)` — the integration pass.

Test cases to cover:
- `#[offload]` with no queue argument → queue = "default"
- `#[offload(queue = "reports")]` → queue = "reports"
- Methods interleaved: `#[offload]` method followed by a non-offload method — only the offload one is picked up
- Zero params beyond `&self` → `params: []`
- Single param, simple owned type
- Multiple params with generic type (`Vec<String>`, `HashMap<K, V>`)
- `&str` param → `rust_type: "String"` (owned substitution)
- `&[T]` param → `rust_type: "Vec<T>"`
- Multi-line signature (params split across lines)
- Existing non-offload service `{name, binding_type}` output unchanged

**Test suite collapse gotcha:** The ferro project has a known issue where multiple `#[tokio::test]` functions inside one test binary collapse via `OnceLock` races. Since offload parser tests are synchronous (`#[test]` not `#[tokio::test]`), this gotcha does not apply. Use plain `#[test]` in a `#[cfg(test)] mod tests` block inside `list_services.rs`, matching the `route_dependencies.rs` pattern.

---

## Finding 8: Documentation Mechanics [VERIFIED: source read]

**mdBook nav file:** `docs/src/SUMMARY.md`

Pages under `# Features` are registered as:
```
- [Page Name](features/filename.md)
```
No `book.toml` change is needed to add a new features page; SUMMARY.md is the only edit point for nav registration.

**Current Features section** (SUMMARY.md lines 21–59): `offload.md` does not yet exist. The natural insertion point is between `Queues & Background Jobs` and `Notifications`:
```
- [Queues & Background Jobs](features/queues.md)
- [Work Distribution (Offload)](features/offload.md)   ← insert here
- [Notifications](features/notifications.md)
```

**Content to relocate from `queues.md`:**

| Section | Lines (approx.) | Action |
|---------|-----------------|--------|
| §"Offloading Service Methods" (authoring surface, typed handle, success-type contract, serializable contract) | 188–289 | Relocate to `offload.md` |
| §"Subscribe and await an offloaded result" (channel convention, request side, server-side consumer, browser read-back, delta payload, redaction, migration note) | 291–end | Relocate to `offload.md` |

After relocation, `queues.md` §Offloading is replaced with a cross-link paragraph:

```markdown
## Offloading Service Methods

For full documentation of `#[offload]`, the result-handle/streaming pattern,
the deployable worker scaling model, and the non-goals, see the dedicated page:
[Work Distribution (Offload)](offload.md).
```

**`deployments.md` cross-link:** `docs/src/features/deployments.md` covers `ferro-deployments` (immutable deployment rows, artifact storage) — this is a different subsystem from the offload worker deployment described in `offload.md`. The cross-link from `deployments.md` to `offload.md` should be a brief callout in a "See also" block rather than a section:

```markdown
> **Horizontal scaling with background workers:** For running the application
> binary's `worker` subcommand at N replicas as an independent queue consumer,
> see [Work Distribution (Offload)](offload.md#scaling-model).
```

This keeps `deployments.md` focused on the artifact/pointer-swap content it already covers.

---

## Finding 9: Scaling Model Source Facts [VERIFIED: spec + 248-CONTEXT.md]

The following facts are confirmed-shipped (Phase 248 complete) and safe to document in `offload.md`:

**Deploy recipe (§Scaling model):**

- The worker is the application's own binary under the `worker` subcommand, not a separate `ferro` CLI binary.
- CLI surface: `<app-bin> worker` (no `--queue` = all registered queues), `<app-bin> worker --queue <name>` (repeatable for multiple queues).
- Web replicas: `<app-bin> serve --no-worker` (disables the in-process WorkerLoop).
- Single-binary development: `<app-bin> serve` (retains the in-process WorkerLoop, drains all queues).
- Scale-out shape: N web replicas (`serve --no-worker`) + M worker replicas (`worker --queue <class>`), sharing the same queue backend and broadcast transport.
- Worker class = queue set; independent fault domain — a saturated `media` class does not starve `default`.
- N is external (operator, platform, cluster scheduler); no framework autoscaler.

**Honest Limitations (D-10, sourced from 248-CONTEXT.md Deferred + spec §Honest limitations):**

1. **Connection ceiling:** `DB_MAX_CONNECTIONS` defaults to 10 per process. At N replicas (web + worker), total connections = 10 × N against a Postgres ceiling of ~100. At typical scales (5 web + 5 workers) this is 100 connections — already at the ceiling. PgBouncer or equivalent is the standard mitigation. The docs should state this plainly and recommend PgBouncer for scale-out deployments.

2. **No built-in metrics or OTel export:** Generated deployment manifests (DigitalOcean App Platform YAML, Docker Compose) do not include an OpenTelemetry collector sidecar or Prometheus scrape config. Monitoring of worker throughput and queue depth requires a separately provisioned observability stack; the framework does not emit one.

3. **Latency:** Result latency is worker-scheduling-bound. The offload path is unsuited for sub-second interactive computation that must complete before a response is rendered. It is the right shape for deferred-result work (report generation, imports, model inference).

**2.0 Non-Goals (D-11, sourced from spec §Future direction):**

- Elastic scale-to-zero via KEDA `ScaledObject` derived from queue depth.
- Warm-start / CRIU-style checkpoint-restore for fast scale-from-zero cold starts.
- Non-Kubernetes actuation via a `WorkerFleetProvider` port (e.g. Nomad).
- WASM/WASI isolates as a lighter execution unit.

These are framed as future work in the spec and must be documented as such in `offload.md`, not as commitments.

**Claims that require code verification before documentation:**

The following facts come from spec/context documents and should be spot-verified against actual implementation before the doc is merged:

- `enqueue_and_mark_pending` exists at `ferro::offload::enqueue_and_mark_pending` [ASSUMED: referenced in `queues.md` L307 — verify the re-export path in `framework/src/lib.rs`].
- `resolve`, `read_result_redacted`, `read_result` exist at `ferro::offload::*` [ASSUMED: referenced in `queues.md` — verify re-exports].
- `projection_snapshots` migration is `CreateProjectionSnapshotsTable` from `ferro_projection` [ASSUMED: referenced in `queues.md` L373 — verify crate].
- `serve --no-worker` flag is actually implemented and wired in the app's `main.rs` clap Commands [ASSUMED: Phase 248 is marked complete — verify `app/src/main.rs` has the `no_worker` flag].

These are all verifiable with one `grep` or `Read` per item in the implementation task and are low risk — they derive from completed phases' context files and existing `queues.md` prose.

---

## Architecture Patterns

### Static Parser State Machine

```
┌──────────────────────────────────────────────────────────────────┐
│  scan_services_from_files (per .rs file)                        │
│                                                                  │
│  Line-by-line iteration                                          │
│                                                                  │
│  State: Idle                                                     │
│    ├─ sees "#[service(...)]"  → push ServiceItem, stay Idle     │
│    ├─ sees "#[offload]"       → State: OffloadPending(queue:"default") │
│    ├─ sees "#[offload(queue…)]" → State: OffloadPending(queue:"x") │
│    └─ other                  → stay Idle                         │
│                                                                  │
│  State: OffloadPending(queue)                                    │
│    ├─ sees "async fn name(" or "fn name(" → State: FnCollecting │
│    ├─ sees "///", "//", "#[…]"           → stay OffloadPending  │
│    └─ sees "#[offload…]"                 → update queue, stay   │
│                                                                  │
│  State: FnCollecting(method, queue, buf)                         │
│    ├─ append line to buf                                         │
│    ├─ if buf contains closing ")"        → parse params from buf │
│    │    → push OffloadableMethod, State: Idle                    │
│    └─ else                              → stay FnCollecting      │
└──────────────────────────────────────────────────────────────────┘
```

### Param Text Parsing

```
Input:  "&self, tenant_id: i64, month: Month"
1. Strip leading "&self" / "self" / "&mut self" (first segment)
2. Bracket-aware split on "," (depth-track < > [ ])
3. For each segment: split on first ":" → (name_part, type_part)
4. Strip whitespace from both
5. Apply owned_type substitution on type_part (text rules):
   - "&str"   → "String"
   - "&[T]"   → "Vec<T>"  (T = everything between [ and ])
   - "&T"     → "T"        (strip leading &)
   - other    → verbatim
Output: Vec<OffloadParam>
```

### `offload.md` Document Structure

```
# Work Distribution (Offload)

## Authoring an offloadable method
  [relocated from queues.md §"Offloading Service Methods"]
  - Authoring surface code example
  - Typed handle
  - Success-type contract table
  - Serializable contract + compile-time enforcement

## Result path
  [relocated from queues.md §"Subscribe and await"]
  - Channel convention
  - Request side / enqueue_and_mark_pending
  - Server-side consumer (race-safe resolve)
  - Browser / client-side read-back
  - Delta payload and redaction
  - Migration (CreateProjectionSnapshotsTable)

## Scaling model
  [new — from spec + 248 decisions]
  - Deploy recipe (serve --no-worker + worker --queue)
  - Worker class = queue set, independent fault domain
  - Deployment shape diagram (prose)
  - Honest limitations subsection (PgBouncer, no OTel, latency)

## Non-goals (2.0 direction)
  [new — from spec §Future direction]
  - Scale-to-zero / KEDA
  - Warm-start
  - Non-Kubernetes actuation
  - WASM isolates
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| AST parsing of Rust source | A recursive descent parser for `syn`-level grammar | Line-based state machine (see §Finding 3) | The grammar of valid `#[offload]` signatures is narrow; a full parser adds a dep and is overkill for a best-effort read surface |
| Full JSON Schema emission | `schemars::JsonSchema` derivation | Typed param list `[{name, rust_type}]` | D-06 explicitly rejects schemars; the typed list is the decided shape |

---

## Common Pitfalls

### Pitfall 1: `ServiceItem` round-trip breakage
**What goes wrong:** The runtime path deserializes `/_ferro/services` into `ServiceItem` via the `Deserialize` derive. Adding a non-optional `methods` field without a `#[serde(default)]` or `skip_serializing_if` will break deserialization of existing runtime payloads that do not carry `methods`.
**How to avoid:** `Vec<OffloadableMethod>` must be decorated with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`. The `Deserialize` derive on `ServiceItem` is used by `fetch_runtime_services` to map `RuntimeServiceInfo` — but note that `fetch_runtime_services` actually maps through `RuntimeServiceInfo` first (lines 47–51) and constructs `ServiceItem` manually; `ServiceItem`'s `Deserialize` is not exercised in the runtime path. However, keeping `serde(default)` is still correct practice and costs nothing.

### Pitfall 2: Param split failure on generic types
**What goes wrong:** Splitting on `,` naively breaks `HashMap<K, V>` — the comma inside the angle brackets is not a param separator.
**How to avoid:** Bracket-aware split: maintain a depth counter, increment on `<`/`[`, decrement on `>`/`]`, only split at depth 0.

### Pitfall 3: `#[service]` must appear in the same block as `#[offload]`
**What goes wrong:** The static parser identifies offloadable methods by file; it needs to correlate them with the correct `ServiceItem`. A method in a `#[service]` trait is inside the trait item block. The parser currently detects `#[service(ConcreteType)]` on one line and extracts `name = ConcreteType`. For offload augmentation, the parser needs to know which trait (service) a given `#[offload]` method belongs to.
**How to avoid:** The state machine must also track which `#[service(...)]` trait is currently open (detect `pub trait TraitName` following a `#[service(...)]`, track opening/closing `{` `}`). Match offload method entries back to the enclosing service. Alternatively — simpler — run a two-pass approach: pass 1 builds `Vec<ServiceItem>` as today; pass 2 scans for `#[offload]` methods and correlates them to `ServiceItem` by matching the trait name. If no matching `ServiceItem` is found for an offload method, skip it (unknown service, not registered).

### Pitfall 4: MCP tool description accuracy bar
**What goes wrong:** Tool descriptions in `service.rs` are part of the framework surface and held to the Rust-API accuracy bar (CLAUDE.md). An imprecise description (e.g., claiming `methods` is always present) will mislead agents.
**How to avoid:** The description must clearly state that `methods` is omitted for services with no offloadable methods. The concrete JSON example in the description or in the docs page should show both cases.

### Pitfall 5: `queues.md` content stranded
**What goes wrong:** Relocating content from `queues.md` to `offload.md` without replacing it with a pointer leaves `queues.md` without a section, breaking any inbound links (e.g., from `docs/src/getting-started/quickstart.md` or external links).
**How to avoid:** Replace the relocated sections in `queues.md` with a short pointer paragraph (see §Finding 8). Grep for `queues.md#offload` or `queues.md#offloading` inbound anchors before relocation. [ASSUMED: no external anchor links found without a grep — verify during implementation.]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (synchronous helpers) |
| Config file | `Cargo.toml` (no separate test config) |
| Quick run command | `cargo test -p ferro-mcp list_services` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OFFLOAD-06 (MCP) | `detect_offload_attr` returns correct queue | unit | `cargo test -p ferro-mcp detect_offload` | ❌ Wave 0 |
| OFFLOAD-06 (MCP) | `extract_method_params` bracket-aware split | unit | `cargo test -p ferro-mcp extract_method_params` | ❌ Wave 0 |
| OFFLOAD-06 (MCP) | Full static parse of a file with two offload methods | unit | `cargo test -p ferro-mcp scan_offload_methods` | ❌ Wave 0 |
| OFFLOAD-06 (MCP) | Non-offload service output unchanged | unit | `cargo test -p ferro-mcp plain_service_unchanged` | ❌ Wave 0 |
| OFFLOAD-06 (docs) | `offload.md` exists and is registered in SUMMARY.md | manual | n/a — doc existence check | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p ferro-mcp`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features`
- **Phase gate:** Full suite green before verification

### Wave 0 Gaps
- [ ] `ferro-mcp/src/tools/list_services.rs` — add `#[cfg(test)] mod tests` block with the five test cases above
- [ ] No new test file needed; tests live inline with the module (matching the `route_dependencies.rs` pattern)

---

## Security Domain

This phase is a read-only MCP surface extension (static file analysis, no network writes) and a docs authoring task. No new authentication surface, no new input validation boundary, no cryptography. ASVS categories V2/V3/V4/V6 do not apply. V5 (input validation) is trivially satisfied: the static parser reads local source files under a trusted project root already gated by the MCP server's project-root initialization.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `enqueue_and_mark_pending`, `resolve`, `read_result_redacted`, `read_result` are re-exported from `ferro::offload::*` | §Scaling Model Source Facts | Doc code examples reference wrong paths; fix during implementation with one grep |
| A2 | `CreateProjectionSnapshotsTable` lives in the `ferro_projection` crate (not `ferro_queue`) | §Scaling Model Source Facts | Incorrect migration import in docs; fix with one grep |
| A3 | `serve --no-worker` is present in `app/src/main.rs` (Phase 248 complete) | §Scaling Model Source Facts | Docs describe a flag that doesn't exist yet; verify before authoring §Deploy recipe |
| A4 | No inbound anchor links to `queues.md#offloading-service-methods` from other doc pages | Pitfall 5 | Broken cross-doc links after relocation; verify with `grep -r "queues.md#" docs/src/` |

---

## Open Questions

1. **Two-pass vs. single-pass service correlation (Pitfall 3)**
   - What we know: the current parser does not track trait block boundaries.
   - What's unclear: is two-pass (build `ServiceItem` list first, then augment) simpler than maintaining `{` `}` depth in the single pass?
   - Recommendation: two-pass is simpler and correct given the source structure. The planner should structure the implementation as: `scan_services_from_files` (unchanged output), then a new `scan_offload_methods_from_files` → augment the `ServiceItem` vector. The second pass only needs to know which trait name each `#[offload]` method belongs to, not the full `{}` nesting depth.

2. **`generation_context` offload field inclusion**
   - What we know: cheap to add (~15 lines).
   - What's unclear: whether the CONTEXT.md "acceptable if cheap" threshold is satisfied now that the scope is known to be < 20 lines.
   - Recommendation: include it. The planner should add `offload: &'static str` (or a thin struct) to `GenerationContext` in the same task that creates `offload.md`, since both need to agree on the docs pointer.

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — this phase extends an existing Rust module and authors Markdown files).

---

## Sources

### Primary (HIGH confidence)
- `ferro-mcp/src/tools/list_services.rs` — full source read; `ServiceItem`, `scan_services_from_files`, dual-path architecture confirmed
- `ferro-macros/src/offload.rs` — full source read; `OffloadMethodInfo` struct, `collect_info`, `owned_type`, `emit_job_items` confirmed
- `ferro-macros/src/service.rs` lines 183–219 — `#[offload]` detection + queue arg parsing loop confirmed
- `ferro-mcp/src/service.rs` lines 600–616 — current `list_services` tool description confirmed
- `ferro-mcp/src/tools/generation_context.rs` — `GenerationContext` struct + `execute()` function structure confirmed
- `ferro-mcp/src/tools/route_dependencies.rs` lines 220–313 — test pattern (inline `#[test]` on sync helpers) confirmed
- `docs/src/SUMMARY.md` — full nav confirmed; `offload.md` not yet registered
- `docs/src/features/queues.md` lines 188–end — sections to relocate confirmed
- `docs/superpowers/specs/2026-06-24-offload-work-distribution-design.md` — §Scaling model, §Honest limitations, §Future direction, §Non-Goals verified
- `.planning/phases/248-deployable-ferro-worker-runtime/248-CONTEXT.md` — decided Phase 248 worker surface verified (D-01 through D-08)

### Secondary (MEDIUM confidence)
- `.planning/REQUIREMENTS.md` OFFLOAD-06 — requirement text confirmed
- `.planning/phases/249-ferro-mcp-introspection-docs/249-CONTEXT.md` — decisions D-01 through D-11 and canonical refs confirmed

---

## Metadata

**Confidence breakdown:**

- Static parser extension: HIGH — source code read in full; state-machine approach is well-grounded
- Serde shape: HIGH — verified against existing codebase conventions
- Tool description change: HIGH — current text read verbatim
- Docs mechanics: HIGH — SUMMARY.md and queues.md sections read; exact line ranges confirmed
- Scaling model facts: HIGH for spec / MEDIUM for implementation (four facts tagged ASSUMED pending implementation grep)
- Test pattern: HIGH — route_dependencies.rs pattern confirmed; plain `#[test]` applicable

**Research date:** 2026-08-15
**Valid until:** 2026-09-15 (stable; no external deps, purely internal codebase extension)
