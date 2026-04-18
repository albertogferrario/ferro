//! Conditional visibility rules for JSON-UI components.
//!
//! Visibility rules determine whether a component is rendered based
//! on data conditions. Conditions reference data paths (JSONPath-style)
//! and support logical composition with AND, OR, and NOT operators.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Comparison operators for visibility conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Visibility {
    And { and: Vec<Visibility> },
    Or { or: Vec<Visibility> },
    Not { not: Box<Visibility> },
    Condition(VisibilityCondition),
}

impl Visibility {
    /// Evaluates this visibility rule against handler data.
    ///
    /// Infallible: malformed conditions, missing paths, and type mismatches all
    /// resolve to `false` (visibility hides the element) without panicking. This
    /// is the contract Phase 116's renderer relies on per CONTEXT D-13.
    ///
    /// Edge cases (per Phase 116 RESEARCH §"Visibility ... TO BE ADDED" and
    /// Assumptions Log A1):
    /// - `Eq` against a missing path → `false` (no value to compare).
    /// - `NotEq` against a missing path → `true` (no value, so it's "not equal" to anything).
    /// - Numeric comparators (`Gt`/`Lt`/`Gte`/`Lte`) against non-numeric or missing → `false`.
    /// - `Contains` on a scalar value → `false`.
    /// - `NotEmpty`/`Empty` treat numbers and booleans as non-empty.
    ///
    /// Visibility is a presentation-layer concern only. Never treat
    /// `visible: false` as a server-side access check — enforce authorization
    /// in the routing/handler layer, not here.
    pub fn evaluate(&self, data: &serde_json::Value) -> bool {
        match self {
            Visibility::And { and } => and.iter().all(|v| v.evaluate(data)),
            Visibility::Or { or } => or.iter().any(|v| v.evaluate(data)),
            Visibility::Not { not } => !not.evaluate(data),
            Visibility::Condition(c) => evaluate_condition(c, data),
        }
    }
}

fn evaluate_condition(c: &VisibilityCondition, data: &serde_json::Value) -> bool {
    use crate::data::resolve_path;
    let resolved = resolve_path(data, &c.path);
    match c.operator {
        VisibilityOperator::Exists => matches!(resolved, Some(v) if !v.is_null()),
        VisibilityOperator::NotExists => !matches!(resolved, Some(v) if !v.is_null()),
        VisibilityOperator::Eq => match (resolved, c.value.as_ref()) {
            (Some(v), Some(target)) => v == target,
            _ => false,
        },
        VisibilityOperator::NotEq => match (resolved, c.value.as_ref()) {
            (Some(v), Some(target)) => v != target,
            (None, _) => true, // missing path is "not equal" per A1
            (Some(_), None) => true,
        },
        VisibilityOperator::Gt => numeric_cmp(resolved, c.value.as_ref(), |a, b| a > b),
        VisibilityOperator::Lt => numeric_cmp(resolved, c.value.as_ref(), |a, b| a < b),
        VisibilityOperator::Gte => numeric_cmp(resolved, c.value.as_ref(), |a, b| a >= b),
        VisibilityOperator::Lte => numeric_cmp(resolved, c.value.as_ref(), |a, b| a <= b),
        VisibilityOperator::Contains => match (resolved, c.value.as_ref()) {
            (Some(serde_json::Value::String(s)), Some(serde_json::Value::String(t))) => {
                s.contains(t)
            }
            (Some(serde_json::Value::Array(arr)), Some(target)) => arr.iter().any(|v| v == target),
            _ => false,
        },
        VisibilityOperator::NotEmpty => match resolved {
            Some(serde_json::Value::String(s)) => !s.is_empty(),
            Some(serde_json::Value::Array(arr)) => !arr.is_empty(),
            Some(serde_json::Value::Object(obj)) => !obj.is_empty(),
            Some(serde_json::Value::Number(_)) | Some(serde_json::Value::Bool(_)) => true,
            _ => false,
        },
        VisibilityOperator::Empty => match resolved {
            Some(serde_json::Value::String(s)) => s.is_empty(),
            Some(serde_json::Value::Array(arr)) => arr.is_empty(),
            Some(serde_json::Value::Object(obj)) => obj.is_empty(),
            Some(serde_json::Value::Number(_)) | Some(serde_json::Value::Bool(_)) => false,
            None | Some(serde_json::Value::Null) => true,
        },
    }
}

fn numeric_cmp(
    resolved: Option<&serde_json::Value>,
    target: Option<&serde_json::Value>,
    op: fn(f64, f64) -> bool,
) -> bool {
    match (resolved, target) {
        (Some(serde_json::Value::Number(a)), Some(serde_json::Value::Number(b))) => {
            match (a.as_f64(), b.as_f64()) {
                (Some(af), Some(bf)) => op(af, bf),
                _ => false,
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
                "operator {op:?} should serialize to {expected}"
            );
        }
    }

    #[test]
    fn evaluate_exists_true_for_present_non_null() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/user".into(),
            operator: VisibilityOperator::Exists,
            value: None,
        });
        assert!(vis.evaluate(&json!({"user": "alice"})));
    }

    #[test]
    fn evaluate_exists_false_for_missing() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/missing".into(),
            operator: VisibilityOperator::Exists,
            value: None,
        });
        assert!(!vis.evaluate(&json!({"user": "alice"})));
    }

    #[test]
    fn evaluate_exists_false_for_null() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/user".into(),
            operator: VisibilityOperator::Exists,
            value: None,
        });
        assert!(!vis.evaluate(&json!({"user": null})));
    }

    #[test]
    fn evaluate_not_exists_inverse_of_exists() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/missing".into(),
            operator: VisibilityOperator::NotExists,
            value: None,
        });
        assert!(vis.evaluate(&json!({})));
    }

    #[test]
    fn evaluate_eq_matches_value() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/role".into(),
            operator: VisibilityOperator::Eq,
            value: Some(json!("admin")),
        });
        assert!(vis.evaluate(&json!({"role": "admin"})));
        assert!(!vis.evaluate(&json!({"role": "user"})));
    }

    #[test]
    fn evaluate_eq_missing_path_returns_false() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/missing".into(),
            operator: VisibilityOperator::Eq,
            value: Some(json!("x")),
        });
        assert!(!vis.evaluate(&json!({})));
    }

    #[test]
    fn evaluate_not_eq_missing_path_returns_true() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/missing".into(),
            operator: VisibilityOperator::NotEq,
            value: Some(json!("x")),
        });
        assert!(vis.evaluate(&json!({})));
    }

    #[test]
    fn evaluate_numeric_comparators() {
        for (op, expect_for_5_vs_3) in [
            (VisibilityOperator::Gt, true),
            (VisibilityOperator::Gte, true),
            (VisibilityOperator::Lt, false),
            (VisibilityOperator::Lte, false),
        ] {
            let vis = Visibility::Condition(VisibilityCondition {
                path: "/n".into(),
                operator: op,
                value: Some(json!(3)),
            });
            assert_eq!(vis.evaluate(&json!({"n": 5})), expect_for_5_vs_3);
        }
    }

    #[test]
    fn evaluate_numeric_comparator_on_string_returns_false() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/n".into(),
            operator: VisibilityOperator::Gt,
            value: Some(json!(3)),
        });
        assert!(!vis.evaluate(&json!({"n": "five"})));
    }

    #[test]
    fn evaluate_contains_string_substring() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/s".into(),
            operator: VisibilityOperator::Contains,
            value: Some(json!("ell")),
        });
        assert!(vis.evaluate(&json!({"s": "hello"})));
        assert!(!vis.evaluate(&json!({"s": "world"})));
    }

    #[test]
    fn evaluate_contains_array_membership() {
        let vis = Visibility::Condition(VisibilityCondition {
            path: "/tags".into(),
            operator: VisibilityOperator::Contains,
            value: Some(json!("admin")),
        });
        assert!(vis.evaluate(&json!({"tags": ["user", "admin"]})));
        assert!(!vis.evaluate(&json!({"tags": ["user", "guest"]})));
    }

    #[test]
    fn evaluate_not_empty_for_string_array_object_number_bool() {
        let make = |op| {
            Visibility::Condition(VisibilityCondition {
                path: "/v".into(),
                operator: op,
                value: None,
            })
        };
        assert!(make(VisibilityOperator::NotEmpty).evaluate(&json!({"v": "x"})));
        assert!(make(VisibilityOperator::NotEmpty).evaluate(&json!({"v": [1]})));
        assert!(make(VisibilityOperator::NotEmpty).evaluate(&json!({"v": {"a": 1}})));
        assert!(make(VisibilityOperator::NotEmpty).evaluate(&json!({"v": 0})));
        assert!(make(VisibilityOperator::NotEmpty).evaluate(&json!({"v": false})));
        assert!(!make(VisibilityOperator::NotEmpty).evaluate(&json!({"v": ""})));
        assert!(!make(VisibilityOperator::NotEmpty).evaluate(&json!({"v": []})));
        assert!(!make(VisibilityOperator::NotEmpty).evaluate(&json!({})));
    }

    #[test]
    fn evaluate_compound_and_or_not() {
        let admin = Visibility::Condition(VisibilityCondition {
            path: "/role".into(),
            operator: VisibilityOperator::Eq,
            value: Some(json!("admin")),
        });
        let active = Visibility::Condition(VisibilityCondition {
            path: "/active".into(),
            operator: VisibilityOperator::Eq,
            value: Some(json!(true)),
        });
        let both = Visibility::And {
            and: vec![admin.clone(), active.clone()],
        };
        let either = Visibility::Or {
            or: vec![admin.clone(), active.clone()],
        };
        let neither = Visibility::Not {
            not: Box::new(either.clone()),
        };

        let data = json!({"role": "admin", "active": false});
        assert!(!both.evaluate(&data));
        assert!(either.evaluate(&data));
        assert!(!neither.evaluate(&data));
    }
}
