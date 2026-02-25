---
phase: 69-static-file-serving
plan: 01
subsystem: infra
tags: [static-files, hyper, mime-guess, caching, security]

# Dependency graph
requires:
  - phase: none
    provides: n/a
provides:
  - Built-in static file serving from public/ directory
  - Differentiated cache headers for hashed vs unhashed assets
  - Security protections (dotfiles, traversal, null bytes)
affects: [ferro-inertia, deployment, production]

# Tech tracking
tech-stack:
  added: [mime_guess 2]
  patterns: [direct hyper::Response<Full<Bytes>> for binary-safe responses, path canonicalization for security]

key-files:
  created: [framework/src/static_files.rs, docs/src/features/static-files.md]
  modified: [framework/Cargo.toml, framework/src/lib.rs, framework/src/server.rs, docs/src/SUMMARY.md]

key-decisions:
  - "Build hyper::Response<Full<Bytes>> directly to avoid binary corruption through HttpResponse's String body"
  - "Testable try_serve_from_dir() helper accepts base path parameter; try_serve_static_file() delegates with public/"
  - "Static files checked before fallback handler to prevent SPA catch-all from serving HTML for asset requests"

patterns-established:
  - "Static file responses bypass HttpResponse to preserve binary integrity"
  - "Differentiated caching: immutable for /assets/*, must-revalidate for root files"

# Metrics
duration: 12min
completed: 2026-02-25
---

# Phase 69: Static File Serving Summary

**Built-in static file serving from public/ with immutable caching for Vite assets, dotfile/traversal security, and binary-safe responses**

## Performance

- **Duration:** 12 min
- **Started:** 2026-02-25
- **Completed:** 2026-02-25
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Zero-config static file serving for all Ferro apps (no more 404s on /assets/* in production)
- Differentiated cache headers: 1-year immutable for Vite hashed output, must-revalidate for root files
- Security protections: dotfile rejection, directory traversal via canonicalization, null byte rejection
- Binary-safe file serving using hyper::Response directly (bypasses HttpResponse String body)
- 12 tests covering path validation, MIME detection, cache headers, binary integrity, and edge cases

## Task Commits

Each task was committed atomically:

1. **Task 1: Create static_files module with try_serve_static_file()** - `345818b` (feat)
2. **Task 2: Integrate static file serving in server.rs and add tests** - `c93bbc4` (feat)
3. **Task 3: Add static files documentation** - `2c4c2f5` (docs)

## Files Created/Modified
- `framework/src/static_files.rs` - Core static file serving logic with path validation, MIME detection, cache headers, and security
- `framework/src/server.rs` - Integration point: static file check before fallback handler in handle_request()
- `framework/src/lib.rs` - Module declaration for static_files (pub(crate))
- `framework/Cargo.toml` - Added mime_guess dependency, tempfile dev-dependency
- `docs/src/features/static-files.md` - Documentation covering behavior, caching, security, dev vs prod
- `docs/src/SUMMARY.md` - Added Static Files entry to Features section

## Decisions Made
- Used direct hyper::Response<Full<Bytes>> construction instead of HttpResponse to avoid binary corruption (HttpResponse stores body as String)
- Added testable try_serve_from_dir() helper that accepts a base path parameter; try_serve_static_file() delegates to it with Path::new("public")
- Static files are checked before the fallback handler (not after) to prevent SPA catch-alls from intercepting asset requests
- Only GET/HEAD methods trigger filesystem checks; POST/PUT/DELETE skip entirely

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added tempfile dev-dependency**
- **Found during:** Task 2 (test implementation)
- **Issue:** Tests use TempDir for filesystem integration tests, but tempfile wasn't in framework's dev-dependencies
- **Fix:** Added `tempfile = "3"` to `[dev-dependencies]` in framework/Cargo.toml
- **Files modified:** framework/Cargo.toml
- **Verification:** Tests compile and pass
- **Committed in:** c93bbc4 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary for test compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Static file serving complete and tested
- All Ferro apps with Vite frontends will serve assets correctly in production
- No further phases planned

---
*Phase: 69-static-file-serving*
*Completed: 2026-02-25*
