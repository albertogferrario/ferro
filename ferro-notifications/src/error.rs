//! Error types for the notification system.

use thiserror::Error;

/// Errors that can occur during notification dispatch.
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to send email notification.
    #[error("mail error: {0}")]
    Mail(String),

    /// Failed to send Slack notification.
    #[error("slack error: {0}")]
    Slack(String),

    /// Failed to store database notification.
    #[error("database error: {0}")]
    Database(String),

    /// Failed to send WhatsApp notification (per CONTEXT.md D-05).
    #[error("whatsapp error: {0}")]
    WhatsApp(#[from] ferro_whatsapp::Error),

    /// Failed to publish an in-app broadcast (per RESEARCH.md §InApp Adapter — ferro_broadcast::Error mapping).
    #[error("broadcast error: {0}")]
    Broadcast(String),

    /// Mail attachment exceeded the 25 MB framework cap (per CONTEXT.md D-11).
    #[error("attachment '{filename}' too large: {size} bytes (limit {limit} bytes)")]
    AttachmentTooLarge {
        /// Original filename of the rejected attachment.
        filename: String,
        /// Size of the rejected attachment in bytes.
        size: usize,
        /// Configured limit in bytes (currently 25 * 1024 * 1024).
        limit: usize,
    },

    /// Channel not configured or available.
    #[error("channel not available: {0}")]
    ChannelNotAvailable(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic notification error.
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a mail error.
    pub fn mail(msg: impl Into<String>) -> Self {
        Self::Mail(msg.into())
    }

    /// Create a slack error.
    pub fn slack(msg: impl Into<String>) -> Self {
        Self::Slack(msg.into())
    }

    /// Create a database error.
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    /// Create a broadcast error.
    pub fn broadcast(msg: impl Into<String>) -> Self {
        Self::Broadcast(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_mail_helper() {
        let e = Error::mail("smtp connect failed");
        assert_eq!(e.to_string(), "mail error: smtp connect failed");
    }

    #[test]
    fn test_error_attachment_too_large_displays_correctly() {
        let e = Error::AttachmentTooLarge {
            filename: "report.pdf".into(),
            size: 30 * 1024 * 1024,
            limit: 25 * 1024 * 1024,
        };
        assert_eq!(
            e.to_string(),
            "attachment 'report.pdf' too large: 31457280 bytes (limit 26214400 bytes)"
        );
    }

    #[test]
    fn test_error_whatsapp_from_impl() {
        let wa_err = ferro_whatsapp::Error::RateLimit;
        let n_err: Error = wa_err.into();
        assert!(matches!(
            n_err,
            Error::WhatsApp(ferro_whatsapp::Error::RateLimit)
        ));
        assert!(n_err.to_string().contains("whatsapp error"));
        assert!(n_err.to_string().contains("rate limit exceeded"));
    }

    #[test]
    fn test_error_broadcast_helper() {
        let e = Error::broadcast("channel disconnected");
        assert_eq!(e.to_string(), "broadcast error: channel disconnected");
    }
}
