use crate::Error;

/// Configuration for the Stripe integration.
///
/// Load from environment variables with [`StripeConfig::from_env`].
#[derive(Debug, Clone)]
pub struct StripeConfig {
    /// Stripe secret API key (sk_live_xxx or sk_test_xxx).
    pub api_key: String,
    /// Webhook signing secret for the platform webhook endpoint.
    pub webhook_secret: String,
    /// Webhook signing secret for the Connect webhook endpoint (optional).
    pub connect_webhook_secret: Option<String>,
    /// Platform application fee percentage taken from Connect transactions (optional).
    pub application_fee_percent: Option<f64>,
}

impl StripeConfig {
    /// Loads Stripe configuration from environment variables.
    ///
    /// Required: `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`
    /// Optional: `STRIPE_CONNECT_WEBHOOK_SECRET`, `STRIPE_APPLICATION_FEE_PERCENT`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if a required variable is missing.
    pub fn from_env() -> Result<Self, Error> {
        let api_key = std::env::var("STRIPE_SECRET_KEY")
            .map_err(|_| Error::Config("STRIPE_SECRET_KEY not set".to_string()))?;

        let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
            .map_err(|_| Error::Config("STRIPE_WEBHOOK_SECRET not set".to_string()))?;

        let connect_webhook_secret = std::env::var("STRIPE_CONNECT_WEBHOOK_SECRET").ok();

        let application_fee_percent = std::env::var("STRIPE_APPLICATION_FEE_PERCENT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok());

        Ok(Self {
            api_key,
            webhook_secret,
            connect_webhook_secret,
            application_fee_percent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_returns_config_error_when_key_missing() {
        // Remove env vars if set (test isolation)
        std::env::remove_var("STRIPE_SECRET_KEY");
        std::env::remove_var("STRIPE_WEBHOOK_SECRET");
        let result = StripeConfig::from_env();
        assert!(matches!(result, Err(Error::Config(_))));
    }

    #[test]
    fn config_loads_from_provided_values() {
        let config = StripeConfig {
            api_key: "sk_test_xxx".to_string(),
            webhook_secret: "whsec_xxx".to_string(),
            connect_webhook_secret: Some("whsec_connect_xxx".to_string()),
            application_fee_percent: Some(2.5),
        };
        assert_eq!(config.api_key, "sk_test_xxx");
        assert_eq!(config.webhook_secret, "whsec_xxx");
        assert_eq!(
            config.connect_webhook_secret.as_deref(),
            Some("whsec_connect_xxx")
        );
        assert_eq!(config.application_fee_percent, Some(2.5));
    }
}
