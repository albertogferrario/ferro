//! `GuardedError` — the single error type for the ferro-orm crate.
//!
//! Every variant's `Display` impl prefixes `"guarded: …"` so production
//! log greps stay surgical (matches the workspace convention used by
//! `WalletError` with `"wallet: …"`, `ConfigError` with `"config: …"`).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuardedError {
    /// The conditional UPDATE matched zero rows — the predicate was not
    /// satisfied. For counter mutations this is the load-bearing
    /// "capacity exhausted" signal.
    #[error("guarded: predicate matched no rows")]
    NoRowsAffected,

    /// The conditional UPDATE matched more than one row — every guarded
    /// update is morally a unique-key-equivalent operation; `>1`
    /// indicates an index/uniqueness bug.
    #[error(
        "guarded: predicate matched {affected} rows (expected 1) — likely an index/uniqueness bug"
    )]
    TooManyRows { affected: u64 },

    /// The builder was executed with no `set_*` calls — a programming
    /// error. Without this guard, sea-orm's `Updater::exec` short-circuits
    /// with `rows_affected: 0`, which would silently look like a
    /// predicate miss (see Pitfall 1 in 152-RESEARCH.md).
    #[error("guarded: no columns to set — builder is empty")]
    EmptyUpdate,

    /// Underlying SeaORM database error.
    #[error("guarded: db error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_no_rows_affected_displays_message() {
        assert_eq!(
            GuardedError::NoRowsAffected.to_string(),
            "guarded: predicate matched no rows"
        );
    }

    #[test]
    fn error_too_many_rows_displays_message() {
        assert_eq!(
            GuardedError::TooManyRows { affected: 3 }.to_string(),
            "guarded: predicate matched 3 rows (expected 1) — likely an index/uniqueness bug"
        );
    }

    #[test]
    fn error_empty_update_displays_message() {
        assert_eq!(
            GuardedError::EmptyUpdate.to_string(),
            "guarded: no columns to set — builder is empty"
        );
    }

    #[test]
    fn db_from_sea_orm_dberr() {
        let db_err = sea_orm::DbErr::Custom("test".into());
        let guarded_err: GuardedError = GuardedError::from(db_err);
        assert!(matches!(guarded_err, GuardedError::Db(_)));
    }
}
