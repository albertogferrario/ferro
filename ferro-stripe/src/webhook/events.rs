use ferro_events::Event;

/// Stripe webhook event for `customer.subscription.updated`.
///
/// Emitted when a subscription's status, plan, or billing cycle changes.
#[derive(Debug, Clone)]
pub struct StripeSubscriptionUpdated {
    /// The raw JSON body of the Stripe event.
    pub event_json: String,
    /// The Stripe subscription ID (sub_xxx).
    pub subscription_id: String,
    /// The Stripe customer ID (cus_xxx).
    pub customer_id: String,
}

impl Event for StripeSubscriptionUpdated {
    fn name(&self) -> &'static str {
        "stripe.customer.subscription.updated"
    }
}

/// Stripe webhook event for `customer.subscription.deleted`.
///
/// Emitted when a subscription is canceled and the billing period ends.
#[derive(Debug, Clone)]
pub struct StripeSubscriptionDeleted {
    /// The raw JSON body of the Stripe event.
    pub event_json: String,
    /// The Stripe subscription ID (sub_xxx).
    pub subscription_id: String,
    /// The Stripe customer ID (cus_xxx).
    pub customer_id: String,
}

impl Event for StripeSubscriptionDeleted {
    fn name(&self) -> &'static str {
        "stripe.customer.subscription.deleted"
    }
}

/// Stripe webhook event for `checkout.session.completed`.
///
/// Emitted when a checkout session finishes successfully.
#[derive(Debug, Clone)]
pub struct StripeCheckoutCompleted {
    /// The raw JSON body of the Stripe event.
    pub event_json: String,
    /// The Stripe checkout session ID (cs_xxx).
    pub session_id: String,
    /// The Stripe customer ID if present (cus_xxx).
    pub customer_id: Option<String>,
}

impl Event for StripeCheckoutCompleted {
    fn name(&self) -> &'static str {
        "stripe.checkout.session.completed"
    }
}

/// Stripe webhook event for `invoice.paid`.
///
/// Emitted when an invoice is paid successfully.
#[derive(Debug, Clone)]
pub struct StripeInvoicePaid {
    /// The raw JSON body of the Stripe event.
    pub event_json: String,
    /// The Stripe invoice ID (in_xxx).
    pub invoice_id: String,
    /// The Stripe customer ID (cus_xxx).
    pub customer_id: String,
}

impl Event for StripeInvoicePaid {
    fn name(&self) -> &'static str {
        "stripe.invoice.paid"
    }
}

/// Stripe webhook event for `payment_intent.succeeded` on a Connect account.
///
/// Emitted when a payment intent succeeds on a connected Stripe account.
#[derive(Debug, Clone)]
pub struct StripeConnectPaymentSucceeded {
    /// The raw JSON body of the Stripe event.
    pub event_json: String,
    /// The Stripe payment intent ID (pi_xxx).
    pub payment_intent_id: String,
    /// The connected Stripe account ID (acct_xxx).
    pub connect_account_id: String,
}

impl Event for StripeConnectPaymentSucceeded {
    fn name(&self) -> &'static str {
        "stripe.connect.payment_intent.succeeded"
    }
}

/// Background job that processes a Stripe webhook event asynchronously.
///
/// Webhook handlers dispatch this job immediately after signature verification,
/// returning HTTP 200 to Stripe without blocking on event processing.
/// The job then dispatches the appropriate ferro-events Event based on event_type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessStripeWebhook {
    /// The Stripe event type string (e.g. "customer.subscription.updated").
    pub event_type: String,
    /// The raw JSON body of the Stripe event.
    pub event_json: String,
    /// The connected account ID for Connect webhooks (None for platform webhooks).
    pub connect_account_id: Option<String>,
}

#[ferro_queue::async_trait]
impl ferro_queue::Job for ProcessStripeWebhook {
    async fn handle(&self) -> Result<(), ferro_queue::Error> {
        match self.event_type.as_str() {
            "customer.subscription.updated" => {
                if let Some(event) = parse_subscription_updated(&self.event_json) {
                    event.dispatch_sync();
                }
            }
            "customer.subscription.deleted" => {
                if let Some(event) = parse_subscription_deleted(&self.event_json) {
                    event.dispatch_sync();
                }
            }
            "checkout.session.completed" => {
                if let Some(event) = parse_checkout_completed(&self.event_json) {
                    event.dispatch_sync();
                }
            }
            "invoice.paid" => {
                if let Some(event) = parse_invoice_paid(&self.event_json) {
                    event.dispatch_sync();
                }
            }
            "payment_intent.succeeded" => {
                if let Some(connect_id) = self.connect_account_id.clone() {
                    if let Some(event) =
                        parse_connect_payment_succeeded(&self.event_json, connect_id)
                    {
                        event.dispatch_sync();
                    }
                }
            }
            // Unknown event types are silently ignored — Stripe sends many event types
            // and only a subset are handled by this integration.
            _ => {}
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "ProcessStripeWebhook"
    }
}

fn parse_subscription_updated(event_json: &str) -> Option<StripeSubscriptionUpdated> {
    let v: serde_json::Value = serde_json::from_str(event_json).ok()?;
    let sub = v.get("data")?.get("object")?;
    let subscription_id = sub.get("id")?.as_str()?.to_string();
    let customer_id = sub.get("customer")?.as_str()?.to_string();
    Some(StripeSubscriptionUpdated {
        event_json: event_json.to_string(),
        subscription_id,
        customer_id,
    })
}

fn parse_subscription_deleted(event_json: &str) -> Option<StripeSubscriptionDeleted> {
    let v: serde_json::Value = serde_json::from_str(event_json).ok()?;
    let sub = v.get("data")?.get("object")?;
    let subscription_id = sub.get("id")?.as_str()?.to_string();
    let customer_id = sub.get("customer")?.as_str()?.to_string();
    Some(StripeSubscriptionDeleted {
        event_json: event_json.to_string(),
        subscription_id,
        customer_id,
    })
}

fn parse_checkout_completed(event_json: &str) -> Option<StripeCheckoutCompleted> {
    let v: serde_json::Value = serde_json::from_str(event_json).ok()?;
    let session = v.get("data")?.get("object")?;
    let session_id = session.get("id")?.as_str()?.to_string();
    let customer_id = session
        .get("customer")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());
    Some(StripeCheckoutCompleted {
        event_json: event_json.to_string(),
        session_id,
        customer_id,
    })
}

fn parse_invoice_paid(event_json: &str) -> Option<StripeInvoicePaid> {
    let v: serde_json::Value = serde_json::from_str(event_json).ok()?;
    let invoice = v.get("data")?.get("object")?;
    let invoice_id = invoice.get("id")?.as_str()?.to_string();
    let customer_id = invoice.get("customer")?.as_str()?.to_string();
    Some(StripeInvoicePaid {
        event_json: event_json.to_string(),
        invoice_id,
        customer_id,
    })
}

fn parse_connect_payment_succeeded(
    event_json: &str,
    connect_account_id: String,
) -> Option<StripeConnectPaymentSucceeded> {
    let v: serde_json::Value = serde_json::from_str(event_json).ok()?;
    let pi = v.get("data")?.get("object")?;
    let payment_intent_id = pi.get("id")?.as_str()?.to_string();
    Some(StripeConnectPaymentSucceeded {
        event_json: event_json.to_string(),
        payment_intent_id,
        connect_account_id,
    })
}

/// Generates a valid Stripe-signature header for testing webhook verification.
///
/// Returns `(signature_header, timestamp)` where signature_header is formatted
/// as `t={timestamp},v1={hmac_sha256}` and timestamp is the current Unix time.
///
/// # Example
///
/// ```rust,ignore
/// let (sig, ts) = signed_webhook_payload(r#"{"id":"evt_1"}"#, "whsec_secret");
/// let event = verify_webhook(r#"{"id":"evt_1"}"#, &sig, "whsec_secret");
/// assert!(event.is_ok());
/// ```
pub fn signed_webhook_payload(payload: &str, secret: &str) -> (String, i64) {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let timestamp = chrono::Utc::now().timestamp();
    let signed_payload = format!("{timestamp}.{payload}");

    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(signed_payload.as_bytes());
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    let header = format!("t={timestamp},v1={signature}");
    (header, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_updated_event_name() {
        let event = StripeSubscriptionUpdated {
            event_json: "{}".to_string(),
            subscription_id: "sub_123".to_string(),
            customer_id: "cus_123".to_string(),
        };
        assert_eq!(event.name(), "stripe.customer.subscription.updated");
    }

    #[test]
    fn subscription_deleted_event_name() {
        let event = StripeSubscriptionDeleted {
            event_json: "{}".to_string(),
            subscription_id: "sub_123".to_string(),
            customer_id: "cus_123".to_string(),
        };
        assert_eq!(event.name(), "stripe.customer.subscription.deleted");
    }

    #[test]
    fn checkout_completed_event_name() {
        let event = StripeCheckoutCompleted {
            event_json: "{}".to_string(),
            session_id: "cs_123".to_string(),
            customer_id: None,
        };
        assert_eq!(event.name(), "stripe.checkout.session.completed");
    }

    #[test]
    fn invoice_paid_event_name() {
        let event = StripeInvoicePaid {
            event_json: "{}".to_string(),
            invoice_id: "in_123".to_string(),
            customer_id: "cus_123".to_string(),
        };
        assert_eq!(event.name(), "stripe.invoice.paid");
    }

    #[test]
    fn connect_payment_succeeded_event_name() {
        let event = StripeConnectPaymentSucceeded {
            event_json: "{}".to_string(),
            payment_intent_id: "pi_123".to_string(),
            connect_account_id: "acct_123".to_string(),
        };
        assert_eq!(event.name(), "stripe.connect.payment_intent.succeeded");
    }

    // Compile-time check: all event types are Clone + Send + Sync
    fn _assert_clone_send_sync<T: Clone + Send + Sync>() {}

    #[test]
    fn events_are_clone_send_sync() {
        _assert_clone_send_sync::<StripeSubscriptionUpdated>();
        _assert_clone_send_sync::<StripeSubscriptionDeleted>();
        _assert_clone_send_sync::<StripeCheckoutCompleted>();
        _assert_clone_send_sync::<StripeInvoicePaid>();
        _assert_clone_send_sync::<StripeConnectPaymentSucceeded>();
    }

    #[test]
    fn signed_webhook_payload_generates_valid_signature() {
        let payload = r#"{"id":"evt_test","type":"invoice.paid"}"#;
        let (sig, _ts) = signed_webhook_payload(payload, "whsec_test");
        // Parse the signature to verify format
        assert!(sig.starts_with("t="), "signature should start with t=");
        assert!(sig.contains(",v1="), "signature should contain ,v1=");
    }

    #[test]
    fn process_stripe_webhook_job_name() {
        let job = ProcessStripeWebhook {
            event_type: "invoice.paid".to_string(),
            event_json: "{}".to_string(),
            connect_account_id: None,
        };
        // Job::name() is a method on the Job trait
        use ferro_queue::Job;
        assert_eq!(job.name(), "ProcessStripeWebhook");
    }
}
