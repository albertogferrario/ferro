---
phase: 158-request-file-multipart-upload-primitive
reviewed: 2026-05-15T00:00:00Z
depth: standard
files_reviewed: 5
files_reviewed_list:
  - framework/Cargo.toml
  - framework/src/http/mod.rs
  - framework/src/http/multipart.rs
  - framework/src/http/request.rs
  - framework/src/lib.rs
findings:
  critical: 1
  warning: 3
  info: 2
  total: 6
status: issues_found
---

# Phase 158: Code Review Report

**Reviewed:** 2026-05-15T00:00:00Z
**Depth:** standard
**Files Reviewed:** 5
**Status:** issues_found

## Summary

This phase introduces `MultipartForm`, `UploadedFile`, and the `Request::multipart()` / `Request::file()` entry points for handling `multipart/form-data` uploads. The implementation is clean and well-tested. One critical issue exists: error responses from all three validation helpers (`parse_multipart_body`, `validate_mime`, `validate_size`) are classified as `FrameworkError::Internal` (HTTP 500) instead of a client-error status code, which means a browser sending a disallowed file type or an oversized upload receives a 500 instead of a 400/422. Three warnings cover a logic flaw in `max_file_bytes()`, a MIME spoofing gap in `validate_mime`, and lossy UTF-8 decoding. Two info items address test duplication and a missing doc note.

## Critical Issues

### CR-01: Client upload errors mapped to HTTP 500

**File:** `framework/src/http/multipart.rs:127-198`

**Issue:** `FrameworkError::internal(...)` maps to HTTP 500 (`status_code()` returns 500 for `FrameworkError::Internal`). Every user-visible rejection — wrong Content-Type, file too large, too many fields, disallowed MIME type — uses `internal()` and therefore returns 500 to the client. These are client faults and should be 400 or 422.

**Fix:** Replace `FrameworkError::internal(...)` with a client-error variant for the four cases where the caller is at fault. Using the existing `FrameworkError::domain()` constructor (which takes a custom HTTP status code) is the least-invasive fix:

```rust
// In parse_multipart_body — wrong/missing Content-Type
FrameworkError::domain("Content-Type is not multipart/form-data or missing boundary", 400)

// Too many fields
FrameworkError::domain("Too many multipart fields", 400)

// In validate_mime
FrameworkError::domain(
    format!("File type '{ct}' is not allowed; accepted: {}", allowed.join(", ")),
    422,
)

// In validate_size
FrameworkError::domain(
    format!("File too large: {} bytes (max {max_bytes})", file.size()),
    422,
)
```

The multer size-limit error (propagated from `field.bytes().await`) is trickier: multer returns its own error type; wrap it as 413 or 422 rather than 500 by inspecting `e.is_size_limit_exceeded()` before the generic fallback:

```rust
let bytes = field.bytes().await.map_err(|e| {
    if e.is_size_limit_exceeded() {
        FrameworkError::domain("Upload field exceeds maximum size", 413)
    } else {
        FrameworkError::internal(format!("Field read error: {e}"))
    }
})?;
```

## Warnings

### WR-01: `max_file_bytes()` parses the env var as MiB but the function name says "bytes"

**File:** `framework/src/http/multipart.rs:213-220`

**Issue:** `UPLOAD_MAX_SIZE_MB` is intended to hold a MiB value (the doc comment says so), and the function multiplies by `1024 * 1024`. However, if the operator writes `UPLOAD_MAX_SIZE_MB=10` the result is 10 MiB, which is correct. The dangerous edge: if an operator reads the function name `max_file_bytes` and sets the var thinking it already is in bytes (e.g. `UPLOAD_MAX_SIZE_MB=10485760`), they will get a 10 TiB per-field limit. The function name is misleading — it returns bytes, but the env var is in MiB. Also, a zero value (`UPLOAD_MAX_SIZE_MB=0`) silently produces a 0-byte limit, which will reject every upload without a clear error.

**Fix:** Guard against zero and document the unit in the function:

```rust
pub(crate) fn max_file_bytes() -> u64 {
    let mb = std::env::var("UPLOAD_MAX_SIZE_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10);
    mb.max(1) * 1024 * 1024   // clamp to at least 1 MiB so 0 is not silent
}
```

Consider renaming to `max_file_size_bytes()` to make the return unit obvious.

### WR-02: `validate_mime` trusts the client-supplied MIME type without magic-byte verification

**File:** `framework/src/http/multipart.rs:188-198`

**Issue:** `content_type` is taken verbatim from the `Content-Disposition`/`Content-Type` header within the multipart part, which the client controls entirely. A user can upload a `.exe` with `Content-Type: image/png` and pass `validate_mime(file, &["image/png"])`. The doc comment on `store()` already acknowledges path-traversal responsibility falls on the caller; this is the analogous gap for content validation.

**Fix:** The function cannot perform magic-byte detection itself without adding a dependency (e.g. `infer`). The minimum required fix is to strengthen the doc comment so callers know the check is declaration-only and not a security gate:

```rust
/// Reject the file if its declared MIME type is not in `allowed`.
///
/// **Security note:** this check is based solely on the client-supplied
/// `Content-Type` header inside the multipart part, which can be forged.
/// For security-sensitive contexts, validate the actual file magic bytes
/// (e.g. with the `infer` crate) in addition to this check.
pub fn validate_mime(file: &UploadedFile, allowed: &[&str]) -> Result<(), FrameworkError> {
```

### WR-03: Text field values decoded with lossy UTF-8

**File:** `framework/src/http/multipart.rs:173`

**Issue:** `String::from_utf8_lossy(&bytes).into_owned()` silently replaces invalid UTF-8 sequences with the replacement character `U+FFFD`. If a form field contains non-UTF-8 bytes (e.g. from a Latin-1 encoded legacy browser form), the replacement character is inserted without any error. The downstream code sees `"hell\u{FFFD}o"` rather than an error and stores corrupted data.

**Fix:** Use strict decoding so the handler can decide how to handle the failure:

```rust
let value = String::from_utf8(bytes.to_vec())
    .map_err(|_| FrameworkError::internal("Multipart text field contains invalid UTF-8"))?;
text_fields.insert(field_name, value);
```

The same pattern applies in the `parse_for_test` helper in the test module (line 325).

## Info

### IN-01: `parse_for_test` duplicates the production parsing loop verbatim

**File:** `framework/src/http/multipart.rs:270-333`

**Issue:** The test helper `parse_for_test` reproduces the full field-iteration logic from `parse_multipart_body` word for word. When the production logic changes (e.g. adding a new field category), the test mirror must be updated independently. This is a maintenance hazard — a divergence between the two would silently invalidate the tests.

**Fix:** Refactor `parse_multipart_body` to accept a generic `futures::Stream` body rather than `hyper::body::Incoming` specifically, or extract the field-iteration logic into a shared private function. The test then calls the shared function with a `Full<Bytes>` stream, eliminating duplication.

### IN-02: `Request::file` doc example references `FrameworkError` without a use path

**File:** `framework/src/http/request.rs:429-434`

**Issue:** The `# Example` block in `Request::file` uses `FrameworkError::internal(...)` in the docstring code but does not show the import. While doc examples in this codebase use `ignore`, misleading imports confuse readers writing real handlers.

**Fix:** Either add `// use ferro_rs::FrameworkError;` to the example or replace the inline reference with a comment explaining how callers surface the missing-file error.

---

_Reviewed: 2026-05-15T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
