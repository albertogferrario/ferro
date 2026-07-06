use crate::Error;

/// Meta Cloud API version used for all API calls.
///
/// Update this constant when Meta releases a new API version.
pub const META_API_VERSION: &str = "v23.0";

/// Configuration for the WhatsApp Business Cloud API integration.
///
/// Load from environment variables with [`WhatsAppConfig::from_env`].
///
/// # Phone Number Format
///
/// The `is_owner` closure receives phone numbers in E.164 format **without** the
/// leading `+` (as Meta delivers them, e.g., `393401234567` not `+393401234567`).
pub struct WhatsAppConfig {
    /// Meta app secret for HMAC-SHA256 webhook signature verification.
    pub app_secret: String,
    /// System user access token for sending messages via the Cloud API.
    pub access_token: String,
    /// Phone Number ID from the Meta developer dashboard.
    pub phone_number_id: String,
    /// Verify token for the GET webhook challenge endpoint.
    pub verify_token: String,
    /// Closure that classifies a phone number as owner (returns true) or customer (returns false).
    ///
    /// Receives the phone number in E.164 format without `+` prefix.
    pub is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>,
}

impl WhatsAppConfig {
    /// Loads WhatsApp configuration from environment variables.
    ///
    /// Required: `WHATSAPP_APP_SECRET`, `WHATSAPP_ACCESS_TOKEN`,
    /// `WHATSAPP_PHONE_NUMBER_ID`, `WHATSAPP_VERIFY_TOKEN`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if any required variable is missing.
    pub fn from_env(is_owner: Box<dyn Fn(&str) -> bool + Send + Sync>) -> Result<Self, Error> {
        let app_secret = std::env::var("WHATSAPP_APP_SECRET")
            .map_err(|_| Error::Config("WHATSAPP_APP_SECRET not set".into()))?;
        let access_token = std::env::var("WHATSAPP_ACCESS_TOKEN")
            .map_err(|_| Error::Config("WHATSAPP_ACCESS_TOKEN not set".into()))?;
        let phone_number_id = std::env::var("WHATSAPP_PHONE_NUMBER_ID")
            .map_err(|_| Error::Config("WHATSAPP_PHONE_NUMBER_ID not set".into()))?;
        let verify_token = std::env::var("WHATSAPP_VERIFY_TOKEN")
            .map_err(|_| Error::Config("WHATSAPP_VERIFY_TOKEN not set".into()))?;

        Ok(Self {
            app_secret,
            access_token,
            phone_number_id,
            verify_token,
            is_owner,
        })
    }

    /// Returns the Meta Cloud API URL for sending messages.
    pub fn api_url(&self) -> String {
        format!(
            "https://graph.facebook.com/{META_API_VERSION}/{}/messages",
            self.phone_number_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_fails_when_required_vars_missing() {
        std::env::remove_var("WHATSAPP_APP_SECRET");
        std::env::remove_var("WHATSAPP_ACCESS_TOKEN");
        std::env::remove_var("WHATSAPP_PHONE_NUMBER_ID");
        std::env::remove_var("WHATSAPP_VERIFY_TOKEN");

        let result = WhatsAppConfig::from_env(Box::new(|_| false));
        assert!(matches!(result, Err(Error::Config(_))));
    }

    #[test]
    fn from_env_fails_when_one_var_missing() {
        std::env::set_var("WHATSAPP_APP_SECRET", "secret");
        std::env::set_var("WHATSAPP_ACCESS_TOKEN", "token");
        std::env::set_var("WHATSAPP_PHONE_NUMBER_ID", "12345");
        std::env::remove_var("WHATSAPP_VERIFY_TOKEN");

        let result = WhatsAppConfig::from_env(Box::new(|_| false));
        assert!(matches!(result, Err(Error::Config(_))));

        std::env::remove_var("WHATSAPP_APP_SECRET");
        std::env::remove_var("WHATSAPP_ACCESS_TOKEN");
        std::env::remove_var("WHATSAPP_PHONE_NUMBER_ID");
    }

    #[test]
    fn api_url_includes_phone_number_id_and_version() {
        let config = WhatsAppConfig {
            app_secret: "s".into(),
            access_token: "t".into(),
            phone_number_id: "99887766".into(),
            verify_token: "v".into(),
            is_owner: Box::new(|_| false),
        };
        assert_eq!(
            config.api_url(),
            "https://graph.facebook.com/v23.0/99887766/messages"
        );
    }
}
