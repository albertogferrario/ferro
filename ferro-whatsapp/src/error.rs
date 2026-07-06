/// Errors that can occur in ferro-whatsapp operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error (missing env var or invalid value).
    #[error("configuration error: {0}")]
    Config(String),

    /// Webhook signature verification failed.
    #[error("webhook verification failed: {0}")]
    WebhookVerification(String),

    /// Meta API rate limit exceeded (HTTP 429).
    #[error("rate limit exceeded")]
    RateLimit,

    /// Recipient phone number is invalid or not registered on WhatsApp.
    #[error("invalid phone number")]
    InvalidNumber,

    /// Authentication error — access token invalid or expired (HTTP 401).
    #[error("authentication error")]
    AuthError,

    /// Network-level error contacting the Meta Cloud API.
    #[error("network error: {0}")]
    NetworkError(String),

    /// Meta Cloud API returned a non-2xx response.
    #[error("api error {status}: {message}")]
    ApiError {
        /// HTTP status code from Meta API.
        status: u16,
        /// Error message from Meta API response body.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_config_displays_message() {
        let e = Error::Config("WHATSAPP_APP_SECRET not set".into());
        assert_eq!(
            e.to_string(),
            "configuration error: WHATSAPP_APP_SECRET not set"
        );
    }

    #[test]
    fn error_webhook_verification_displays_message() {
        let e = Error::WebhookVerification("Signature mismatch".into());
        assert_eq!(
            e.to_string(),
            "webhook verification failed: Signature mismatch"
        );
    }

    #[test]
    fn error_rate_limit_displays_correctly() {
        let e = Error::RateLimit;
        assert_eq!(e.to_string(), "rate limit exceeded");
    }

    #[test]
    fn error_invalid_number_displays_correctly() {
        let e = Error::InvalidNumber;
        assert_eq!(e.to_string(), "invalid phone number");
    }

    #[test]
    fn error_auth_error_displays_correctly() {
        let e = Error::AuthError;
        assert_eq!(e.to_string(), "authentication error");
    }

    #[test]
    fn error_network_error_displays_message() {
        let e = Error::NetworkError("connection refused".into());
        assert_eq!(e.to_string(), "network error: connection refused");
    }

    #[test]
    fn error_api_error_displays_status_and_message() {
        let e = Error::ApiError {
            status: 500,
            message: "internal server error".into(),
        };
        assert_eq!(e.to_string(), "api error 500: internal server error");
    }
}
