//! `AuditTarget` — what the audit entry is about (D-07).
//!
//! Two stringly-typed fields: `kind` (dotted-namespace convention per D-08
//! — e.g. `"inventory.unit"`, `"user"`, `"checkout.session"`) and `id`
//! (consumer-stringified primary key). Domain-agnostic by design: a closed
//! enum would force every consumer to upstream their target variants.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditTarget {
    /// Dotted-or-snake namespace, e.g. `"inventory.unit"`, `"user"`,
    /// `"checkout.session"`. Convention only — not enforced at compile time.
    pub kind: String,

    /// Consumer-stringified primary key (any string that uniquely identifies
    /// the target within `kind`).
    pub id: String,
}

impl AuditTarget {
    /// Construct a new `AuditTarget` from any string-like kind and any
    /// stringifiable id.
    pub fn new(kind: impl Into<String>, id: impl ToString) -> Self {
        Self {
            kind: kind.into(),
            id: id.to_string(),
        }
    }
}

impl<K: Into<String>, I: ToString> From<(K, I)> for AuditTarget {
    fn from((kind, id): (K, I)) -> Self {
        Self::new(kind, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_constructs_from_str_and_int() {
        let t = AuditTarget::new("inventory.unit", 42_i64);
        assert_eq!(t.kind, "inventory.unit");
        assert_eq!(t.id, "42");
    }

    #[test]
    fn new_constructs_from_string_and_string() {
        let t = AuditTarget::new(String::from("user"), String::from("u_42"));
        assert_eq!(t.kind, "user");
        assert_eq!(t.id, "u_42");
    }

    #[test]
    fn from_tuple_constructs_target() {
        let t: AuditTarget = ("checkout.session", "ses_abc123").into();
        assert_eq!(t.kind, "checkout.session");
        assert_eq!(t.id, "ses_abc123");
    }

    #[test]
    fn from_tuple_with_numeric_id() {
        let t: AuditTarget = ("inventory.unit", 7_u32).into();
        assert_eq!(t.kind, "inventory.unit");
        assert_eq!(t.id, "7");
    }
}
