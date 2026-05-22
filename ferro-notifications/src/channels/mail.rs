//! Mail notification channel.

use serde::{Deserialize, Serialize};

/// Per-attachment size limit (25 MB), per CONTEXT.md D-11.
///
/// This is a framework-level cap — provider-specific caps (e.g. Resend's 40 MB
/// total per email) are surfaced by the carrier and NOT duplicated here.
pub const MAX_ATTACHMENT_BYTES: usize = 25 * 1024 * 1024;

/// A binary attachment for a mail message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailAttachment {
    /// Filename as it should appear to the recipient (e.g. "invoice.pdf").
    pub filename: String,
    /// MIME content-type (e.g. "application/pdf").
    pub content_type: String,
    /// Raw bytes of the attachment.
    pub content: Vec<u8>,
}

/// A mail message for email notifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MailMessage {
    /// Email subject line.
    pub subject: String,
    /// Plain text body.
    pub body: String,
    /// Optional HTML body.
    pub html: Option<String>,
    /// From address (if different from default).
    pub from: Option<String>,
    /// From display name override (e.g. tenant business name). Overrides `MailConfig::from_name`.
    pub from_name: Option<String>,
    /// Reply-to address.
    pub reply_to: Option<String>,
    /// CC recipients.
    pub cc: Vec<String>,
    /// BCC recipients.
    pub bcc: Vec<String>,
    /// Custom headers.
    pub headers: Vec<(String, String)>,
    /// Inline attachments. Per CONTEXT.md D-09, all attachments are in-memory `Vec<u8>`.
    #[serde(default)]
    pub attachments: Vec<MailAttachment>,
}

impl MailMessage {
    /// Create a new empty mail message.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the subject line.
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = subject.into();
        self
    }

    /// Set the plain text body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Set the HTML body.
    pub fn html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// Set the from address.
    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Override the sender display name for this message (e.g. tenant business name).
    /// Takes precedence over `MailConfig::from_name`.
    pub fn from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }

    /// Set the reply-to address.
    pub fn reply_to(mut self, reply_to: impl Into<String>) -> Self {
        self.reply_to = Some(reply_to.into());
        self
    }

    /// Add a CC recipient.
    pub fn cc(mut self, email: impl Into<String>) -> Self {
        self.cc.push(email.into());
        self
    }

    /// Add a BCC recipient.
    pub fn bcc(mut self, email: impl Into<String>) -> Self {
        self.bcc.push(email.into());
        self
    }

    /// Add a custom header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Add an attachment to the mail message.
    ///
    /// Returns `Err(Error::AttachmentTooLarge { .. })` if `content.len()` exceeds
    /// [`MAX_ATTACHMENT_BYTES`] (25 MB) per CONTEXT.md D-11. The cap is per-attachment;
    /// no cumulative cap is enforced (Resend's 40 MB total is the carrier's responsibility).
    ///
    /// Multiple calls accumulate.
    pub fn attachment(
        mut self,
        filename: impl Into<String>,
        content_type: impl Into<String>,
        content: Vec<u8>,
    ) -> Result<Self, crate::Error> {
        let filename = filename.into();
        if content.len() > MAX_ATTACHMENT_BYTES {
            return Err(crate::Error::AttachmentTooLarge {
                filename,
                size: content.len(),
                limit: MAX_ATTACHMENT_BYTES,
            });
        }
        self.attachments.push(MailAttachment {
            filename,
            content_type: content_type.into(),
            content,
        });
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mail_message_builder() {
        let mail = MailMessage::new()
            .subject("Welcome!")
            .body("Hello, welcome to our service.")
            .html("<h1>Hello!</h1>")
            .from("noreply@example.com")
            .cc("manager@example.com")
            .bcc("archive@example.com");

        assert_eq!(mail.subject, "Welcome!");
        assert_eq!(mail.body, "Hello, welcome to our service.");
        assert_eq!(mail.html, Some("<h1>Hello!</h1>".into()));
        assert_eq!(mail.from, Some("noreply@example.com".into()));
        assert_eq!(mail.cc, vec!["manager@example.com"]);
        assert_eq!(mail.bcc, vec!["archive@example.com"]);
        assert!(mail.attachments.is_empty());
    }

    #[test]
    fn test_mail_attachment_under_limit_succeeds() {
        let mail = MailMessage::new()
            .attachment("a.pdf", "application/pdf", vec![0u8; 1024])
            .expect("under limit should succeed");
        assert_eq!(mail.attachments.len(), 1);
        assert_eq!(mail.attachments[0].filename, "a.pdf");
        assert_eq!(mail.attachments[0].content_type, "application/pdf");
        assert_eq!(mail.attachments[0].content.len(), 1024);
    }

    #[test]
    fn test_mail_attachment_at_exact_limit_succeeds() {
        let mail = MailMessage::new()
            .attachment(
                "edge.bin",
                "application/octet-stream",
                vec![0u8; MAX_ATTACHMENT_BYTES],
            )
            .expect("exactly at limit must succeed (limit is inclusive)");
        assert_eq!(mail.attachments.len(), 1);
        assert_eq!(mail.attachments[0].content.len(), MAX_ATTACHMENT_BYTES);
    }

    #[test]
    fn test_mail_attachment_over_limit_returns_typed_error() {
        let oversize = vec![0u8; MAX_ATTACHMENT_BYTES + 1];
        let result = MailMessage::new().attachment("big.pdf", "application/pdf", oversize);
        match result {
            Err(crate::Error::AttachmentTooLarge {
                filename,
                size,
                limit,
            }) => {
                assert_eq!(filename, "big.pdf");
                assert_eq!(size, MAX_ATTACHMENT_BYTES + 1);
                assert_eq!(limit, MAX_ATTACHMENT_BYTES);
            }
            other => panic!("expected AttachmentTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn test_mail_attachment_accumulates() {
        let mail = MailMessage::new()
            .attachment("a.pdf", "application/pdf", vec![1, 2, 3])
            .unwrap()
            .attachment("b.pdf", "application/pdf", vec![4, 5, 6])
            .unwrap()
            .attachment("c.txt", "text/plain", b"hello".to_vec())
            .unwrap();
        assert_eq!(mail.attachments.len(), 3);
        assert_eq!(mail.attachments[0].filename, "a.pdf");
        assert_eq!(mail.attachments[1].filename, "b.pdf");
        assert_eq!(mail.attachments[2].filename, "c.txt");
        assert_eq!(mail.attachments[2].content, b"hello".to_vec());
    }

    #[test]
    fn test_mail_message_serde_round_trip_with_attachments() {
        let mail = MailMessage::new()
            .subject("with attachment")
            .body("body")
            .attachment("hi.txt", "text/plain", b"hello".to_vec())
            .unwrap();
        let json = serde_json::to_string(&mail).unwrap();
        let back: MailMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.subject, "with attachment");
        assert_eq!(back.attachments.len(), 1);
        assert_eq!(back.attachments[0].filename, "hi.txt");
        assert_eq!(back.attachments[0].content, b"hello".to_vec());
    }

    #[test]
    fn test_mail_message_default_has_empty_attachments() {
        let mail = MailMessage::default();
        assert!(mail.attachments.is_empty());
    }

    #[test]
    fn test_max_attachment_bytes_constant() {
        assert_eq!(MAX_ATTACHMENT_BYTES, 26_214_400);
    }
}
