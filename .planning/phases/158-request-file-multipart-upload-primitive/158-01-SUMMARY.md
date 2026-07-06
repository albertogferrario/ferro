---
phase: 158-request-file-multipart-upload-primitive
plan: "01"
subsystem: framework/http
tags:
  - http
  - multipart
  - upload
  - storage

dependency_graph:
  requires:
    - ferro-storage (Disk, PutOptions — already a framework dep)
    - multer = "3" (new dep)
    - http-body-util (BodyStream — already present)
    - futures-util (StreamExt — already present)
  provides:
    - MultipartForm (parsed multipart body with file + text accessors)
    - UploadedFile (buffered file with size/extension/is_image/store methods)
    - validate_mime (MIME allow-list check)
    - validate_size (byte-size cap check)
    - parse_multipart_body (pub(crate) — called by Request::multipart() in plan 02)
    - max_file_bytes (pub(crate) — env-configured limit, plan 02 caller)
    - max_fields (pub(crate) — env-configured limit, plan 02 caller)
  affects:
    - framework/src/http/mod.rs (mod + pub use)
    - framework/src/lib.rs (crate-root re-export)

tech_stack:
  added:
    - multer = "3" (async multipart/form-data parser, hyper 1.x compatible)
  patterns:
    - Body-consuming method pattern (mirrors req.form() / req.json())
    - BodyStream::new(body).filter_map() bridge for hyper 1.x Incoming → multer stream
    - pub(crate) helpers called by sibling module (plan 02 wires Request::multipart())

key_files:
  created:
    - framework/src/http/multipart.rs (234 lines)
  modified:
    - framework/Cargo.toml (multer = "3" added)
    - framework/src/http/mod.rs (mod multipart + pub use)
    - framework/src/lib.rs (four symbols added to pub use http block)

decisions:
  - "D-11 resolved: UploadedFile::store takes &Disk (not &Storage or &dyn DiskDriver) — caller selects disk via storage.disk() before calling store"
  - "D-01 honored: multer crate used, no custom multipart parser"
  - "D-04 honored: MultipartForm exposes file(), files(), field(), fields()"
  - "D-07/D-08 honored: UploadedFile fields and methods match spec exactly"
  - "D-09 honored: store() calls disk.put_with_options with PutOptions::new().content_type()"
  - "D-12 honored: SizeLimit::per_field(max_file_bytes) in multer Constraints; UPLOAD_MAX_SIZE_MB env var"
  - "D-13 honored: manual field_count counter in iteration loop; UPLOAD_MAX_FIELDS env var"
  - "D-14 honored: validate_mime and validate_size free functions implemented"
  - "D-16 honored: module at framework/src/http/multipart.rs, re-exported from mod.rs and lib.rs"
  - "D-17 honored: multer = 3 added to framework/Cargo.toml only, no new workspace crate"
  - "D-18 honored: boundary-missing error uses exact message Content-Type is not multipart/form-data or missing boundary"

metrics:
  duration: "311 seconds (~5 minutes)"
  completed: "2026-05-15"
  tasks_completed: 3
  files_modified: 4
---

# Phase 158 Plan 01: Multipart Upload Primitive — Core Module Summary

**One-liner:** `multer`-backed multipart parser producing `UploadedFile` + `MultipartForm` with `store(&disk, path)` as single-line persistence via `ferro-storage`.

## What Was Built

A self-contained `framework/src/http/multipart.rs` module (234 lines) providing:

- `UploadedFile` — buffered file with `size()`, `extension()`, `is_image()`, and `async store(&Disk, &str)` for one-line persistence to local disk or S3.
- `MultipartForm` — parsed body with `file()` (first match), `files()` (all matches), `field()` (text field), `fields()` (all text fields).
- `parse_multipart_body` (pub(crate)) — bridges `hyper::body::Incoming` to multer via `BodyStream::new(body).filter_map(...)`, enforces per-field size limit and field count limit.
- `validate_mime` / `validate_size` — free functions for handler-level MIME and size validation.
- `max_file_bytes` / `max_fields` (pub(crate)) — env-configured defaults (10 MB, 100 fields).

All four public symbols (`MultipartForm`, `UploadedFile`, `validate_mime`, `validate_size`) are wired through `http/mod.rs` and re-exported at the `ferro-rs` crate root so handlers can write `use ferro::{MultipartForm, UploadedFile, validate_mime, validate_size}`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] clippy `manual_contains` lint in `validate_mime`**
- **Found during:** Task 3 (clippy -D warnings run)
- **Issue:** `allowed.iter().any(|a| *a == ct)` triggers `clippy::manual_contains` — clippy prefers `allowed.contains(&ct)` as more efficient.
- **Fix:** Changed to `allowed.contains(&ct)`.
- **Files modified:** `framework/src/http/multipart.rs`
- **Commit:** 5ae7fb89

**2. [Rule 2 - Missing critical functionality] `#[allow(dead_code)]` on `pub(crate)` helpers**
- **Found during:** Task 3 (clippy -D warnings run)
- **Issue:** `parse_multipart_body`, `max_file_bytes`, and `max_fields` are `pub(crate)` but have no callers until plan 02 wires `Request::multipart()`. Clippy -D warnings treats unused pub(crate) functions as errors.
- **Fix:** Added `#[allow(dead_code)]` with comments noting these are called by plan 02. The attributes will be removed when plan 02 adds the callers.
- **Files modified:** `framework/src/http/multipart.rs`
- **Commit:** 5ae7fb89

**3. [Rule 1 - Style] rustfmt reformatted three files**
- **Found during:** Task 3 (cargo fmt --all -- --check)
- **Issue:** Manual line-break choices in multipart.rs, mod.rs pub use ordering, and lib.rs wrap column differed from rustfmt output.
- **Fix:** `cargo fmt --all` applied; no logic changes.
- **Commit:** 5ae7fb89

**Note on Task 1 vs Task 2 split:** The plan specified creating types in Task 1 and appending the parser/validators in Task 2. Since the complete file was written in a single `Write` operation during Task 1 (the content was fully specified in the plan), both tasks' content landed in the first commit (03b878ea). No functional deviation — all specified content is present and correct.

## Known Stubs

None. All functions are fully implemented. `parse_multipart_body`, `max_file_bytes`, and `max_fields` are stub-free; they simply lack callers until plan 02.

## Threat Flags

No new trust boundaries beyond what the plan's threat model covers. All six threats (T-158-01 through T-158-06) are addressed in the implementation:

| Threat | Mitigation Implemented |
|--------|----------------------|
| T-158-01 DoS oversized file | `SizeLimit::new().per_field(max_file_bytes)` in multer Constraints |
| T-158-02 DoS field count | `field_count > max_fields` counter in iteration loop |
| T-158-03 MIME spoofing | `validate_mime` free function (opt-in allow-list) |
| T-158-04 Path traversal | Documented in `# Security` section of `store()` rustdoc; caller responsibility |
| T-158-05 Error info disclosure | `FrameworkError::internal` wraps multer errors; generic 500 to HTTP clients |
| T-158-06 Missing boundary DoS | `multer::parse_boundary` returns Err on missing/wrong Content-Type; short-circuits before body read |

## Self-Check: PASSED

- FOUND: `framework/src/http/multipart.rs` (234 lines)
- FOUND: `multer = "3"` in `framework/Cargo.toml`
- FOUND: `mod multipart` and `pub use multipart::` in `framework/src/http/mod.rs`
- FOUND: `MultipartForm`, `UploadedFile`, `validate_mime`, `validate_size` in `framework/src/lib.rs`
- FOUND: commit 03b878ea (Task 1+2 content)
- FOUND: commit 5ae7fb89 (Task 3 wiring + fixes)
- `cargo build -p ferro-rs`: exit 0, zero errors
- `cargo clippy -p ferro-rs --all-targets -- -D warnings`: exit 0
- `cargo fmt --all -- --check`: exit 0
- `cargo doc --no-deps -p ferro-rs`: exit 0
