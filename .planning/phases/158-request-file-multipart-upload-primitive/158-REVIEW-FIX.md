---
phase: 158-request-file-multipart-upload-primitive
fixed_at: 2026-05-15T00:00:00Z
review_path: .planning/phases/158-request-file-multipart-upload-primitive/158-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 158: Code Review Fix Report

**Fixed at:** 2026-05-15T00:00:00Z
**Source review:** .planning/phases/158-request-file-multipart-upload-primitive/158-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### CR-01: Client upload errors mapped to HTTP 500

**Files modified:** `framework/src/http/multipart.rs`
**Commit:** 6895bce2
**Applied fix:**
- `parse_multipart_body`: wrong/missing Content-Type boundary now returns `FrameworkError::domain(..., 400)` instead of `internal`.
- `parse_multipart_body`: too-many-fields guard now returns `FrameworkError::domain(..., 400)`.
- `parse_multipart_body`: `field.bytes().await` error now matches on `multer::Error::FieldSizeExceeded` and `multer::Error::StreamSizeExceeded` to return `FrameworkError::domain("Upload field exceeds maximum size", 413)`; all other read errors fall through to `FrameworkError::internal`.
- `validate_mime`: rejection now returns `FrameworkError::domain(..., 422)`.
- `validate_size`: rejection now returns `FrameworkError::domain(..., 422)`.

Note: `multer::Error` does not have an `is_size_limit_exceeded()` method (multer 3.1.0). The fix matches on the `FieldSizeExceeded` and `StreamSizeExceeded` enum variants directly, which is the correct approach for that version.

### WR-01: `max_file_bytes()` does not guard against zero

**Files modified:** `framework/src/http/multipart.rs`
**Commit:** 6895bce2
**Applied fix:** Added `mb.max(1)` clamp before multiplying by `1024 * 1024`, so `UPLOAD_MAX_SIZE_MB=0` produces a 1 MiB limit rather than a silent 0-byte limit. Added a doc comment explaining the MiB unit and the clamp rationale.

### WR-02: `validate_mime` doc comment missing security warning

**Files modified:** `framework/src/http/multipart.rs`
**Commit:** 6895bce2
**Applied fix:** Added a `**Security note:**` paragraph to the `validate_mime` doc comment explaining that the check is based solely on the client-supplied `Content-Type` header and can be forged, and that magic-byte verification (e.g. `infer` crate) is required for security-sensitive contexts.

### WR-03: Text field values decoded with lossy UTF-8

**Files modified:** `framework/src/http/multipart.rs`
**Commit:** 6895bce2
**Applied fix:** Replaced `String::from_utf8_lossy(&bytes).into_owned()` with `String::from_utf8(bytes.to_vec()).map_err(|_| FrameworkError::internal("Multipart text field contains invalid UTF-8"))?` in both the production `parse_multipart_body` function and the test-module `parse_for_test` helper.

---

_Fixed: 2026-05-15T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
