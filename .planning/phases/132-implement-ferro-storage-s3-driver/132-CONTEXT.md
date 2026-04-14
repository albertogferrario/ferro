# Phase 132: Implement ferro-storage S3 Driver - Context

**Gathered:** 2026-04-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the stub `S3Driver` in `ferro-storage/src/drivers/s3.rs` with a working implementation using `aws-sdk-s3`. All framework wiring exists (feature gate, DiskConfig fields, env var parsing, Error::S3 variant). Only the 15 `StorageDriver` trait methods need real implementations.

</domain>

<decisions>
## Implementation Decisions

### S3 Client Initialization
- **D-01:** `S3Driver` becomes a struct holding `aws_sdk_s3::Client`, `bucket: String`, and `url_base: Option<String>` (replacing the current unit struct)
- **D-02:** Client built from `DiskConfig` fields at driver creation time in `facade.rs::create_driver()`. Use `aws_config::defaults(BehaviorVersion::latest())` with the standard credential chain (env vars -> profile -> instance metadata)
- **D-03:** Custom endpoint configured via `AWS_URL` env var for S3-compatible providers (DigitalOcean Spaces, MinIO, Cloudflare R2). Passed as `endpoint_url()` on the S3 config builder. `force_path_style` enabled when custom endpoint is set.

### URL Generation
- **D-04:** `url()` returns `{url_base}/{path}` when `url_base` is configured (CDN or public bucket URL). Falls back to `https://{bucket}.s3.{region}.amazonaws.com/{path}` when no `url_base` is set.
- **D-05:** `temporary_url()` always generates a presigned `GetObject` URL using `aws_sdk_s3::presigning::PresigningConfig` with the caller-specified expiration duration.

### Visibility / ACL
- **D-06:** Default: skip per-object ACL entirely. Most S3-compatible providers (DO Spaces, R2) use bucket-level access. AWS itself deprecated per-object ACLs with S3 Object Ownership.
- **D-07:** When `PutOptions::visibility` is `Public`, set `x-amz-acl: public-read` on the PutObject request. When `Private`, omit the ACL header (bucket default applies). This is a best-effort hint — providers that ignore ACLs will simply use their bucket policy.

### Directory Operations
- **D-08:** `files(dir)` uses `ListObjectsV2` with `prefix=dir` and `delimiter=/`, returning object keys (excluding CommonPrefixes).
- **D-09:** `all_files(dir)` uses `ListObjectsV2` with `prefix=dir` only (no delimiter), returning all keys recursively. Handles pagination via `continuation_token`.
- **D-10:** `directories(dir)` extracts `CommonPrefixes` from a delimited `ListObjectsV2` call.
- **D-11:** `make_directory(path)` creates a zero-byte object at `{path}/.keep` as a directory marker.
- **D-12:** `delete_directory(path)` lists all objects with the prefix and issues `DeleteObjects` batch delete (up to 1000 per batch, paginated).

### Error Handling
- **D-13:** All AWS SDK errors map to `Error::S3(message)` using the error's display string. `NoSuchKey` maps to `Error::NotFound`.

### Testing
- **D-14:** Integration tests gated behind `s3-tests` feature flag. Tests run against a real bucket (configured via env vars). Not run in CI by default.
- **D-15:** Unit tests for URL construction and path normalization use no network calls.

### Claude's Discretion
- Path normalization (leading slash stripping, trailing slash for directories)
- Pagination batch size for listing operations
- Content-Type detection strategy for `put()` (mime_guess from extension, or PutOptions override)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### ferro-storage crate
- `ferro-storage/src/storage.rs` — StorageDriver trait definition (15 required + 3 default methods)
- `ferro-storage/src/drivers/s3.rs` — Current stub implementation (replace entirely)
- `ferro-storage/src/facade.rs` — DiskConfig, DiskDriver::S3 arm, create_driver() factory
- `ferro-storage/src/config.rs` — StorageConfig::from_env() with AWS_* env var parsing
- `ferro-storage/src/error.rs` — Error::S3 variant already defined
- `ferro-storage/Cargo.toml` — aws-sdk-s3 v1 + aws-config v1 declared as optional deps

### Existing driver patterns
- `ferro-storage/src/drivers/local.rs` — Reference implementation of all 15 trait methods
- `ferro-storage/src/drivers/memory.rs` — Simpler reference for trait compliance

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `DiskConfig` already has `bucket`, `region`, `url` fields behind `s3` feature gate
- `StorageConfig::from_env()` already reads `AWS_BUCKET`, `AWS_DEFAULT_REGION`, `AWS_URL`
- `Error::S3(String)` variant ready for use
- `mime_guess` crate already in dependencies — usable for content-type detection on `put()`
- `LocalDriver` provides pattern for all 15 methods — S3 driver follows the same signatures

### Established Patterns
- All drivers implement `StorageDriver` via `#[async_trait]`
- `PutOptions` carries visibility and content_type
- `FileMetadata` has `path`, `size`, `last_modified`, `mime_type`
- Error constructors: `Error::not_found()`, `Error::not_implemented()`

### Integration Points
- `facade.rs:200-206` — S3 arm of `create_driver()` currently creates unit struct + logs warning. This is the wiring point.
- `DiskConfig` needs no changes — fields already exist.
- `StorageConfig::from_env()` needs no changes — already wires S3.

</code_context>

<specifics>
## Specific Ideas

- gestiscilo field test: replace `src/services/storage.rs` with `ferro::Storage::disk("s3")` using `AWS_*` vars (currently uses `SPACES_*` vars with hand-rolled DO Spaces client)
- DigitalOcean Spaces is the primary S3-compatible target — uses `{region}.digitaloceanspaces.com` endpoint format

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 132-implement-ferro-storage-s3-driver*
*Context gathered: 2026-04-14*
