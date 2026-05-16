//! `ReservationContext` — per-call audit metadata bundle (D-29).
//!
//! Bundles four pieces of metadata that the kernel threads to every audit
//! entry it writes during a state transition: the `actor`, an optional
//! `correlation_id` tying the operation to a request / job, an optional
//! `tenant_id` (D-36 stringly-typed), and an optional `reason`.
//!
//! Constructors map directly to the [`ferro_audit::AuditActor`] variants;
//! consuming `with_*` builder methods follow the workspace convention
//! (every setter takes `mut self` and returns `Self`).

use ferro_audit::AuditActor;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ReservationContext {
    pub actor: AuditActor,
    pub correlation_id: Option<Uuid>,
    pub tenant_id: Option<String>,
    pub reason: Option<String>,
}

impl ReservationContext {
    /// Context for system-initiated operations (cron, queue workers,
    /// sweeper-internal calls). Sets `actor = AuditActor::System`; the
    /// audit log will persist `actor_id = NULL` for this entry.
    pub fn system() -> Self {
        Self {
            actor: AuditActor::System,
            correlation_id: None,
            tenant_id: None,
            reason: None,
        }
    }

    /// Context for an end-user-initiated operation. `user_id` is any
    /// consumer-chosen stable identifier (i64.to_string(), Uuid rendered,
    /// slug — anything stable).
    pub fn user(user_id: impl Into<String>) -> Self {
        Self {
            actor: AuditActor::User(user_id.into()),
            correlation_id: None,
            tenant_id: None,
            reason: None,
        }
    }

    /// Context for a queued-job-driven operation. `name` is the job name
    /// (e.g. `"stripe.webhook.payment_failed"`).
    pub fn job(name: impl Into<String>) -> Self {
        Self {
            actor: AuditActor::Job(name.into()),
            correlation_id: None,
            tenant_id: None,
            reason: None,
        }
    }

    /// Context for unauthenticated public actions (rare but valid).
    /// Sets `actor = AuditActor::Anonymous`; the audit log will persist
    /// `actor_id = NULL`.
    pub fn anonymous() -> Self {
        Self {
            actor: AuditActor::Anonymous,
            correlation_id: None,
            tenant_id: None,
            reason: None,
        }
    }

    /// Attach a correlation id (request id, job id, Stripe payment intent
    /// id rendered) to all audit entries written during this state
    /// transition. The audit row's `correlation_id` column is set when
    /// this is `Some` (workspace pattern: never call the audit builder's
    /// `.correlation()` with a synthetic UUID — see RESEARCH Pitfall 4).
    pub fn with_correlation(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Attach a tenant id (D-36). Stored on the reservation row and on
    /// each audit entry. `Option<String>` — pass any string-convertible
    /// value; the audit log filters by `tenant_id IS NULL` when this is
    /// `None`.
    pub fn with_tenant(mut self, t: impl Into<String>) -> Self {
        self.tenant_id = Some(t.into());
        self
    }

    /// Attach a free-form reason recorded on the audit entry (e.g.
    /// `"order_committed"`, `"manual_admin_override"`). Distinct from
    /// `ReleaseReason` (which is a typed enum recorded on the
    /// reservation row + event).
    pub fn with_reason(mut self, r: impl Into<String>) -> Self {
        self.reason = Some(r.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_builder_full_chain() {
        // system() defaults
        let c = ReservationContext::system();
        assert!(matches!(c.actor, AuditActor::System));
        assert!(c.correlation_id.is_none());
        assert!(c.tenant_id.is_none());
        assert!(c.reason.is_none());

        // user(...) populates User variant
        let c = ReservationContext::user("u_42");
        assert!(matches!(c.actor, AuditActor::User(ref s) if s == "u_42"));
        assert!(c.correlation_id.is_none());

        // job(...) populates Job variant
        let c = ReservationContext::job("stripe.webhook.payment_failed");
        assert!(matches!(c.actor, AuditActor::Job(ref s) if s == "stripe.webhook.payment_failed"));

        // anonymous()
        let c = ReservationContext::anonymous();
        assert!(matches!(c.actor, AuditActor::Anonymous));

        // Full builder chain
        let correlation = Uuid::new_v4();
        let c = ReservationContext::user("u_42")
            .with_correlation(correlation)
            .with_tenant("tenant_a")
            .with_reason("manual_override");

        assert!(matches!(c.actor, AuditActor::User(ref s) if s == "u_42"));
        assert_eq!(c.correlation_id, Some(correlation));
        assert_eq!(c.tenant_id.as_deref(), Some("tenant_a"));
        assert_eq!(c.reason.as_deref(), Some("manual_override"));
    }
}
