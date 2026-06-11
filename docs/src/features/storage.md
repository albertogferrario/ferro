# Storage

Ferro provides a unified file storage abstraction inspired by Laravel's filesystem. Work with local files, memory storage, and cloud providers through a consistent API.

## Configuration

### Environment Variables

Configure storage in your `.env` file:

```env
# Default disk (local, public, or s3)
FILESYSTEM_DISK=local

# Local disk settings
FILESYSTEM_LOCAL_ROOT=./storage
FILESYSTEM_LOCAL_URL=

# Public disk settings (for web-accessible files)
FILESYSTEM_PUBLIC_ROOT=./storage/public
FILESYSTEM_PUBLIC_URL=/storage

# S3 disk settings (requires s3 feature)
AWS_ACCESS_KEY_ID=your-key
AWS_SECRET_ACCESS_KEY=your-secret
AWS_DEFAULT_REGION=us-east-1
AWS_BUCKET=your-bucket
AWS_URL=https://your-bucket.s3.amazonaws.com
```

### Bootstrap Setup

In `src/bootstrap.rs`, configure storage:

```rust
use ferro::{App, Storage, StorageConfig};
use std::sync::Arc;

pub async fn register() {
    // ... other setup ...

    // Create storage with environment config
    let config = StorageConfig::from_env();
    let storage = Arc::new(Storage::with_storage_config(config));

    // Store in app state for handlers to access
    App::set_storage(storage);
}
```

### Manual Configuration

```rust
use ferro::{Storage, StorageConfig, DiskConfig};

let config = StorageConfig::new("local")
    .disk("local", DiskConfig::local("./storage"))
    .disk("public", DiskConfig::local("./storage/public").with_url("/storage"))
    .disk("uploads", DiskConfig::local("./uploads").with_url("/uploads"));

let storage = Storage::with_storage_config(config);
```

## Basic Usage

### Storing Files

```rust
use ferro::Storage;

// Store string content
storage.put("documents/report.txt", "Report content").await?;

// Store bytes
storage.put("images/photo.jpg", image_bytes).await?;

// Store with visibility options
use ferro::PutOptions;

storage.put_with_options(
    "private/secret.txt",
    "secret content",
    PutOptions::new().visibility(Visibility::Private),
).await?;
```

### Retrieving Files

```rust
// Get as bytes
let contents = storage.get("documents/report.txt").await?;

// Get as string
let text = storage.get_string("documents/report.txt").await?;

// Check if file exists
if storage.exists("documents/report.txt").await? {
    println!("File exists!");
}
```

### Deleting Files

```rust
// Delete a single file
storage.delete("temp/cache.txt").await?;

// Delete a directory and all contents
storage.disk("local")?.delete_directory("temp").await?;
```

### Copying and Moving

```rust
// Copy a file
storage.copy("original.txt", "backup/original.txt").await?;

// Move/rename a file
storage.rename("old-name.txt", "new-name.txt").await?;
```

## Multiple Disks

### Switching Disks

```rust
// Use the default disk
storage.put("file.txt", "content").await?;

// Use a specific disk
let public_disk = storage.disk("public")?;
public_disk.put("images/logo.png", logo_bytes).await?;

// Get file from specific disk
let file = storage.disk("uploads")?.get("user-upload.pdf").await?;
```

### Disk Configuration

Each disk is configured independently:

```rust
use ferro::{StorageConfig, DiskConfig};

let config = StorageConfig::new("local")
    // Main storage disk
    .disk("local", DiskConfig::local("./storage/app"))
    // Publicly accessible files
    .disk("public", DiskConfig::local("./storage/public").with_url("/storage"))
    // Temporary files
    .disk("temp", DiskConfig::local("/tmp/app"))
    // Memory disk for testing
    .disk("testing", DiskConfig::memory());
```

## File URLs

### Public URLs

```rust
// Get the public URL for a file
let url = storage.disk("public")?.url("images/logo.png").await?;
// Returns: /storage/images/logo.png

// With a custom URL base
let config = DiskConfig::local("./uploads")
    .with_url("https://cdn.example.com/uploads");
// url() returns: https://cdn.example.com/uploads/images/logo.png
```

### Temporary URLs

For files that need time-limited access:

```rust
use std::time::Duration;

// Get a temporary URL (useful for S3 presigned URLs)
let disk = storage.disk("s3")?;
let temp_url = disk.temporary_url(
    "private/document.pdf",
    Duration::from_secs(3600), // 1 hour
).await?;
```

## File Information

### Metadata

```rust
let disk = storage.disk("local")?;

// Get file size
let size = disk.size("document.pdf").await?;
println!("File size: {} bytes", size);

// Get full metadata
let metadata = disk.metadata("document.pdf").await?;
println!("Path: {}", metadata.path);
println!("Size: {}", metadata.size);
println!("MIME type: {:?}", metadata.mime_type);
println!("Last modified: {:?}", metadata.last_modified);
```

### Visibility

```rust
use ferro::{PutOptions, Visibility};

// Store with private visibility
storage.put_with_options(
    "private/data.json",
    json_data,
    PutOptions::new().visibility(Visibility::Private),
).await?;

// Store with public visibility
storage.put_with_options(
    "public/image.jpg",
    image_data,
    PutOptions::new().visibility(Visibility::Public),
).await?;
```

## Directory Operations

### Listing Files

```rust
let disk = storage.disk("local")?;

// List files in a directory (non-recursive)
let files = disk.files("documents").await?;
for file in files {
    println!("File: {}", file);
}

// List all files recursively
let all_files = disk.all_files("documents").await?;
for file in all_files {
    println!("File: {}", file);
}

// List directories
let dirs = disk.directories("documents").await?;
for dir in dirs {
    println!("Directory: {}", dir);
}
```

### Creating Directories

```rust
let disk = storage.disk("local")?;

// Create a directory
disk.make_directory("uploads/2024/01").await?;

// Delete a directory and contents
disk.delete_directory("temp").await?;
```

## Available Drivers

### Local Driver

Stores files on the local filesystem:

```rust
let config = DiskConfig::local("./storage")
    .with_url("https://example.com/storage");
```

### Memory Driver

Stores files in memory (useful for testing):

```rust
let config = DiskConfig::memory()
    .with_url("https://cdn.example.com");
```

### S3 Driver

Enable the `s3` feature:

```toml
[dependencies]
ferro = { version = "0.1", features = ["s3"] }
```

## Example: File Upload Handler

```rust
use ferro::{Request, Response, Storage};
use std::sync::Arc;

async fn upload_file(
    request: Request,
    storage: Arc<Storage>,
) -> Response {
    // Get uploaded file from multipart form
    let file = request.file("document")?;

    // Generate unique filename
    let filename = format!(
        "uploads/{}/{}",
        chrono::Utc::now().format("%Y/%m/%d"),
        file.name()
    );

    // Store the file
    storage.disk("public")?
        .put(&filename, file.bytes())
        .await?;

    // Get the public URL
    let url = storage.disk("public")?
        .url(&filename)
        .await?;

    Response::json(&serde_json::json!({
        "success": true,
        "url": url,
    }))
}
```

## Example: Avatar Upload with Validation

```rust
use ferro::{Request, Response, Storage, PutOptions, Visibility};
use std::sync::Arc;

async fn upload_avatar(
    request: Request,
    storage: Arc<Storage>,
    user_id: i64,
) -> Response {
    let file = request.file("avatar")?;

    // Validate file type
    let allowed_types = ["image/jpeg", "image/png", "image/webp"];
    if !allowed_types.contains(&file.content_type()) {
        return Response::bad_request("Invalid file type");
    }

    // Validate file size (max 5MB)
    if file.size() > 5 * 1024 * 1024 {
        return Response::bad_request("File too large");
    }

    // Delete old avatar if exists
    let old_path = format!("avatars/{}.jpg", user_id);
    if storage.exists(&old_path).await? {
        storage.delete(&old_path).await?;
    }

    // Store new avatar
    let path = format!("avatars/{}.{}", user_id, file.extension());
    storage.disk("public")?
        .put_with_options(
            &path,
            file.bytes(),
            PutOptions::new().visibility(Visibility::Public),
        )
        .await?;

    let url = storage.disk("public")?.url(&path).await?;

    Response::json(&serde_json::json!({
        "avatar_url": url,
    }))
}
```

## CDN

### CDN Edge URLs

Configure a CDN base URL for a disk to serve stored files from an edge node rather than the storage origin. The primary configuration surface is the provider-agnostic quartet:

```env
# Provider-agnostic CDN quartet (preferred)
# CDN_PROVIDER: none | digitalocean | bunny | cloudflare
CDN_URL=https://cdn.example.com
CDN_PROVIDER=digitalocean
CDN_PURGE_TOKEN=your-api-token
CDN_PURGE_ZONE=your-cdn-endpoint-id
```

`CDN_URL` drives `Disk::cdn_url()`. `CDN_PROVIDER` selects the purge adapter; `CDN_PROVIDER=none` makes `purge()` a logged no-op. `CDN_URL` (display) and `CDN_PROVIDER` (purge) are independent — a deployment can serve assets through a CDN URL with no purge provider configured.

**Deprecated fallbacks (one release window):** the following legacy env vars are still read as fallbacks and emit a `tracing::warn!` deprecation notice on use. Migrate to the quartet.

| Deprecated var | Replacement |
|---|---|
| `AWS_CDN_URL` | `CDN_URL` |
| `BUNNY_CDN_URL` | `CDN_URL` + `CDN_PROVIDER=bunny` |
| `CF_CDN_URL` | `CDN_URL` + `CDN_PROVIDER=cloudflare` |
| `DO_SPACES_CDN_ID` | `CDN_PURGE_ZONE` |
| `CF_ZONE_ID` | `CDN_PURGE_ZONE` |
| `DIGITALOCEAN_ACCESS_TOKEN` | `CDN_PURGE_TOKEN` |
| `CF_API_TOKEN` | `CDN_PURGE_TOKEN` |
| `BUNNY_ACCESS_KEY` | `CDN_PURGE_TOKEN` |

Or set it programmatically:

```rust
use ferro_storage::DiskConfig;

let config = DiskConfig::local("./storage/public")
    .with_url("https://origin.example.com/storage")
    .with_cdn_url("https://cdn.example.com/storage");
```

`Disk::cdn_url(path)` and `Storage::cdn_url(path)` return the CDN edge URL when a CDN base is configured, or fall back to the origin `url()` otherwise:

```rust
// With CDN configured — returns "https://cdn.example.com/storage/images/logo.png"
let url = storage.disk("public")?.cdn_url("images/logo.png").await?;

// Without CDN configured — falls back to origin URL
let url = storage.disk("local")?.cdn_url("images/logo.png").await?;
```

The CDN URL computation is pure string composition at the facade layer — no network call is made. Double slashes are normalized: a trailing slash on the base and a leading slash on the path produce a single slash in the result.

### Cache Invalidation

The `PurgeApi` trait abstracts CDN cache invalidation across providers:

```rust
use ferro_storage::PurgeApi;

#[async_trait]
pub trait PurgeApi: Send + Sync {
    async fn purge(&self, paths: &[String]) -> Result<(), Error>;
}
```

Paths are relative (e.g. `"index.html"`, `"assets/*"`). Implementations handle batching, rate limiting, and full-URL construction internally.

### DigitalOcean Spaces CDN Adapter

The `DoSpacesCdn` adapter is the default, batteries-included implementation. It calls the DigitalOcean CDN purge API (`DELETE /v2/cdn/endpoints/{id}/cache`) and encapsulates:

- **Batching:** at most 50 file paths per request (the DO API limit).
- **Rate limiting:** an internal sliding-window throttle enforces at most 5 requests per 10-second window.
- **Wildcard paths:** `"assets/*"` counts as one file slot, not an expanded set.
- **Missing endpoint id:** when `CDN_PURGE_ZONE` is unset, `purge()` is a logged no-op that returns `Ok(())`. Applications without a CDN endpoint continue to work without error.

```env
CDN_PROVIDER=digitalocean
CDN_PURGE_ZONE=your-cdn-endpoint-id
CDN_PURGE_TOKEN=your-do-api-token
```

```rust
use ferro_storage::{DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};

let purger = DoSpacesCdn::new(DoSpacesCdnConfig::from_env());

// Purge a set of paths after a deployment promote
purger.purge(&[
    "index.html".to_string(),
    "de/index.html".to_string(),
]).await?;
```

The `DIGITALOCEAN_ACCESS_TOKEN` is never written to logs. `DoSpacesCdnConfig` implements a hand-written `Debug` that prints `<redacted>` for the token field.

### Feature-gated Adapters

Bunny CDN and Cloudflare CDN adapters are available behind optional cargo features. They are not compiled into the default dependency graph:

```toml
[dependencies]
ferro-storage = { version = "0.2", features = ["cdn-bunny"] }
# or
ferro-storage = { version = "0.2", features = ["cdn-cloudflare"] }
```

**Bunny CDN** (`cdn-bunny`): calls `POST https://api.bunny.net/purge?url={full_url}&async=false` per path with an `AccessKey` header. Set `CDN_PROVIDER=bunny`, `CDN_URL`, and `CDN_PURGE_TOKEN` (the Bunny access key).

```rust
use ferro_storage::{BunnyCdn, BunnyCdnConfig, PurgeApi};

let purger = BunnyCdn::new(BunnyCdnConfig::from_env());
purger.purge(&["index.html".to_string()]).await?;
```

**Cloudflare CDN** (`cdn-cloudflare`): calls `POST /zones/{zone_id}/purge_cache` with `{"files": [...full_urls...]}` and Bearer auth. Set `CDN_PROVIDER=cloudflare`, `CDN_URL`, `CDN_PURGE_TOKEN` (the CF API token), and `CDN_PURGE_ZONE` (the CF zone id).

```rust
use ferro_storage::{CloudflareCdn, CloudflareCdnConfig, PurgeApi};

let purger = CloudflareCdn::new(CloudflareCdnConfig::from_env());
purger.purge(&["index.html".to_string()]).await?;
```

### Promote → Purge Sequence

The standard deployment workflow is a two-call sequence: promote the new deployment, then purge the affected HTML keys:

```rust
use ferro_storage::{DoSpacesCdn, DoSpacesCdnConfig, PurgeApi};

// After ferro_deployments::promote(...)
let purger = DoSpacesCdn::new(DoSpacesCdnConfig::from_env());
purger.purge(&html_keys).await?;
```

**Purge policy:** purge only the non-hashed HTML keys after a promote. Content-hashed asset URLs (e.g. `app.a1b2c3d4.js`) are immutable — their content never changes at the same URL, so purging them is unnecessary and consumes rate-limit budget. Purging `*` invalidates the entire CDN cache; reserve that for deliberate full-cache invalidation.

## Environment Variables Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `FILESYSTEM_DISK` | Default disk name | `local` |
| `FILESYSTEM_LOCAL_ROOT` | Local disk root path | `./storage` |
| `FILESYSTEM_LOCAL_URL` | Local disk URL base | - |
| `FILESYSTEM_PUBLIC_ROOT` | Public disk root path | `./storage/public` |
| `FILESYSTEM_PUBLIC_URL` | Public disk URL base | `/storage` |
| `AWS_ACCESS_KEY_ID` | S3 access key | - |
| `AWS_SECRET_ACCESS_KEY` | S3 secret key | - |
| `AWS_DEFAULT_REGION` | S3 region | `us-east-1` |
| `AWS_BUCKET` | S3 bucket name | - |
| `AWS_URL` | S3 URL base | - |

## Best Practices

1. **Use meaningful disk names** - `public`, `uploads`, `backups` instead of `disk1`
2. **Set appropriate visibility** - Use private for sensitive files
3. **Organize files by date** - `uploads/2024/01/file.pdf` prevents directory bloat
4. **Use the public disk for web assets** - Images, CSS, JS that need URLs
5. **Use memory driver for tests** - Fast and isolated testing
6. **Clean up temporary files** - Delete files that are no longer needed
7. **Validate uploads** - Check file types and sizes before storing

## MCP Tools

Use `code_templates` with the `storage` category to generate file upload and storage handler patterns.

### `code_templates`

Returns ready-to-use code snippets for file upload handling, including multipart parsing, extension validation, and disk selection. Pass `category: "storage"` to get templates for single-file upload, avatar upload with validation, and temporary URL generation.
