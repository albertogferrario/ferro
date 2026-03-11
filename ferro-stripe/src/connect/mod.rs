/// A Stripe Connect account linked to a tenant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectAccount {
    /// Stripe Connect account ID (acct_xxx).
    pub account_id: String,
    /// True when the connected account has completed Stripe onboarding.
    pub onboarding_complete: bool,
}

pub mod checkout;
