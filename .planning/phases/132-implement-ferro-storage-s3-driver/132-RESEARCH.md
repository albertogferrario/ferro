# Phase 132: Implement ferro-storage S3 Driver - Research

**Researched:** 2026-04-14
**Domain:** aws-sdk-s3 v1 Rust SDK, S3-compatible object storage (DigitalOcean Spaces, MinIO, R2)
**Confidence:** HIGH

## Summary

The ferro-storage crate has a complete stub `S3Driver` that returns `Error::not_implemented()` for all 15 trait methods. Everything around it is wired: `DiskConfig` has `bucket`/`region`/`url` fields, `StorageConfig::from_env()` reads `AWS_*` env vars, `Error::S3(String)` is defined, and `aws-sdk-s3 = "1"` plus `aws-config = "1"` are already optional deps behind the `s3` feature gate.

The aws-sdk-s3 v1 SDK uses a fluent builder pattern for all operations. All S3 `config::Builder` methods (credentials, region, endpoint_url, force_path_style) are **synchronous** — this is critical because `create_driver()` in `facade.rs` is a sync function. The approach is to build the `aws_sdk_s3::Client` entirely from synchronous builder calls, reading env vars directly rather than going through `aws_config::defaults().load().await` (which would require making `create_driver()` async). The CONTEXT.md D-02 says "standard credential chain (env vars -> profile -> instance metadata)" — the env-var tier of that chain is what `Credentials::from_keys()` implements; instance-metadata fallback would require async and is out of scope for this phase.

DigitalOcean Spaces is the primary S3-compatible target. Its quirk: region must be set to `us-east-1` in the SDK config (AWS convention), but the actual datacenter is encoded in the endpoint URL (`{region}.digitaloceanspaces.com`). `force_path_style` should be `false` for DO Spaces (it uses virtual-hosted style).

**Primary recommendation:** Build `S3Driver` as a struct holding `aws_sdk_s3::Client` + `bucket: String` + `region: String` + `url_base: Option<String>`. Construct the client synchronously in `create_driver()` using `aws_sdk_s3::config::Builder` with `Credentials::from_keys()` for credentials. All 15 trait methods map to straightforward SDK calls.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**S3 Client Initialization**
- **D-01:** `S3Driver` becomes a struct holding `aws_sdk_s3::Client`, `bucket: String`, and `url_base: Option<String>` (replacing the current unit struct)
- **D-02:** Client built from `DiskConfig` fields at driver creation time in `facade.rs::create_driver()`. Use `aws_config::defaults(BehaviorVersion::latest())` with the standard credential chain (env vars -> profile -> instance metadata)
- **D-03:** Custom endpoint configured via `AWS_URL` env var for S3-compatible providers (DigitalOcean Spaces, MinIO, Cloudflare R2). Passed as `endpoint_url()` on the S3 config builder. `force_path_style` enabled when custom endpoint is set.

**URL Generation**
- **D-04:** `url()` returns `{url_base}/{path}` when `url_base` is configured (CDN or public bucket URL). Falls back to `https://{bucket}.s3.{region}.amazonaws.com/{path}` when no `url_base` is set.
- **D-05:** `temporary_url()` always generates a presigned `GetObject` URL using `aws_sdk_s3::presigning::PresigningConfig` with the caller-specified expiration duration.

**Visibility / ACL**
- **D-06:** Default: skip per-object ACL entirely.
- **D-07:** When `PutOptions::visibility` is `Public`, set `x-amz-acl: public-read` on the PutObject request. When `Private`, omit the ACL header.

**Directory Operations**
- **D-08:** `files(dir)` uses `ListObjectsV2` with `prefix=dir` and `delimiter=/`, returning object keys (excluding CommonPrefixes).
- **D-09:** `all_files(dir)` uses `ListObjectsV2` with `prefix=dir` only (no delimiter), returning all keys recursively. Handles pagination via `continuation_token`.
- **D-10:** `directories(dir)` extracts `CommonPrefixes` from a delimited `ListObjectsV2` call.
- **D-11:** `make_directory(path)` creates a zero-byte object at `{path}/.keep` as a directory marker.
- **D-12:** `delete_directory(path)` lists all objects with the prefix and issues `DeleteObjects` batch delete (up to 1000 per batch, paginated).

**Error Handling**
- **D-13:** All AWS SDK errors map to `Error::S3(message)` using the error's display string. `NoSuchKey` maps to `Error::NotFound`.

**Testing**
- **D-14:** Integration tests gated behind `s3-tests` feature flag. Tests run against a real bucket (configured via env vars). Not run in CI by default.
- **D-15:** Unit tests for URL construction and path normalization use no network calls.

### Claude's Discretion
- Path normalization (leading slash stripping, trailing slash for directories)
- Pagination batch size for listing operations
- Content-Type detection strategy for `put()` (mime_guess from extension, or PutOptions override)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

## Project Constraints (from CLAUDE.md)

- Run `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test --all-features` before every commit
- No co-author lines in commits
- Keep functions small and focused
- Delete old code when replacing — no versioned functions
- `fmt.Errorf` Rust equivalent: `format!("context: {}", err)` — preserve error chains via Display
- Update docs (`docs/src/`) when framework changes

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| aws-sdk-s3 | 1.119.0 (locked) | S3 API operations | Official AWS Rust SDK v1 |
| aws-config | 1.8.12 (locked) | SDK config + credential chain | Required companion for aws-sdk-s3 |
| aws-credential-types | (transitive) | `Credentials::from_keys()` static creds | Part of aws-sdk-s3 dep tree |

Both are already declared as optional deps in `ferro-storage/Cargo.toml`. No new dependencies needed.

**Version verification:** Cargo.lock shows `aws-sdk-s3 = 1.119.0` and `aws-config = 1.8.12`. Latest registry shows `aws-sdk-s3 = 1.129.0`. The locked versions will be used; no version bump needed for this phase.

### Supporting (already present)
| Library | Version | Purpose |
|---------|---------|---------|
| mime_guess | 2 | Content-Type detection from file extension in `put()` |
| bytes | 1 | `Bytes` ↔ `ByteStream` conversion |
| async-trait | 0.1 | Required for `StorageDriver` trait impl |

**Installation:** No new `cargo add` needed — all deps are already declared.

## Architecture Patterns

### S3Driver Struct (replaces unit struct)

```rust
// Source: CONTEXT.md D-01
#[cfg(feature = "s3")]
pub struct S3Driver {
    client: aws_sdk_s3::Client,
    bucket: String,
    region: String,
    url_base: Option<String>,
}
```

### Sync Client Construction (critical pattern)

`create_driver()` in `facade.rs` is **synchronous**. `aws_config::defaults().load().await` cannot be called from sync context without spawning a runtime. The solution: build the client directly from `aws_sdk_s3::config::Builder` with explicit env-var credentials. This covers the "env vars" tier of D-02's credential chain without needing async.

```rust
// Source: docs.rs aws_sdk_s3::config::Builder + discussion #444
use aws_sdk_s3::config::{Builder, Region};
use aws_credential_types::Credentials;

let creds = Credentials::from_keys(
    std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
    std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
    None, // session token
);

let mut config_builder = Builder::new()
    .region(Region::new(region.clone()))
    .credentials_provider(creds);

if let Some(endpoint) = &endpoint_url {
    config_builder = config_builder
        .endpoint_url(endpoint)
        .force_path_style(true); // D-03: enable when custom endpoint set
}

let client = aws_sdk_s3::Client::from_conf(config_builder.build());
```

Note: `aws_credential_types` is a transitive dependency of `aws-sdk-s3`. It does NOT need an explicit `Cargo.toml` entry if imported only in code — but `Cargo.toml` should list it explicitly if using `features = ["hardcoded-credentials"]`.

**Alternative (if Cargo.toml allows):** The `aws_sdk_s3::config::Credentials` re-export may work without the extra feature flag. Verify at compile time.

### facade.rs S3 Arm Wiring

```rust
// Source: facade.rs:200-206 (current stub to replace)
#[cfg(feature = "s3")]
DiskDriver::S3 => {
    let bucket = config.bucket.clone().unwrap_or_default();
    let region = config.region.clone().unwrap_or_else(|| "us-east-1".to_string());
    let url_base = config.url.clone();
    let endpoint_url = std::env::var("AWS_URL").ok();
    Arc::new(S3Driver::new(bucket, region, url_base, endpoint_url))
}
```

### Presigned URL (temporary_url)

```rust
// Source: docs.aws.amazon.com/sdk-for-rust/latest/dg/presigned-urls.html
use aws_sdk_s3::presigning::PresigningConfig;

let presigning_config = PresigningConfig::builder()
    .expires_in(expiration)
    .build()
    .map_err(|e| Error::S3(e.to_string()))?;

let presigned = self.client
    .get_object()
    .bucket(&self.bucket)
    .key(path)
    .presigned(presigning_config)
    .await
    .map_err(|e| Error::S3(e.to_string()))?;

Ok(presigned.uri().to_string())
```

### put_object with ByteStream

```rust
// Source: docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_s3_code_examples.html
use aws_sdk_s3::primitives::ByteStream;

let body = ByteStream::from(contents); // contents: Bytes — direct conversion

let mut req = self.client
    .put_object()
    .bucket(&self.bucket)
    .key(path)
    .body(body);

// Content-Type: prefer PutOptions override, fall back to mime_guess
let content_type = options.content_type.clone()
    .or_else(|| mime_guess::from_path(path).first().map(|m| m.to_string()));
if let Some(ct) = content_type {
    req = req.content_type(ct);
}

// D-07: ACL for public visibility
if options.visibility == Visibility::Public {
    req = req.acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead);
}

req.send().await.map_err(|e| Error::S3(e.to_string()))?;
```

### ListObjectsV2 with Pagination (all_files / delete_directory)

```rust
// Source: docs.rs/aws-sdk-s3 ListObjectsV2Paginator
// Use .into_paginator() for clean pagination — no manual continuation_token loop needed

let mut paginator = self.client
    .list_objects_v2()
    .bucket(&self.bucket)
    .prefix(normalized_dir)
    // omit delimiter for all_files; add .delimiter("/") for files/directories
    .into_paginator()
    .send();

let mut keys = Vec::new();
while let Some(page) = paginator.next().await {
    let page = page.map_err(|e| Error::S3(e.to_string()))?;
    for obj in page.contents() {
        if let Some(key) = obj.key() {
            keys.push(key.to_string());
        }
    }
}
```

### delete_objects Batch

```rust
// Source: docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_s3_code_examples.html
use aws_sdk_s3::types::{Delete, ObjectIdentifier};

let identifiers: Result<Vec<_>, _> = keys.iter()
    .map(|key| ObjectIdentifier::builder().key(key).build()
        .map_err(|e| Error::S3(e.to_string())))
    .collect();

let delete = Delete::builder()
    .set_objects(Some(identifiers?))
    .build()
    .map_err(|e| Error::S3(e.to_string()))?;

self.client
    .delete_objects()
    .bucket(&self.bucket)
    .delete(delete)
    .send()
    .await
    .map_err(|e| Error::S3(e.to_string()))?;
```

### Error Mapping (D-13)

The SDK wraps errors in `SdkError<ServiceError>`. For `NoSuchKey` detection, `head_object` returns HTTP 404 but may not carry a typed `NoSuchKey` variant in its response body (S3 quirk — HEAD requests have no body). Check the raw HTTP status code as a fallback.

```rust
// For get/delete operations — typed error matching
.map_err(|e| {
    // Check if it's a NoSuchKey service error
    if e.code() == Some("NoSuchKey") {
        Error::not_found(path)
    } else {
        Error::S3(e.to_string())
    }
})?
```

For `head_object` (used in `exists()`, `size()`, `metadata()`):

```rust
match result {
    Ok(output) => { /* found */ }
    Err(e) => {
        // HEAD responses have no body, so typed error may be unmodeled
        // Check HTTP status directly
        if e.as_service_error()
            .map(|se| se.is_not_found())
            .unwrap_or(false)
        || e.raw_response()
            .map(|r| r.status().as_u16() == 404)
            .unwrap_or(false)
        {
            return Err(Error::not_found(path));
        }
        return Err(Error::S3(e.to_string()));
    }
}
```

### copy_object copy_source Format

The `copy_source` parameter uses URL-encoded `{bucket}/{key}` format:

```rust
// Source: AWS S3 CopyObject API docs
let copy_source = format!("{}/{}", self.bucket, from);
self.client
    .copy_object()
    .copy_source(copy_source)
    .bucket(&self.bucket)
    .key(to)
    .send()
    .await
    .map_err(|e| Error::S3(e.to_string()))?;
```

### Path Normalization (Claude's discretion)

Consistent with `MemoryDriver::normalize_path`:

```rust
fn normalize_path(path: &str) -> &str {
    path.trim_start_matches('/')
}

fn normalize_dir_prefix(dir: &str) -> String {
    let d = dir.trim_start_matches('/');
    if d.is_empty() {
        String::new()
    } else {
        format!("{d}/")
    }
}
```

### Recommended Project Structure (files to touch)

```
ferro-storage/
├── src/
│   ├── drivers/
│   │   └── s3.rs         ← full replacement (currently stub)
│   └── facade.rs         ← update S3 arm in create_driver()
├── Cargo.toml            ← may need aws-credential-types if not transitive
└── tests/
    └── s3_integration.rs ← new, gated behind s3-tests feature
```

### Anti-Patterns to Avoid

- **Calling `aws_config::defaults().load().await` in `create_driver()`:** `create_driver()` is sync. This would require `block_on()` which panics inside an existing Tokio runtime. Use `aws_sdk_s3::config::Builder` directly.
- **Manual pagination loop with `continuation_token`:** The SDK's `.into_paginator()` handles this cleanly; prefer it over manual loops.
- **Using `ListObjectsV1`:** Always use `ListObjectsV2` — V1 is legacy.
- **URL-encoding `copy_source` manually:** The SDK handles URL-encoding internally for the `copy_source()` builder method; just pass `bucket/key` as a string.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Pagination | Manual continuation-token loop | `.into_paginator().send()` | Paginator handles all token management |
| Presigned URLs | Custom HMAC signing | `PresigningConfig` + `.presigned()` | Signing is complex, time-sensitive, SigV4 |
| Batch delete | Loop of single deletes | `delete_objects()` with `Delete::builder()` | 1000 objects per call, atomic, cheaper |
| Content-Type detection | Extension map | `mime_guess::from_path()` | Already a dep, covers 800+ MIME types |
| Credentials wiring | Custom env-var parser | `Credentials::from_keys()` | Standard SDK type, implements `ProvideCredentials` |

**Key insight:** The aws-sdk-s3 paginator API is the right abstraction for listing — it handles continuation tokens, rate limiting, and page iteration without manual state management.

## Common Pitfalls

### Pitfall 1: async config init in sync create_driver()
**What goes wrong:** Calling `aws_config::defaults(BehaviorVersion::latest()).load().await` inside the sync `create_driver()` function panics at runtime ("cannot call block_in_place inside a Tokio context without multi-thread").
**Why it happens:** `create_driver()` is not `async fn`, and the S3 SDK config loader is inherently async.
**How to avoid:** Use `aws_sdk_s3::config::Builder::new()` directly. Read `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` from env, pass to `Credentials::from_keys()`. The standard credential chain is fully covered for the env-var tier (which is the only one DO Spaces users will use).
**Warning signs:** Compiler error "future cannot be awaited in non-async context" or runtime panic with "cannot use block_in_place inside a Tokio context".

### Pitfall 2: HeadObject 404 not returning typed NoSuchKey
**What goes wrong:** `head_object` returns HTTP 404 when the object doesn't exist, but because HEAD has no response body, the SDK cannot parse a typed `NoSuchKey` error variant — it may return a generic 404 error instead.
**Why it happens:** S3 error details are in the XML response body; HEAD requests have no body.
**How to avoid:** For `exists()`, check both the typed `is_not_found()` on the service error AND the raw HTTP 404 status. Map both to `Ok(false)` for `exists()`, or `Error::NotFound` for `size()`/`metadata()`.
**Warning signs:** `exists()` returns `Err(S3(...))` instead of `Ok(false)` for missing keys.

### Pitfall 3: DigitalOcean Spaces + force_path_style
**What goes wrong:** Setting `force_path_style(true)` for DO Spaces generates bucket paths like `https://nyc3.digitaloceanspaces.com/mybucket/key` instead of `https://mybucket.nyc3.digitaloceanspaces.com/key`.
**Why it happens:** DO Spaces uses virtual-hosted-style addressing (default). `force_path_style` should only be `true` for MinIO/local S3 that don't support virtual-hosted style. D-03 says "enable when custom endpoint is set" — this needs care: for DO Spaces, `force_path_style` should probably be `false` or configurable.
**How to avoid:** D-03 locks the behavior to "force_path_style enabled when custom endpoint is set". This aligns with MinIO and R2 requirements. For DO Spaces, virtual-hosted style is the correct default; the locked decision (force_path_style=true for custom endpoints) may need a follow-up if DO Spaces breaks. Document this as a known limitation.
**Warning signs:** ListObjects returns empty results or 403; URLs have the bucket in the path instead of subdomain.

### Pitfall 4: copy_source URL-encoding
**What goes wrong:** If the key contains spaces or special chars and you manually URL-encode `copy_source`, you may double-encode it since the SDK also performs encoding.
**How to avoid:** Pass the raw `"{bucket}/{key}"` string directly to `.copy_source()`. The SDK handles URL-encoding internally.

### Pitfall 5: Empty prefix for root-level listing
**What goes wrong:** `files("")` or `all_files("")` with an empty prefix lists the entire bucket, which may be very large or time out.
**How to avoid:** The behavior is correct by S3 semantics — empty prefix means "all objects". For `files("")`, use `delimiter="/"` and no prefix to get only root-level objects. Document that callers should provide a specific prefix for large buckets.

### Pitfall 6: s3-tests feature vs s3 feature
**What goes wrong:** Integration tests that depend on `aws-sdk-s3` must be behind a feature that implies the `s3` feature. If `s3-tests` is declared without `s3` in its requirements, compilation fails.
**How to avoid:** Declare `s3-tests = ["s3"]` in `[features]` of `Cargo.toml`. Tests then compile only when both features are active.

## Code Examples

Verified patterns from official sources:

### Client construction (sync, no async needed)
```rust
// Source: docs.rs/aws-sdk-s3/latest/aws_sdk_s3/config/struct.Builder.html
use aws_sdk_s3::config::{Builder as S3ConfigBuilder, Region};

let creds = aws_credential_types::Credentials::from_keys(
    std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_default(),
    std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_default(),
    None,
);

let config = S3ConfigBuilder::new()
    .region(Region::new(region))
    .credentials_provider(creds)
    .build();

let client = aws_sdk_s3::Client::from_conf(config);
```

### List with delimiter (files / directories)
```rust
// Source: docs.rs/aws-sdk-s3 ListObjectsV2FluentBuilder
let output = self.client
    .list_objects_v2()
    .bucket(&self.bucket)
    .prefix(format!("{dir}/"))
    .delimiter("/")
    .send()
    .await
    .map_err(|e| Error::S3(e.to_string()))?;

// Files: output.contents() — keys not in common_prefixes
// Directories: output.common_prefixes() — strip trailing "/"
```

### exists() using head_object
```rust
// Source: AWS SDK for Rust error handling docs
match self.client.head_object().bucket(&self.bucket).key(path).send().await {
    Ok(_) => Ok(true),
    Err(e) => {
        let is_404 = e.as_service_error()
            .map(|se| se.is_not_found())
            .unwrap_or(false)
            || e.raw_response().map(|r| r.status().as_u16() == 404).unwrap_or(false);
        if is_404 { Ok(false) } else { Err(Error::S3(e.to_string())) }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| rusoto_s3 crate | aws-sdk-s3 v1 | 2022–2023 | rusoto is unmaintained; aws-sdk-s3 is the official AWS-maintained SDK |
| Manual pagination loops | `.into_paginator()` | aws-sdk-s3 v1 | Cleaner code, correct state management |
| ListObjects (V1) | ListObjectsV2 | S3 API evolution | V2 is recommended; V1 deprecated for new code |

**Deprecated/outdated:**
- rusoto: Unmaintained since 2023. aws-sdk-s3 is the replacement.
- `aws_config::from_env()`: Replaced by `aws_config::defaults(BehaviorVersion::latest())`.

## Open Questions

1. **`Credentials::from_keys()` crate import**
   - What we know: It lives in `aws-credential-types` with `features = ["hardcoded-credentials"]`, OR may be accessible via `aws_sdk_s3::config::Credentials` re-export.
   - What's unclear: Whether `aws-credential-types` is already in the transitive dep tree without explicit declaration, and whether the `hardcoded-credentials` feature is enabled by default.
   - Recommendation: At Wave 0, attempt `use aws_sdk_s3::config::Credentials;` first. If that doesn't compile, add `aws-credential-types = { version = "1", features = ["hardcoded-credentials"] }` to `ferro-storage/Cargo.toml`.

2. **D-03 force_path_style for DO Spaces**
   - What we know: DO Spaces uses virtual-hosted style (force_path_style=false). D-03 locks "force_path_style enabled when custom endpoint is set". This works for MinIO/R2 but may conflict with DO Spaces behavior.
   - What's unclear: Whether DO Spaces breaks with force_path_style=true in practice (it may work since it also supports path-style).
   - Recommendation: Implement D-03 as locked. Flag for validation in the gestiscilo field test. If DO Spaces rejects path-style requests, D-03 can be refined to add a `force_path_style` field to `DiskConfig`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| DigitalOcean Spaces bucket | Integration tests (D-14) | Unknown | — | Skip `s3-tests` run; unit tests pass without it |
| `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` env vars | Integration tests | Unknown | — | Tests gated behind `s3-tests` feature; not run in CI |
| cargo (Rust toolchain) | All compilation | Yes | workspace uses stable | — |

**Missing dependencies with no fallback:** None — all blocking functionality is covered by the existing `s3` feature gate.
**Missing dependencies with fallback:** Real S3 bucket for integration tests — gated behind `s3-tests` feature per D-14.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (tokio::test for async) |
| Config file | Cargo.toml [features] — `s3-tests = ["s3"]` |
| Quick run command | `cargo test -p ferro-storage --features s3` |
| Full suite command | `cargo test --all-features` |
| Integration tests | `cargo test -p ferro-storage --features s3-tests` (requires real bucket) |

### Phase Requirements → Test Map
| Behavior | Test Type | Automated Command | Notes |
|----------|-----------|-------------------|-------|
| URL construction (url_base set) | unit | `cargo test -p ferro-storage --features s3 url_with_base` | No network |
| URL construction (no url_base) | unit | `cargo test -p ferro-storage --features s3 url_fallback` | No network |
| Path normalization | unit | `cargo test -p ferro-storage --features s3 normalize_path` | No network |
| All 15 trait methods return real results | integration | `cargo test -p ferro-storage --features s3-tests` | Requires bucket |
| `Storage::disk("s3").put().await` works against DO Spaces | manual/integration | field test via gestiscilo | Requires live bucket |

### Wave 0 Gaps
- [ ] `ferro-storage/Cargo.toml` — add `s3-tests = ["s3"]` feature
- [ ] `ferro-storage/tests/s3_integration.rs` — integration test file (Wave 1+, not blocking implementation)

*(Unit tests for URL construction can be added inline in `s3.rs` — no new files needed for unit coverage)*

## Sources

### Primary (HIGH confidence)
- [docs.rs/aws-sdk-s3/latest/aws_sdk_s3/config/struct.Builder.html](https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/config/struct.Builder.html) — Builder method signatures confirmed sync
- [AWS SDK for Rust presigned URLs guide](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/presigned-urls.html) — PresigningConfig pattern verified
- [AWS SDK for Rust S3 code examples](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_s3_code_examples.html) — put_object, get_object, delete_objects, list_objects_v2 patterns
- [AWS SDK for Rust error handling](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/error-handling.html) — NoSuchKey detection pattern
- [AWS SDK for Rust client configuration](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/config-code.html) — sync Builder confirmed, no defaults loaded without aws_config

### Secondary (MEDIUM confidence)
- [DigitalOcean Spaces + AWS SDK guide](https://docs.digitalocean.com/products/spaces/how-to/use-aws-sdks/) — DO Spaces endpoint format, force_path_style=false recommendation
- [GitHub awslabs/aws-sdk-rust discussion #444](https://github.com/awslabs/aws-sdk-rust/discussions/444) — Credentials::from_keys pattern for static credentials
- [docs.rs ListObjectsV2Paginator](https://docs.rs/aws-sdk-s3/latest/aws_sdk_s3/operation/list_objects_v2/paginator/struct.ListObjectsV2Paginator.html) — paginator pattern

### Tertiary (LOW confidence)
- HeadObject NoSuchKey behavior across SDKs — documented as inconsistent issue in multiple SDK repos; pattern of checking raw 404 status is a workaround

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — deps already locked in Cargo.lock
- Architecture: HIGH — sync Builder pattern verified from official docs
- Pitfalls: HIGH (sync init), MEDIUM (HeadObject 404) — both verified from SDK docs and known SDK behavior
- DO Spaces specifics: MEDIUM — official DO docs confirm endpoint format; force_path_style interaction needs field validation

**Research date:** 2026-04-14
**Valid until:** 2026-07-14 (stable SDK, 90-day estimate)
