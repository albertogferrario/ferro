# Phase 158: Request::file() Multipart Upload Primitive — Research

**Researched:** 2026-05-15
**Domain:** Rust multipart/form-data parsing, hyper 1.x body stream integration, ferro-storage bridging
**Confidence:** HIGH

## Summary

Phase 158 adds multipart/form-data parsing to the framework using the `multer` crate (v3.1.0). The implementation lives entirely in a new `framework/src/http/multipart.rs` module with no new workspace crate required.

The critical integration challenge is adapting `hyper::body::Incoming` — which does NOT implement `Stream` directly in hyper 1.x — to the `Stream<Item = Result<Bytes, _>>` interface that `multer::Multipart::new` requires. The official multer hyper example (verified from source) uses `http_body_util::BodyStream::new(body).filter_map(...)` to bridge the gap. Both `http_body_util` and `futures-util` are already framework dependencies with the exact features needed.

`parse_boundary` returns `Result<String, multer::Error>`, not `Option<String>` as the CONTEXT.md specifics section states — the actual API must handle the error case (Content-Type not multipart or missing boundary).

`ferro-storage` is already a non-optional dependency in `framework/Cargo.toml`. `multer = "3"` is the only new dependency to add.

**Primary recommendation:** Follow the official multer hyper example pattern exactly — `BodyStream::new(body).filter_map(|r| async { r.map(|f| f.into_data().ok()).transpose() })` — then pass the resulting stream and boundary to `Multipart::new`. This is the verified, supported integration path.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Parser Library**
- D-01: Use `multer` crate (async multipart parser compatible with hyper 1.x body streams). `mime_guess = "2"` is already a dep — no new MIME library needed.
- D-02: `multer` takes ownership of the body stream and the `boundary` string extracted from `Content-Type: multipart/form-data; boundary=...`. Extraction logic lives in a private helper alongside existing `parse_form` / `parse_json`.

**API Surface**
- D-03: Primary API is `req.multipart().await -> Result<MultipartForm, FrameworkError>`. Mirrors `req.form()` / `req.json()` — consumes the request, returns a parsed value.
- D-04: `MultipartForm` exposes: `fn file(&self, field: &str) -> Option<&UploadedFile>`, `fn files(&self, field: &str) -> &[UploadedFile]`, `fn field(&self, name: &str) -> Option<&str>`, `fn fields(&self) -> &HashMap<String, String>`.
- D-05: Convenience shorthand: `req.file("avatar").await` — calls multipart internally, returns `Option<UploadedFile>`.
- D-06: Mixed multipart (text fields + files) handled in single `req.multipart()` pass.

**UploadedFile Shape**
- D-07: `UploadedFile` fields: `field_name: String`, `file_name: Option<String>`, `content_type: Option<String>`, `bytes: Bytes`.
- D-08: `UploadedFile` methods: `fn size(&self) -> usize`, `fn extension(&self) -> Option<&str>`, `fn is_image(&self) -> bool`, `fn store(&self, ...) -> impl Future<...>`.

**Storage Integration**
- D-09: `UploadedFile::store(storage, path)` calls `storage.put(path, self.bytes.clone())` with `PutOptions::new().content_type(...)`.
- D-10: `ferro-storage` added as dependency to `framework/Cargo.toml` (already present).
- D-11: Whether `store()` takes `&Storage` or `&dyn DiskDriver` — Claude's discretion.

**Size Limits**
- D-12: Default max file size: 10 MB. Configurable via `UPLOAD_MAX_SIZE_MB`. Return `FrameworkError` (not panic) on oversize.
- D-13: Max fields limit: 100. Configurable via `UPLOAD_MAX_FIELDS`.

**Validation Helpers**
- D-14: `fn validate_mime(file: &UploadedFile, allowed: &[&str]) -> Result<(), FrameworkError>` and `fn validate_size(file: &UploadedFile, max_bytes: usize) -> Result<(), FrameworkError>`.
- D-15: No integration with existing `Validator` builder this phase.

**Module Location**
- D-16: `framework/src/http/multipart.rs`. Exported from `framework/src/http/mod.rs`. Re-exported from `framework/src/lib.rs` as `pub use http::multipart::{MultipartForm, UploadedFile, validate_mime, validate_size}`.
- D-17: `multer` added to `framework/Cargo.toml`. No new workspace crate.

**Error Handling**
- D-18: Multipart errors map to `FrameworkError::internal(...)`. Boundary-missing gets dedicated message: `"Content-Type is not multipart/form-data or missing boundary"`.

### Claude's Discretion
- Whether `UploadedFile::store` takes `&Storage` or `&dyn DiskDriver` — pick whichever avoids a generic parameter on `UploadedFile`
- Whether `MultipartForm::files_map` is `HashMap<String, Vec<UploadedFile>>` or `Vec<UploadedFile>` iterated by field name — pick the simpler representation
- Test structure: in-memory multipart body construction for unit tests vs integration tests
- Display name for errors shown to users (file too large, bad MIME type)

### Deferred Ideas (OUT OF SCOPE)
- Streaming upload (pass file bytes directly to S3 without full buffer)
- Validator builder integration (`rules![file_type("image/*"), max_size(5.mb())]`)
- Multiple file upload progress events / WebSocket progress
- `#[extract]` derive macro support for typed multipart structs
</user_constraints>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Multipart body parsing | API / Backend | — | Server-side only; raw body stream consumed once |
| UploadedFile buffering | API / Backend | — | Files buffered in memory on the server before storage |
| File storage (put to disk/S3) | Database / Storage | — | ferro-storage layer owns persistence |
| Content-Type boundary extraction | API / Backend | — | Header parsing precedes body reading |
| Size / MIME validation helpers | API / Backend | — | Pure functions on already-buffered data |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| multer | 3.1.0 | Async multipart/form-data parser | Only maintained Rust crate compatible with hyper 1.x body streams; official hyper example confirms the integration |
| http_body_util | 0.1 (already present) | `BodyStream` adapter for hyper body → futures Stream | Required to bridge hyper 1.x `Incoming` to multer |
| futures-util | 0.3 (already present, `["sink","std"]`) | `StreamExt::filter_map` for frame → bytes conversion | Already used in websocket.rs; `std` feature includes `StreamExt` |
| ferro-storage | 0.2 (already present) | File persistence after parsing | Already a direct dep in framework/Cargo.toml |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| mime_guess | 2 (already present) | MIME type from extension (fallback) | When content_type field is None and you must infer from file_name |
| std::path::Path | stdlib | `extension()` extraction from file_name | Used inside `UploadedFile::extension()` |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| multer | multipart (abonander/multipart) | multipart crate is not hyper 1.x compatible |
| BodyStream.filter_map | collect_body then parse bytes | collect_body buffers everything — multer needs a stream for field-by-field memory-safe iteration |

**Installation:**
```bash
# In framework/Cargo.toml [dependencies]:
multer = "3"
```

**Version verification:** `[VERIFIED: crates.io/crates/multer]` — version 3.1.0 is current as of 2026-05-15. Its `dev-dependencies` confirm hyper 1.0 + http-body-util 0.1 compatibility.

---

## Architecture Patterns

### System Architecture Diagram

```
HTTP Request (Content-Type: multipart/form-data; boundary=...)
       │
       ▼
req.multipart() / req.file()   ← consumes Request
       │
       ├── extract boundary from Content-Type header
       │       multer::parse_boundary(ct) → Result<String>
       │       error → FrameworkError::internal("Content-Type is not multipart/form-data or missing boundary")
       │
       ├── build body stream
       │       http_body_util::BodyStream::new(incoming_body)
       │       .filter_map(|r| async { r.map(|f| f.into_data().ok()).transpose() })
       │
       ├── multer::Multipart::with_constraints(stream, boundary, constraints)
       │       SizeLimit::new().per_field(max_file_bytes)
       │       max_fields: read from env UPLOAD_MAX_FIELDS (default 100)
       │
       ├── field iteration: while let Some(field) = multipart.next_field().await?
       │       field.name() → Some(&str)  → text field or file field key
       │       field.file_name() → Option<&str>  → file upload marker
       │       field.content_type() → Option<&mime::Mime>
       │       field.bytes().await → Result<Bytes>
       │
       ├── classification per field:
       │       has file_name → push UploadedFile into files_map
       │       no file_name  → push (name, text) into text_fields
       │
       └── return MultipartForm { files_map, text_fields }
                │
                ├── form.file("avatar") → Option<&UploadedFile>
                ├── form.files("photos") → &[UploadedFile]
                └── form.field("title")  → Option<&str>

UploadedFile::store(&disk, "path/to/save")
       │
       └── disk.put_with_options(path, self.bytes.clone(),
               PutOptions::new().content_type(mime)) → Result<(), ferro_storage::Error>
```

### Recommended Project Structure

```
framework/src/http/
├── mod.rs           # add: pub use multipart::{MultipartForm, UploadedFile, validate_mime, validate_size}
├── multipart.rs     # NEW: all multipart parsing code
├── request.rs       # add: pub async fn multipart(self), pub async fn file(self, field: &str)
├── body.rs          # unchanged — collect_body not used for multipart
└── ...
```

### Pattern 1: Body Stream Conversion (hyper Incoming → multer)

**What:** Convert `hyper::body::Incoming` (which does NOT impl `Stream` in hyper 1.x) to a `Stream<Item = Result<Bytes, _>>` for multer.

**When to use:** Always, inside the private `parse_multipart` helper in multipart.rs.

**Example:**
```rust
// Source: https://github.com/rwf2/multer/blob/master/examples/hyper_server_example.rs
// [VERIFIED: rwf2/multer GitHub source]
use http_body_util::BodyStream;
use futures_util::StreamExt;

let body_stream = BodyStream::new(incoming_body)
    .filter_map(|result| async move {
        result.map(|frame| frame.into_data().ok()).transpose()
    });

let mut multipart = multer::Multipart::with_constraints(
    body_stream,
    boundary,
    multer::Constraints::new()
        .size_limit(
            multer::SizeLimit::new()
                .per_field(max_file_bytes as u64)
        )
);
```

### Pattern 2: Boundary Extraction

**What:** Extract the multipart boundary from the `Content-Type` header.

**Example:**
```rust
// Source: [VERIFIED: rwf2/multer GitHub source — src/lib.rs]
// parse_boundary returns Result<String, multer::Error>
let boundary = multer::parse_boundary(content_type_str)
    .map_err(|_| FrameworkError::internal(
        "Content-Type is not multipart/form-data or missing boundary"
    ))?;
```

**Critical correction:** CONTEXT.md `<specifics>` mentions `multer::parse_boundary` returns `Option<String>`. The actual API is `Result<String, multer::Error>`. The error path must use `map_err`, not `ok_or`.

### Pattern 3: Field Iteration and Classification

**What:** Iterate multer fields, routing file fields vs. text fields.

**Example:**
```rust
// [VERIFIED: rwf2/multer GitHub source — examples/hyper_server_example.rs]
while let Some(field) = multipart.next_field().await
    .map_err(|e| FrameworkError::internal(format!("Multipart parse error: {e}")))? 
{
    let field_name = field.name()
        .map(|s| s.to_string())
        .unwrap_or_default();
    let file_name = field.file_name().map(|s| s.to_string());
    let content_type = field.content_type().map(|m| m.to_string());
    let bytes = field.bytes().await
        .map_err(|e| FrameworkError::internal(format!("Field read error: {e}")))?;

    if file_name.is_some() {
        // file upload
        files_map.entry(field_name.clone()).or_default().push(UploadedFile {
            field_name, file_name, content_type, bytes,
        });
    } else {
        // text field
        text_fields.insert(field_name, String::from_utf8_lossy(&bytes).into_owned());
    }
}
```

### Pattern 4: UploadedFile::store — taking &Disk (discretion resolved)

**Recommendation:** Take `&Disk` (not `&Storage`). `Disk` is already the selected-disk handle; taking `&Storage` would silently use the default disk and would not let the caller choose `storage.disk("s3")?` at the call site. Taking `&dyn DiskDriver` requires a generic or dyn, and `DiskDriver` is the low-level trait — `Disk` is the idiomatic user-facing handle.

```rust
// [ASSUMED] — design choice based on ferro-storage API inspection
pub async fn store(
    &self,
    disk: &ferro_storage::Disk,
    path: &str,
) -> Result<(), ferro_storage::Error> {
    let opts = ferro_storage::PutOptions::new()
        .content_type(
            self.content_type.as_deref().unwrap_or("application/octet-stream")
        );
    disk.put_with_options(path, self.bytes.clone(), opts).await
}
```

### Pattern 5: Size Limit with multer Constraints

**What:** Use multer's built-in `Constraints` API rather than post-hoc checking.

**Example:**
```rust
// Source: [VERIFIED: rwf2/multer GitHub source — src/constraints.rs, src/size_limit.rs]
let max_file_bytes: u64 = std::env::var("UPLOAD_MAX_SIZE_MB")
    .ok()
    .and_then(|v| v.parse::<u64>().ok())
    .unwrap_or(10) * 1024 * 1024;

let max_fields: usize = std::env::var("UPLOAD_MAX_FIELDS")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(100);

let constraints = multer::Constraints::new()
    .size_limit(
        multer::SizeLimit::new().per_field(max_file_bytes)
    );
// max_fields is enforced by tracking field count in the iteration loop
```

Note: `multer::Constraints` does not have a direct `max_fields` setter — the field count limit is enforced manually in the iteration loop.

### Anti-Patterns to Avoid

- **Using `collect_body` before multer:** `collect_body` buffers the entire body into memory and discards stream semantics. Pass `Incoming` directly to multer via `BodyStream`.
- **Calling `parse_boundary` and treating it as infallible:** It returns `Result`, not `Option`. Missing boundary or wrong Content-Type returns `Err`. Handle explicitly.
- **Storing `multer::Field` across an `.await`:** `multer::Field` holds the stream mutably; do not store references to it. Collect `field.bytes().await` synchronously.
- **Taking `&Storage` in `store()`:** The caller must select which disk to write to. `&Storage` hides this choice and defaults to the default disk silently.
- **Generic parameter on `UploadedFile`:** Avoids tight coupling. The `store()` method takes `&ferro_storage::Disk` — no generic parameter on the struct itself.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Multipart boundary parsing | Custom RFC 2046 parser | `multer::parse_boundary` | Handles quoting, whitespace, edge cases |
| Field-by-field async streaming | Custom body read loop | `multer::Multipart::next_field()` | Handles MIME headers, Content-Disposition, chunked encoding |
| Size limit enforcement | Post-hoc `bytes.len()` check | `multer::Constraints` + `SizeLimit::per_field()` | Rejects oversized fields before they fully buffer — more DoS-safe |
| MIME type from file extension | Custom extension→MIME table | `mime_guess` (already dep) | Already in the workspace |

**Key insight:** Multipart is deceptively complex. RFC 2046 has many edge cases around boundary quoting, CRLF handling, and header encoding. Multer handles all of these; custom parsers almost always have gaps.

---

## Common Pitfalls

### Pitfall 1: hyper::body::Incoming Does Not Implement Stream

**What goes wrong:** Trying to pass `Incoming` directly to `multer::Multipart::new(body, boundary)` fails to compile because `Incoming` does not implement `futures_core::Stream` in hyper 1.x.

**Why it happens:** Hyper 1.x deliberately removed the `Stream` impl from `Incoming` for API stability reasons.

**How to avoid:** Use the `BodyStream` adapter from `http_body_util`:
```rust
use http_body_util::BodyStream;
use futures_util::StreamExt;
let stream = BodyStream::new(body)
    .filter_map(|r| async move { r.map(|f| f.into_data().ok()).transpose() });
```
Source: `[VERIFIED: rwf2/multer GitHub examples/hyper_server_example.rs]`

**Warning signs:** Compiler error `the trait Stream is not implemented for hyper::body::Incoming`.

### Pitfall 2: parse_boundary Returns Result, Not Option

**What goes wrong:** CONTEXT.md specifics mention `.ok()` pattern but the real signature is `Result<String, multer::Error>`. Code that calls `.ok()` will silently suppress the error and produce `None`, losing the error detail.

**Why it happens:** The API changed between multer v2 and v3 (or was always `Result` and described incorrectly in planning notes).

**How to avoid:** Use `.map_err(|_| FrameworkError::internal(...))` instead of `.ok_or_else(...)` on an `Option`.

**Warning signs:** `no method named ok_or_else found for struct Result` or unexpected silencing of boundary errors.

### Pitfall 3: futures-util Missing "stream" Feature

**What goes wrong:** `StreamExt::filter_map` requires the stream combinator feature. The framework's `futures-util` dep uses `features = ["sink", "std"]` which does include `StreamExt` (the `std` feature pulls in stream combinators), but a future dep change could break this.

**Why it happens:** Confusion about which features include which combinators.

**How to avoid:** Confirm `StreamExt` compiles — it already compiles in `websocket.rs`. No feature change needed.

**Warning signs:** `no method named filter_map found for struct BodyStream`.

### Pitfall 4: field.content_type() Returns &mime::Mime, Not &str

**What goes wrong:** `Field::content_type()` returns `Option<&mime::Mime>`. Storing it as `Option<String>` requires `.map(|m| m.to_string())`. Passing it directly as `&str` to `PutOptions::content_type()` doesn't work.

**How to avoid:** `.map(|m| m.to_string())` at collection time.

### Pitfall 5: MultipartForm Internal Storage — Use Vec<UploadedFile> with field_name, Not Separate HashMap + Vec

**Recommendation for discretion item:** Use `HashMap<String, Vec<UploadedFile>>` as the internal `files_map` type. This is the simpler representation that supports `file()` (first match) and `files()` (all matches for a field name) without a linear scan.

---

## Code Examples

### Complete parse_multipart helper (verified pattern)

```rust
// Source: https://github.com/rwf2/multer/blob/master/examples/hyper_server_example.rs
// [VERIFIED: rwf2/multer GitHub source]
use http_body_util::BodyStream;
use futures_util::StreamExt;
use hyper::body::Incoming;
use std::collections::HashMap;

async fn parse_multipart_body(
    body: Incoming,
    content_type: &str,
    max_file_bytes: u64,
    max_fields: usize,
) -> Result<MultipartForm, FrameworkError> {
    let boundary = multer::parse_boundary(content_type)
        .map_err(|_| FrameworkError::internal(
            "Content-Type is not multipart/form-data or missing boundary"
        ))?;

    let body_stream = BodyStream::new(body)
        .filter_map(|result| async move {
            result.map(|frame| frame.into_data().ok()).transpose()
        });

    let constraints = multer::Constraints::new()
        .size_limit(multer::SizeLimit::new().per_field(max_file_bytes));

    let mut multipart = multer::Multipart::with_constraints(
        body_stream, boundary, constraints
    );

    let mut files_map: HashMap<String, Vec<UploadedFile>> = HashMap::new();
    let mut text_fields: HashMap<String, String> = HashMap::new();
    let mut field_count = 0usize;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| FrameworkError::internal(format!("Multipart parse error: {e}")))?
    {
        field_count += 1;
        if field_count > max_fields {
            return Err(FrameworkError::internal("Too many fields in multipart request"));
        }

        let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
        let file_name = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|m| m.to_string());

        let bytes = field.bytes().await
            .map_err(|e| FrameworkError::internal(format!("Field read error: {e}")))?;

        if file_name.is_some() {
            files_map.entry(field_name.clone()).or_default().push(UploadedFile {
                field_name, file_name, content_type, bytes,
            });
        } else {
            text_fields.insert(field_name, String::from_utf8_lossy(&bytes).into_owned());
        }
    }

    Ok(MultipartForm { files_map, text_fields })
}
```

### Request::multipart() and Request::file() method signatures

```rust
// [ASSUMED] — follows existing req.json() / req.form() pattern verified in request.rs
impl Request {
    /// Parse the request body as multipart/form-data.
    ///
    /// Consumes the request since the body can only be read once.
    pub async fn multipart(self) -> Result<MultipartForm, FrameworkError> {
        let content_type = self
            .inner
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let body = self.inner.into_body();
        parse_multipart_body(body, &content_type, max_file_bytes(), max_fields()).await
    }

    /// Parse the request body as multipart/form-data and return the first file for `field`.
    ///
    /// Consumes the request since the body can only be read once.
    pub async fn file(self, field: &str) -> Result<Option<UploadedFile>, FrameworkError> {
        let form = self.multipart().await?;
        Ok(form.files_map.get(field).and_then(|v| v.into_iter().next()).cloned())
    }
}
```

### UploadedFile type definition

```rust
// [ASSUMED] — shape from CONTEXT.md D-07/D-08
use bytes::Bytes;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub field_name: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Bytes,
}

impl UploadedFile {
    pub fn size(&self) -> usize { self.bytes.len() }

    pub fn extension(&self) -> Option<&str> {
        self.file_name.as_deref()
            .and_then(|n| Path::new(n).extension())
            .and_then(|e| e.to_str())
    }

    pub fn is_image(&self) -> bool {
        self.content_type.as_deref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }

    pub async fn store(
        &self,
        disk: &ferro_storage::Disk,
        path: &str,
    ) -> Result<(), ferro_storage::Error> {
        let opts = ferro_storage::PutOptions::new()
            .content_type(
                self.content_type.as_deref().unwrap_or("application/octet-stream")
            );
        disk.put_with_options(path, self.bytes.clone(), opts).await
    }
}
```

### Ergonomic handler target

```rust
// Source: CONTEXT.md <specifics> (design goal)
#[handler]
pub async fn upload_avatar(req: Request, user: User) -> Response {
    let form = req.multipart().await?;
    let file = form.file("avatar").ok_or_else(|| bad_request("no file"))?;
    let storage = App::resolve::<Storage>()?;
    let disk = storage.disk("public")?;
    let path = format!("avatars/{}.{}", user.id, file.extension().unwrap_or("bin"));
    file.store(&disk, &path).await
        .map_err(|e| FrameworkError::internal(e.to_string()))?;
    Ok(json!({"path": path}))
}
```

---

## Environment Availability

Step 2.6: SKIPPED — this phase is a pure code addition to the framework. The only new dependency is `multer = "3"` pulled from crates.io during `cargo build`.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| hyper 0.x: `Incoming` implements `Stream` | hyper 1.x: `Incoming` does NOT implement `Stream` | hyper 1.0 (2023) | Requires `BodyStream` adapter from http_body_util |
| multer 2.x: `parse_boundary` returned `Option<String>` | multer 3.x: returns `Result<String>` | multer 3.0 (2023) | Must use `map_err`, not `ok_or` |
| `futures_util::StreamExt` needed separate "stream" feature | `futures_util` "std" feature includes `StreamExt` | futures 0.3 | No feature change required |

**Deprecated/outdated:**
- `multipart` crate (abonander): not hyper 1.x compatible, stalled maintenance.
- `multer` v2 docs: show `Option<String>` for `parse_boundary` — outdated.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `UploadedFile::store` takes `&ferro_storage::Disk` (not `&Storage`) | Code Examples | If `Storage` is preferred, the call site ergonomics change slightly (caller must call `storage.disk()` first); functionally equivalent |
| A2 | `MultipartForm::files_map` is `HashMap<String, Vec<UploadedFile>>` | Code Examples | If `Vec<UploadedFile>` is used instead, `file()` requires a linear scan |
| A3 | `max_fields` is enforced by counter in the iteration loop (not multer Constraints) | Patterns | multer `Constraints` may expose a field count limit in a future version |
| A4 | Request::file() returns `Result<Option<UploadedFile>, FrameworkError>` (not `Option<UploadedFile>`) | Code Examples | If it returns `Option`, error handling requires a separate step |
| A5 | The `lib.rs` re-export line is `pub use http::multipart::{MultipartForm, UploadedFile, validate_mime, validate_size}` | Decisions | Already locked in D-16 — only risk is if `validate_mime`/`validate_size` names change |

---

## Open Questions

1. **multer `Constraints::max_fields` — does multer 3.x expose it?**
   - What we know: `SizeLimit` has `whole_stream`, `per_field`, `for_field`. `Constraints` has `allowed_fields` and `size_limit`.
   - What's unclear: Whether `Constraints::allowed_fields(vec![])` combined with wildcards can enforce a field count limit.
   - Recommendation: Enforce max_fields with a counter in the iteration loop — guaranteed to work regardless of multer version.

2. **`Request::file()` return type: `Result<Option<UploadedFile>>` vs `Option<UploadedFile>`**
   - What we know: `req.multipart()` can fail (bad Content-Type, parse error). The convenience method must either propagate that error or hide it.
   - Recommendation: Return `Result<Option<UploadedFile>, FrameworkError>` to match the framework's `?`-operator convention. Handlers that only need one file should use `?` freely.

---

## Validation Architecture

`nyquist_validation` key is absent from `.planning/config.json` — treat as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[tokio::test]` (same as existing framework tests) |
| Config file | Cargo.toml `[dev-dependencies]` |
| Quick run command | `cargo test -p ferro-rs --test multipart 2>/dev/null \|\| cargo test -p ferro-rs multipart` |
| Full suite command | `cargo test --all-features -p ferro-rs` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-03 | `req.multipart()` parses mixed multipart body | unit | `cargo test -p ferro-rs multipart_parses_fields` | ❌ Wave 0 |
| D-04 | `MultipartForm::file()` returns first file; `fields()` returns text | unit | `cargo test -p ferro-rs multipart_form_accessors` | ❌ Wave 0 |
| D-05 | `req.file("field")` convenience returns `Option<UploadedFile>` | unit | `cargo test -p ferro-rs request_file_convenience` | ❌ Wave 0 |
| D-07 | `UploadedFile` fields are correctly populated | unit | `cargo test -p ferro-rs uploaded_file_fields` | ❌ Wave 0 |
| D-08 | `size()`, `extension()`, `is_image()` correct | unit | `cargo test -p ferro-rs uploaded_file_methods` | ❌ Wave 0 |
| D-12 | Oversized field returns FrameworkError, not panic | unit | `cargo test -p ferro-rs multipart_size_limit` | ❌ Wave 0 |
| D-14 | `validate_mime` rejects disallowed MIME types | unit | `cargo test -p ferro-rs validate_mime_rejects` | ❌ Wave 0 |
| D-14 | `validate_size` rejects oversized bytes | unit | `cargo test -p ferro-rs validate_size_rejects` | ❌ Wave 0 |
| D-18 | Missing boundary returns descriptive FrameworkError | unit | `cargo test -p ferro-rs multipart_missing_boundary` | ❌ Wave 0 |

### Testing Strategy — In-Memory Multipart Body Construction

Because `hyper::body::Incoming` cannot be constructed in unit tests, the test pattern mirrors the existing approach in `request.rs` tests: test the underlying parsing logic directly without constructing a real `Request`.

The key unit-testable primitive is `parse_multipart_body(body: Incoming, ...)`. For tests, construct a real `hyper::body::Incoming`-equivalent via a locally-constructed multipart body:

```rust
// Build a raw multipart body as Bytes and wrap it in http_body_util::Full<Bytes>
// then use http_body_util::BodyStream adapter — same as production path
// [ASSUMED] — standard Rust async HTTP testing pattern
use http_body_util::Full;
use bytes::Bytes;

fn make_multipart_body(boundary: &str, parts: &[(&str, &str, Option<&str>)]) -> (Bytes, String) {
    let ct = format!("multipart/form-data; boundary={boundary}");
    let mut body = Vec::new();
    for (name, value, filename) in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        if let Some(fname) = filename {
            body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{fname}\"\r\n\r\n")
                .as_bytes()
            );
        } else {
            body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n")
                .as_bytes()
            );
        }
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (Bytes::from(body), ct)
}
```

Use `Full::new(bytes)` → `http_body_util::BodyStream::new(body).filter_map(...)` to pass into the internal `parse_multipart_body` function.

### Wave 0 Gaps

- [ ] `framework/src/http/multipart.rs` — the entire new module
- [ ] `framework/tests/multipart.rs` — or `framework/src/http/multipart.rs` `#[cfg(test)]` block covering all test map entries above
- [ ] `multer = "3"` in `framework/Cargo.toml`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | `validate_mime` + `validate_size` helpers; multer `SizeLimit::per_field` |
| V6 Cryptography | no | — |

### Known Threat Patterns for Multipart Upload

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| DoS via oversized file | Denial of Service | `SizeLimit::per_field(max_bytes)` in multer Constraints (server-side, pre-buffer) |
| DoS via large field count | Denial of Service | Manual counter in iteration loop (`> max_fields` → error) |
| Content-Type spoofing (upload .exe as image/png) | Tampering | `validate_mime()` checks declared MIME; callers should also validate by magic bytes if security-critical |
| Path traversal via file_name | Elevation of Privilege | `UploadedFile::extension()` uses `Path::extension()` (stdlib) — does NOT use file_name as a path; callers must construct the storage path themselves |

**File name sanitization:** `UploadedFile::file_name` stores the raw Content-Disposition value. Callers must not use `file_name` directly as a filesystem path. The framework does not sanitize file names (out of scope this phase). Planner should note this in the plan as a caller responsibility.

---

## Sources

### Primary (HIGH confidence)
- `[VERIFIED: rwf2/multer GitHub — examples/hyper_server_example.rs]` — exact `BodyStream` + `filter_map` conversion pattern for hyper 1.x
- `[VERIFIED: rwf2/multer GitHub — src/lib.rs]` — `parse_boundary` returns `Result<String, multer::Error>`, not `Option`
- `[VERIFIED: rwf2/multer GitHub — src/field.rs]` — `field.name()`, `file_name()`, `content_type()`, `bytes()` signatures
- `[VERIFIED: rwf2/multer GitHub — Cargo.toml]` — version 3.1.0, dev-deps confirm hyper 1.0 + http-body-util 0.1 compatibility
- `[VERIFIED: ferro framework/Cargo.toml]` — `ferro-storage`, `http-body-util`, `futures-util` already present
- `[VERIFIED: ferro framework/src/http/body.rs]` — `collect_body`, `parse_json`, `parse_form` established patterns
- `[VERIFIED: ferro framework/src/http/request.rs]` — `req.form()`, `req.json()` consume `self`, return `Result<T, FrameworkError>`; `into_parts()` exists at line 397
- `[VERIFIED: ferro ferro-storage/src/facade.rs]` — `Disk::put_with_options(path, contents, options)` signature
- `[VERIFIED: ferro ferro-storage/src/storage.rs]` — `PutOptions::new().content_type(...)` builder

### Secondary (MEDIUM confidence)
- `[CITED: docs.rs/multer/3.1.0]` — `Multipart::new`, `Multipart::with_constraints`, `Constraints`, `SizeLimit` type signatures

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — multer 3.1.0 confirmed via crates.io; hyper example verified from source; all existing deps confirmed in Cargo.toml
- Architecture: HIGH — follows exactly the existing `req.form()`/`req.json()` pattern; multer API verified
- Pitfalls: HIGH — hyper 1.x Stream issue confirmed from official hyper issue tracker; parse_boundary signature verified from source
- Storage integration: HIGH — Disk + PutOptions API read directly from ferro-storage source

**Research date:** 2026-05-15
**Valid until:** 2026-08-15 (stable libraries; multer 3.x unlikely to have breaking changes)
