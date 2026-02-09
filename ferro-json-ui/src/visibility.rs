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
    Condition(VisibilityCondition),
    And { and: Vec<Visibility> },
    Or { or: Vec<Visibility> },
    Not { not: Box<Visibility> },
}
