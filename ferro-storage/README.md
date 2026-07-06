# ferro-storage

File storage abstraction for the Ferro framework.

## Features

- Local filesystem driver
- In-memory driver (for testing)
- Amazon S3 driver (with `s3` feature)
- Unified API across all backends

## Usage

```rust
use ferro_storage::{Storage, DiskConfig};

// Create storage with configuration
let storage = Storage::with_config(
    "local",
    vec![
        ("local", DiskConfig::local("storage/app")),
        ("public", DiskConfig::local("storage/public").with_url("/storage")),
    ],
);

// Store a file
storage.put("documents/report.pdf", file_contents).await?;

// Get a file
let contents = storage.get("documents/report.pdf").await?;

// Check existence
if storage.exists("documents/report.pdf").await? {
    println!("File exists!");
}

// Get public URL
let url = storage.disk("public")?.url("images/logo.png").await?;

// Delete a file
storage.delete("documents/old-report.pdf").await?;
```

## Multiple Disks

```rust
// Configure multiple disks
let storage = Storage::with_config(
    "local",
    vec![
        ("local", DiskConfig::local("storage/app")),
        ("s3", DiskConfig::s3("my-bucket", "us-east-1")),
    ],
);

// Use specific disk
let s3 = storage.disk("s3")?;
s3.put("backups/data.json", data).await?;
```

## S3 Support

Enable the `s3` feature:

```toml
[dependencies]
ferro-storage = { version = "0.2", features = ["s3"] }
```

## License

MIT
