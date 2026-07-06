//! Inspect a service projection's structure: fields, relationships, actions, state machine.

use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// Detailed structure of a single projection.
#[derive(Debug, Serialize)]
pub struct ProjectionDetail {
    pub name: String,
    pub file: String,
    pub service_name: String,
    pub display_name: Option<String>,
    pub fields: Vec<FieldInfo>,
    pub relationships: Vec<String>,
    pub actions: Vec<String>,
    pub has_state_machine: bool,
    pub intent_hints: Vec<String>,
}

/// A parsed field from the projection source.
#[derive(Debug, Serialize)]
pub struct FieldInfo {
    pub name: String,
    pub data_type: String,
    pub meaning: String,
    pub readable: bool,
    pub writable: bool,
}

/// Error returned when a projection is not found.
#[derive(Debug, Serialize)]
pub struct ProjectionNotFound {
    pub name: String,
    pub available: Vec<String>,
}

/// Result of inspecting a projection.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum InspectResult {
    Found(ProjectionDetail),
    NotFound(ProjectionNotFound),
}

/// Inspect a named projection by scanning source files in `src/projections/`.
pub fn execute(project_root: &Path, name: &str) -> InspectResult {
    let projections = super::list_projections::execute(project_root, None);
    let available: Vec<String> = projections
        .projections
        .iter()
        .map(|p| p.name.clone())
        .collect();

    let info = projections.projections.into_iter().find(|p| p.name == name);

    let info = match info {
        Some(i) => i,
        None => {
            return InspectResult::NotFound(ProjectionNotFound {
                name: name.to_string(),
                available,
            })
        }
    };

    let file_path = project_root.join(&info.file);
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => {
            return InspectResult::NotFound(ProjectionNotFound {
                name: name.to_string(),
                available,
            })
        }
    };

    let fields = parse_fields(&content);
    let relationships = parse_relationships(&content);
    let actions = parse_actions(&content);
    let has_state_machine = content.contains(".state_machine(");
    let intent_hints = parse_intent_hints(&content);

    InspectResult::Found(ProjectionDetail {
        name: info.name,
        file: info.file,
        service_name: info.service_name.unwrap_or_default(),
        display_name: info.display_name,
        fields,
        relationships,
        actions,
        has_state_machine,
        intent_hints,
    })
}

/// Parse `.field("name", DataType::X, FieldMeaning::Y)` calls.
fn parse_fields(content: &str) -> Vec<FieldInfo> {
    let mut fields = Vec::new();

    // Standard .field() calls — readable + writable
    let field_re =
        Regex::new(r#"\.field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#).unwrap();
    for cap in field_re.captures_iter(content) {
        fields.push(FieldInfo {
            name: cap[1].to_string(),
            data_type: cap[2].to_string(),
            meaning: cap[3].to_string(),
            readable: true,
            writable: true,
        });
    }

    // .optional_field() — readable + writable
    let optional_re =
        Regex::new(r#"\.optional_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in optional_re.captures_iter(content) {
        fields.push(FieldInfo {
            name: cap[1].to_string(),
            data_type: cap[2].to_string(),
            meaning: cap[3].to_string(),
            readable: true,
            writable: true,
        });
    }

    // .read_only_field() — readable, not writable
    let ro_re =
        Regex::new(r#"\.read_only_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in ro_re.captures_iter(content) {
        fields.push(FieldInfo {
            name: cap[1].to_string(),
            data_type: cap[2].to_string(),
            meaning: cap[3].to_string(),
            readable: true,
            writable: false,
        });
    }

    // .write_only_field() — not readable, writable
    let wo_re =
        Regex::new(r#"\.write_only_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in wo_re.captures_iter(content) {
        fields.push(FieldInfo {
            name: cap[1].to_string(),
            data_type: cap[2].to_string(),
            meaning: cap[3].to_string(),
            readable: false,
            writable: true,
        });
    }

    fields
}

/// Parse relationship builder calls.
fn parse_relationships(content: &str) -> Vec<String> {
    let mut rels = Vec::new();

    // .relationship(RelationshipDef::new("name", ...))
    let rel_re = Regex::new(r#"\.relationship\(RelationshipDef::new\("([^"]+)""#).unwrap();
    for cap in rel_re.captures_iter(content) {
        rels.push(cap[1].to_string());
    }

    // .has_many("name", "target")
    let hm_re = Regex::new(r#"\.has_many\("([^"]+)""#).unwrap();
    for cap in hm_re.captures_iter(content) {
        rels.push(cap[1].to_string());
    }

    // .belongs_to("name", "target")
    let bt_re = Regex::new(r#"\.belongs_to\("([^"]+)""#).unwrap();
    for cap in bt_re.captures_iter(content) {
        rels.push(cap[1].to_string());
    }

    // .has_one("name", "target")
    let ho_re = Regex::new(r#"\.has_one\("([^"]+)""#).unwrap();
    for cap in ho_re.captures_iter(content) {
        rels.push(cap[1].to_string());
    }

    // .belongs_to_many("name", "target")
    let btm_re = Regex::new(r#"\.belongs_to_many\("([^"]+)""#).unwrap();
    for cap in btm_re.captures_iter(content) {
        rels.push(cap[1].to_string());
    }

    rels
}

/// Parse `.action(ActionDef::new("name"))` calls.
fn parse_actions(content: &str) -> Vec<String> {
    let re = Regex::new(r#"\.action\(ActionDef::new\("([^"]+)"\)"#).unwrap();
    re.captures_iter(content)
        .map(|c| c[1].to_string())
        .collect()
}

/// Parse `.intent_hint(IntentHint::Primary(Intent::X))` and similar.
fn parse_intent_hints(content: &str) -> Vec<String> {
    let re = Regex::new(r#"\.intent_hint\(IntentHint::(\w+)\(Intent::(\w+)\)\)"#).unwrap();
    re.captures_iter(content)
        .map(|c| format!("{}({})", &c[1], &c[2]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_not_found() {
        let non_existent = PathBuf::from("/tmp/non_existent_ferro_projections_test");
        let result = execute(&non_existent, "nonexistent");
        match result {
            InspectResult::NotFound(nf) => {
                assert_eq!(nf.name, "nonexistent");
                assert!(nf.available.is_empty());
            }
            InspectResult::Found(_) => panic!("expected NotFound"),
        }
    }

    #[test]
    fn test_serialization() {
        let detail = ProjectionDetail {
            name: "user_service".to_string(),
            file: "src/projections/user.rs".to_string(),
            service_name: "user".to_string(),
            display_name: Some("User".to_string()),
            fields: vec![FieldInfo {
                name: "email".to_string(),
                data_type: "String".to_string(),
                meaning: "Email".to_string(),
                readable: true,
                writable: true,
            }],
            relationships: vec!["orders".to_string()],
            actions: vec!["activate".to_string()],
            has_state_machine: true,
            intent_hints: vec!["Primary(Browse)".to_string()],
        };

        let json = serde_json::to_string(&InspectResult::Found(detail));
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("user_service"));
        assert!(json_str.contains("email"));
        assert!(json_str.contains("Email"));
        assert!(json_str.contains("orders"));
        assert!(json_str.contains("activate"));
        assert!(json_str.contains("has_state_machine"));
        assert!(json_str.contains("intent_hints"));
    }

    #[test]
    fn test_not_found_serialization() {
        let nf = InspectResult::NotFound(ProjectionNotFound {
            name: "missing".to_string(),
            available: vec!["user_service".to_string()],
        });

        let json = serde_json::to_string(&nf);
        assert!(json.is_ok(), "Should serialize NotFound to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("missing"));
        assert!(json_str.contains("user_service"));
        assert!(json_str.contains("available"));
    }

    #[test]
    fn test_parse_fields() {
        let content = r#"
            ServiceDef::new("test")
                .field("id", DataType::Integer, FieldMeaning::Identifier)
                .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
                .write_only_field("password", DataType::String, FieldMeaning::Sensitive)
                .optional_field("notes", DataType::String, FieldMeaning::FreeText)
        "#;

        let fields = parse_fields(content);
        assert_eq!(fields.len(), 4);

        assert_eq!(fields[0].name, "id");
        assert!(fields[0].readable);
        assert!(fields[0].writable);

        assert_eq!(fields[1].name, "notes");
        assert!(fields[1].readable);
        assert!(fields[1].writable);

        assert_eq!(fields[2].name, "created_at");
        assert!(fields[2].readable);
        assert!(!fields[2].writable);

        assert_eq!(fields[3].name, "password");
        assert!(!fields[3].readable);
        assert!(fields[3].writable);
    }

    #[test]
    fn test_parse_relationships() {
        let content = r#"
            .has_many("orders", "order")
            .belongs_to("customer", "customer")
            .has_one("profile", "profile")
            .belongs_to_many("tags", "tag")
            .relationship(RelationshipDef::new("items", "item", Cardinality::OneToMany))
        "#;

        let rels = parse_relationships(content);
        assert_eq!(rels.len(), 5);
        assert!(rels.contains(&"orders".to_string()));
        assert!(rels.contains(&"customer".to_string()));
        assert!(rels.contains(&"profile".to_string()));
        assert!(rels.contains(&"tags".to_string()));
        assert!(rels.contains(&"items".to_string()));
    }

    #[test]
    fn test_parse_actions() {
        let content = r#"
            .action(ActionDef::new("submit"))
            .action(ActionDef::new("approve").display_name("Approve"))
        "#;

        let actions = parse_actions(content);
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0], "submit");
        assert_eq!(actions[1], "approve");
    }

    #[test]
    fn test_parse_intent_hints() {
        let content = r#"
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .intent_hint(IntentHint::Exclude(Intent::Process))
        "#;

        let hints = parse_intent_hints(content);
        assert_eq!(hints.len(), 2);
        assert_eq!(hints[0], "Primary(Browse)");
        assert_eq!(hints[1], "Exclude(Process)");
    }
}
