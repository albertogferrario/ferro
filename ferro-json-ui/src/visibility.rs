//! Conditional visibility rules for JSON-UI components.
//!
//! Visibility rules determine whether a component is rendered based
//! on data conditions. Conditions reference data paths (JSONPath-style)
//! and support logical composition with AND, OR, and NOT operators.

use serde::{Deserialize, Serialize};

/// Comparison operators for visibility conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityOperator {
    Exists,
    NotExists,
    Eq,
    NotEq,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
    NotEmpty,
    Empty,
}

/// A single visibility condition comparing a data path against a value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisibilityCondition {
    /// JSONPath-style reference to data.
    pub path: String,
    pub operator: VisibilityOperator,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// Visibility rule with logical composition support.
///
/// Uses `#[serde(untagged)]` to support clean JSON:
/// - Simple: `{"path": "/data/users", "operator": "not_empty"}`
/// - Compound: `{"and": [...]}`
/// - Nested: `{"not": {"path": ..., "operator": ...}}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Visibility {
    And { and: Vec<Visibility> },
    Or { or: Vec<Visibility> },
    Not { not: Box<Visibility> },
    Condition(VisibilityCondition),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_condition_round_trips() {
        let json = r#"{"path": "/data/users", "operator": "not_empty"}"#;
        let vis: Visibility = serde_json::from_str(json).unwrap();
        match &vis {
            Visibility::Condition(c) => {
                assert_eq!(c.path, "/data/users");
                assert_eq!(c.operator, VisibilityOperator::NotEmpty);
                assert!(c.value.is_none());
            }
            _ => panic!("expected Condition variant"),
        }
        let serialized = serde_json::to_string(&vis).unwrap();
        let reparsed: Visibility = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vis, reparsed);
    }

    #[test]
    fn condition_with_value() {
        let json = r#"{"path": "/auth/user/role", "operator": "eq", "value": "admin"}"#;
        let vis: Visibility = serde_json::from_str(json).unwrap();
        match &vis {
            Visibility::Condition(c) => {
                assert_eq!(c.operator, VisibilityOperator::Eq);
                assert_eq!(
                    c.value,
                    Some(serde_json::Value::String("admin".to_string()))
                );
            }
            _ => panic!("expected Condition variant"),
        }
    }

    #[test]
    fn compound_and_condition() {
        let json = r#"{
            "and": [
                {"path": "/auth/user", "operator": "exists"},
                {"path": "/auth/user/role", "operator": "eq", "value": "admin"}
            ]
        }"#;
        let vis: Visibility = serde_json::from_str(json).unwrap();
        match &vis {
            Visibility::And { and } => {
                assert_eq!(and.len(), 2);
            }
            _ => panic!("expected And variant"),
        }
        let serialized = serde_json::to_string(&vis).unwrap();
        let reparsed: Visibility = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vis, reparsed);
    }

    #[test]
    fn compound_or_condition() {
        let json = r#"{
            "or": [
                {"path": "/data/status", "operator": "eq", "value": "active"},
                {"path": "/data/status", "operator": "eq", "value": "pending"}
            ]
        }"#;
        let vis: Visibility = serde_json::from_str(json).unwrap();
        assert!(matches!(vis, Visibility::Or { .. }));
    }

    #[test]
    fn nested_not_condition() {
        let json = r#"{"not": {"path": "/data/deleted", "operator": "exists"}}"#;
        let vis: Visibility = serde_json::from_str(json).unwrap();
        match &vis {
            Visibility::Not { not } => match not.as_ref() {
                Visibility::Condition(c) => {
                    assert_eq!(c.path, "/data/deleted");
                    assert_eq!(c.operator, VisibilityOperator::Exists);
                }
                _ => panic!("expected Condition inside Not"),
            },
            _ => panic!("expected Not variant"),
        }
    }

    #[test]
    fn all_operators_serialize() {
        let operators = vec![
            (VisibilityOperator::Exists, "exists"),
            (VisibilityOperator::NotExists, "not_exists"),
            (VisibilityOperator::Eq, "eq"),
            (VisibilityOperator::NotEq, "not_eq"),
            (VisibilityOperator::Gt, "gt"),
            (VisibilityOperator::Lt, "lt"),
            (VisibilityOperator::Gte, "gte"),
            (VisibilityOperator::Lte, "lte"),
            (VisibilityOperator::Contains, "contains"),
            (VisibilityOperator::NotEmpty, "not_empty"),
            (VisibilityOperator::Empty, "empty"),
        ];
        for (op, expected) in operators {
            let json = serde_json::to_value(&op).unwrap();
            assert_eq!(
                json, expected,
                "operator {:?} should serialize to {}",
                op, expected
            );
        }
    }
}
