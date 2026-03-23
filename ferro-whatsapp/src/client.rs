use crate::{Error, Message, SendResult, WhatsAppConfig};
use std::sync::OnceLock;

static WA_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static WA_CONFIG: OnceLock<WhatsAppConfig> = OnceLock::new();

/// Static facade for the WhatsApp Business Cloud API client.
///
/// Initialize once at application startup with [`WhatsApp::init`],
/// then call [`WhatsApp::send`] from any handler.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_whatsapp::{WhatsApp, WhatsAppConfig, Message};
///
/// let config = WhatsAppConfig::from_env(Box::new(|phone| phone == "393401234567"))
///     .expect("WhatsApp config not set");
/// WhatsApp::init(config);
///
/// // Later, in a handler
/// let result = WhatsApp::send("393409999999", Message::Text { body: "Hello!".into() }).await?;
/// println!("Sent: {}", result.wamid);
/// ```
pub struct WhatsApp;

impl WhatsApp {
    /// Initializes the static WhatsApp client with the given configuration.
    ///
    /// Safe to call multiple times; subsequent calls are no-ops.
    pub fn init(config: WhatsAppConfig) {
        let client = reqwest::Client::new();
        WA_CLIENT.set(client).ok();
        WA_CONFIG.set(config).ok();
    }

    /// Returns a reference to the WhatsApp configuration.
    ///
    /// # Panics
    ///
    /// Panics if [`WhatsApp::init`] has not been called.
    pub fn config() -> &'static WhatsAppConfig {
        WA_CONFIG.get().expect("WhatsApp::init() not called")
    }

    /// Sends a WhatsApp message to the given recipient.
    ///
    /// # Errors
    ///
    /// Returns typed [`Error`] variants for rate limits, auth failures,
    /// invalid phone numbers, network errors, and API errors.
    ///
    /// # Panics
    ///
    /// Panics if [`WhatsApp::init`] has not been called.
    pub async fn send(to: &str, message: Message) -> Result<SendResult, Error> {
        let config = Self::config();
        let client = WA_CLIENT.get().expect("WhatsApp::init() not called");
        send_message(client, config, to, message).await
    }
}

/// Sends a message via the Meta Cloud API.
async fn send_message(
    client: &reqwest::Client,
    config: &WhatsAppConfig,
    to: &str,
    message: Message,
) -> Result<SendResult, Error> {
    let payload = message.to_api_payload(to);
    let url = config.api_url();

    let response = client
        .post(&url)
        .bearer_auth(&config.access_token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    let status = response.status();
    let body_text = response
        .text()
        .await
        .map_err(|e| Error::NetworkError(e.to_string()))?;

    if status.is_success() {
        let body: serde_json::Value =
            serde_json::from_str(&body_text).map_err(|e| Error::ApiError {
                status: status.as_u16(),
                message: e.to_string(),
            })?;
        let wamid = parse_wamid(&body)?;
        Ok(SendResult { wamid })
    } else {
        Err(map_response_error(status.as_u16(), &body_text))
    }
}

/// Extracts the wamid from a successful Meta API response.
fn parse_wamid(body: &serde_json::Value) -> Result<String, Error> {
    body["messages"][0]["id"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| Error::ApiError {
            status: 200,
            message: "missing messages[0].id in response".into(),
        })
}

/// Maps HTTP status codes to typed [`Error`] variants.
fn map_response_error(status: u16, body: &str) -> Error {
    match status {
        429 => Error::RateLimit,
        401 => Error::AuthError,
        400 if body.to_lowercase().contains("invalid") => Error::InvalidNumber,
        _ => Error::ApiError {
            status,
            message: body.to_string(),
        },
    }
}

/// Returns the (url, payload) for a given message and phone_number_id, for testing.
#[cfg(any(test, feature = "test-helpers"))]
pub(crate) fn build_api_payload(
    to: &str,
    message: &Message,
    phone_number_id: &str,
) -> (String, serde_json::Value) {
    use crate::config::META_API_VERSION;
    let url = format!("https://graph.facebook.com/{META_API_VERSION}/{phone_number_id}/messages");
    let payload = message.to_api_payload(to);
    (url, payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn build_api_payload_forms_correct_url() {
        let msg = Message::Text {
            body: "test".into(),
        };
        let (url, payload) = build_api_payload("393401234567", &msg, "11223344");
        assert_eq!(url, "https://graph.facebook.com/v23.0/11223344/messages");
        assert_eq!(payload["to"], "393401234567");
    }

    #[test]
    fn parse_wamid_extracts_from_success_response() {
        let body = serde_json::json!({
            "messaging_product": "whatsapp",
            "contacts": [{"input": "393401234567", "wa_id": "393401234567"}],
            "messages": [{"id": "wamid.HBgNMzkzNDAxMjM0NTY3FQIAERgSM2Y3NDk"}]
        });
        let wamid = parse_wamid(&body).unwrap();
        assert_eq!(wamid, "wamid.HBgNMzkzNDAxMjM0NTY3FQIAERgSM2Y3NDk");
    }

    #[test]
    fn parse_wamid_returns_api_error_when_missing() {
        let body = serde_json::json!({"messages": []});
        let err = parse_wamid(&body).unwrap_err();
        assert!(matches!(err, Error::ApiError { status: 200, .. }));
    }

    #[test]
    fn map_response_error_429_returns_rate_limit() {
        let err = map_response_error(429, "rate limit exceeded");
        assert!(matches!(err, Error::RateLimit));
    }

    #[test]
    fn map_response_error_401_returns_auth_error() {
        let err = map_response_error(401, "unauthorized");
        assert!(matches!(err, Error::AuthError));
    }

    #[test]
    fn map_response_error_400_with_invalid_returns_invalid_number() {
        let err = map_response_error(400, r#"{"error": "invalid phone number format"}"#);
        assert!(matches!(err, Error::InvalidNumber));
    }

    #[test]
    fn map_response_error_500_returns_api_error() {
        let err = map_response_error(500, "internal server error");
        assert!(matches!(err, Error::ApiError { status: 500, .. }));
    }
}
