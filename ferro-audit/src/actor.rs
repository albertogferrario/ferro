//! `AuditActor` — typed actor enum stringly-keyed to keep `ferro-audit`
//! domain-agnostic (D-05).
//!
//! The DB representation is `(actor_kind: String, actor_id: Option<String>)`.
//! `actor_kind` is the snake_case variant name returned by [`AuditActor::kind`];
//! `actor_id` is the contained string returned by [`AuditActor::id`], or `NULL`
//! for `System` and `Anonymous` (no specific identity).

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuditActor {
    /// Concrete end-user. `String` is consumer-chosen: `i64.to_string()`,
    /// `Uuid` rendered, slug — anything stable for the consumer.
    User(String),

    /// Background process with no specific user identity (cron, queue worker,
    /// system-driven mutation). Persists `actor_id = NULL`.
    System,

    /// Queued job — the contained string is the job name
    /// (e.g. `"stripe.webhook.subscription_updated"`).
    Job(String),

    /// API client — the contained string is the API key id / OAuth client id.
    ApiClient(String),

    /// Unauthenticated public action (rare but valid). Persists `actor_id = NULL`.
    Anonymous,
}

impl AuditActor {
    /// Returns the snake_case actor kind. Persisted in the `actor_kind` column.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::User(_) => "user",
            Self::System => "system",
            Self::Job(_) => "job",
            Self::ApiClient(_) => "api_client",
            Self::Anonymous => "anonymous",
        }
    }

    /// Returns the actor id, if the variant carries one. `System` and
    /// `Anonymous` return `None` and persist `actor_id = NULL`.
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::User(id) | Self::Job(id) | Self::ApiClient(id) => Some(id.as_str()),
            Self::System | Self::Anonymous => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_kind_and_id() {
        let a = AuditActor::User("u_42".into());
        assert_eq!(a.kind(), "user");
        assert_eq!(a.id(), Some("u_42"));
    }

    #[test]
    fn system_kind_and_id() {
        let a = AuditActor::System;
        assert_eq!(a.kind(), "system");
        assert_eq!(a.id(), None);
    }

    #[test]
    fn job_kind_and_id() {
        let a = AuditActor::Job("stripe.webhook.subscription_updated".into());
        assert_eq!(a.kind(), "job");
        assert_eq!(a.id(), Some("stripe.webhook.subscription_updated"));
    }

    #[test]
    fn api_client_kind_and_id() {
        let a = AuditActor::ApiClient("oauth_client_xyz".into());
        assert_eq!(a.kind(), "api_client");
        assert_eq!(a.id(), Some("oauth_client_xyz"));
    }

    #[test]
    fn anonymous_kind_and_id() {
        let a = AuditActor::Anonymous;
        assert_eq!(a.kind(), "anonymous");
        assert_eq!(a.id(), None);
    }
}
