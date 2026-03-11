use crate::Error;
use stripe::{
    AccountLink, AccountLinkType, CheckoutSession, CheckoutSessionMode, CreateAccountLink,
    CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCheckoutSessionPaymentIntentData,
    CreateCheckoutSessionPaymentIntentDataTransferData,
};

/// Creates a Stripe Connect Checkout Session for a one-time payment.
///
/// Uses the destination charges pattern: payment processed on behalf of the connected
/// account, with an optional application fee going to the platform.
pub async fn create_connect_checkout(
    connect_account_id: &str,
    price_cents: i64,
    currency: &str,
    success_url: &str,
    cancel_url: &str,
    application_fee_cents: Option<i64>,
) -> Result<String, Error> {
    let client = crate::Stripe::client();

    let mut params = CreateCheckoutSession::new();
    params.success_url = Some(success_url);
    params.cancel_url = Some(cancel_url);
    params.mode = Some(CheckoutSessionMode::Payment);
    params.line_items = Some(vec![CreateCheckoutSessionLineItems {
        quantity: Some(1),
        price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
            currency: currency
                .parse()
                .map_err(|_| Error::Stripe(format!("invalid currency: {currency}")))?,
            unit_amount: Some(price_cents),
            product_data: Some(stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                name: "Payment".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }]);

    params.payment_intent_data = Some(CreateCheckoutSessionPaymentIntentData {
        application_fee_amount: application_fee_cents,
        transfer_data: Some(CreateCheckoutSessionPaymentIntentDataTransferData {
            destination: connect_account_id.to_string(),
            ..Default::default()
        }),
        on_behalf_of: Some(connect_account_id.to_string()),
        ..Default::default()
    });

    let session = CheckoutSession::create(client, params).await?;
    Ok(session.url.unwrap_or_default())
}

/// Creates a Stripe Connect account link for onboarding a connected account.
///
/// Returns a time-limited, single-use onboarding URL. Redirect the tenant to this URL
/// to complete Stripe Connect onboarding (KYC, bank account, etc.).
pub async fn create_account_link(
    account_id: &str,
    refresh_url: &str,
    return_url: &str,
) -> Result<String, Error> {
    let client = crate::Stripe::client();

    let account: stripe::AccountId = account_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid account id: {account_id}")))?;

    let mut params = CreateAccountLink::new(account, AccountLinkType::AccountOnboarding);
    params.refresh_url = Some(refresh_url);
    params.return_url = Some(return_url);

    let link = AccountLink::create(client, params).await?;
    Ok(link.url)
}
