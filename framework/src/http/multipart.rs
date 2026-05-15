//! Multipart/form-data parsing utilities for HTTP requests.
//!
//! Provides `MultipartForm` and `UploadedFile` types, the `parse_multipart_body`
//! helper, and the `validate_mime` / `validate_size` free functions used by
//! handlers to accept file uploads. Mirrors the body-parsing pattern in
//! `body.rs` but operates on the raw `hyper::body::Incoming` stream so the
//! `multer` crate can iterate fields without buffering the full request body.

use crate::error::FrameworkError;
use bytes::Bytes;
use ferro_storage::{Disk, PutOptions};
use futures_util::StreamExt;
use http_body_util::BodyStream;
use hyper::body::Incoming;
use std::collections::HashMap;
use std::path::Path;

/// A single uploaded file extracted from a multipart/form-data request.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// Name of the form field this file was attached to (e.g. `"avatar"`).
    pub field_name: String,
    /// Original filename from the part's `Content-Disposition` header, if present.
    pub file_name: Option<String>,
    /// MIME type from the part headers, if present.
    pub content_type: Option<String>,
    /// Buffered file content.
    pub bytes: Bytes,
}

impl UploadedFile {
    /// Size of the uploaded payload in bytes.
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    /// File extension derived from `file_name` via `std::path::Path::extension()`.
    ///
    /// Returns `None` when `file_name` is `None` or has no extension.
    pub fn extension(&self) -> Option<&str> {
        self.file_name
            .as_deref()
            .and_then(|n| Path::new(n).extension())
            .and_then(|e| e.to_str())
    }

    /// `true` if `content_type` is present and starts with `"image/"`.
    pub fn is_image(&self) -> bool {
        self.content_type
            .as_deref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }

    /// Persist the buffered bytes to the given storage disk.
    ///
    /// The content type stored alongside the object defaults to
    /// `"application/octet-stream"` when this file has no declared MIME type.
    /// The caller is responsible for selecting the disk via
    /// `storage.disk("public")?` (or another configured name).
    ///
    /// # Security
    ///
    /// `path` is passed verbatim to the storage driver. Callers MUST sanitize
    /// any user-supplied component (e.g. `self.file_name`) before constructing
    /// the path — this method does not perform path-traversal checks.
    pub async fn store(&self, disk: &Disk, path: &str) -> Result<(), ferro_storage::Error> {
        let opts = PutOptions::new().content_type(
            self.content_type
                .as_deref()
                .unwrap_or("application/octet-stream"),
        );
        disk.put_with_options(path, self.bytes.clone(), opts).await
    }
}

/// A parsed multipart/form-data body.
///
/// Holds every file part keyed by form field name as well as every text part.
#[derive(Debug)]
pub struct MultipartForm {
    pub(crate) files_map: HashMap<String, Vec<UploadedFile>>,
    pub(crate) text_fields: HashMap<String, String>,
}

impl MultipartForm {
    /// First file uploaded under `field`, if any.
    pub fn file(&self, field: &str) -> Option<&UploadedFile> {
        self.files_map.get(field).and_then(|v| v.first())
    }

    /// All files uploaded under `field`. Returns an empty slice if the field
    /// is absent.
    pub fn files(&self, field: &str) -> &[UploadedFile] {
        self.files_map
            .get(field)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Value of the text field `name`, if present.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.text_fields.get(name).map(|s| s.as_str())
    }

    /// All text fields keyed by name.
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.text_fields
    }
}

/// Parse a `hyper::body::Incoming` request body as multipart/form-data.
///
/// This is the low-level entry point. `Request::multipart()` and
/// `Request::file()` (added in plan 02) are the public-facing wrappers.
///
/// Bridges `Incoming` (which does not implement `futures::Stream` in
/// hyper 1.x) to the stream interface multer expects via
/// `http_body_util::BodyStream` + `StreamExt::filter_map`.
// Called by Request::multipart() added in plan 02.
#[allow(dead_code)]
pub(crate) async fn parse_multipart_body(
    body: Incoming,
    content_type: &str,
    max_file_bytes: u64,
    max_fields: usize,
) -> Result<MultipartForm, FrameworkError> {
    let boundary = multer::parse_boundary(content_type).map_err(|_| {
        FrameworkError::internal("Content-Type is not multipart/form-data or missing boundary")
    })?;

    let body_stream = BodyStream::new(body)
        .filter_map(|result| async move { result.map(|frame| frame.into_data().ok()).transpose() });

    let constraints =
        multer::Constraints::new().size_limit(multer::SizeLimit::new().per_field(max_file_bytes));

    let mut multipart = multer::Multipart::with_constraints(body_stream, boundary, constraints);

    let mut files_map: HashMap<String, Vec<UploadedFile>> = HashMap::new();
    let mut text_fields: HashMap<String, String> = HashMap::new();
    let mut field_count: usize = 0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| FrameworkError::internal(format!("Multipart parse error: {e}")))?
    {
        field_count += 1;
        if field_count > max_fields {
            return Err(FrameworkError::internal(
                "Too many fields in multipart request",
            ));
        }

        let field_name = field.name().map(|s| s.to_string()).unwrap_or_default();
        let file_name = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|m| m.to_string());
        let bytes = field
            .bytes()
            .await
            .map_err(|e| FrameworkError::internal(format!("Field read error: {e}")))?;

        if file_name.is_some() {
            files_map
                .entry(field_name.clone())
                .or_default()
                .push(UploadedFile {
                    field_name,
                    file_name,
                    content_type,
                    bytes,
                });
        } else {
            text_fields.insert(field_name, String::from_utf8_lossy(&bytes).into_owned());
        }
    }

    Ok(MultipartForm {
        files_map,
        text_fields,
    })
}

/// Reject the file if its declared MIME type is not in `allowed`.
///
/// A file with no `content_type` is treated as the empty string and will
/// only pass if `allowed` contains `""` — which is never useful, so callers
/// should treat content_type-less uploads as rejections.
pub fn validate_mime(file: &UploadedFile, allowed: &[&str]) -> Result<(), FrameworkError> {
    let ct = file.content_type.as_deref().unwrap_or("");
    if allowed.contains(&ct) {
        Ok(())
    } else {
        Err(FrameworkError::internal(format!(
            "File type '{ct}' is not allowed; accepted: {}",
            allowed.join(", ")
        )))
    }
}

/// Reject the file if `file.size() > max_bytes`.
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

/// Read the per-field byte limit from `UPLOAD_MAX_SIZE_MB` (default 10 MiB).
// Called by Request::multipart() added in plan 02.
#[allow(dead_code)]
pub(crate) fn max_file_bytes() -> u64 {
    std::env::var("UPLOAD_MAX_SIZE_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10)
        * 1024
        * 1024
}

/// Read the per-request field limit from `UPLOAD_MAX_FIELDS` (default 100).
// Called by Request::multipart() added in plan 02.
#[allow(dead_code)]
pub(crate) fn max_fields() -> usize {
    std::env::var("UPLOAD_MAX_FIELDS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100)
}
