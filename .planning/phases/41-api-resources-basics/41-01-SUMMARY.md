---
phase: 41-api-resources-basics
plan: 01
subsystem: api
tags: [resource, serde_json, builder-pattern, api-response, json]

requires:
  - phase: none
    provides: n/a
provides:
  - Resource trait with to_resource, to_response, to_wrapped_response, to_response_with
  - ResourceMap builder with field, when, unless, merge_when, when_some, build
  - Both types exported from ferro::Resource and ferro::ResourceMap
affects: [41-02-derive-macro, 41-03-collection-resources, 42-api-resources-advanced]

tech-stack:
  added: [serde_json/preserve_order]
  patterns: [resource-map-builder, tcp-loopback-test-helper]

key-files:
  created:
    - framework/src/http/resources/mod.rs
    - framework/src/http/resources/resource.rs
    - framework/src/http/resources/resource_map.rs
  modified:
    - framework/src/http/mod.rs
    - framework/src/lib.rs
    - framework/Cargo.toml

key-decisions:
  - "Enable serde_json preserve_order for insertion-order field output in ResourceMap"
  - "TCP loopback helper for constructing real hyper::body::Incoming in unit tests"
  - "ResourceValue enum with Present/Missing for conditional field support"

patterns-established:
  - "ResourceMap builder: conditional field inclusion via when/unless/merge_when/when_some"
  - "Resource trait: request-aware model-to-JSON transformation"
  - "with_test_request helper: TCP loopback to create real Request for unit tests"

duration: 13min
completed: 2026-02-10
---

# Phase 41 Plan 01: Resource Trait and ResourceMap Builder Summary

**Resource trait with request-aware to_resource/to_response methods and ResourceMap conditional-field builder using serde_json preserve_order**

## Performance

- **Duration:** 13 min
- **Started:** 2026-02-10T04:23:54Z
- **Completed:** 2026-02-10T04:37:44Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- ResourceMap builder with field(), when(), unless(), merge_when(), when_some(), build() for conditional JSON field inclusion
- Resource trait with to_resource(), to_response(), to_wrapped_response(), to_response_with() for model-to-JSON transformation
- Both types exported from ferro::Resource and ferro::ResourceMap in the public API
- 13 unit tests covering all ResourceMap methods and Resource trait implementations

## Task Commits

Each task was committed atomically:

1. **Task 1: Create ResourceMap builder** - `ee14a2a` (feat)
2. **Task 2: Create Resource trait with response helpers** - `38947cc` (feat)
3. **Task 3: Add unit tests for ResourceMap and Resource trait** - `0d9907f` (test)

## Files Created/Modified
- `framework/src/http/resources/mod.rs` - Module definition with re-exports of Resource and ResourceMap
- `framework/src/http/resources/resource.rs` - Resource trait definition with default response methods
- `framework/src/http/resources/resource_map.rs` - ResourceMap builder with conditional field support
- `framework/src/http/mod.rs` - Added resources module and re-exports
- `framework/src/lib.rs` - Added Resource and ResourceMap to public API
- `framework/Cargo.toml` - Enabled serde_json preserve_order feature

## Decisions Made
- **serde_json preserve_order:** Enabled the `preserve_order` feature so ResourceMap::build() preserves field insertion order. Without it, serde_json::Map uses BTreeMap which sorts keys alphabetically. This is important for predictable API output.
- **TCP loopback test helper:** Created `with_test_request()` helper that spawns a one-shot HTTP server on localhost to construct a real `hyper::body::Incoming`-backed Request. This is necessary because Incoming has no public constructor in hyper 1.x.
- **ResourceValue::Missing sentinel:** Used a Present/Missing enum internally even though `when(false)` simply doesn't add the field. This keeps the code consistent and extensible for future merge/override operations.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Enabled serde_json preserve_order feature**
- **Found during:** Task 3 (unit tests for field order preservation)
- **Issue:** serde_json::Map uses BTreeMap by default, which sorts keys alphabetically. The plan specifies "preserving insertion order" for ResourceMap output.
- **Fix:** Added `features = ["preserve_order"]` to serde_json dependency in framework/Cargo.toml
- **Files modified:** framework/Cargo.toml, Cargo.lock
- **Verification:** test_field_order_preserved passes with correct insertion order
- **Committed in:** 0d9907f (Task 3 commit)

**2. [Rule 3 - Blocking] Created TCP loopback helper for Request construction in tests**
- **Found during:** Task 3 (Resource trait unit tests)
- **Issue:** hyper::body::Incoming has no public constructor, making it impossible to construct a Request in unit tests
- **Fix:** Created with_test_request() helper that uses TCP loopback + hyper::server::conn::http1 to construct a real Request
- **Files modified:** framework/src/http/resources/resource.rs
- **Verification:** All 3 Resource trait tests pass
- **Committed in:** 0d9907f (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were necessary for test correctness. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Resource trait and ResourceMap builder are complete and exported from the public API
- Ready for Plan 02 (ApiResource derive macro) which will generate Resource impls automatically
- The with_test_request() helper pattern can be reused in Plan 02 and 03 tests

---
*Phase: 41-api-resources-basics*
*Completed: 2026-02-10*
