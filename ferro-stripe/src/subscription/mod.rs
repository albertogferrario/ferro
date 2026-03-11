/// Subscription status variants matching Stripe's 8 possible states.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    /// Subscription is in trial period.
    Trialing,
    /// Subscription is active and paid.
    Active,
    /// Payment pending for first invoice.
    Incomplete,
    /// First invoice payment failed and expired.
    IncompleteExpired,
    /// Payment failed on renewal invoice.
    PastDue,
    /// Subscription was canceled.
    Canceled,
    /// Invoice payment failed multiple times.
    Unpaid,
    /// Subscription is paused (pause_collection).
    Paused,
}

/// Subscription state for a tenant, loaded from the tenant_billing table.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubscriptionInfo {
    /// Stripe subscription ID (sub_xxx).
    pub stripe_subscription_id: String,
    /// Plan identifier: "free" | "pro" | "enterprise".
    pub plan: String,
    /// Stripe subscription status.
    pub status: SubscriptionStatus,
    /// When the trial ends (None if not on trial).
    pub trial_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    /// True when subscription is canceled but billing period hasn't ended yet.
    pub cancel_at_period_end: bool,
    /// When the current billing period ends.
    pub current_period_end: chrono::DateTime<chrono::Utc>,
    /// Stripe Connect account ID for this tenant (None if not connected).
    pub stripe_connect_account_id: Option<String>,
}

impl SubscriptionInfo {
    /// Returns true when the subscription is in a trial period.
    pub fn on_trial(&self) -> bool {
        self.status == SubscriptionStatus::Trialing
    }

    /// Returns true when the subscription grants access (active or trialing).
    pub fn subscribed(&self) -> bool {
        matches!(
            self.status,
            SubscriptionStatus::Active | SubscriptionStatus::Trialing
        )
    }

    /// Returns true when the subscription is scheduled to cancel but the period is still active.
    pub fn on_grace_period(&self) -> bool {
        self.cancel_at_period_end && self.subscribed()
    }
}

/// Returns true if `tenant_plan` satisfies `required_plan` in the plan hierarchy.
///
/// Plan ordering: enterprise > pro > free. Higher tiers satisfy lower requirements.
/// Unknown plans only satisfy themselves.
///
/// # Examples
///
/// ```
/// use ferro_stripe::plan_satisfies;
///
/// assert!(plan_satisfies("enterprise", "pro"));
/// assert!(plan_satisfies("pro", "free"));
/// assert!(!plan_satisfies("free", "pro"));
/// assert!(plan_satisfies("free", "free"));
/// ```
pub fn plan_satisfies(tenant_plan: &str, required_plan: &str) -> bool {
    const TIERS: &[&str] = &["free", "pro", "enterprise"];
    let tenant_rank = TIERS.iter().position(|&p| p == tenant_plan);
    let required_rank = TIERS.iter().position(|&p| p == required_plan);
    match (tenant_rank, required_rank) {
        (Some(t), Some(r)) => t >= r,
        _ => tenant_plan == required_plan,
    }
}

pub mod checkout;
pub mod sync;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info(status: SubscriptionStatus, cancel_at_period_end: bool) -> SubscriptionInfo {
        SubscriptionInfo {
            stripe_subscription_id: "sub_test".to_string(),
            plan: "pro".to_string(),
            status,
            trial_ends_at: None,
            cancel_at_period_end,
            current_period_end: chrono::Utc::now(),
            stripe_connect_account_id: None,
        }
    }

    #[test]
    fn on_trial_returns_true_only_for_trialing() {
        assert!(make_info(SubscriptionStatus::Trialing, false).on_trial());
        assert!(!make_info(SubscriptionStatus::Active, false).on_trial());
        assert!(!make_info(SubscriptionStatus::Incomplete, false).on_trial());
        assert!(!make_info(SubscriptionStatus::IncompleteExpired, false).on_trial());
        assert!(!make_info(SubscriptionStatus::PastDue, false).on_trial());
        assert!(!make_info(SubscriptionStatus::Canceled, false).on_trial());
        assert!(!make_info(SubscriptionStatus::Unpaid, false).on_trial());
        assert!(!make_info(SubscriptionStatus::Paused, false).on_trial());
    }

    #[test]
    fn subscribed_returns_true_for_active_and_trialing() {
        assert!(make_info(SubscriptionStatus::Active, false).subscribed());
        assert!(make_info(SubscriptionStatus::Trialing, false).subscribed());
        assert!(!make_info(SubscriptionStatus::Incomplete, false).subscribed());
        assert!(!make_info(SubscriptionStatus::IncompleteExpired, false).subscribed());
        assert!(!make_info(SubscriptionStatus::PastDue, false).subscribed());
        assert!(!make_info(SubscriptionStatus::Canceled, false).subscribed());
        assert!(!make_info(SubscriptionStatus::Unpaid, false).subscribed());
        assert!(!make_info(SubscriptionStatus::Paused, false).subscribed());
    }

    #[test]
    fn on_grace_period_requires_cancel_at_period_end_and_subscribed() {
        assert!(make_info(SubscriptionStatus::Active, true).on_grace_period());
        assert!(make_info(SubscriptionStatus::Trialing, true).on_grace_period());
        assert!(!make_info(SubscriptionStatus::Active, false).on_grace_period());
        assert!(!make_info(SubscriptionStatus::Trialing, false).on_grace_period());
        assert!(!make_info(SubscriptionStatus::Canceled, true).on_grace_period());
        assert!(!make_info(SubscriptionStatus::PastDue, true).on_grace_period());
        assert!(!make_info(SubscriptionStatus::Unpaid, true).on_grace_period());
        assert!(!make_info(SubscriptionStatus::Paused, true).on_grace_period());
    }

    #[test]
    fn plan_satisfies_enterprise_satisfies_pro_and_free() {
        assert!(plan_satisfies("enterprise", "enterprise"));
        assert!(plan_satisfies("enterprise", "pro"));
        assert!(plan_satisfies("enterprise", "free"));
    }

    #[test]
    fn plan_satisfies_pro_satisfies_free_but_not_enterprise() {
        assert!(plan_satisfies("pro", "pro"));
        assert!(plan_satisfies("pro", "free"));
        assert!(!plan_satisfies("pro", "enterprise"));
    }

    #[test]
    fn plan_satisfies_free_satisfies_only_itself() {
        assert!(plan_satisfies("free", "free"));
        assert!(!plan_satisfies("free", "pro"));
        assert!(!plan_satisfies("free", "enterprise"));
    }

    #[test]
    fn plan_satisfies_unknown_plans_match_exact() {
        assert!(plan_satisfies("custom", "custom"));
        assert!(!plan_satisfies("custom", "pro"));
        assert!(!plan_satisfies("pro", "custom"));
    }

    #[test]
    fn subscription_status_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::PastDue).unwrap(),
            "\"past_due\""
        );
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::IncompleteExpired).unwrap(),
            "\"incomplete_expired\""
        );
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::Trialing).unwrap(),
            "\"trialing\""
        );
    }

    #[test]
    fn subscription_info_serializes_to_json_with_all_fields() {
        let info = make_info(SubscriptionStatus::Active, false);
        let json = serde_json::to_value(&info).unwrap();
        assert!(json["stripe_subscription_id"].is_string());
        assert!(json["plan"].is_string());
        assert!(json["status"].is_string());
        assert!(json["cancel_at_period_end"].is_boolean());
        assert!(json["current_period_end"].is_string());
    }
}
