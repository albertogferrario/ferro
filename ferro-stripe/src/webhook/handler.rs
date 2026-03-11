use crate::client::Stripe;
use crate::webhook::events::ProcessStripeWebhook;
use crate::webhook::verify_webhook;
use crate::Error;

/// Handles an incoming platform webhook from Stripe.
///
/// Verifies the HMAC signature using the configured `webhook_secret`,
/// then dispatches a [`ProcessStripeWebhook`] job for asynchronous processing.
/// Returns immediately with `Ok(())` to acknowledge the webhook to Stripe.
///
/// The actual event processing (DB updates, cache invalidation) runs in
/// the background job, decoupled from the HTTP response.
///
/// # Errors
///
/// Returns [`Error::WebhookVerification`] if the signature is invalid.
pub async fn handle_platform_webhook(raw_body: &str, signature: &str) -> Result<(), Error> {
    let event = verify_webhook(raw_body, signature, &Stripe::config().webhook_secret)?;

    let job = ProcessStripeWebhook {
        event_type: event.type_.to_string(),
        event_json: raw_body.to_string(),
        connect_account_id: None,
    };

    ferro_queue::dispatch(job)
        .await
        .map_err(|e| Error::Stripe(format!("failed to dispatch webhook job: {e}")))?;

    Ok(())
}

/// Handles an incoming Connect webhook from Stripe.
///
/// Verifies the HMAC signature using the configured `connect_webhook_secret`,
/// then dispatches a [`ProcessStripeWebhook`] job with the connected account ID.
/// Returns immediately with `Ok(())` to acknowledge the webhook to Stripe.
///
/// # Errors
///
/// Returns [`Error::Config`] if `connect_webhook_secret` is not configured.
/// Returns [`Error::WebhookVerification`] if the signature is invalid.
pub async fn handle_connect_webhook(raw_body: &str, signature: &str) -> Result<(), Error> {
    let connect_secret = Stripe::config()
        .connect_webhook_secret
        .as_deref()
        .ok_or_else(|| Error::Config("STRIPE_CONNECT_WEBHOOK_SECRET not set".to_string()))?;

    let event = verify_webhook(raw_body, signature, connect_secret)?;

    // Connect events include the connected account ID in the event.account field.
    let connect_account_id = event.account.map(|id| id.to_string());

    let job = ProcessStripeWebhook {
        event_type: event.type_.to_string(),
        event_json: raw_body.to_string(),
        connect_account_id,
    };

    ferro_queue::dispatch(job)
        .await
        .map_err(|e| Error::Stripe(format!("failed to dispatch webhook job: {e}")))?;

    Ok(())
}
