---
phase: 195-close-the-loop-by-default
reviewed: 2026-06-10T00:00:00Z
depth: standard
files_reviewed: 6
files_reviewed_list:
  - ferro-mcp/src/tools/checkpoint_projection.rs
  - ferro-mcp/src/tools/generate_projection.rs
  - ferro-mcp/src/tools/json_ui_generate.rs
  - ferro-mcp/src/tools/projection_coverage.rs
  - ferro-mcp/src/tools/application_info.rs
  - ferro-mcp/src/service.rs
findings:
  critical: 0
  warning: 5
  info: 3
  total: 8
status: issues_found
---

# Phase 195: Code Review Report

**Reviewed:** 2026-06-10
**Depth:** standard
**Files Reviewed:** 6
**Status:** issues_found

## Summary

Phase 195 wires four wrapper seams into `checkpoint_projection`, embeds compact `VerdictSummary` values in `generate_projection` and `json_ui_generate`, and surfaces ambient cache reads in `projection_coverage` and `application_info`. The anti-reimplementation invariant (CHK-09/SC-4) is respected throughout: all wrapper seams dispatch to existing validators and carry the correct `source` provenance strings. The seam cascade logic (`seam_1_failed` → skip seams 4+5; `seam_4_failed` → skip seam 5) is correctly structured and well-tested.

The dominant issue class is one-way logic errors and scoping risks: `decide_seam4` gates on `Fail` only but the cascade in `run_for` gates on `Fail` only as well — consistent, but the `Warn` path is unchecked (seam 4 runs on seam 1 `Warn`, which is correct per spec, but the asymmetry is not tested). More significantly, seam 5's route filter is a bare substring match that can produce false-negative or false-positive contract checks, documented in the code but worth flagging. There are also a handful of `async fn` bodies (`rendered_view_seam`, `props_to_contract_seam`) that are called from an `async` context but are themselves synchronous — this is fine today but introduces a non-obvious coupling that should be noted. The `run_for` step numbering comment has an off-by-one in the inline comments (step 5 appears twice). Minor issues include the `to_snake_case` abbreviation for acronyms and a test helper using `source: "checkpoint"` for a seam that is not `field_to_column`, which contradicts the SC-4 guard the test is trying to verify.

---

## Warnings

### WR-01: Seam 5 route-filter substring match can over-match adjacent routes

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:582-583`
**Issue:** `props_to_contract_seam` passes `service_name.to_lowercase()` as the `route_filter` to `validate_contracts::execute`. This is a substring match (documented as "Pitfall 6: may include adjacent routes sharing the substring"). A service named `"booking"` will match routes containing `"booking"` anywhere in the path, including `/admin/rebooking`, `/pre_booking_confirm`, etc. The consequence is false-positive contract violations surfaced against unrelated routes, producing misleading findings and potentially degrading `seam5` to `Fail` for a projection that is itself clean. The inverse (under-match) can occur when the service name does not appear verbatim in any route path.
**Fix:** Until Phase 196 implements exact scoping, at minimum apply a word-boundary guard. Wrap the filter value with a path-segment delimiter: prefer matching `/{filter}/` or `/{filter}` as a prefix, rather than an arbitrary substring. If `validate_contracts` does not support regex filters yet, a post-filter step in `props_to_contract_seam` can discard matches whose route path does not contain `/{filter}` or start with `/{filter}`:

```rust
// Rough guard: only keep validations whose route path contains /filter/ or ends with /filter
let filter_segment = format!("/{filter}");
// After collecting `result.validations`, filter:
let relevant: Vec<_> = result.validations.iter()
    .filter(|v| v.route.contains(&filter_segment))
    .collect();
```

---

### WR-02: `run_for` step-numbering comment off-by-one — step 5 appears twice

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:195-233`
**Issue:** The inline step comments label the routes pre-load as step 5 (line 196) and then the verdict-aggregate block also as step 5 (line 221). The cache-write block (line 233) continues with step 6. This means step 5 is used for two distinct operations and there is no labeled step 6 for what is actually step 6. In a 237-line function with non-trivial control flow, this makes auditing harder and can hide logical reordering bugs.
**Fix:** Renumber consistently:

```
// 5. Pre-load routes once for seam 3 (async I/O done here, find_handler is sync).
// 6. Seam cascade (D-06):
// 7. Aggregate verdict (D-09).
// 8. Write status cache (D-11).
```

---

### WR-03: `decide_seam4`/`decide_seam5` pure helpers are `#[cfg(test)]`-only but the production cascade duplicates their logic

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:653-672`, `208-219`
**Issue:** The pure functions `decide_seam4` and `decide_seam5` (lines 654-672) exist only in `#[cfg(test)]` scope and are never called from `run_for`. The production cascade logic at lines 208-219 re-expresses the same if-chains inline. This is a divergence risk: a future change to the cascade conditions in `run_for` will not be tested by the `decide_seam4`/`decide_seam5` unit tests, which test a stale copy. The existing test `decide_seam5_pure` and `cascade_seam1_fail` are not actually testing the production code path — they test the test-only helpers.
**Fix:** Move `decide_seam4` and `decide_seam5` to production scope (remove `#[cfg(test)]`) and call them from `run_for`. Then the existing unit tests cover the real logic:

```rust
// Production scope (remove #[cfg(test)])
fn decide_seam4(seam1_status: &SeamStatus) -> Option<&'static str> { ... }
fn decide_seam5(seam1_status: &SeamStatus, seam4_status: &SeamStatus) -> Option<&'static str> { ... }

// In run_for:
let seam4 = match decide_seam4(&seam1.status) {
    Some(reason) => make_not_checked("rendered_view", "render_projection", reason),
    None => rendered_view_seam(project_root, name),
};
let seam5 = match decide_seam5(&seam1.status, &seam4.status) {
    Some(reason) => make_not_checked("props_to_contract", "validate_contracts", reason),
    None => props_to_contract_seam(project_root, &detail.service_name),
};
```

---

### WR-04: `read_ambient_status` path construction is NOT guarded by `validate_name` at the call sites in `projection_coverage` and `application_info`

**File:** `ferro-mcp/src/tools/projection_coverage.rs:104-108`, `ferro-mcp/src/tools/application_info.rs:435`
**Issue:** `read_ambient_status` is called with `proj.name` (the projection function name from `list_projections`) and `proj.name` (from `list_projections`) in `check_projection_checkpoint`. The comment on `read_ambient_status` at line 819 states the name is "trusted" because it originates from the projection scan. This trust assumption is documented but not enforced — nothing stops a projection file from having a function named with characters outside `[a-zA-Z0-9_-]` (e.g. unicode identifiers, or a crafted filename) reaching `read_ambient_status`. The path construction `format!("{name}.json")` is directly used inside `project_root.join(".ferro/checkpoints/{name}.json")`.

In practice the risk is low: Rust function names are already restricted by the compiler. However, the code comment says "The write path is already `validate_name`-guarded (T-195-01)" but the read path has no such guard. If an attacker can place a file in `src/projections/` with a crafted function name (e.g. via symlink or OS-level manipulation), the read path would not reject it.
**Fix:** Add `validate_name` to `read_ambient_status` as a defensive check that returns `"unverified"` on rejection, consistent with the general "bad input → unverified" contract:

```rust
pub(crate) fn read_ambient_status(project_root: &Path, name: &str) -> &'static str {
    if validate_name(name).is_err() {
        return "unverified";
    }
    // ... existing logic
}
```

---

### WR-05: `sc4_no_checkpoint_source_on_wrapper_seams` test uses `source: "checkpoint"` for a synthetic seam that is not `field_to_column`

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:1940-1946`
**Issue:** Inside the SC-4 guard test, seam 2 is constructed as:

```rust
let seam2 = SeamResult {
    seam: "field_to_column".to_string(),
    status: SeamStatus::Pass,
    source: "checkpoint".to_string(),
    ...
};
```

This is correctly `field_to_column`, so SC-4 does allow `source: "checkpoint"` here. The test is valid. However, the aggregate helper test `make_seam` (line 1236) hard-codes `source: "checkpoint"` for all seams including non-`field_to_column` ones:

```rust
fn make_seam(seam: &str, status: SeamStatus, findings: Vec<Finding>) -> SeamResult {
    SeamResult {
        source: "checkpoint".to_string(),
        ...
    }
}
```

Tests like `aggregate_status_fail_wins_over_not_checked` and `next_steps_ranked_deduped` produce `SeamResult` values with `seam: "projection_well_formed"` and `source: "checkpoint"`. This directly violates the SC-4 invariant (`source == "checkpoint"` must appear only on `field_to_column`) in test data, which means a future code scan or property test that validates SC-4 on all `SeamResult` values in the codebase would flag these tests as violations.
**Fix:** Update `make_seam` to accept `source` as a parameter, or default to `"test"` rather than `"checkpoint"`:

```rust
fn make_seam(seam: &str, status: SeamStatus, findings: Vec<Finding>) -> SeamResult {
    SeamResult {
        seam: seam.to_string(),
        status,
        source: "test".to_string(),   // neutral; not "checkpoint"
        findings,
        reason: None,
    }
}
```

---

## Info

### IN-01: `rendered_view_seam` and `props_to_contract_seam` are sync functions called in an async context — the pattern is fine but `render_projection::execute` is blocking I/O

**File:** `ferro-mcp/src/tools/checkpoint_projection.rs:494`, `582`
**Issue:** Both `rendered_view_seam` and `props_to_contract_seam` are synchronous (`fn`, not `async fn`). They perform filesystem I/O (`render_projection::execute` reads files; `validate_contracts::execute` parses `src/routes.rs`). These are called from the async `run_for` function. Since Tokio's `block_in_place` or `spawn_blocking` are not used, the blocking I/O runs on the async executor thread. This is consistent with the rest of the `ferro-mcp` tool architecture (other sync tools also do filesystem reads directly), so it is not a regression introduced by Phase 195. No action is required unless the executor model changes, but worth noting as a pattern to track.
**Fix:** No immediate action. If the project moves to `tokio::spawn_blocking` for tool I/O, these two functions should be migrated at the same time.

---

### IN-02: `to_snake_case` in `projection_coverage.rs` produces verbose output for acronyms

**File:** `ferro-mcp/src/tools/projection_coverage.rs:182-195`
**Issue:** `to_snake_case("HTMLParser")` → `"h_t_m_l_parser"` (as confirmed by the test at line 283). The test explicitly asserts this behavior, meaning it is intentional. However, the output is used in the `suggestion` field exposed to agents: `"ferro make:projection h_t_m_l_parser --from-model"`. Agents following this suggestion would create a projection with an unexpected name for acronym-containing models.
**Fix:** This is a pre-existing limitation, not introduced by Phase 195. The test acknowledges it. If acronym handling becomes a real concern, a lookup table or heuristic (consecutive uppercase → treat as single token) can be added. For now, document the limitation in the function's doc comment.

---

### IN-03: `generate_projection.rs` calls `checkpoint_projection::run_for` with `chrono::Utc::now()` rather than an injected timestamp

**File:** `ferro-mcp/src/tools/generate_projection.rs:99`
**Issue:** `run_for(project_root, &anchor, chrono::Utc::now())` reads the wall clock inline. The `run_for` function was designed with an injected timestamp precisely to avoid this (D-11: "do not read wall-clock inside pure logic"). The same pattern appears in `json_ui_generate.rs` at line 129. Because the timestamp is used only for the `checked_at` field in the cache file, this is low risk. However, it means that test code that calls `execute` (the public entry point calling `run_for` with `Utc::now()`) cannot assert on the specific `checked_at` value.
**Fix:** This is a minor issue. For testability, `generate_projection::execute` could accept an optional timestamp, or rely solely on `checkpoint_projection::execute` (which wraps `run_for` with `Utc::now()`) rather than calling `run_for` directly. Since this is the same pattern as `checkpoint_projection::execute`, it is consistent. No action required unless test coverage of `checked_at` becomes important.

---

_Reviewed: 2026-06-10_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
