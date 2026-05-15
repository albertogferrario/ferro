# Phase 158: Request::file() Multipart Upload Primitive — Pattern Map

**Mapped:** 2026-05-15
**Files analyzed:** 5
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `framework/src/http/multipart.rs` | utility (body parser + types) | file-I/O / request-response | `framework/src/http/body.rs` | role-match |
| `framework/src/http/request.rs` | request abstraction | request-response | self (add two methods) | exact — mirror `req.json()` / `req.form()` |
| `framework/src/http/mod.rs` | module registry | — | self (add one line) | exact — mirror `body` module pattern |
| `framework/src/lib.rs` | public API surface | — | self (add one pub use) | exact — mirror `http::{…}` re-export block |
| `framework/Cargo.toml` | config | — | self | exact — add one dependency |

---

## Pattern Assignments

### `framework/src/http/multipart.rs` (utility, file-I/O / request-response)

**Analog:** `framework/src/http/body.rs`

**Imports pattern** (`body.rs` lines 1–9):
```rust
use crate::error::FrameworkError;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use serde::de::DeserializeOwned;
```

For `multipart.rs`, the import block becomes:
```rust
use crate::error::FrameworkError;
use bytes::Bytes;
use ferro_storage::{Disk, PutOptions};
use futures_util::StreamExt;
use http_body_util::BodyStream;
use hyper::body::Incoming;
use std::collections::HashMap;
use std::path::Path;
```

**Error construction pattern** (`body.rs` lines 12–17, 20–23, 26–29):
```rust
.map_err(|e| FrameworkError::internal(format!("Failed to read request body: {e}")))
```
Every fallible operation calls `FrameworkError::internal(format!("…: {e}"))`. Multipart follows the same pattern:
- boundary missing → `FrameworkError::internal("Content-Type is not multipart/form-data or missing boundary")`
- parse error → `FrameworkError::internal(format!("Multipart parse error: {e}"))`
- field read error → `FrameworkError::internal(format!("Field read error: {e}"))`

**Core parsing pattern** — body stream conversion (verified from multer hyper example):
```rust
// Inside private async fn parse_multipart_body(body: Incoming, content_type: &str, …)
let boundary = multer::parse_boundary(content_type)
    .map_err(|_| FrameworkError::internal(
        "Content-Type is not multipart/form-data or missing boundary",
    ))?;

let body_stream = BodyStream::new(body)
    .filter_map(|result| async move {
        result.map(|frame| frame.into_data().ok()).transpose()
    });

let constraints = multer::Constraints::new()
    .size_limit(multer::SizeLimit::new().per_field(max_file_bytes));

let mut multipart = multer::Multipart::with_constraints(body_stream, boundary, constraints);
```

**Field iteration and classification pattern**:
```rust
let mut files_map: HashMap<String, Vec<UploadedFile>> = HashMap::new();
let mut text_fields: HashMap<String, String> = HashMap::new();
let mut field_count = 0usize;

while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| FrameworkError::internal(format!("Multipart parse error: {e}")))?
{
    field_count += 1;
    if field_count > max_fields {
        return Err(FrameworkError::internal("Too many fields in multipart request"));
    }

    let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
    let file_name = field.file_name().map(|s| s.to_string());
    let content_type = field.content_type().map(|m| m.to_string());
    let bytes = field
        .bytes()
        .await
        .map_err(|e| FrameworkError::internal(format!("Field read error: {e}")))?;

    if file_name.is_some() {
        files_map.entry(field_name.clone()).or_default().push(UploadedFile {
            field_name,
            file_name,
            content_type,
            bytes,
        });
    } else {
        text_fields.insert(field_name, String::from_utf8_lossy(&bytes).into_owned());
    }
}

Ok(MultipartForm { files_map, text_fields })
```

**UploadedFile type pattern** (shape from D-07/D-08; `Path::extension` is stdlib):
```rust
#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub field_name: String,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Bytes,
}

impl UploadedFile {
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    pub fn extension(&self) -> Option<&str> {
        self.file_name
            .as_deref()
            .and_then(|n| Path::new(n).extension())
            .and_then(|e| e.to_str())
    }

    pub fn is_image(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }

    pub async fn store(&self, disk: &Disk, path: &str) -> Result<(), ferro_storage::Error> {
        let opts = PutOptions::new().content_type(
            self.content_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
        );
        disk.put_with_options(path, self.bytes.clone(), opts).await
    }
}
```

`Disk::put_with_options` signature confirmed from `ferro-storage/src/facade.rs` lines 331–338:
```rust
pub async fn put_with_options(
    &self,
    path: &str,
    contents: impl Into<Bytes>,
    options: PutOptions,
) -> Result<(), Error> {
    self.driver.put(path, contents.into(), options).await
}
```

**Validation helper pattern** — standalone free functions:
```rust
pub fn validate_mime(file: &UploadedFile, allowed: &[&str]) -> Result<(), FrameworkError> {
    let ct = file.content_type.as_deref().unwrap_or("");
    if allowed.iter().any(|&a| ct == a) {
        Ok(())
    } else {
        Err(FrameworkError::internal(format!(
            "File type '{ct}' is not allowed; accepted: {}",
            allowed.join(", ")
        )))
    }
}

pub fn validate_size(file: &UploadedFile, max_bytes: usize) -> Result<(), FrameworkError> {
    if file.size() <= max_bytes {
        Ok(())
    } else {
        Err(FrameworkError::internal(format!(
            "File too large: {} bytes (max {max_bytes})",
            file.size()
        )))
    }
}
```

**Environment limit helpers** — private, call once from the parsing entry point:
```rust
fn max_file_bytes() -> u64 {
    std::env::var("UPLOAD_MAX_SIZE_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10)
        * 1024
        * 1024
}

fn max_fields() -> usize {
    std::env::var("UPLOAD_MAX_FIELDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
}
```

**Test pattern** — mirrors `request.rs` test approach: test the private parsing function directly without constructing `hyper::body::Incoming`. Use `http_body_util::Full<Bytes>` + `BodyStream` to feed a raw multipart body:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http_body_util::{BodyStream, Full};

    fn make_multipart_body(boundary: &str, parts: &[(&str, &str, Option<&str>)]) -> (Bytes, String) {
        let ct = format!("multipart/form-data; boundary={boundary}");
        let mut body = Vec::new();
        for (name, value, filename) in parts {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            match filename {
                Some(fname) => body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"; filename=\"{fname}\"\r\n\r\n")
                        .as_bytes(),
                ),
                None => body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n")
                        .as_bytes(),
                ),
            }
            body.extend_from_slice(value.as_bytes());
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (Bytes::from(body), ct)
    }
}
```

---

### `framework/src/http/request.rs` (request abstraction, request-response)

**Analog:** self — add `multipart()` and `file()` following the existing `json()` / `form()` pattern.

**Existing body-consuming method pattern** (`request.rs` lines 353–376):
```rust
/// Parse the request body as JSON
///
/// Consumes the request since the body can only be read once.
pub async fn json<T: DeserializeOwned>(self) -> Result<T, FrameworkError> {
    let (_, bytes) = self.body_bytes().await?;
    parse_json(&bytes)
}

/// Parse the request body as form-urlencoded
///
/// Consumes the request since the body can only be read once.
pub async fn form<T: DeserializeOwned>(self) -> Result<T, FrameworkError> {
    let (_, bytes) = self.body_bytes().await?;
    parse_form(&bytes)
}
```

**`into_parts()` — raw body extraction** (`request.rs` lines 397–415):
```rust
pub fn into_parts(self) -> (RequestParts, hyper::body::Incoming) {
    let content_type = self
        .inner
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let params = self.params;
    let body = self.inner.into_body();

    (RequestParts { params, content_type }, body)
}
```

New methods copy this pattern — extract content-type header then call `self.inner.into_body()` to get the `Incoming` stream:
```rust
/// Parse the request body as multipart/form-data.
///
/// Consumes the request since the body can only be read once.
pub async fn multipart(self) -> Result<super::multipart::MultipartForm, FrameworkError> {
    let content_type = self
        .inner
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();
    let body = self.inner.into_body();
    super::multipart::parse_multipart_body(
        body,
        &content_type,
        super::multipart::max_file_bytes(),
        super::multipart::max_fields(),
    )
    .await
}

/// Parse the request body as multipart/form-data and return the first file for `field`.
///
/// Consumes the request since the body can only be read once.
pub async fn file(
    self,
    field: &str,
) -> Result<Option<super::multipart::UploadedFile>, FrameworkError> {
    let mut form = self.multipart().await?;
    Ok(form.files_map.remove(field).and_then(|mut v| {
        if v.is_empty() { None } else { Some(v.swap_remove(0)) }
    }))
}
```

**Import line to add at top of `request.rs`** — no new import needed; `multipart` is a sibling module accessed via `super::multipart::`.

---

### `framework/src/http/mod.rs` (module registry)

**Analog:** self — follow the existing `body` module registration pattern.

**Existing pattern** (`mod.rs` lines 1–11):
```rust
mod body;
pub mod cookie;
mod extract;
mod form_request;
mod request;
pub mod request_context;
/// API resource and pagination types.
pub mod resources;
mod response;

pub use body::{collect_body, parse_form, parse_json};
```

**Change:** Add `mod multipart;` in the `mod` block and one `pub use` line:
```rust
mod multipart;
// …existing mods…

pub use multipart::{validate_mime, validate_size, MultipartForm, UploadedFile};
```

---

### `framework/src/lib.rs` (public API surface)

**Analog:** self — follow the existing `http::{…}` re-export block pattern.

**Existing re-export block** (`lib.rs` lines 105–109):
```rust
pub use http::{
    bytes, json, request_host, text, Cookie, CookieOptions, FormRequest, FromParam, FromRequest,
    HttpResponse, InertiaRedirect, PaginationLinks, PaginationMeta, Redirect, Request, Resource,
    ResourceCollection, ResourceMap, Response, ResponseExt, SameSite,
};
```

**Change:** Add four symbols from `http::multipart` to this same block (or a separate adjacent line per D-16):
```rust
pub use http::multipart::{validate_mime, validate_size, MultipartForm, UploadedFile};
```

The four names match exactly what D-16 specifies.

---

### `framework/Cargo.toml` (config)

**Analog:** self — follow existing workspace-crate dependency declarations.

**Existing pattern** (`Cargo.toml` lines 34–48):
```toml
ferro-storage = { path = "../ferro-storage", version = "0.2" }
```

`ferro-storage` is already present as a non-optional dependency (line 39). No change needed there.

**Change:** Add `multer` under `[dependencies]` in the same style as the other external crates:
```toml
multer = "3"
```

`futures-util` (line 68) and `http-body-util` (line 27) are already present with the features needed — no version or feature changes required.

---

## Shared Patterns

### Error construction
**Source:** `framework/src/error.rs` lines 368–372 and `framework/src/http/body.rs` lines 12–17
**Apply to:** all fallible operations in `multipart.rs`
```rust
FrameworkError::internal(format!("…: {e}"))
// or for user-visible messages:
FrameworkError::internal("Content-Type is not multipart/form-data or missing boundary")
```

### Body-consuming method signature
**Source:** `framework/src/http/request.rs` lines 353–356 and 373–376
**Apply to:** `Request::multipart()` and `Request::file()`
- `self` (consuming, not `&self`)
- `async fn`
- Return `Result<T, FrameworkError>` — always propagates with `?`

### Module registration
**Source:** `framework/src/http/mod.rs` lines 1–11
**Apply to:** `mod multipart;` declaration and `pub use multipart::{…}` line

### Top-level re-export
**Source:** `framework/src/lib.rs` lines 105–109
**Apply to:** `pub use http::multipart::{MultipartForm, UploadedFile, validate_mime, validate_size};`

### Storage put pattern
**Source:** `ferro-storage/src/facade.rs` lines 331–338 (`Disk::put_with_options`)
**Apply to:** `UploadedFile::store(&self, disk: &Disk, path: &str)`
```rust
disk.put_with_options(path, self.bytes.clone(), opts).await
```
`PutOptions` builder: `PutOptions::new().content_type("…")` — confirmed from `ferro-storage/src/storage.rs` lines 68–83.

---

## No Analog Found

None — all five files have close analogs in the codebase.

---

## Critical Implementation Notes for Planner

1. **`parse_boundary` returns `Result`, not `Option`.** Use `.map_err(|_| FrameworkError::internal("…"))`, not `.ok_or_else(…)` on an `Option`. This is the multer 3.x API.

2. **`hyper::body::Incoming` does not implement `Stream` in hyper 1.x.** Must use `http_body_util::BodyStream::new(body).filter_map(…)` before passing to `multer::Multipart::with_constraints`. Both deps are already present.

3. **`field.content_type()` returns `Option<&mime::Mime>`, not `Option<&str>`.** Store as `Option<String>` via `.map(|m| m.to_string())`.

4. **`UploadedFile::store` takes `&ferro_storage::Disk`, not `&Storage`.** This lets the caller pick the disk (`storage.disk("s3")?`). No generic parameter on `UploadedFile`.

5. **`MultipartForm::files_map` is `HashMap<String, Vec<UploadedFile>>`.** Supports `file()` (first match via index 0) and `files()` (full slice) without linear scan. Text fields go into a separate `HashMap<String, String>`.

6. **max_fields is enforced by a counter in the iteration loop** (multer `Constraints` has no direct `max_fields` method in 3.x). Increment counter per field, return `FrameworkError::internal("Too many fields…")` when exceeded.

7. **`make_multipart_body` helper is defined `#[cfg(test)]` inside `multipart.rs`** — uses `http_body_util::Full<Bytes>` to avoid needing a live `hyper::body::Incoming`.

---

## Metadata

**Analog search scope:** `framework/src/http/`, `ferro-storage/src/`
**Files scanned:** 7 source files
**Pattern extraction date:** 2026-05-15
