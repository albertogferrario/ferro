//! Render a service projection to JSON-UI output by reconstructing a ServiceDef from source.

use serde::Serialize;
use std::path::Path;

use ferro_projections::{
    derive_intents, ActionDef, Cardinality, DataType, FieldMeaning, IntentHint, JsonUiRenderer,
    RenderContext, RenderMode, Renderer, ServiceDef, StateDef, StateMachine, Transition,
};
use regex::Regex;
use std::fs;

use super::inspect_projection::InspectResult;

/// Rendered projection output.
#[derive(Debug, Serialize)]
pub struct RenderResult {
    pub service_name: String,
    pub intent: String,
    pub confidence: f64,
    pub mode: String,
    pub json_ui: serde_json::Value,
    pub all_intents: Vec<IntentInfo>,
}

/// Intent scoring information.
#[derive(Debug, Serialize)]
pub struct IntentInfo {
    pub intent: String,
    pub confidence: f64,
    pub signals: Vec<String>,
}

/// Render a named projection to JSON-UI.
pub fn execute(
    project_root: &Path,
    name: &str,
    mode: Option<&str>,
    intent_index: Option<usize>,
) -> Result<RenderResult, String> {
    // Find the projection via inspect
    let inspect = super::inspect_projection::execute(project_root, name);
    let detail = match inspect {
        InspectResult::Found(d) => d,
        InspectResult::NotFound(nf) => {
            return Err(format!(
                "projection '{}' not found. Available: {:?}",
                nf.name, nf.available
            ))
        }
    };

    // Read the source file for full reconstruction
    let file_path = project_root.join(&detail.file);
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("failed to read {}: {}", detail.file, e))?;

    // Reconstruct ServiceDef from source
    let service = reconstruct_service_def(&detail.service_name, &detail.display_name, &content)?;

    // Derive intents
    let intents = derive_intents(&service);

    // Parse mode
    let render_mode = match mode {
        Some("input") => RenderMode::Input,
        _ => RenderMode::Display,
    };

    let idx = intent_index.unwrap_or(0);
    let ctx = RenderContext {
        intent_index: idx,
        current_state: None,
        mode: render_mode,
    };

    // Render
    let renderer = JsonUiRenderer;
    let json_ui = renderer
        .render(&service, &intents, &ctx)
        .map_err(|e| format!("render error: {e}"))?;

    let selected = intents
        .get(idx)
        .ok_or_else(|| format!("intent_index {idx} out of bounds"))?;

    let all_intents: Vec<IntentInfo> = intents
        .iter()
        .map(|is| IntentInfo {
            intent: format!("{:?}", is.intent),
            confidence: is.confidence,
            signals: is.matching_signals.clone(),
        })
        .collect();

    Ok(RenderResult {
        service_name: detail.service_name,
        intent: format!("{:?}", selected.intent),
        confidence: selected.confidence,
        mode: match render_mode {
            RenderMode::Display => "display".to_string(),
            RenderMode::Input => "input".to_string(),
        },
        json_ui,
        all_intents,
    })
}

/// Reconstruct a ServiceDef from source code using regex parsing.
pub(crate) fn reconstruct_service_def(
    service_name: &str,
    display_name: &Option<String>,
    content: &str,
) -> Result<ServiceDef, String> {
    let mut service = ServiceDef::new(service_name);

    if let Some(dn) = display_name {
        service = service.display_name(dn.clone());
    }

    // Parse description
    let desc_re = Regex::new(r#"\.description\("([^"]+)"\)"#).unwrap();
    if let Some(cap) = desc_re.captures(content) {
        service = service.description(cap[1].to_string());
    }

    // Parse fields
    service = parse_and_add_fields(service, content);

    // Parse relationships
    service = parse_and_add_relationships(service, content);

    // Parse actions
    service = parse_and_add_actions(service, content);

    // Parse state machine
    if content.contains(".state_machine(") {
        if let Some(sm) = parse_state_machine(content) {
            service = service.state_machine(sm);
        }
    }

    // Parse intent hints
    service = parse_and_add_intent_hints(service, content);

    Ok(service)
}

/// Parse and add fields from source.
fn parse_and_add_fields(mut service: ServiceDef, content: &str) -> ServiceDef {
    // .field("name", DataType::X, FieldMeaning::Y)
    let field_re =
        Regex::new(r#"\.field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#).unwrap();
    for cap in field_re.captures_iter(content) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            service = service.field(&cap[1], dt, fm);
        }
    }

    // .optional_field("name", DataType::X, FieldMeaning::Y)
    let opt_re =
        Regex::new(r#"\.optional_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in opt_re.captures_iter(content) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            service = service.optional_field(&cap[1], dt, fm);
        }
    }

    // .read_only_field("name", DataType::X, FieldMeaning::Y)
    let ro_re =
        Regex::new(r#"\.read_only_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in ro_re.captures_iter(content) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            service = service.read_only_field(&cap[1], dt, fm);
        }
    }

    // .write_only_field("name", DataType::X, FieldMeaning::Y)
    let wo_re =
        Regex::new(r#"\.write_only_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in wo_re.captures_iter(content) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            service = service.write_only_field(&cap[1], dt, fm);
        }
    }

    service
}

/// Parse and add relationships from source.
fn parse_and_add_relationships(mut service: ServiceDef, content: &str) -> ServiceDef {
    // .has_many("name", "target")
    let hm_re = Regex::new(r#"\.has_many\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in hm_re.captures_iter(content) {
        service = service.has_many(&cap[1], &cap[2]);
    }

    // .belongs_to("name", "target")
    let bt_re = Regex::new(r#"\.belongs_to\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in bt_re.captures_iter(content) {
        service = service.belongs_to(&cap[1], &cap[2]);
    }

    // .has_one("name", "target")
    let ho_re = Regex::new(r#"\.has_one\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in ho_re.captures_iter(content) {
        service = service.has_one(&cap[1], &cap[2]);
    }

    // .belongs_to_many("name", "target")
    let btm_re = Regex::new(r#"\.belongs_to_many\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in btm_re.captures_iter(content) {
        service = service.belongs_to_many(&cap[1], &cap[2]);
    }

    // .relationship(RelationshipDef::new("name", "target", Cardinality::X))
    let rel_re = Regex::new(
        r#"\.relationship\(RelationshipDef::new\("([^"]+)",\s*"([^"]+)",\s*Cardinality::(\w+)\)"#,
    )
    .unwrap();
    for cap in rel_re.captures_iter(content) {
        if let Some(card) = parse_cardinality(&cap[3]) {
            use ferro_projections::RelationshipDef;
            service = service.relationship(RelationshipDef::new(&cap[1], &cap[2], card));
        }
    }

    service
}

/// Parse and add actions from source.
fn parse_and_add_actions(mut service: ServiceDef, content: &str) -> ServiceDef {
    // Match .action(ActionDef::new("name")...) — capture the action name
    let action_re = Regex::new(r#"\.action\(ActionDef::new\("([^"]+)"\)"#).unwrap();
    for cap in action_re.captures_iter(content) {
        let action_name = &cap[1];
        let action = ActionDef::new(action_name);
        service = service.action(action);
    }

    service
}

/// Parse intent hints and add them to the service.
fn parse_and_add_intent_hints(mut service: ServiceDef, content: &str) -> ServiceDef {
    let re = Regex::new(r#"\.intent_hint\(IntentHint::(\w+)\(Intent::(\w+)\)\)"#).unwrap();
    for cap in re.captures_iter(content) {
        let intent = match parse_intent(&cap[2]) {
            Some(i) => i,
            None => continue,
        };
        let hint = match &cap[1] {
            "Primary" => IntentHint::Primary(intent),
            "Exclude" => IntentHint::Exclude(intent),
            _ => continue,
        };
        service = service.intent_hint(hint);
    }

    service
}

/// Parse a minimal state machine from source.
fn parse_state_machine(content: &str) -> Option<StateMachine> {
    // StateMachine::new("name")
    let name_re = Regex::new(r#"StateMachine::new\("([^"]+)"\)"#).unwrap();
    let name = name_re.captures(content).map(|c| c[1].to_string())?;

    // .initial("state")
    let initial_re = Regex::new(r#"\.initial\("([^"]+)"\)"#).unwrap();
    let initial = initial_re
        .captures(content)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "initial".to_string());

    let mut machine = StateMachine::new(&name).initial(&initial);

    // Collect state names that are marked as final_state
    let final_state_re = Regex::new(r#"StateDef::new\("([^"]+)"\)[^;]*\.final_state\(\)"#).unwrap();
    let final_states: std::collections::HashSet<String> = final_state_re
        .captures_iter(content)
        .map(|c| c[1].to_string())
        .collect();

    // .state(StateDef::new("name"))
    let state_re = Regex::new(r#"StateDef::new\("([^"]+)"\)"#).unwrap();
    for cap in state_re.captures_iter(content) {
        let state_name = cap[1].to_string();
        let mut state = StateDef::new(&state_name);
        if final_states.contains(&state_name) {
            state = state.final_state();
        }
        machine = machine.state(state);
    }

    // .transition(Transition::new("from", "event", "to"))
    let trans_re = Regex::new(r#"Transition::new\("([^"]+)",\s*"([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in trans_re.captures_iter(content) {
        machine = machine.transition(Transition::new(&cap[1], &cap[2], &cap[3]));
    }

    Some(machine)
}

/// Map a string to a DataType variant.
fn parse_data_type(s: &str) -> Option<DataType> {
    match s {
        "String" => Some(DataType::String),
        "Integer" => Some(DataType::Integer),
        "Float" => Some(DataType::Float),
        "Boolean" => Some(DataType::Boolean),
        "DateTime" => Some(DataType::DateTime),
        "Date" => Some(DataType::Date),
        "Json" => Some(DataType::Json),
        "Binary" => Some(DataType::Binary),
        "Uuid" => Some(DataType::Uuid),
        "Enum" => Some(DataType::Enum),
        _ => None,
    }
}

/// Map a string to a FieldMeaning variant.
fn parse_field_meaning(s: &str) -> Option<FieldMeaning> {
    match s {
        "Identifier" => Some(FieldMeaning::Identifier),
        "ForeignKey" => Some(FieldMeaning::ForeignKey),
        "EntityName" => Some(FieldMeaning::EntityName),
        "Email" => Some(FieldMeaning::Email),
        "Phone" => Some(FieldMeaning::Phone),
        "Url" => Some(FieldMeaning::Url),
        "ImageUrl" => Some(FieldMeaning::ImageUrl),
        "Money" => Some(FieldMeaning::Money),
        "Percentage" => Some(FieldMeaning::Percentage),
        "Quantity" => Some(FieldMeaning::Quantity),
        "Status" => Some(FieldMeaning::Status),
        "Category" => Some(FieldMeaning::Category),
        "Boolean" => Some(FieldMeaning::Boolean),
        "FreeText" => Some(FieldMeaning::FreeText),
        "CreatedAt" => Some(FieldMeaning::CreatedAt),
        "UpdatedAt" => Some(FieldMeaning::UpdatedAt),
        "DateTime" => Some(FieldMeaning::DateTime),
        "Sensitive" => Some(FieldMeaning::Sensitive),
        other => Some(FieldMeaning::Custom(other.to_string())),
    }
}

/// Map a string to a Cardinality variant.
fn parse_cardinality(s: &str) -> Option<Cardinality> {
    match s {
        "OneToOne" => Some(Cardinality::OneToOne),
        "OneToMany" => Some(Cardinality::OneToMany),
        "ManyToOne" => Some(Cardinality::ManyToOne),
        "ManyToMany" => Some(Cardinality::ManyToMany),
        _ => None,
    }
}

/// Map a string to an Intent variant.
fn parse_intent(s: &str) -> Option<ferro_projections::Intent> {
    use ferro_projections::Intent;
    match s {
        "Browse" => Some(Intent::Browse),
        "Focus" => Some(Intent::Focus),
        "Collect" => Some(Intent::Collect),
        "Process" => Some(Intent::Process),
        "Summarize" => Some(Intent::Summarize),
        "Analyze" => Some(Intent::Analyze),
        "Track" => Some(Intent::Track),
        other => Some(Intent::Custom(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_result_serialization() {
        let result = RenderResult {
            service_name: "user".to_string(),
            intent: "Browse".to_string(),
            confidence: 0.85,
            mode: "display".to_string(),
            json_ui: serde_json::json!({"$schema": "ferro-json-ui/v1"}),
            all_intents: vec![IntentInfo {
                intent: "Browse".to_string(),
                confidence: 0.85,
                signals: vec!["entity_name".to_string()],
            }],
        };

        let json = serde_json::to_string(&result);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("user"));
        assert!(json_str.contains("Browse"));
        assert!(json_str.contains("0.85"));
        assert!(json_str.contains("display"));
        assert!(json_str.contains("ferro-json-ui/v1"));
        assert!(json_str.contains("all_intents"));
    }

    #[test]
    fn test_mode_parsing() {
        // Default mode
        assert!(matches!(
            match None::<&str> {
                Some("input") => RenderMode::Input,
                _ => RenderMode::Display,
            },
            RenderMode::Display
        ));

        // Explicit display
        assert!(matches!(
            match Some("display") {
                Some("input") => RenderMode::Input,
                _ => RenderMode::Display,
            },
            RenderMode::Display
        ));

        // Explicit input
        assert!(matches!(
            match Some("input") {
                Some("input") => RenderMode::Input,
                _ => RenderMode::Display,
            },
            RenderMode::Input
        ));
    }

    #[test]
    fn test_parse_data_type_all_variants() {
        assert_eq!(parse_data_type("String"), Some(DataType::String));
        assert_eq!(parse_data_type("Integer"), Some(DataType::Integer));
        assert_eq!(parse_data_type("Float"), Some(DataType::Float));
        assert_eq!(parse_data_type("Boolean"), Some(DataType::Boolean));
        assert_eq!(parse_data_type("DateTime"), Some(DataType::DateTime));
        assert_eq!(parse_data_type("Date"), Some(DataType::Date));
        assert_eq!(parse_data_type("Json"), Some(DataType::Json));
        assert_eq!(parse_data_type("Binary"), Some(DataType::Binary));
        assert_eq!(parse_data_type("Uuid"), Some(DataType::Uuid));
        assert_eq!(parse_data_type("Enum"), Some(DataType::Enum));
        assert_eq!(parse_data_type("Unknown"), None);
    }

    #[test]
    fn test_parse_field_meaning_all_variants() {
        assert_eq!(
            parse_field_meaning("Identifier"),
            Some(FieldMeaning::Identifier)
        );
        assert_eq!(parse_field_meaning("Money"), Some(FieldMeaning::Money));
        assert_eq!(parse_field_meaning("Status"), Some(FieldMeaning::Status));
        assert_eq!(
            parse_field_meaning("CustomThing"),
            Some(FieldMeaning::Custom("CustomThing".to_string()))
        );
    }

    #[test]
    fn test_parse_cardinality() {
        assert_eq!(parse_cardinality("OneToOne"), Some(Cardinality::OneToOne));
        assert_eq!(parse_cardinality("OneToMany"), Some(Cardinality::OneToMany));
        assert_eq!(parse_cardinality("ManyToOne"), Some(Cardinality::ManyToOne));
        assert_eq!(
            parse_cardinality("ManyToMany"),
            Some(Cardinality::ManyToMany)
        );
        assert_eq!(parse_cardinality("Invalid"), None);
    }

    #[test]
    fn test_reconstruct_minimal_service() {
        let content = r#"
pub fn user_service() -> ServiceDef {
    ServiceDef::new("user")
        .display_name("User")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
}
        "#;

        let result = reconstruct_service_def("user", &Some("User".to_string()), content);
        assert!(result.is_ok());

        let service = result.unwrap();
        assert_eq!(service.name, "user");
        assert_eq!(service.display_name.as_deref(), Some("User"));
        assert_eq!(service.fields.len(), 2);
        assert_eq!(service.fields[0].name, "id");
        assert_eq!(service.fields[1].name, "name");
    }

    #[test]
    fn test_reconstruct_with_relationships() {
        let content = r#"
ServiceDef::new("order")
    .has_many("line_items", "line_item")
    .belongs_to("customer", "customer")
        "#;

        let result = reconstruct_service_def("order", &None, content);
        assert!(result.is_ok());

        let service = result.unwrap();
        assert_eq!(service.relationships.len(), 2);
    }

    #[test]
    fn test_not_found_project() {
        let result = execute(
            std::path::Path::new("/tmp/nonexistent_test_project"),
            "missing",
            None,
            None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
