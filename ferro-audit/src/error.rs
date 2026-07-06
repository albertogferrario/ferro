//! `AuditError` — the single error type for the ferro-audit crate.
//!
//! Every variant's `Display` impl prefixes `"audit: …"` so production
//! log greps stay surgical (matches the workspace convention used by
//! `GuardedError` with `"guarded: …"`, `WalletError` with `"wallet: …"`,
//! `ConfigError` with `"config: …"`).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    /// The builder was executed with an empty `action` string. An audit
    /// entry with no action is uninterpretable — the action verb is the
    /// only required field on every audit entry (D-10, D-16).
    #[error("audit: action is required")]
    MissingAction,

    /// Underlying SeaORM database error.
    #[error("audit: db error: {0}")]
    Db(#[from] sea_orm::DbErr),

    /// JSON serialization / deserialization error on the `before` / `after`
    /// payload. In practice this fires only if a caller hands the builder
    /// a malformed `serde_json::Value` (very rare; included for completeness).
    #[error("audit: json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_missing_action_displays_message() {
        assert_eq!(
            AuditError::MissingAction.to_string(),
            "audit: action is required"
        );
    }

    #[test]
    fn db_from_sea_orm_dberr() {
        let db_err = sea_orm::DbErr::Custom("test".into());
        let audit_err: AuditError = AuditError::from(db_err);
        assert!(matches!(audit_err, AuditError::Db(_)));
        assert!(audit_err.to_string().starts_with("audit: db error: "));
    }

    #[test]
    fn json_from_serde_json_error() {
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let audit_err: AuditError = AuditError::from(json_err);
        assert!(matches!(audit_err, AuditError::Json(_)));
        assert!(audit_err
            .to_string()
            .starts_with("audit: json serialization error: "));
    }
}
