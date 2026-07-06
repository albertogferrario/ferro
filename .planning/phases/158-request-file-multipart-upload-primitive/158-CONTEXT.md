# Phase 158: Request::file() multipart upload primitive - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Add multipart/form-data parsing to the framework so handlers can receive uploaded files via `req.multipart()` and `req.file("field")`. Include an `UploadedFile` type with a `store()` helper that bridges directly to `ferro-storage`. Out of scope: streaming upload (files are buffered into memory), multipart nested bodies, chunk resumption, progress events.

The killer feature: a handler can receive an uploaded file and persist it to local disk or S3 in three lines, using the same `ferro-storage` API already wired into the app.

</domain>

<decisions>
## Implementation Decisions

### Parser Library
- **D-01:** Use `multer` crate (async multipart parser compatible with hyper 1.x body streams). `mime_guess = "2"` is already a dep — no new MIME library needed.
- **D-02:** `multer` takes ownership of the body stream and the `boundary` string extracted from `Content-Type: multipart/form-data; boundary=...`. Extraction logic lives in a private helper alongside existing `parse_form` / `parse_json`.

### API Surface
- **D-03:** Primary API is `req.multipart().await -> Result<MultipartForm, FrameworkError>`. This mirrors the existing `req.form()` / `req.json()` pattern — consumes the request, returns a parsed value.
- **D-04:** `MultipartForm` exposes:
  - `fn file(&self, field: &str) -> Option<&UploadedFile>` — first file with that field name
  - `fn files(&self, field: &str) -> &[UploadedFile]` — all files with that field name
  - `fn field(&self, name: &str) -> Option<&str>` — text field value
  - `fn fields(&self) -> &HashMap<String, String>` — all text fields
- **D-05:** Convenience shorthand on `Request`: `req.file("avatar").await` — calls multipart internally and returns `Option<UploadedFile>`. For handlers that only need one file.
- **D-06:** Mixed multipart (text fields + files) is handled in a single `req.multipart()` pass. Text fields go into `MultipartForm::fields`, file fields into `MultipartForm::files_map`. No need for separate text-only parsing.

### UploadedFile Shape
- **D-07:** `UploadedFile` fields:
  - `field_name: String` — form field name (e.g. `"avatar"`)
  - `file_name: Option<String>` — original filename from Content-Disposition header
  - `content_type: Option<String>` — MIME type from part headers
  - `bytes: Bytes` — buffered file content
- **D-08:** `UploadedFile` methods:
  - `fn size(&self) -> usize` — `self.bytes.len()`
  - `fn extension(&self) -> Option<&str>` — derived from `file_name` via `Path::extension()`
  - `fn is_image(&self) -> bool` — checks content_type starts with `image/`
  - `fn store(&self, storage: &Storage, path: &str) -> impl Future<Output = Result<(), ferro_storage::Error>>` — puts bytes to storage disk

### Storage Integration
- **D-09:** `UploadedFile::store(storage, path)` calls `storage.put(path, self.bytes.clone())` with `PutOptions::new().content_type(self.content_type.as_deref().unwrap_or("application/octet-stream"))`.
- **D-10:** `ferro-storage` is added as a dependency to `framework/Cargo.toml` (it already exists in the workspace). `UploadedFile::store` lives in `framework/src/http/multipart.rs` — no separate crate needed.
- **D-11:** `storage.disk("public")?.put(...)` variant is handled by the caller (`storage.disk("s3")?` etc.) — `store()` takes `&dyn DiskDriver` or `&Storage` directly (Claude's discretion on which).

### Size Limits
- **D-12:** Default max file size: 10 MB. Configurable via env var `UPLOAD_MAX_SIZE_MB` (parsed at first multipart call, cached). If a part exceeds the limit, return `FrameworkError` with a user-friendly message — do not panic.
- **D-13:** Max fields limit: 100 (prevents DoS from degenerate multipart bodies). Also configurable via `UPLOAD_MAX_FIELDS`.

### Validation Helpers
- **D-14:** Add two free functions in `framework/src/http/multipart.rs`:
  - `fn validate_mime(file: &UploadedFile, allowed: &[&str]) -> Result<(), FrameworkError>` — checks content_type is in the allow-list
  - `fn validate_size(file: &UploadedFile, max_bytes: usize) -> Result<(), FrameworkError>` — manual size guard
- **D-15:** No integration with the existing `Validator` builder in this phase — that's a follow-on. These helpers are standalone.

### Module Location
- **D-16:** New module `framework/src/http/multipart.rs`. Exported from `framework/src/http/mod.rs`. Re-exported from `framework/src/lib.rs` as `pub use http::multipart::{MultipartForm, UploadedFile, validate_mime, validate_size}`.
- **D-17:** `multer` added to `framework/Cargo.toml`. No new workspace crate.

### Error Handling
- **D-18:** Multipart errors map to `FrameworkError::internal(...)` with descriptive message. Boundary-missing error gets a dedicated message: `"Content-Type is not multipart/form-data or missing boundary"`.

### Claude's Discretion
- Whether `UploadedFile::store` takes `&Storage` or `&dyn DiskDriver` — pick whichever avoids a generic parameter on `UploadedFile`
- Whether `MultipartForm::files_map` is `HashMap<String, Vec<UploadedFile>>` or `Vec<UploadedFile>` iterated by field name — pick the simpler representation
- Test structure: in-memory multipart body construction for unit tests vs integration tests
- Display name for errors shown to users (file too large, bad MIME type)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core implementation targets
- `framework/src/http/body.rs` — existing body parsing pattern to follow
- `framework/src/http/request.rs` — Request struct where `req.multipart()` and `req.file()` are added
- `framework/src/http/mod.rs` — module re-export location
- `framework/src/lib.rs` — top-level re-exports

### Storage integration
- `ferro-storage/src/lib.rs` — Storage, Disk, DiskDriver, PutOptions public API
- `ferro-storage/src/storage.rs` — PutOptions struct and builder methods

### Framework error type
- `framework/src/error.rs` — FrameworkError, how internal errors are constructed

### Cargo dependencies
- `framework/Cargo.toml` — add `multer = "3"` (latest stable for hyper 1.x)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `collect_body(body: Incoming) -> Result<Bytes, FrameworkError>` in `body.rs` — used by existing parsers; `multer` takes the raw `Incoming` stream directly (before collection), so this is NOT used for multipart. `multer::Multipart::new(body, boundary)` takes `Incoming` directly.
- `mime_guess` dep already present — can validate extensions against MIME types if needed
- `ferro-storage::PutOptions::new().content_type(...)` — exact API for storing with MIME type

### Established Patterns
- All body-consuming methods on `Request` are async and return `Result<T, FrameworkError>`
- `req.form()` and `req.json()` both consume `self` — `req.multipart()` should too
- FrameworkError uses `internal(message: String)` for wrapping low-level errors
- Re-exports go through `framework/src/lib.rs` for user-facing types

### Integration Points
- `framework/src/http/request.rs:397` — `into_parts()` gives raw `hyper::body::Incoming`; `req.multipart()` can use this to extract boundary then pass body to `multer`
- `ferro-storage` is a workspace crate — add as `ferro-storage = { path = "../ferro-storage" }` in framework/Cargo.toml

</code_context>

<specifics>
## Specific Ideas

- Ergonomic target for a handler:
  ```rust
  #[handler]
  pub async fn upload_avatar(req: Request, user: User) -> Response {
      let form = req.multipart().await?;
      let file = form.file("avatar").ok_or_else(|| bad_request("no file"))?;
      let path = format!("avatars/{}.{}", user.id, file.extension().unwrap_or("bin"));
      file.store(&storage, &path).await?;
      Ok(json!({"path": path}))
  }
  ```
- `multer` v3 is the current version compatible with hyper 1.x body streams.
- Boundary extraction: `multer::parse_boundary(content_type_str)` is the standard approach.

</specifics>

<deferred>
## Deferred Ideas

- Streaming upload (pass file bytes directly to S3 without full buffer) — requires different multer API surface, future phase
- Validator builder integration (`rules![file_type("image/*"), max_size(5.mb())]`) — future phase
- Multiple file upload progress events / WebSocket progress — future phase
- `#[extract]` derive macro support for typed multipart structs — future phase

</deferred>

---

*Phase: 158-request-file-multipart-upload-primitive*
*Context gathered: 2026-05-15*
