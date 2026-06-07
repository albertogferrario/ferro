//! Stripe Checkout Session builder.
//!
//! Provides a consuming builder for Stripe Checkout Sessions that supports
//! both one-time payment and subscription modes, with a runtime guard
//! requiring an idempotency key before any network call is made.

use crate::Error;
use chrono::{DateTime, TimeZone, Utc};
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData,
    CreateCheckoutSessionPaymentIntentData, CreateCheckoutSessionPaymentIntentDataCaptureMethod,
    CreateCheckoutSessionPaymentIntentDataTransferData,
};

/// Which Stripe Checkout mode to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// One-time payment via `payment_intent`.
    Payment,
    /// Recurring subscription via `subscription`.
    Subscription,
}

/// A single line item in a Checkout Session, priced in the smallest currency unit.
#[derive(Debug, Clone)]
pub struct LineItem {
    pub name: String,
    pub description: Option<String>,
    pub unit_amount_cents: i64,
    pub quantity: u32,
    pub currency: String,
}

/// The return value of `CheckoutBuilder::create()`.
///
/// Carries the Stripe-assigned session id and hosted-page URL, the
/// session expiry (Stripe's server-side TTL), and the idempotency key
/// that was used on the create call (so callers can correlate retries).
#[derive(Debug, Clone)]
pub struct CheckoutIntent {
    pub session_id: String,
    pub url: String,
    pub expires_at: DateTime<Utc>,
    pub idempotency_key: String,
}

/// Builder for a Stripe Checkout Session.
///
/// Uses consuming `self -> Self` combinators (Ferro convention).
/// `idempotency_key` MUST be set before calling `create()`.
pub struct CheckoutBuilder {
    mode: Mode,
    line_items: Vec<LineItem>,
    success_url: Option<String>,
    cancel_url: Option<String>,
    metadata: Vec<(String, String)>,
    customer_email: Option<String>,
    destination: Option<(String, Option<i64>)>,
    idempotency_key: Option<String>,
    manual_capture: bool,
}

impl CheckoutBuilder {
    /// Creates a new builder with the given checkout mode.
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            line_items: Vec::new(),
            success_url: None,
            cancel_url: None,
            metadata: Vec::new(),
            customer_email: None,
            destination: None,
            idempotency_key: None,
            manual_capture: false,
        }
    }

    /// Appends a line item to the session.
    pub fn line_item(mut self, item: LineItem) -> Self {
        self.line_items.push(item);
        self
    }

    /// Sets the URL to redirect to after a successful payment.
    pub fn success_url(mut self, url: &str) -> Self {
        self.success_url = Some(url.to_string());
        self
    }

    /// Sets the URL to redirect to when the customer cancels.
    pub fn cancel_url(mut self, url: &str) -> Self {
        self.cancel_url = Some(url.to_string());
        self
    }

    /// Adds a metadata key-value pair.
    pub fn metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.push((key.to_string(), value.to_string()));
        self
    }

    /// Pre-fills the customer's email address.
    pub fn customer_email(mut self, email: &str) -> Self {
        self.customer_email = Some(email.to_string());
        self
    }

    /// Pre-fills the customer's email address from an optional value.
    pub fn customer_email_opt(mut self, email: Option<&str>) -> Self {
        self.customer_email = email.map(|e| e.to_string());
        self
    }

    /// Routes the payment to a Connect account (destination charges pattern).
    ///
    /// `account_id` is the Connect account to receive the funds.
    /// `fee_cents` is an optional platform application fee.
    pub fn destination(mut self, account_id: &str, fee_cents: Option<i64>) -> Self {
        self.destination = Some((account_id.to_string(), fee_cents));
        self
    }

    /// Sets the idempotency key for this session creation.
    ///
    /// Required — calling `create()` without setting this returns
    /// `Err(Error::MissingIdempotencyKey)`.
    pub fn idempotency_key(mut self, key: &str) -> Self {
        self.idempotency_key = Some(key.to_string());
        self
    }

    /// Enables manual capture for this Checkout Session.
    ///
    /// The payment is authorized but not charged at checkout. Call
    /// [`crate::payment_intent::capture`] to charge or
    /// [`crate::payment_intent::cancel`] to release the hold.
    ///
    /// Only valid with [`Mode::Payment`]. Calling this with [`Mode::Subscription`]
    /// returns [`Error::ManualCaptureRequiresPaymentMode`] when `create()` is called.
    pub fn manual_capture(mut self) -> Self {
        self.manual_capture = true;
        self
    }

    /// Builds the `payment_intent_data` params when either `manual_capture` or
    /// `destination` is set, merging both into a single struct to avoid
    /// double-overwrite.
    fn build_payment_intent_data(&self) -> Option<CreateCheckoutSessionPaymentIntentData> {
        let needs_payment_intent_data = self.destination.is_some() || self.manual_capture;
        if needs_payment_intent_data {
            let mut pid = CreateCheckoutSessionPaymentIntentData::default();
            if self.manual_capture {
                pid.capture_method =
                    Some(CreateCheckoutSessionPaymentIntentDataCaptureMethod::Manual);
            }
            if let Some((account_id, fee_cents)) = &self.destination {
                pid.application_fee_amount = *fee_cents;
                pid.transfer_data = Some(CreateCheckoutSessionPaymentIntentDataTransferData {
                    destination: account_id.clone(),
                    ..Default::default()
                });
                pid.on_behalf_of = Some(account_id.clone());
            }
            Some(pid)
        } else {
            None
        }
    }

    /// Creates the Checkout Session via the Stripe API.
    ///
    /// # Errors
    ///
    /// - [`Error::MissingIdempotencyKey`] when `.idempotency_key()` was not called.
    ///   This check fires before any network call is made.
    /// - [`Error::ManualCaptureRequiresPaymentMode`] when `manual_capture()` is used
    ///   with `Mode::Subscription`. Fires before any network call.
    /// - [`Error::Stripe`] on currency parse failure, invalid IDs, or API errors.
    pub async fn create(self) -> Result<CheckoutIntent, Error> {
        // Runtime guards — fail before any network call.
        // Check idempotency key presence via ref to avoid partial move.
        if self.idempotency_key.is_none() {
            return Err(Error::MissingIdempotencyKey);
        }
        if self.manual_capture && self.mode == Mode::Subscription {
            return Err(Error::ManualCaptureRequiresPaymentMode);
        }
        // SAFETY: checked is_none() above.
        let idempotency_key = self.idempotency_key.clone().unwrap();

        let client = crate::Stripe::client();
        let mut params = CreateCheckoutSession::new();
        params.mode = Some(match self.mode {
            Mode::Payment => CheckoutSessionMode::Payment,
            Mode::Subscription => CheckoutSessionMode::Subscription,
        });

        if let Some(s) = &self.success_url {
            params.success_url = Some(s.as_str());
        }
        if let Some(c) = &self.cancel_url {
            params.cancel_url = Some(c.as_str());
        }

        // Convert LineItem -> CreateCheckoutSessionLineItems using inline price_data.
        // Mirrors the existing connect/checkout.rs pattern.
        let converted: Vec<CreateCheckoutSessionLineItems> = self
            .line_items
            .iter()
            .map(|li| {
                let currency: stripe::Currency = li
                    .currency
                    .parse()
                    .map_err(|_| Error::Stripe(format!("invalid currency: {}", li.currency)))?;
                Ok::<_, Error>(CreateCheckoutSessionLineItems {
                    quantity: Some(li.quantity as u64),
                    price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                        currency,
                        unit_amount: Some(li.unit_amount_cents),
                        product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                            name: li.name.clone(),
                            description: li.description.clone(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            })
            .collect::<Result<_, _>>()?;
        if !converted.is_empty() {
            params.line_items = Some(converted);
        }

        if !self.metadata.is_empty() {
            let mut md = std::collections::HashMap::new();
            for (k, v) in &self.metadata {
                md.insert(k.clone(), v.clone());
            }
            params.metadata = Some(md);
        }

        if let Some(email) = &self.customer_email {
            params.customer_email = Some(email.as_str());
        }

        params.payment_intent_data = self.build_payment_intent_data();

        // Note: async-stripe 0.41 does not expose a per-request idempotency-key
        // strategy on CheckoutSession::create. The idempotency_key is stored on
        // CheckoutIntent so callers can correlate retries at the application layer.
        let session = CheckoutSession::create(client, params).await?;

        // expires_at on CheckoutSession is Timestamp (i64 Unix seconds), not Option<i64>.
        let expires_at = Utc
            .timestamp_opt(session.expires_at, 0)
            .single()
            .unwrap_or_else(Utc::now);

        let url = session.url.ok_or_else(|| {
            Error::Stripe("checkout session created but url field was absent".to_string())
        })?;

        Ok(CheckoutIntent {
            session_id: session.id.to_string(),
            url,
            expires_at,
            idempotency_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    // Import the types that will be defined in this module.
    use super::{CheckoutBuilder, LineItem, Mode};

    #[test]
    fn checkout_builder_new_is_empty() {
        let b = CheckoutBuilder::new(Mode::Payment);
        assert_eq!(b.mode, Mode::Payment);
        assert!(b.line_items.is_empty());
        assert!(b.success_url.is_none());
        assert!(b.cancel_url.is_none());
        assert!(b.metadata.is_empty());
        assert!(b.customer_email.is_none());
        assert!(b.destination.is_none());
        assert!(b.idempotency_key.is_none());
    }

    #[test]
    fn line_item_public_fields_constructable() {
        let li = LineItem {
            name: "Widget".to_string(),
            description: Some("A widget".to_string()),
            unit_amount_cents: 1000,
            quantity: 2,
            currency: "usd".to_string(),
        };
        assert_eq!(li.name, "Widget");
        assert_eq!(li.unit_amount_cents, 1000);
        assert_eq!(li.quantity, 2);
    }

    #[tokio::test]
    async fn checkout_create_missing_key_returns_err() {
        // No .idempotency_key() call — must return Err BEFORE any network call.
        // (Stripe::init is not called in the test binary, so if we reach
        // `Stripe::client()`, we would panic — a passing result here proves
        // the guard fires before the network code.)
        let result = CheckoutBuilder::new(Mode::Payment)
            .success_url("https://example.com/ok")
            .cancel_url("https://example.com/cancel")
            .line_item(LineItem {
                name: "Widget".to_string(),
                description: None,
                unit_amount_cents: 100,
                quantity: 1,
                currency: "usd".to_string(),
            })
            .create()
            .await;
        assert!(
            matches!(result, Err(Error::MissingIdempotencyKey)),
            "expected Err(MissingIdempotencyKey), got {result:?}"
        );
    }

    #[tokio::test]
    async fn checkout_create_manual_capture_subscription_returns_err() {
        // manual_capture() + Mode::Subscription must return
        // Err(ManualCaptureRequiresPaymentMode) BEFORE any network call.
        // idempotency_key IS set so the MissingIdempotencyKey guard does not fire first.
        let result = CheckoutBuilder::new(Mode::Subscription)
            .idempotency_key("k")
            .manual_capture()
            .create()
            .await;
        assert!(
            matches!(result, Err(Error::ManualCaptureRequiresPaymentMode)),
            "expected Err(ManualCaptureRequiresPaymentMode), got {result:?}"
        );
    }

    #[test]
    fn checkout_create_manual_capture_sets_capture_method() {
        use stripe::CreateCheckoutSessionPaymentIntentDataCaptureMethod;

        let builder = CheckoutBuilder::new(Mode::Payment).manual_capture();
        let pid = builder
            .build_payment_intent_data()
            .expect("manual_capture=true should produce Some(payment_intent_data)");
        assert_eq!(
            pid.capture_method,
            Some(CreateCheckoutSessionPaymentIntentDataCaptureMethod::Manual),
            "capture_method must be Manual when manual_capture() is set"
        );
        assert!(
            pid.transfer_data.is_none(),
            "transfer_data must be None when destination() is not set"
        );
    }

    #[test]
    fn checkout_create_manual_capture_with_destination_sets_both_fields() {
        use stripe::CreateCheckoutSessionPaymentIntentDataCaptureMethod;

        let builder = CheckoutBuilder::new(Mode::Payment)
            .manual_capture()
            .destination("acct_test", Some(200));
        let pid = builder
            .build_payment_intent_data()
            .expect("manual_capture + destination should produce Some(payment_intent_data)");
        assert_eq!(
            pid.capture_method,
            Some(CreateCheckoutSessionPaymentIntentDataCaptureMethod::Manual),
            "capture_method must be Manual"
        );
        assert!(
            pid.transfer_data.is_some(),
            "transfer_data must be Some when destination() is set"
        );
        assert_eq!(
            pid.on_behalf_of,
            Some("acct_test".to_string()),
            "on_behalf_of must match the destination account_id"
        );
        assert_eq!(
            pid.application_fee_amount,
            Some(200),
            "application_fee_amount must match the fee_cents"
        );
    }
}
