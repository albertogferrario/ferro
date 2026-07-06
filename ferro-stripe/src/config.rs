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

    /// Computes the platform application fee for a Connect destination charge.
    ///
    /// Returns `Some(round(amount_cents * percent / 100))` when
    /// [`StripeConfig::application_fee_percent`] is set to a strictly positive
    /// value, and `None` when the percentage is unset or non-positive
    /// (`<= 0.0`). The returned fee is clamped to `[0, amount_cents]` so it is
    /// never negative and never exceeds the charge amount.
    ///
    /// Feed the result directly into
    /// [`CheckoutBuilder::destination`](crate::CheckoutBuilder::destination):
    ///
    /// ```ignore
    /// let fee = Stripe::config().application_fee_for(amount_cents);
    /// CheckoutBuilder::new(Mode::Payment).destination(account_id, fee);
    /// ```
    pub fn application_fee_for(&self, amount_cents: i64) -> Option<i64> {
        let percent = self.application_fee_percent?;
        if percent <= 0.0 {
            return None;
        }
        let fee = (amount_cents as f64 * percent / 100.0).round() as i64;
        Some(fee.clamp(0, amount_cents.max(0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_returns_config_error_when_key_missing() {
        // Save existing values so other tests are not affected by the removal.
        let old_key = std::env::var("STRIPE_SECRET_KEY").ok();
        let old_secret = std::env::var("STRIPE_WEBHOOK_SECRET").ok();

        std::env::remove_var("STRIPE_SECRET_KEY");
        std::env::remove_var("STRIPE_WEBHOOK_SECRET");

        let result = StripeConfig::from_env();
        assert!(matches!(result, Err(Error::Config(_))));

        // Restore to avoid polluting the process-global env for other tests.
        if let Some(k) = old_key {
            std::env::set_var("STRIPE_SECRET_KEY", k);
        }
        if let Some(s) = old_secret {
            std::env::set_var("STRIPE_WEBHOOK_SECRET", s);
        }
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

    fn config_with_percent(percent: Option<f64>) -> StripeConfig {
        StripeConfig {
            api_key: "sk_test_xxx".to_string(),
            webhook_secret: "whsec_xxx".to_string(),
            connect_webhook_secret: None,
            application_fee_percent: percent,
        }
    }

    #[test]
    fn application_fee_for_returns_none_when_unset() {
        let config = config_with_percent(None);
        assert_eq!(config.application_fee_for(2000), None);
    }

    #[test]
    fn application_fee_for_returns_none_when_zero_percent() {
        let config = config_with_percent(Some(0.0));
        assert_eq!(config.application_fee_for(2000), None);
    }

    #[test]
    fn application_fee_for_returns_none_when_negative_percent() {
        let config = config_with_percent(Some(-1.0));
        assert_eq!(config.application_fee_for(2000), None);
    }

    #[test]
    fn application_fee_for_computes_normal_case() {
        // 5% of 2000 cents = 100 cents.
        let config = config_with_percent(Some(5.0));
        assert_eq!(config.application_fee_for(2000), Some(100));
    }

    #[test]
    fn application_fee_for_rounds_to_nearest_cent() {
        // 2.5% of 1999 = 49.975 → rounds to 50.
        let config = config_with_percent(Some(2.5));
        assert_eq!(config.application_fee_for(1999), Some(50));
    }

    #[test]
    fn application_fee_for_clamps_to_amount_upper_bound() {
        // 150% of 2000 = 3000, clamped down to the charge amount.
        let config = config_with_percent(Some(150.0));
        assert_eq!(config.application_fee_for(2000), Some(2000));
    }

    #[test]
    fn application_fee_for_clamps_negative_amount_to_zero() {
        // A non-positive charge amount can never yield a negative fee.
        let config = config_with_percent(Some(5.0));
        assert_eq!(config.application_fee_for(0), Some(0));
        assert_eq!(config.application_fee_for(-100), Some(0));
    }
}
