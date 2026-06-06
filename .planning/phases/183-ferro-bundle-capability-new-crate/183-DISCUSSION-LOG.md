# Phase 183: `ferro-bundle` capability (new crate) — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 183-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-06
**Phase:** 183-ferro-bundle-capability-new-crate
**Mode:** `--auto` (all gray areas auto-resolved with recommended option; choices logged below)
**Areas discussed:** Storage type, Registry mechanism, URL routing, Hash truncation, ETag format, Registration timing, Hash algorithm, Alias mechanism, Crate dependencies, README requirement, Workspace + publish.yml integration, Publish bootstrap, Test isolation

---

## Storage type

| Option | Description | Selected |
|--------|-------------|----------|
| `&'static [u8]` only (compile-time `include_bytes!`) | Matches locked API signature; simplest contract | ✓ |
| `Cow<'static, [u8]>` (compile-time OR runtime bytes) | Broader applicability, larger surface | |
| `Arc<[u8]>` (runtime bytes only) | Anti-pattern: defeats the "immutable static blob" model | |

**Auto-selected:** `&'static [u8]` only. Roadmap locks the API signature. Runtime-bytes deferred to a v2 if a real consumer surfaces the need.

---

## Registry mechanism

| Option | API shape | Selected |
|--------|-----------|----------|
| Process-global `OnceLock<DashMap<String, BundleEntry>>` keyed by URL path | `Bundle::serve(req)` looks up `req.path()` | ✓ |
| Per-`App` registration via `app.bundle(b)` | Would require `Bundle::serve(req, &registry)` — conflicts with locked API | |
| `lazy_static`/`once_cell::Lazy` | Functionally equivalent; extra dep cost | |

**Auto-selected:** `OnceLock<DashMap<…>>`. The locked API `Bundle::serve(req)` takes only the request — the registry MUST be process-global for the lookup to work. `std::sync::OnceLock` is stable since Rust 1.70, no extra dep over `dashmap` (which is already a transitive in multiple ferro-* crates).

---

## URL routing

| Option | Description | Selected |
|--------|-------------|----------|
| `Bundle::serve(req)` dispatches by `req.path()` (registry lookup) | Single handler, consumer routes `/bundles/{filename}` to it | ✓ |
| Per-bundle dynamic route registration | Would require the framework to know the URL at registration time; chicken-and-egg with hash | |

**Auto-selected:** Single-handler dispatch via the registry. Consumer mounts `Bundle::serve` as the handler for the `/bundles/…` namespace (and for any registered alias paths).

---

## Hash truncation

| Option | Description | Selected |
|--------|-------------|----------|
| 8 hex chars (32 bits, ~4.3B collision space) | Matches roadmap example URL | ✓ |
| 16 hex chars (64 bits) | Overkill for practical bundle counts | |
| Full 64 hex chars | Bloats the URL | |

**Auto-selected:** 8 hex chars. Documented in the README so future contributors know the collision-space tradeoff. Extendable in v2 without breaking the URL format (the chars are a left-prefix of the full SHA-256).

---

## ETag format

| Option | Description | Selected |
|--------|-------------|----------|
| Strong, full SHA-256 hex, quoted | RFC 7232 compliant; full content-integrity match | ✓ |
| Strong, 8-hex-char truncation | Conflates URL hash with content-integrity hash | |
| Weak (`W/"…"`) | Wrong semantics — bytes are byte-exact, not semantically equivalent | |

**Auto-selected:** Strong, full SHA-256 hex, quoted per RFC 7232 §2.3. The URL uses the truncated hash as a cache-busting handle; the ETag uses the full hash for content-integrity matching. Same algorithm, different precision, different purpose.

---

## Registration timing

| Option | Description | Selected |
|--------|-------------|----------|
| Eager at `Bundle::new()` call; panic on duplicate name | Predictable startup-time error for developer mistakes | ✓ |
| Lazy on first `.hashed_url()` call | Adds a hidden ordering dependency between `Bundle::new()` and `Bundle::serve()` | |
| Builder pattern with explicit `.register(&app)` | Conflicts with locked `Bundle::serve(req)` API shape | |

**Auto-selected:** Eager at `Bundle::new()`. Hashing bytes happens once, on construction. Duplicate-name panic is loud, immediate, and fixable by the developer at startup — far better than a silent override at runtime.

---

## Hash algorithm

| Option | Description | Selected |
|--------|-------------|----------|
| SHA-256 | Locked by roadmap | ✓ |

**Auto-selected:** SHA-256. Industry-standard for content addressing. Crate dep `sha2`.

---

## Alias mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Stored on Bundle, registered in parallel global registry, 301 redirect | Permanent backward-compat for old URLs | ✓ |
| 302 redirect (temporary) | Wrong semantics — the alias is permanent | |
| Server-side rewrite (no redirect, just serve at the alias URL) | Loses the cache benefit of the hashed URL on the client | |

**Auto-selected:** 301 redirect from alias path to current hashed URL. Locked by Success Criterion #3. Multiple aliases per bundle are supported (`.with_alias("/a.js").with_alias("/b.js")`).

---

## Crate dependencies

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: `sha2`, `hex`, `dashmap`, `framework` | Smallest possible surface | ✓ |
| Add `http` crate for standalone HTTP types | Decouples from `framework`, adds duplication | |
| Re-implement HTTP types | Anti-pattern; framework owns them | |

**Auto-selected:** `sha2` + `hex` + `dashmap` + `framework` (published as `ferro-rs`). `framework` dependency makes Phase 183's crate a Wave1B publish.

---

## README requirement

| Option | Description | Selected |
|--------|-------------|----------|
| README with bundle-vs-filesystem-split section, code example, usage notes | Locked by Success Criterion #5 | ✓ |
| README minimal (description only) | Violates SC-5 | |
| Defer to `cargo doc` | `cargo doc` doesn't surface design-rationale framing | |

**Auto-selected:** Full README. The "two parallel asset-serving paths intentional" architectural note must live somewhere a future contributor's `ls ferro-bundle/` browse will find it. The README is the right place.

---

## Workspace + publish.yml integration

| Option | Description | Selected |
|--------|-------------|----------|
| Add to `Cargo.toml` `workspace.members` + `.github/workflows/publish.yml` Wave1B | Matches CLAUDE.md project rule + dep graph | ✓ |
| Add to Wave1A | Wrong: `ferro-bundle` depends on `framework` (Wave0/1A) | |
| Skip publish.yml update | Violates CLAUDE.md project rule for new crates | |

**Auto-selected:** Wave1B placement. Researcher confirms the dep graph during planning.

---

## Publish bootstrap

| Option | Description | Selected |
|--------|-------------|----------|
| Manual `cargo publish -p ferro-bundle` from local terminal at first publish | Memory `project_ferro_publish_token_scoping.md` — CI token can't create new crates | ✓ |
| Assume CI handles first publish | Will fail with "no permission to create crate" | |
| Skip bootstrap, only allow consumption after second version | Adds an unnecessary version | |

**Auto-selected:** Manual bootstrap. Planner adds an explicit "manual `cargo publish` from local terminal" task with notes; CI workflow takes over from the second version onward.

---

## Test isolation

| Option | Description | Selected |
|--------|-------------|----------|
| `#[cfg(test)] pub(crate) fn reset()` clears the registries | Tests within one binary can isolate; integration tests already isolated by `cargo test` binary boundaries | ✓ |
| Thread-local registries | Wrong: production code is multi-threaded, thread-locals would break runtime behavior | |
| Per-`#[test]` registry created from a builder | Conflicts with the locked `Bundle::serve(req)` API shape | |

**Auto-selected:** `#[cfg(test)] reset()` helper. Each unit test that registers bundles calls `reset()` first. Integration tests run in separate processes by default, so OS-level isolation is sufficient there.

---

## Claude's Discretion

Delegated to planner/executor:

- Exact crate metadata fields (`keywords`, `categories`, `description`) — follow `ferro-storage` template.
- File layout under `ferro-bundle/src/` — single `lib.rs` or split into `bundle.rs` + `registry.rs` + `serve.rs`. Planner's call.
- Specific `thiserror`-derived error variant names.
- Exact integration-test layout under `ferro-bundle/tests/`.

---

## Deferred Ideas

Captured in 183-CONTEXT.md `<deferred>` section. Summary:

- Runtime-mutable bundle bytes (`Cow<'static, [u8]>` / `Arc<[u8]>`).
- Pre-deflated `Accept-Encoding` variants (gzip/br).
- Composite bundles / manifests.
- Streaming serve for very large bundles.
- Re-export `Bundle` through `framework` for ergonomic discoverability.
- Content-type sniffing (explicitly excluded by SC-4).

### Reviewed Todos (not folded)

None — `gsd-tools todo match-phase 183` was not run because the phase directory did not exist when `init phase-op` was invoked. If pending todos surface relevant to Phase 183 during planning, the planner can fold them into PLAN.md task descriptions at that point.
