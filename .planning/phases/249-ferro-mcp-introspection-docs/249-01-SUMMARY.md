---
phase: 249-ferro-mcp-introspection-docs
plan: "01"
subsystem: ferro-mcp
tags: [mcp, list_services, offload, static-parser, generation-context]
dependency_graph:
  requires: []
  provides: [offload-introspection-mcp, list_services-offload-methods]
  affects: [ferro-mcp]
tech_stack:
  added: []
  patterns: [three-state-machine-parser, bracket-aware-split, additive-serde-field]
key_files:
  created: []
  modified:
    - ferro-mcp/src/tools/list_services.rs
    - ferro-mcp/src/service.rs
    - ferro-mcp/src/tools/generation_context.rs
decisions:
  - "D-01 honored: list_services extended in place, no separate offload tool"
  - "D-02 honored: plain services serialize unchanged (proven by plain_service_unchanged test)"
  - "D-03 honored: tool description states methods array and omit-when-empty"
  - "D-04 honored: offload facts derived from static source parse inside ferro-mcp"
  - "D-05 honored: scan_offload_methods_from_files wired on both execute() branches; /_ferro/services endpoint untouched"
  - "D-06 honored: typed param list [{name, rust_type}], no schemars, no new trait bound"
metrics:
  duration: 502s
  completed: "2026-08-15"
  tasks_completed: 3
  files_modified: 3
---

# Phase 249 Plan 01: ferro-mcp Offload Introspection Summary

Static parser extension giving `list_services` offload-method awareness: `OffloadableMethod` + `OffloadParam` structs, a three-state-machine second-pass walker (`scan_offload_methods_from_files`), dual-branch wiring in `execute()`, updated tool description, and a read-only `offload` field on `GenerationContext`.

## What Was Built

### `ferro-mcp/src/tools/list_services.rs`

Two new serializable structs added above `ServiceItem`:

- `OffloadParam { name: String, rust_type: String }` — one non-self parameter with its owned type string.
- `OffloadableMethod { name, queue, params: Vec<OffloadParam> }` — one `#[offload]`-annotated method; `params` omitted when empty via `skip_serializing_if`.

`ServiceItem` gained an additive `methods: Vec<OffloadableMethod>` field with `#[serde(default, skip_serializing_if = "Vec::is_empty")]`. The `default` attribute preserves round-trip deserialization of runtime payloads that carry no `methods` key; `skip_serializing_if` keeps plain-service JSON output byte-for-byte identical to the previous format (D-02).

Three helper functions implement the parse surface:

- `detect_offload_attr(trimmed: &str) -> Option<String>` — returns the declared queue or `"default"` for bare `#[offload]`; `None` for non-matching lines.
- `extract_method_params(inner: &str) -> Vec<OffloadParam>` — bracket-aware comma split (depth-tracks `<`/`[` vs `>`/`]`), drops the receiver segment, splits each segment on the first `:`, and applies `owned_type` substitution (`&str` → `String`, `&[T]` → `Vec<T>`, `&T` → `T`).
- `scan_offload_methods_from_files(project_root: &Path, services: &mut [ServiceItem])` — second-pass walker over `src/**/*.rs` using the same WalkDir filter as `scan_services_from_files`. Runs a three-state machine (Idle → OffloadPending → FnCollecting) per file, tracks `#[service(ConcreteType)]` and the following `trait TraitName` to correlate discovered methods to the right `ServiceItem` by concrete or trait name. Methods for unregistered services are discarded.

`execute()` calls `scan_offload_methods_from_files` on both the runtime path (after fetching from `/_ferro/services`) and the static path, so agents always see offload data regardless of whether the app is running (D-05).

Six inline unit tests in `#[cfg(test)] mod tests`:

| Test | Behavior |
|------|----------|
| `detect_offload_attr_bare_returns_default` | bare `#[offload]` → queue `"default"` |
| `detect_offload_attr_reads_declared_queue` | `#[offload(queue = "reports")]` → `"reports"`; non-offload line → `None` |
| `extract_method_params_bracket_aware` | `HashMap<K, V>` inner comma does not split |
| `extract_method_params_owned_substitution` | `&str` → `String`, `&[Tag]` → `Vec<Tag>` |
| `scan_offload_methods` | temp-dir fixture with 2 offload methods and 1 non-offload; asserts `methods.len() == 2` |
| `plain_service_unchanged` | zero-methods `ServiceItem` serializes with no `"methods"` key |

### `ferro-mcp/src/service.rs`

`list_services` tool description updated (D-03): `**When to use:**` extended with `or discovering which service methods are offloadable`; `**Returns:**` rewritten to state that services with `#[offload]` methods include a `methods` array and that plain services omit the field.

### `ferro-mcp/src/tools/generation_context.rs`

`GenerationContext` struct gained a flat `offload: &'static str` field immediately after `live_projection`, mirroring the `memoize: &'static str` pattern. Populated in `execute()` with a one-sentence description of `#[offload]`, queue defaulting, worker deploy shape, and a pointer to `docs/src/features/offload.md`. No authoring template was added (deferred per plan).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added `Deserialize` to `OffloadParam` and `OffloadableMethod`**
- **Found during:** Task 1 compile
- **Issue:** `ServiceItem` derives `Deserialize`; its `methods: Vec<OffloadableMethod>` field requires `OffloadableMethod: Deserialize`. The plan specified `#[derive(Debug, Serialize, Clone)]` for both new structs, omitting `Deserialize`. The compiler rejected the struct extension.
- **Fix:** Added `Deserialize` to both `OffloadParam` and `OffloadableMethod`.
- **Files modified:** `ferro-mcp/src/tools/list_services.rs`
- **Commit:** bc612982

**2. [Rule 1 - Bug] Changed `&mut Vec<ServiceItem>` to `&mut [ServiceItem]`**
- **Found during:** Task 1/2 clippy pass
- **Issue:** `clippy::ptr_arg` flagged both `scan_offload_methods_from_files` and `flush_block` for taking `&mut Vec<T>` where `&mut [T]` is sufficient. `-D warnings` makes this a build error.
- **Fix:** Changed both signatures to `&mut [ServiceItem]`.
- **Files modified:** `ferro-mcp/src/tools/list_services.rs`
- **Commit:** bc612982

### TDD Gate Note

Tasks 1 and 2 were implemented atomically in a single write. The plan called for a RED commit (structs + tests referencing not-yet-existing helpers) followed by a GREEN commit (implement helpers). Because the helpers were authored together with the structs, the RED state was never persisted to git. All six tests pass from the first commit. The behavior contract is fully covered; only the two-step commit sequence was consolidated.

## Known Stubs

None. All new fields carry real values; no placeholder text flows to any output surface.

## Threat Flags

None. The changes add a local source read (same trust boundary as the existing `scan_services_from_files` walk) and additive JSON output. No new network endpoint, auth path, or untrusted input boundary introduced. Consistent with the plan's threat model (T-249-01..T-249-03, all `accept`).

## Self-Check: PASSED

| Item | Status |
|------|--------|
| `ferro-mcp/src/tools/list_services.rs` | FOUND |
| `ferro-mcp/src/service.rs` | FOUND |
| `ferro-mcp/src/tools/generation_context.rs` | FOUND |
| commit bc612982 (Tasks 1+2) | FOUND |
| commit b73ac92a (Task 3) | FOUND |
| `cargo test -p ferro-mcp` 320 passed, 0 failed | PASSED |
| `cargo clippy -p ferro-mcp --all-targets -- -D warnings` | CLEAN |
