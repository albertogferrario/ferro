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
    CreateCheckoutSessionPaymentIntentData, CreateCheckoutSessionPaymentIntentDataTransferData,
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

    /// Creates the Checkout Session via the Stripe API.
    ///
    /// # Errors
    ///
    /// - [`Error::MissingIdempotencyKey`] when `.idempotency_key()` was not called.
    ///   This check fires before any network call is made.
    /// - [`Error::Stripe`] on currency parse failure, invalid IDs, or API errors.
    pub async fn create(self) -> Result<CheckoutIntent, Error> {
        // Runtime guard — fail before any network call.
        let idempotency_key = self.idempotency_key.ok_or(Error::MissingIdempotencyKey)?;

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

        if let Some((account_id, fee_cents)) = &self.destination {
            params.payment_intent_data = Some(CreateCheckoutSessionPaymentIntentData {
                application_fee_amount: *fee_cents,
                transfer_data: Some(CreateCheckoutSessionPaymentIntentDataTransferData {
                    destination: account_id.clone(),
                    ..Default::default()
                }),
                on_behalf_of: Some(account_id.clone()),
                ..Default::default()
            });
        }

        // Note: async-stripe 0.41 does not expose a per-request idempotency-key
        // strategy on CheckoutSession::create. The idempotency_key is stored on
        // CheckoutIntent so callers can correlate retries at the application layer.
        let session = CheckoutSession::create(client, params).await?;

        // expires_at on CheckoutSession is Timestamp (i64 Unix seconds), not Option<i64>.
        let expires_at = Utc
            .timestamp_opt(session.expires_at, 0)
            .single()
            .unwrap_or_else(Utc::now);

        Ok(CheckoutIntent {
            session_id: session.id.to_string(),
            url: session.url.unwrap_or_default(),
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
}
