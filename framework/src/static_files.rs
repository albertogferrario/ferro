use bytes::Bytes;
use http_body_util::Full;
use std::path::Path;

/// Try to serve a static file from the given base directory.
///
/// Returns `None` if the path is invalid, the file doesn't exist,
/// or security checks fail (dotfiles, directory traversal).
pub(crate) async fn try_serve_from_dir(
    base_dir: &Path,
    request_path: &str,
) -> Option<hyper::Response<Full<Bytes>>> {
    // Reject empty paths and paths with null bytes
    if request_path.is_empty() || request_path.contains('\0') {
        return None;
    }

    // Reject dotfiles and hidden directories (prevents .env, .git/, etc.)
    if request_path
        .split('/')
        .any(|segment| segment.starts_with('.'))
    {
        return None;
    }

    // Build filesystem path
    let relative_path = request_path.trim_start_matches('/');
    let file_path = base_dir.join(relative_path);

    // Canonicalize both paths for directory traversal protection
    let canonical_base = base_dir.canonicalize().ok()?;
    let canonical_file = file_path.canonicalize().ok()?;

    if !canonical_file.starts_with(&canonical_base) {
        return None;
    }

    // Don't serve directories
    if canonical_file.is_dir() {
        return None;
    }

    // Read the file
    let bytes = tokio::fs::read(&canonical_file).await.ok()?;

    // Detect MIME type from file extension
    let content_type = mime_guess::from_path(&canonical_file)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Differentiated cache headers:
    // - /assets/* : Vite hashed output, immutable
    // - Everything else: must-revalidate (favicon.ico, robots.txt, etc.)
    let cache_control = if request_path.starts_with("/assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=0, must-revalidate"
    };

    let response = hyper::Response::builder()
        .status(200)
        .header("Content-Type", &content_type)
        .header("Content-Length", bytes.len().to_string())
        .header("Cache-Control", cache_control)
        .body(Full::new(Bytes::from(bytes)))
        .unwrap();

    Some(response)
}

/// Try to serve a static file from the `public/` directory.
///
/// This is the entry point called from `server.rs` for unmatched routes.
pub(crate) async fn try_serve_static_file(
    request_path: &str,
) -> Option<hyper::Response<Full<Bytes>>> {
    try_serve_from_dir(Path::new("public"), request_path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::TempDir;

    // --- Path validation tests (no filesystem needed) ---

    #[tokio::test]
    async fn empty_path_returns_none() {
        let dir = TempDir::new().unwrap();
        assert!(try_serve_from_dir(dir.path(), "").await.is_none());
    }

    #[tokio::test]
    async fn null_byte_path_returns_none() {
        let dir = TempDir::new().unwrap();
        assert!(try_serve_from_dir(dir.path(), "/file\0.txt")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn dotfile_path_returns_none() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".env"), "SECRET=x").unwrap();
        assert!(try_serve_from_dir(dir.path(), "/.env").await.is_none());
    }

    #[tokio::test]
    async fn hidden_directory_returns_none() {
        let dir = TempDir::new().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "data").unwrap();
        assert!(try_serve_from_dir(dir.path(), "/.git/config")
            .await
            .is_none());
    }

    #[tokio::test]
    async fn dotdot_segment_returns_none() {
        let dir = TempDir::new().unwrap();
        // ".." starts with "." so it's caught by the dotfile check
        assert!(try_serve_from_dir(dir.path(), "/../etc/passwd")
            .await
            .is_none());
    }

    // --- Filesystem integration tests ---

    #[tokio::test]
    async fn serves_existing_file_with_correct_content_type() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("style.css"), "body {}").unwrap();

        let resp = try_serve_from_dir(dir.path(), "/style.css").await.unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.headers()["Content-Type"], "text/css");
        assert_eq!(resp.headers()["Content-Length"], "7");

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"body {}");
    }

    #[tokio::test]
    async fn assets_path_gets_immutable_cache() {
        let dir = TempDir::new().unwrap();
        let assets_dir = dir.path().join("assets");
        fs::create_dir(&assets_dir).unwrap();
        fs::write(assets_dir.join("main.js"), "console.log()").unwrap();

        let resp = try_serve_from_dir(dir.path(), "/assets/main.js")
            .await
            .unwrap();
        assert_eq!(
            resp.headers()["Cache-Control"],
            "public, max-age=31536000, immutable"
        );
    }

    #[tokio::test]
    async fn root_file_gets_must_revalidate_cache() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("favicon.ico"), [0u8; 4]).unwrap();

        let resp = try_serve_from_dir(dir.path(), "/favicon.ico")
            .await
            .unwrap();
        assert_eq!(
            resp.headers()["Cache-Control"],
            "public, max-age=0, must-revalidate"
        );
    }

    #[tokio::test]
    async fn nonexistent_file_returns_none() {
        let dir = TempDir::new().unwrap();
        assert!(try_serve_from_dir(dir.path(), "/nope.txt").await.is_none());
    }

    #[tokio::test]
    async fn directory_path_returns_none() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        assert!(try_serve_from_dir(dir.path(), "/subdir").await.is_none());
    }

    #[tokio::test]
    async fn binary_file_not_corrupted() {
        let dir = TempDir::new().unwrap();
        // Write bytes that are invalid UTF-8
        let binary_data: Vec<u8> = (0..=255).collect();
        fs::write(dir.path().join("data.bin"), &binary_data).unwrap();

        let resp = try_serve_from_dir(dir.path(), "/data.bin").await.unwrap();
        assert_eq!(resp.headers()["Content-Type"], "application/octet-stream");

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], &binary_data[..]);
    }

    #[tokio::test]
    async fn nonexistent_base_dir_returns_none() {
        let result = try_serve_from_dir(Path::new("/nonexistent/dir"), "/file.txt").await;
        assert!(result.is_none());
    }
}
