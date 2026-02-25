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
