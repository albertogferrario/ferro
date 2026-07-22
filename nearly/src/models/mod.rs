//! Application models.
//!
//! Each model is a SeaORM entity plus Ferro's `Model`/`ModelMut` query helpers.
//! Timestamps are stored as ISO-8601 strings (matching the framework's SQLite
//! conventions), so migrations use `.timestamp()` columns.

pub mod place;
pub mod presence;
pub mod profile;
pub mod trillo;
pub mod user;

/// Current UTC time as an ISO-8601 string, used for timestamp columns.
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
