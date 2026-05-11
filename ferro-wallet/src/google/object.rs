//! Google Wallet `eventTicketObject` JSON construction.
//!
//! The single per-pass JSON object Google Wallet's save endpoint consumes. The class
//! it points at (`classId`) is the v1 fixed `"{issuer_id}.booking"` per D-07; per-pass
//! identity lives in `id = "{issuer_id}.{subject.serial()}"`.
//!
//! No unit tests in this file — the full pipeline (object → JWT → roundtrip-decode)
//! is exercised in `tests/google_jwt.rs` (PLAN-08).

use serde_json::{json, Value};

use super::jwt::pass_type_id_default;
use super::GoogleWalletBuilder;
use crate::subject::WalletSubject;
use crate::WalletError;

/// Build the `eventTicketObject` JSON for a single subject (D-07 ID format + spec §3 shape).
///
/// - `id = "{issuer_id}.{subject.serial()}"`
/// - `classId = "{issuer_id}.{pass_type_id_default()}"` (any `.` in the suffix mapped to `_`;
///   no-op for `"booking"`, kept future-proof).
/// - `state = "active"`.
/// - `barcode = { type: "qrCode", value: subject.barcode_token() }`.
/// - `ticketHolderName = primary.value`.
/// - `eventName.defaultValue = { language: "en", value: primary.value }`.
///
/// # Errors
///
/// Never errors in v1 — every field is derivable from the supplied subject + builder.
/// The `Result` return is retained for forward compatibility with future fields that
/// may validate input (e.g. timezone parse for `dateTime`).
pub(crate) fn build_event_ticket_object<S: WalletSubject>(
    builder: &GoogleWalletBuilder,
    subject: &S,
) -> Result<Value, WalletError> {
    // Future-proof the dot-substitution rule from D-07 (no-op for the v1 "booking" suffix).
    let class_suffix = pass_type_id_default().replace('.', "_");
    let class_id = format!("{}.{}", builder.issuer_id, class_suffix);
    let object_id = format!("{}.{}", builder.issuer_id, subject.serial());

    let primary = subject.primary();

    Ok(json!({
        "id": object_id,
        "classId": class_id,
        "state": "active",
        "barcode": {
            "type": "qrCode",
            "value": subject.barcode_token(),
        },
        "ticketHolderName": primary.value,
        "eventName": {
            "defaultValue": {
                "language": "en",
                "value": primary.value,
            }
        }
    }))
}
