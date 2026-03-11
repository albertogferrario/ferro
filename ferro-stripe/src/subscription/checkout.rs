use crate::Error;
use stripe::{
    BillingPortalSession, CheckoutSession, CheckoutSessionMode, CreateBillingPortalSession,
    CreateCheckoutSession, CreateCheckoutSessionLineItems,
};

/// Creates a Stripe Checkout Session for a subscription upgrade.
///
/// Returns the hosted checkout page URL to redirect the tenant to.
pub async fn create_subscription_checkout(
    customer_id: &str,
    price_id: &str,
    success_url: &str,
    cancel_url: &str,
) -> Result<String, Error> {
    let client = crate::Stripe::client();

    let mut params = CreateCheckoutSession::new();
    params.success_url = Some(success_url);
    params.cancel_url = Some(cancel_url);
    params.customer = Some(
        customer_id
            .parse()
            .map_err(|_| Error::Stripe(format!("invalid customer id: {customer_id}")))?,
    );
    params.mode = Some(CheckoutSessionMode::Subscription);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price: Some(price_id.to_string()),
        ..Default::default()
    }]);

    let session = CheckoutSession::create(client, params).await?;
    Ok(session.url.unwrap_or_default())
}

/// Creates a Stripe Billing Portal session for self-service subscription management.
///
/// Returns the hosted portal URL to redirect the tenant to.
pub async fn billing_portal_url(customer_id: &str, return_url: &str) -> Result<String, Error> {
    let client = crate::Stripe::client();

    let customer_id_parsed: stripe::CustomerId = customer_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid customer id: {customer_id}")))?;

    let mut params = CreateBillingPortalSession::new(customer_id_parsed);
    params.return_url = Some(return_url);

    let session = BillingPortalSession::create(client, params).await?;
    Ok(session.url)
}
