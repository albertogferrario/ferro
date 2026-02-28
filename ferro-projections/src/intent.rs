use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Structurally-derivable intent classification for a service.
///
/// Intents answer "what IS this service?" based on its structural shape
/// (fields, relationships, state machine), not "what can a user DO?"
///
/// Known variants are tried first during deserialization; any unrecognized
/// string falls through to `Custom(String)`.
///
/// `Custom(String)` must remain the last variant for correct serde deserialization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(
    description = "Structural intent classification. Known variants: browse, focus, collect, process, summarize, analyze, track. Any other string is a custom domain-specific intent."
)]
pub enum Intent {
    /// Collection navigation: has_many relationships, EntityName fields.
    Browse,
    /// Single-entity deep view: FreeText/ImageUrl/Url fields.
    Focus,
    /// Data capture: many writable fields, write_only present.
    Collect,
    /// Workflow with state progression: guarded transitions.
    Process,
    /// Overview dashboard: read-only Money/Percentage/Quantity fields.
    Summarize,
    /// Time-series exploration: DateTime + numeric measures.
    Analyze,
    /// Timeline/audit trail: Status + temporal ordering.
    Track,
    /// Escape hatch for intents not structurally derivable.
    #[serde(untagged)]
    Custom(String),
}

/// A scored intent with confidence and the structural signals that contributed.
///
/// Produced by the structural analysis engine (Phase 89). Confidence ranges
/// from 0.0 (no signal) to 1.0 (strong structural match).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct IntentScore {
    /// The classified intent.
    pub intent: Intent,
    /// Confidence score from 0.0 to 1.0.
    pub confidence: f64,
    /// Structural signals that contributed to this classification.
    pub matching_signals: Vec<String>,
}

/// Manual override for intent derivation when structural analysis is wrong.
///
/// `Primary` forces an intent as the top classification.
/// `Exclude` removes an intent from consideration entirely.
///
/// Serializes as externally tagged: `{"primary": "browse"}` or `{"exclude": "process"}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntentHint {
    /// Force this intent as the primary classification.
    Primary(Intent),
    /// Exclude this intent from consideration.
    Exclude(Intent),
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Intent construction and serde --

    #[test]
    fn intent_known_variants_serde_round_trip() {
        let known = [
            Intent::Browse,
            Intent::Focus,
            Intent::Collect,
            Intent::Process,
            Intent::Summarize,
            Intent::Analyze,
            Intent::Track,
        ];
        for intent in known {
            let json = serde_json::to_string(&intent).unwrap();
            let parsed: Intent = serde_json::from_str(&json).unwrap();
            assert_eq!(intent, parsed);
        }
    }

    #[test]
    fn intent_custom_fallback() {
        let parsed: Intent = serde_json::from_str(r#""dashboard""#).unwrap();
        assert_eq!(parsed, Intent::Custom("dashboard".to_string()));
    }

    #[test]
    fn intent_custom_round_trip() {
        let custom = Intent::Custom("my_intent".into());
        let json = serde_json::to_string(&custom).unwrap();
        let parsed: Intent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Intent::Custom("my_intent".into()));
    }

    #[test]
    fn intent_known_not_custom() {
        // "browse" must match Browse variant, not Custom("browse")
        let parsed: Intent = serde_json::from_str(r#""browse""#).unwrap();
        assert_eq!(parsed, Intent::Browse);
        assert_ne!(parsed, Intent::Custom("browse".into()));
    }

    #[test]
    fn intent_snake_case_serialization() {
        assert_eq!(
            serde_json::to_string(&Intent::Browse).unwrap(),
            r#""browse""#
        );
        assert_eq!(
            serde_json::to_string(&Intent::Summarize).unwrap(),
            r#""summarize""#
        );
    }

    #[test]
    fn intent_eq_and_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Intent::Browse);
        set.insert(Intent::Browse);
        set.insert(Intent::Custom("x".into()));
        assert_eq!(set.len(), 2);
    }

    // -- IntentScore construction and serde --

    #[test]
    fn intent_score_serde_round_trip() {
        let score = IntentScore {
            intent: Intent::Browse,
            confidence: 0.85,
            matching_signals: vec!["has_many_relationships".into(), "entity_name_fields".into()],
        };
        let json = serde_json::to_string(&score).unwrap();
        let parsed: IntentScore = serde_json::from_str(&json).unwrap();
        assert_eq!(score, parsed);
    }

    #[test]
    fn intent_score_with_custom_intent() {
        let score = IntentScore {
            intent: Intent::Custom("reporting".into()),
            confidence: 0.6,
            matching_signals: vec!["date_range_fields".into()],
        };
        let json = serde_json::to_string(&score).unwrap();
        let parsed: IntentScore = serde_json::from_str(&json).unwrap();
        assert_eq!(score, parsed);
    }

    // -- IntentHint construction and serde --

    #[test]
    fn intent_hint_primary_serde_round_trip() {
        let hint = IntentHint::Primary(Intent::Browse);
        let json = serde_json::to_string(&hint).unwrap();
        let parsed: IntentHint = serde_json::from_str(&json).unwrap();
        assert_eq!(hint, parsed);
    }

    #[test]
    fn intent_hint_exclude_serde_round_trip() {
        let hint = IntentHint::Exclude(Intent::Process);
        let json = serde_json::to_string(&hint).unwrap();
        let parsed: IntentHint = serde_json::from_str(&json).unwrap();
        assert_eq!(hint, parsed);
    }

    #[test]
    fn intent_hint_json_structure() {
        // Primary serializes as {"primary": "browse"}
        let primary = IntentHint::Primary(Intent::Browse);
        let json = serde_json::to_string(&primary).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("primary").is_some());
        assert_eq!(value["primary"], "browse");

        // Exclude serializes as {"exclude": "process"}
        let exclude = IntentHint::Exclude(Intent::Process);
        let json = serde_json::to_string(&exclude).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.get("exclude").is_some());
        assert_eq!(value["exclude"], "process");
    }

    #[test]
    fn intent_hint_with_custom_intent() {
        let hint = IntentHint::Primary(Intent::Custom("wizard".into()));
        let json = serde_json::to_string(&hint).unwrap();
        let parsed: IntentHint = serde_json::from_str(&json).unwrap();
        assert_eq!(hint, parsed);
    }

    // -- JSON Schema --

    #[test]
    fn intent_json_schema_has_description() {
        let schema = schemars::schema_for!(Intent);
        let value = schema.to_value();
        let desc = value
            .get("description")
            .expect("Intent schema must have description");
        let desc_str = desc.as_str().unwrap();
        assert!(
            desc_str.contains("Known variants"),
            "description should document known variants, got: {desc_str}"
        );
    }

    #[test]
    fn intent_score_json_schema() {
        let schema = schemars::schema_for!(IntentScore);
        let value = schema.to_value();
        let props = value
            .get("properties")
            .expect("IntentScore schema must have properties");
        let obj = props.as_object().unwrap();
        assert!(obj.contains_key("intent"), "missing 'intent' property");
        assert!(
            obj.contains_key("confidence"),
            "missing 'confidence' property"
        );
        assert!(
            obj.contains_key("matching_signals"),
            "missing 'matching_signals' property"
        );
    }

    #[test]
    fn intent_hint_json_schema() {
        let schema = schemars::schema_for!(IntentHint);
        let value = schema.to_value();
        // IntentHint is an externally tagged enum, so it should have oneOf
        let one_of = value.get("oneOf");
        assert!(one_of.is_some(), "IntentHint schema must have oneOf");
    }
}
