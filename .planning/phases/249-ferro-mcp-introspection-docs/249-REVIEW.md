---
phase: 249-ferro-mcp-introspection-docs
reviewed: 2026-08-15T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - ferro-mcp/src/tools/list_services.rs
  - ferro-mcp/src/service.rs
  - ferro-mcp/src/tools/generation_context.rs
  - docs/src/features/offload.md
  - docs/src/features/queues.md
  - docs/src/features/deployments.md
  - docs/src/SUMMARY.md
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 249: Code Review Report

**Reviewed:** 2026-08-15
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The change adds a best-effort static source parser (`scan_offload_methods_from_files`)
to `list_services`, surfacing `#[offload]`-annotated `#[service]` trait methods with
their queue and typed parameter list, plus supporting `generation_context` prose and a
new canonical `offload.md` doc page. I reviewed the parser for the concerns flagged in
the brief — unbounded accumulation, panics, off-by-one in bracket/paren tracking, the
owned-type substitution mirroring, and serialization additivity — and cross-checked the
docs against the actual macro surface (`ferro-macros/src/offload.rs`,
`ferro-macros/src/service.rs`).

No crash, panic, or unbounded-growth path was found: all slice indexing is guarded by a
preceding `find`, depth counters only feed comparisons (never index), and per-file state
is bounded by file size. The serialization additivity requirement holds — the
`skip_serializing_if` guards and the `plain_service_unchanged` test correctly ensure a
plain service serializes byte-for-byte unchanged. The owned-type substitution in
`owned_type` faithfully mirrors the macro's `&str -> String`, `&[T] -> Vec<T>`,
`&T -> T` rules.

The two warnings both concern the parser's recognition of the `#[service(...)]`
attribute. The parser handles only the positional `#[service(ConcreteType)]` form, but
the `#[service]` macro also accepts the named form `#[service(impl = ConcreteType,
fake = ...)]` — and the named form is the one used in the new `offload.md` canonical
example. On the named form the concrete-name correlation silently fails; the feature is
saved only by the trait-name fallback, and then only in files where the trait
declaration is present and matched. This is worth fixing because the documented, blessed
authoring surface is exactly the case that degrades.

## Warnings

### WR-01: `#[service(impl = X)]` named form is not recognized by the static parser

**File:** `ferro-mcp/src/tools/list_services.rs:151-162` (also `366-368`)
**Issue:**
The `#[service]` macro accepts two syntaxes (`ferro-macros/src/service.rs:31-64`):
- positional — `#[service(ReportBuilder)]`
- named — `#[service(impl = ReportBuilder)]` and `#[service(impl = Real, fake = Fake)]`

Both the initial scan (`scan_services_from_files`) and the offload walker
(`scan_offload_methods_from_files`) extract the concrete name as the raw substring
between the first `(` and the first `)`:

```rust
let impl_name = &trimmed[start + 1..end];      // scan_services_from_files:154
let impl_name = trimmed[start + 1..end].trim(); // scan_offload_methods_from_files:368
```

For `#[service(impl = ReportBuilder)]` this yields the string `"impl = ReportBuilder"`,
not `ReportBuilder`. Consequences:
- In the static-analysis fallback, the service is registered under the bogus name
  `"impl = ReportBuilder"`.
- In `flush_block` (`:559-566`), concrete-name correlation compares
  `Some("impl = ReportBuilder")` against the service names and fails to match; offload
  methods attach only if the *trait name* fallback matches (requires the `pub trait ...`
  line to be present and parsed in the same block).

This matters because the canonical example the phase itself ships uses the named form:
`docs/src/features/offload.md:17` → `#[service(impl = ReportBuilder)]`. The blessed
authoring surface is the exact input the parser mishandles.

**Fix:** Normalize the arg before use — strip an optional `impl =` prefix and keep only
the first comma-separated segment (to drop `fake = ...`):

```rust
fn service_impl_name(arg: &str) -> &str {
    let first = arg.split(',').next().unwrap_or(arg).trim();
    first
        .strip_prefix("impl")
        .map(str::trim_start)
        .and_then(|s| s.strip_prefix('='))
        .map(str::trim)
        .unwrap_or(first)
}
```

Apply at both extraction sites (`:154` and `:368`).

### WR-02: `#[service(...)]` with the attribute args split across lines is missed

**File:** `ferro-mcp/src/tools/list_services.rs:151-162, 365-382`
**Issue:**
Both scanners require `#[service(` … `)` to open and close on the same trimmed line
(`trimmed.find('(')` then `trimmed.find(')')` on the *same* `line`). When the concrete
type is long enough to wrap — e.g.

```rust
#[service(
    impl = some_module::VeryLongConcreteReportBuilder,
)]
```

— the opening line has no `)`, the `if let Some(end) = trimmed.find(')')` guard fails,
and the block is never opened. No panic, but the service (and any `#[offload]` methods
under it) is silently dropped from the output. Given this is a fallback introspection
surface, silent omission is the failure mode to avoid.

**Fix:** This is lower-frequency than WR-01; the pragmatic fix is to accumulate the
`#[service(...)]` attribute across lines with the same paren-depth technique already used
for `fn` parameter lists (`extract_inner_params`), or to document the single-line
constraint as a known limitation. If left as-is, note it in the parser's module comment
so the boundary is explicit rather than surprising.

## Info

### IN-01: Macro doc examples disagree with the derived Job name (docs are correct, macro comments are stale)

**File:** `ferro-mcp/src/tools/generation_context.rs:598-602` and `docs/src/features/offload.md:26-28`
**Issue:**
`offload.md` states the derived Job is `<TraitPascalCase><MethodPascalCase>Job`, giving
`ReportsServiceBuildMonthlyJob` for `trait ReportsService` + `build_monthly` — which
matches the macro's actual output (`format_ident!("{}{}Job", trait_ident, method_pascal)`
at `ferro-macros/src/offload.rs:144`, using the full trait ident). However, the *macro's
own* doc comments and examples use `ReportsBuildMonthlyJob` (dropping `Service`):
`offload.rs:28, 49`. The new docs are right; the discrepancy is inside `ferro-macros`
(out of this phase's file scope) and is worth a one-line follow-up there so the two
sources agree. No action required inside phase 249 — flagged for traceability per the
audit-discrepancies convention.

### IN-02: `detect_offload_attr` only matches `queue = "` with exactly one space each side

**File:** `ferro-mcp/src/tools/list_services.rs:221`
**Issue:**
The queue name is located via `trimmed.find("queue = \"")` — a literal match requiring
exactly `queue = "`. A rustfmt-conformant attribute always produces this spacing, so in
practice it holds; but `#[offload(queue="reports")]` or `#[offload( queue = "x" )]`
(hand-typed, pre-fmt) would fall through to the "bare / default queue" branch (`:228`)
and mislabel the queue as `"default"`. This is a best-effort read-only surface, so it is
Info rather than Warning, but a tolerant match (split on `queue`, then find `"`) would
remove the spacing dependency.
**Fix:** Locate `queue`, then the next `"` and its closing `"`, ignoring interior
whitespace/`=`.

### IN-03: `OffloadPending` state can leak across a new `#[service(...)]` in malformed source

**File:** `ferro-mcp/src/tools/list_services.rs:364-382, 408-416`
**Issue:**
The `#[service(...)]` block-detection branch ends in `continue` (`:381`) before the state
machine runs, and the `OffloadPending` arm `continue`s on any `#[`-prefixed line
(`:412`). So an `#[offload]` immediately followed by a new `#[service(...)]` line (no
intervening `fn`) leaves the machine in `OffloadPending` across the block boundary; the
next `fn` encountered — now in the new block — is attributed as offloadable. This only
arises in malformed/interleaved source (an `#[offload]` with no method), causes no panic
and no unbounded growth, and mis-attributes at most one method. Acceptable for a
best-effort parser; noted for completeness.
**Fix (optional):** Reset `state = State::Idle` when opening a new `#[service(...)]`
block.

### IN-04: `walkdir` traversal is duplicated across the two scan passes

**File:** `ferro-mcp/src/tools/list_services.rs:140-144, 332-336`
**Issue:**
`scan_services_from_files` and `scan_offload_methods_from_files` each run an independent
`WalkDir` over `{project_root}/src` with the identical `.rs` filter, reading every file
twice per `list_services` call (the offload pass also runs after the runtime path at
`:110`). Not a correctness issue and off the v1 performance-scope, but the two passes
could share one traversal if this tool is ever called hot. Noted only as a maintenance
observation.

---

**Docs check (offload.md, queues.md, deployments.md, SUMMARY.md):** relative links and
anchors resolve — `deployments.md:9` → `offload.md#scaling-model` matches the
`## Scaling model` heading (`offload.md:199`); `queues.md:192` → `offload.md` (no anchor)
is valid; `SUMMARY.md:25` registers `features/offload.md` in nav. Code fences are
balanced and language-tagged. The prose claims verified against the code surface are
accurate: default queue `"default"`, `#[offload(queue = "name")]` override, worker recipe
`<app-bin> worker --queue <class>` / `serve --no-worker`, and the owned-type mapping table
all match `ferro-macros/src/offload.rs`. The only doc/code disagreement is IN-01, which
is internal to `ferro-macros` and not a phase-249 file.

---

_Reviewed: 2026-08-15_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
