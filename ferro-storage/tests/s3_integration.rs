// Run with: cargo test -p ferro-storage --features s3-tests -- --test-threads=1
//
// Requires environment variables:
//   AWS_ACCESS_KEY_ID
//   AWS_SECRET_ACCESS_KEY
//   AWS_BUCKET
//   AWS_DEFAULT_REGION (default: us-east-1)
//   AWS_URL (optional, for DigitalOcean Spaces / MinIO)
//
// Tests are skipped (not failed) when AWS_BUCKET is not set.
#![cfg(feature = "s3-tests")]

use bytes::Bytes;
use ferro_storage::{Disk, PutOptions, Storage, StorageConfig};
use std::time::Duration;

fn setup() -> Storage {
    let config = StorageConfig::from_env();
    Storage::with_storage_config(config)
}

/// Return the S3 disk, or `None` if env vars are not configured (skips the test).
fn s3_disk_or_skip() -> Option<Disk> {
    if std::env::var("AWS_BUCKET").is_err() {
        eprintln!("Skipping S3 integration test: AWS_BUCKET not set");
        return None;
    }
    let storage = setup();
    storage.disk("s3").ok()
}

#[tokio::test]
async fn test_put_get_delete() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let path = "integration-test/hello.txt";
    let content = Bytes::from("hello s3");

    disk.put(path, content.clone()).await.unwrap();

    let retrieved = disk.get(path).await.unwrap();
    assert_eq!(retrieved, content);

    disk.delete(path).await.unwrap();

    let exists = disk.exists(path).await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_exists_missing() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let exists = disk
        .exists("integration-test/nonexistent-key-12345")
        .await
        .unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_put_with_content_type() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let path = "integration-test/image.png";
    let content = Bytes::from(b"\x89PNG\r\n".as_ref());
    let options = PutOptions::new().content_type("image/png");

    disk.put_with_options(path, content, options).await.unwrap();

    let meta = disk.metadata(path).await.unwrap();
    assert_eq!(meta.mime_type.as_deref(), Some("image/png"));

    disk.delete(path).await.unwrap();
}

#[tokio::test]
async fn test_size() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let path = "integration-test/size-test.txt";
    let content = Bytes::from("0123456789");

    disk.put(path, content).await.unwrap();

    let size = disk.size(path).await.unwrap();
    assert_eq!(size, 10);

    disk.delete(path).await.unwrap();
}

#[tokio::test]
async fn test_copy() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let original = "integration-test/copy-original.txt";
    let copy = "integration-test/copy-destination.txt";
    let content = Bytes::from("copy test content");

    disk.put(original, content.clone()).await.unwrap();
    disk.copy(original, copy).await.unwrap();

    let retrieved = disk.get(copy).await.unwrap();
    assert_eq!(retrieved, content);

    disk.delete(original).await.unwrap();
    disk.delete(copy).await.unwrap();
}

#[tokio::test]
async fn test_url() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let path = "integration-test/url-test.txt";
    let bucket = std::env::var("AWS_BUCKET").unwrap_or_default();

    disk.put(path, Bytes::from("url test")).await.unwrap();

    let url = disk.url(path).await.unwrap();
    assert!(!url.is_empty());

    // URL must contain either the bucket name or a configured url_base host
    let url_base = std::env::var("AWS_URL").unwrap_or_default();
    if url_base.is_empty() {
        assert!(
            url.contains(&bucket),
            "URL should contain bucket name: {url}"
        );
    }

    disk.delete(path).await.unwrap();
}

#[tokio::test]
async fn test_temporary_url() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let path = "integration-test/presigned-test.txt";

    disk.put(path, Bytes::from("presigned content"))
        .await
        .unwrap();

    let url = disk
        .temporary_url(path, Duration::from_secs(300))
        .await
        .unwrap();

    assert!(
        url.contains("X-Amz-Signature") || url.contains("x-amz-signature"),
        "presigned URL should contain X-Amz-Signature: {url}"
    );

    disk.delete(path).await.unwrap();
}

#[tokio::test]
async fn test_files_and_directories() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    let file_a = "integration-test/dir/a.txt";
    let file_b = "integration-test/dir/sub/b.txt";

    disk.put(file_a, Bytes::from("file a")).await.unwrap();
    disk.put(file_b, Bytes::from("file b")).await.unwrap();

    // files() returns only direct children (no subdirs)
    let files = disk.files("integration-test/dir").await.unwrap();
    assert!(
        files.iter().any(|k| k.contains("a.txt")),
        "files() should include a.txt: {files:?}"
    );
    assert!(
        !files.iter().any(|k| k.contains("b.txt")),
        "files() should not include b.txt in subdir: {files:?}"
    );

    // directories() returns direct child directories
    let dirs = disk.directories("integration-test/dir").await.unwrap();
    assert!(
        dirs.iter().any(|d| d.contains("sub")),
        "directories() should include sub: {dirs:?}"
    );

    // all_files() returns everything recursively
    let all = disk.all_files("integration-test/dir").await.unwrap();
    assert!(
        all.iter().any(|k| k.contains("a.txt")),
        "all_files() should include a.txt: {all:?}"
    );
    assert!(
        all.iter().any(|k| k.contains("b.txt")),
        "all_files() should include b.txt: {all:?}"
    );

    disk.delete(file_a).await.unwrap();
    disk.delete(file_b).await.unwrap();
}

#[tokio::test]
async fn test_make_delete_directory() {
    let Some(disk) = s3_disk_or_skip() else {
        return;
    };

    disk.make_directory("integration-test/newdir")
        .await
        .unwrap();

    let marker_exists = disk.exists("integration-test/newdir/.keep").await.unwrap();
    assert!(
        marker_exists,
        ".keep marker should exist after make_directory"
    );

    disk.delete_directory("integration-test/newdir")
        .await
        .unwrap();

    let marker_gone = disk.exists("integration-test/newdir/.keep").await.unwrap();
    assert!(
        !marker_gone,
        ".keep marker should be gone after delete_directory"
    );
}
