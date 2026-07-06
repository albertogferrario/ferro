//! `reconstruct_state` — pure JSON fold (D-24, the "replay" primitive).
//!
//! Iterates `entries` in order, folding each entry's `after` JSON into a
//! running state. The fold is a **shallow object merge**: keys from newer
//! entries overwrite older keys at the top level only. Nested objects and
//! arrays are replaced wholesale, not deep-merged.
//!
//! Returns `None` if the slice is empty or no entry has a non-None `after`.
//! Returns `Some(Value::Object(map))` for the typical case (object merge).
//! Returns `Some(non_object)` if any entry's `after` is a non-object value
//! — that value replaces the entire running state from that point on.
//!
//! Consumers needing deep-merge run their own fold over the
//! `Vec<AuditEntry>` — `reconstruct_state` ships shallow-merge in v0.

use serde_json::{Map, Value};

use crate::entry::AuditEntry;

/// Fold the `after` payloads of an audit entry sequence into a single
/// reconstructed state value. See module-level docs for shallow-merge
/// semantics.
pub fn reconstruct_state(entries: &[AuditEntry]) -> Option<Value> {
    let mut state: Map<String, Value> = Map::new();
    let mut seen_any = false;

    for entry in entries {
        match &entry.after {
            Some(Value::Object(after_map)) => {
                for (k, v) in after_map {
                    state.insert(k.clone(), v.clone());
                }
                seen_any = true;
            }
            Some(v) => {
                // Non-object after: replace state wholesale.
                return Some(v.clone());
            }
            None => {}
        }
    }

    if seen_any {
        Some(Value::Object(state))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;
    use uuid::Uuid;

    // Helper: construct a minimal AuditEntry for replay tests. We don't
    // round-trip through the DB; replay is a pure function and only reads
    // the `after` field.
    fn entry_with_after(after: Option<Value>) -> AuditEntry {
        AuditEntry {
            id: Uuid::new_v4(),
            tenant_id: None,
            actor_kind: "system".to_string(),
            actor_id: None,
            action: "test.event".to_string(),
            target_kind: None,
            target_id: None,
            before: None,
            after,
            reason: None,
            correlation_id: None,
            created_at: NaiveDateTime::default(),
        }
    }

    // VALIDATION 153-08-01 case (a): empty slice -> None
    #[test]
    fn reconstruct_state_empty_returns_none() {
        assert_eq!(reconstruct_state(&[]), None);
    }

    // VALIDATION 153-08-01 case (b): all-None afters -> None
    #[test]
    fn reconstruct_state_all_none_returns_none() {
        let entries = vec![
            entry_with_after(None),
            entry_with_after(None),
            entry_with_after(None),
        ];
        assert_eq!(reconstruct_state(&entries), None);
    }

    // VALIDATION 153-08-01 case (c): sequence of object merges
    #[test]
    fn reconstruct_state_object_merges_shallow() {
        let entries = vec![
            entry_with_after(Some(json!({ "quantity": 10, "status": "available" }))),
            entry_with_after(Some(json!({ "quantity": 5 }))),
            entry_with_after(Some(json!({ "status": "reserved", "owner": "u_42" }))),
        ];
        // Final state: quantity=5 (last write), status="reserved" (last write),
        // owner="u_42" (only writer)
        let result = reconstruct_state(&entries).expect("non-empty");
        assert_eq!(
            result,
            json!({ "quantity": 5, "status": "reserved", "owner": "u_42" })
        );
    }

    // VALIDATION 153-08-01 case (d): nested object is replaced wholesale (not deep-merged)
    #[test]
    fn reconstruct_state_nested_object_replaced_wholesale() {
        let entries = vec![
            entry_with_after(Some(json!({ "config": { "retries": 3, "timeout": 30 } }))),
            entry_with_after(Some(json!({ "config": { "retries": 5 } }))),
        ];
        // Shallow merge: the entire `config` object is replaced — timeout=30 is LOST.
        let result = reconstruct_state(&entries).expect("non-empty");
        assert_eq!(result, json!({ "config": { "retries": 5 } }));
    }

    // VALIDATION 153-08-01 case (e): non-object after replaces state wholesale
    #[test]
    fn reconstruct_state_non_object_replaces_state() {
        let entries = vec![
            entry_with_after(Some(json!({ "quantity": 10 }))),
            entry_with_after(Some(json!("DELETED"))), // string after
        ];
        let result = reconstruct_state(&entries).expect("non-empty");
        assert_eq!(result, json!("DELETED"));
    }
}
