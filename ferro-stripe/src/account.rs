//! Stripe Connect account operations — create, link, retrieve, billing portal.
//!
//! Consolidates functions previously scattered across the product-axis
//! `connect/` and `subscription/` modules into a single capability-axis
//! surface. `create_link` and `billing_portal_url` are preserved verbatim
//! from those modules. `create_account` and `retrieve_account` are new.

use crate::Error;
use stripe::{
    AccountLink, AccountLinkType, BillingPortalSession, CreateAccountLink,
    CreateBillingPortalSession,
};

/// Creates a new Stripe Connect account (Standard type by default).
///
/// Returns the created `stripe::Account`. Follow up with [`create_link`]
/// to generate an onboarding URL.
pub async fn create_account() -> Result<stripe::Account, Error> {
    let client = crate::Stripe::client();
    let mut params = stripe::CreateAccount::new();
    params.type_ = Some(stripe::AccountType::Standard);
    let account = stripe::Account::create(client, params).await?;
    Ok(account)
}

/// Retrieves a Stripe Connect account by id.
pub async fn retrieve_account(account_id: &str) -> Result<stripe::Account, Error> {
    let client = crate::Stripe::client();
    let id: stripe::AccountId = account_id
        .parse()
        .map_err(|_| Error::Stripe(format!("invalid account id: {account_id}")))?;
    let account = stripe::Account::retrieve(client, &id, &[]).await?;
    Ok(account)
}

/// Creates a Stripe Connect account onboarding link.
///
/// Returns a time-limited, single-use onboarding URL. Redirect the tenant to
/// this URL to complete Stripe Connect onboarding (KYC, bank account, etc.).
///
/// Preserved verbatim from the retired `connect::checkout::create_account_link`.
pub async fn create_link(
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

/// Creates a Stripe Billing Portal session for self-service subscription management.
///
/// Returns the hosted portal URL to redirect the tenant to.
///
/// Preserved verbatim from the retired `subscription::checkout::billing_portal_url`.
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
