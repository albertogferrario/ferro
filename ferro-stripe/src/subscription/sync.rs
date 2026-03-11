use super::{SubscriptionInfo, SubscriptionStatus};
use chrono::{TimeZone, Utc};

/// Maps a Stripe subscription object to our `SubscriptionInfo` type.
///
/// Used by webhook handlers to update the local tenant_billing state.
/// The plan name is resolved via [`plan_from_subscription`].
pub fn subscription_info_from_stripe(sub: &stripe::Subscription) -> SubscriptionInfo {
    let status = map_status(sub.status);

    let trial_ends_at = sub
        .trial_end
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let current_period_end = Utc
        .timestamp_opt(sub.current_period_end, 0)
        .single()
        .unwrap_or_else(Utc::now);

    let plan = plan_from_subscription(sub);

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

/// Resolves the plan name from a Stripe subscription.
///
/// Resolution order:
/// 1. `metadata["plan"]` on the subscription — explicit plan tag set by your billing code
/// 2. Price `nickname` from the first subscription item — set in your Stripe Dashboard
/// 3. `"unknown"` — fallback when no plan information is available
///
/// Store the resolved plan name in your `tenant_billing` table to use with
/// [`plan_satisfies`](super::plan_satisfies) for access control.
pub fn plan_from_subscription(sub: &stripe::Subscription) -> String {
    // 1. Check subscription metadata for explicit plan tag
    if let Some(plan) = sub.metadata.get("plan") {
        if !plan.is_empty() {
            return plan.clone();
        }
    }

    // 2. Check price nickname on the first item
    if let Some(nickname) = sub
        .items
        .data
        .first()
        .and_then(|item| item.price.as_ref())
        .and_then(|price| price.nickname.as_deref())
    {
        if !nickname.is_empty() {
            return nickname.to_string();
        }
    }

    // 3. Fallback
    "unknown".to_string()
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
