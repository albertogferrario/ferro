use super::{SubscriptionInfo, SubscriptionStatus};
use chrono::{TimeZone, Utc};

/// Maps a Stripe subscription object to our `SubscriptionInfo` type.
///
/// Used by webhook handlers to update the local tenant_billing state.
pub fn subscription_info_from_stripe(sub: &stripe::Subscription) -> SubscriptionInfo {
    let status = map_status(sub.status);

    let trial_ends_at = sub
        .trial_end
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let current_period_end = Utc
        .timestamp_opt(sub.current_period_end, 0)
        .single()
        .unwrap_or_else(Utc::now);

    // Extract price ID from the first subscription item as plan identifier.
    // In practice the caller resolves plan name from price_id via the billing table.
    let plan = sub
        .items
        .data
        .first()
        .and_then(|item| item.price.as_ref())
        .map(|price| price.id.to_string())
        .unwrap_or_else(|| "free".to_string());

    SubscriptionInfo {
        stripe_subscription_id: sub.id.to_string(),
        plan,
        status,
        trial_ends_at,
        cancel_at_period_end: sub.cancel_at_period_end,
        current_period_end,
        stripe_connect_account_id: None, // loaded from tenant_billing, not Stripe object
    }
}

fn map_status(status: stripe::SubscriptionStatus) -> SubscriptionStatus {
    match status {
        stripe::SubscriptionStatus::Trialing => SubscriptionStatus::Trialing,
        stripe::SubscriptionStatus::Active => SubscriptionStatus::Active,
        stripe::SubscriptionStatus::Incomplete => SubscriptionStatus::Incomplete,
        stripe::SubscriptionStatus::IncompleteExpired => SubscriptionStatus::IncompleteExpired,
        stripe::SubscriptionStatus::PastDue => SubscriptionStatus::PastDue,
        stripe::SubscriptionStatus::Canceled => SubscriptionStatus::Canceled,
        stripe::SubscriptionStatus::Unpaid => SubscriptionStatus::Unpaid,
        stripe::SubscriptionStatus::Paused => SubscriptionStatus::Paused,
    }
}
