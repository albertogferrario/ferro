//! Payment intent status enum — TEXT-backed, five variants.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Lifecycle status of a `payment_intents` row.
///
/// Stored as a `TEXT` column; each variant maps to its snake_case `string_value`.
#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Text")]
pub enum PaymentIntentStatus {
    #[sea_orm(string_value = "reserved")]
    Reserved,
    #[sea_orm(string_value = "paid")]
    Paid,
    #[sea_orm(string_value = "released")]
    Released,
    #[sea_orm(string_value = "failed")]
    Failed,
    #[sea_orm(string_value = "refunded")]
    Refunded,
}

#[cfg(test)]
mod tests {
    use super::PaymentIntentStatus;
    use sea_orm::ActiveEnum;

    #[test]
    fn status_string_values_round_trip() {
        for (variant, s) in [
            (PaymentIntentStatus::Reserved, "reserved"),
            (PaymentIntentStatus::Paid, "paid"),
            (PaymentIntentStatus::Released, "released"),
            (PaymentIntentStatus::Failed, "failed"),
            (PaymentIntentStatus::Refunded, "refunded"),
        ] {
            assert_eq!(variant.to_value(), s.to_string());
            assert_eq!(
                PaymentIntentStatus::try_from_value(&s.to_string()).unwrap(),
                variant
            );
        }
    }
}
