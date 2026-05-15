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
pub(crate) fn max_file_bytes() -> u64 {
    std::env::var("UPLOAD_MAX_SIZE_MB")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(10)
        * 1024
        * 1024
}

/// Read the per-request field limit from `UPLOAD_MAX_FIELDS` (default 100).
pub(crate) fn max_fields() -> usize {
    std::env::var("UPLOAD_MAX_FIELDS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http_body_util::{BodyStream, Full};

    /// Build a raw multipart/form-data body and matching Content-Type value.
    ///
    /// Each part is `(name, value, filename)`. `Some(filename)` produces a
    /// file part (Content-Disposition includes `filename="..."` and the bytes
    /// of `value` are placed in the part body); `None` produces a text part.
    fn make_multipart_body(
        boundary: &str,
        parts: &[(&str, &[u8], Option<&str>)],
    ) -> (Bytes, String) {
        let ct = format!("multipart/form-data; boundary={boundary}");
        let mut body: Vec<u8> = Vec::new();
        for (name, value, filename) in parts {
            body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
            match filename {
                Some(fname) => body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{name}\"; filename=\"{fname}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
                    )
                    .as_bytes(),
                ),
                None => body.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n")
                        .as_bytes(),
                ),
            }
            body.extend_from_slice(value);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        (Bytes::from(body), ct)
    }

    /// Mirror of `parse_multipart_body` that accepts an in-memory body so
    /// tests don't need a live `hyper::body::Incoming`.
    async fn parse_for_test(
        raw: Bytes,
        content_type: &str,
        max_bytes: u64,
        max_fields_cap: usize,
    ) -> Result<MultipartForm, FrameworkError> {
        let boundary = multer::parse_boundary(content_type).map_err(|_| {
            FrameworkError::internal("Content-Type is not multipart/form-data or missing boundary")
        })?;

        let body = Full::new(raw);
        let stream = BodyStream::new(body).filter_map(|result| async move {
            result.map(|frame| frame.into_data().ok()).transpose()
        });

        let constraints =
            multer::Constraints::new().size_limit(multer::SizeLimit::new().per_field(max_bytes));

        let mut multipart = multer::Multipart::with_constraints(stream, boundary, constraints);

        let mut files_map: HashMap<String, Vec<UploadedFile>> = HashMap::new();
        let mut text_fields: HashMap<String, String> = HashMap::new();
        let mut field_count: usize = 0;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| FrameworkError::internal(format!("Multipart parse error: {e}")))?
        {
            field_count += 1;
            if field_count > max_fields_cap {
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

    // D-03 / D-04: parsing + accessors

    #[tokio::test]
    async fn multipart_parses_fields() {
        let (raw, ct) = make_multipart_body(
            "BOUNDARY",
            &[
                ("title", b"hello", None),
                ("avatar", b"\x89PNG\r\n\x1a\n", Some("avatar.png")),
            ],
        );
        let form = parse_for_test(raw, &ct, 10 * 1024 * 1024, 100)
            .await
            .expect("parses");
        assert_eq!(form.field("title"), Some("hello"));
        let file = form.file("avatar").expect("avatar present");
        assert_eq!(file.field_name, "avatar");
        assert_eq!(file.file_name.as_deref(), Some("avatar.png"));
        assert_eq!(file.bytes.as_ref(), b"\x89PNG\r\n\x1a\n");
    }

    #[tokio::test]
    async fn multipart_form_accessors() {
        let (raw, ct) = make_multipart_body(
            "B",
            &[
                ("photos", b"AAA", Some("a.jpg")),
                ("photos", b"BBB", Some("b.jpg")),
                ("caption", b"two photos", None),
            ],
        );
        let form = parse_for_test(raw, &ct, 10 * 1024 * 1024, 100)
            .await
            .expect("parses");
        assert_eq!(form.file("photos").unwrap().bytes.as_ref(), b"AAA");
        assert_eq!(form.files("photos").len(), 2);
        assert_eq!(form.files("photos")[1].bytes.as_ref(), b"BBB");
        assert!(form.files("absent").is_empty());
        assert!(form.file("absent").is_none());
        assert_eq!(form.field("caption"), Some("two photos"));
        assert_eq!(form.fields().len(), 1);
    }

    // D-07: UploadedFile fields populated

    #[tokio::test]
    async fn uploaded_file_fields() {
        let (raw, ct) = make_multipart_body("B", &[("doc", b"PDFDATA", Some("report.pdf"))]);
        let form = parse_for_test(raw, &ct, 1024, 100).await.expect("parses");
        let file = form.file("doc").expect("present");
        assert_eq!(file.field_name, "doc");
        assert_eq!(file.file_name.as_deref(), Some("report.pdf"));
        assert_eq!(
            file.content_type.as_deref(),
            Some("application/octet-stream")
        );
        assert_eq!(file.bytes.len(), b"PDFDATA".len());
    }

    // D-08: UploadedFile method coverage

    #[test]
    fn uploaded_file_size_returns_byte_len() {
        let f = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: None,
            bytes: Bytes::from_static(b"12345"),
        };
        assert_eq!(f.size(), 5);
    }

    #[test]
    fn extension_from_filename() {
        let with_ext = UploadedFile {
            field_name: "f".into(),
            file_name: Some("avatar.png".into()),
            content_type: None,
            bytes: Bytes::new(),
        };
        let no_ext = UploadedFile {
            field_name: "f".into(),
            file_name: Some("noext".into()),
            content_type: None,
            bytes: Bytes::new(),
        };
        let none = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: None,
            bytes: Bytes::new(),
        };
        assert_eq!(with_ext.extension(), Some("png"));
        assert_eq!(no_ext.extension(), None);
        assert_eq!(none.extension(), None);
    }

    #[test]
    fn is_image_true_false() {
        let img = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: Some("image/jpeg".into()),
            bytes: Bytes::new(),
        };
        let pdf = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: Some("application/pdf".into()),
            bytes: Bytes::new(),
        };
        let none = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: None,
            bytes: Bytes::new(),
        };
        assert!(img.is_image());
        assert!(!pdf.is_image());
        assert!(!none.is_image());
    }

    // D-18: missing/wrong Content-Type

    #[tokio::test]
    async fn multipart_missing_boundary() {
        let raw = Bytes::from_static(b"irrelevant");
        let err = parse_for_test(raw, "application/json", 1024, 100)
            .await
            .expect_err("must error");
        let msg = format!("{err}");
        assert!(
            msg.contains("Content-Type is not multipart/form-data or missing boundary"),
            "unexpected error message: {msg}"
        );
    }

    // D-12: per-field size limit

    #[tokio::test]
    async fn multipart_size_limit_rejects_oversized_field() {
        let big = vec![b'A'; 50];
        let (raw, ct) = make_multipart_body("B", &[("blob", &big, Some("big.bin"))]);
        let err = parse_for_test(raw, &ct, 10, 100)
            .await
            .expect_err("oversized must error");
        let msg = format!("{err}");
        assert!(
            msg.contains("Multipart parse error") || msg.contains("Field read error"),
            "expected size-limit error from multer, got: {msg}"
        );
    }

    // D-13: per-request field count limit

    #[tokio::test]
    async fn multipart_max_fields_rejects_excess() {
        let (raw, ct) = make_multipart_body(
            "B",
            &[("a", b"1", None), ("b", b"2", None), ("c", b"3", None)],
        );
        let err = parse_for_test(raw, &ct, 1024, 2)
            .await
            .expect_err("must reject excess fields");
        let msg = format!("{err}");
        assert!(
            msg.contains("Too many fields in multipart request"),
            "unexpected error message: {msg}"
        );
    }

    // D-14: validation helpers

    #[test]
    fn validate_mime_accepts_allowed() {
        let f = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: Some("image/png".into()),
            bytes: Bytes::new(),
        };
        validate_mime(&f, &["image/png", "image/jpeg"]).expect("png is allowed");
    }

    #[test]
    fn validate_mime_rejects_disallowed() {
        let f = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: Some("application/x-msdownload".into()),
            bytes: Bytes::new(),
        };
        let err = validate_mime(&f, &["image/png"]).expect_err("must reject exe");
        let msg = format!("{err}");
        assert!(msg.contains("application/x-msdownload"));
        assert!(msg.contains("image/png"));
    }

    #[test]
    fn validate_size_accepts_within_cap() {
        let f = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: None,
            bytes: Bytes::from_static(b"hello"),
        };
        validate_size(&f, 10).expect("5 bytes is within 10");
    }

    #[test]
    fn validate_size_rejects_over_cap() {
        let f = UploadedFile {
            field_name: "f".into(),
            file_name: None,
            content_type: None,
            bytes: Bytes::from_static(b"hello world!!"),
        };
        let err = validate_size(&f, 5).expect_err("13 > 5");
        let msg = format!("{err}");
        assert!(msg.contains("13 bytes"));
        assert!(msg.contains("max 5"));
    }

    // Killer-feature integration: UploadedFile::store() wires to ferro-storage

    #[tokio::test]
    async fn store_to_memory_disk() {
        use ferro_storage::{DiskConfig, Storage};

        let storage = Storage::with_config("mem", vec![("mem", DiskConfig::memory())]);
        let disk = storage.disk("mem").expect("memory disk exists");

        let file = UploadedFile {
            field_name: "avatar".into(),
            file_name: Some("photo.png".into()),
            content_type: Some("image/png".into()),
            bytes: Bytes::from_static(b"\x89PNG\r\n\x1a\n"),
        };

        file.store(&disk, "uploads/photo.png")
            .await
            .expect("store succeeds");

        let stored = disk
            .get("uploads/photo.png")
            .await
            .expect("file readable after store");
        assert_eq!(stored.as_ref(), b"\x89PNG\r\n\x1a\n");
        assert!(disk.exists("uploads/photo.png").await.unwrap());
    }
}
