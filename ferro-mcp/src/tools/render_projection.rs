//! Render a service projection to JSON-UI output by reconstructing a ServiceDef from source.

use serde::Serialize;
use std::path::Path;

use ferro_json_ui::{JsonUiRenderer, RenderMode, VisualContext};
use ferro_projections::render::BaseContext;
use ferro_projections::{
    derive_intents, ActionDef, Cardinality, DataType, FieldMeaning, GuardDef, InputDef, IntentHint,
    Renderer, ServiceDef, StateDef, StateMachine, Transition,
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
    let ctx = VisualContext {
        base: BaseContext {
            intent_index: idx,
            current_state: None,
            ..Default::default()
        },
        mode: render_mode,
        templates: None,
    };

    // Render
    let renderer = JsonUiRenderer;
    let spec = renderer
        .render(&service, &intents, &ctx)
        .map_err(|e| format!("render error: {e}"))?;
    let json_ui = serde_json::to_value(&spec)
        .map_err(|e| format!("failed to serialize Spec to JSON: {e}"))?;

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

    // Parse guards
    service = parse_and_add_guards(service, content);

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
///
/// Extracts full `.action(...)` blocks using parenthesis depth counting, then
/// applies sub-regexes to parse chained builder methods within each block.
fn parse_and_add_actions(mut service: ServiceDef, content: &str) -> ServiceDef {
    for block in extract_action_blocks(content) {
        if let Some(action) = parse_action_block(&block) {
            service = service.action(action);
        }
    }
    service
}

/// Extract each `.action(...)` expression by tracking parenthesis depth.
fn extract_action_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let needle = ".action(";
    let bytes = content.as_bytes();
    let mut search_from = 0;

    while let Some(pos) = content[search_from..].find(needle) {
        let abs_pos = search_from + pos;
        let start = abs_pos + needle.len(); // right after the opening '('
        let mut depth = 1;
        let mut i = start;

        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            i += 1;
        }

        if depth == 0 {
            // Block content is everything between the outer parens (exclusive of closing ')')
            blocks.push(content[start..i - 1].to_string());
        }

        search_from = i;
    }

    blocks
}

/// Parse a single action block into an ActionDef with full builder chain.
fn parse_action_block(block: &str) -> Option<ActionDef> {
    let name_re = Regex::new(r#"ActionDef::new\("([^"]+)"\)"#).unwrap();
    let name = name_re.captures(block)?[1].to_string();
    let mut action = ActionDef::new(&name);

    // .transition_trigger("event_name")
    let tt_re = Regex::new(r#"\.transition_trigger\("([^"]+)"\)"#).unwrap();
    if let Some(cap) = tt_re.captures(block) {
        action = action.transition_trigger(&cap[1]);
    }

    // .precondition("guard_name") — may appear multiple times
    let pc_re = Regex::new(r#"\.precondition\("([^"]+)"\)"#).unwrap();
    for cap in pc_re.captures_iter(block) {
        action = action.precondition(&cap[1]);
    }

    // .display_name("name")
    let dn_re = Regex::new(r#"\.display_name\("([^"]+)"\)"#).unwrap();
    if let Some(cap) = dn_re.captures(block) {
        action = action.display_name(&cap[1]);
    }

    // .input(InputDef::new("name", DataType::X, FieldMeaning::Y))
    let input_re = Regex::new(
        r#"\.input\(InputDef::new\(\s*"([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+),?\s*\)\)"#,
    )
    .unwrap();
    for cap in input_re.captures_iter(block) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            action = action.input(InputDef::new(&cap[1], dt, fm));
        }
    }

    Some(action)
}

/// Parse and add guard definitions from source.
fn parse_and_add_guards(mut service: ServiceDef, content: &str) -> ServiceDef {
    let guard_re =
        Regex::new(r#"\.guard\(GuardDef::new\("([^"]+)"\)(?:\.display_name\("([^"]+)"\))?\)"#)
            .unwrap();
    for cap in guard_re.captures_iter(content) {
        let mut guard = GuardDef::new(&cap[1]);
        if let Some(dn) = cap.get(2) {
            guard = guard.display_name(dn.as_str());
        }
        service = service.guard(guard);
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

    // .transition(Transition::new("from", "event", "to")) with optional .guard("name")
    let trans_re = Regex::new(
        r#"Transition::new\("([^"]+)",\s*"([^"]+)",\s*"([^"]+)"\)(?:\.guard\("([^"]+)"\))?"#,
    )
    .unwrap();
    for cap in trans_re.captures_iter(content) {
        let mut transition = Transition::new(&cap[1], &cap[2], &cap[3]);
        if let Some(guard_match) = cap.get(4) {
            transition = transition.guard(guard_match.as_str());
        }
        machine = machine.transition(transition);
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
            json_ui: serde_json::json!({"$schema": "ferro-json-ui/v2"}),
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
        assert!(json_str.contains("ferro-json-ui/v2"));
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

    #[test]
    fn test_reconstruct_action_with_transition_trigger() {
        let content = r#"
ServiceDef::new("order")
    .action(ActionDef::new("submit").transition_trigger("submit"))
        "#;

        let service = reconstruct_service_def("order", &None, content).unwrap();
        assert_eq!(service.actions.len(), 1);
        assert_eq!(service.actions[0].name, "submit");
        assert_eq!(
            service.actions[0].transition_trigger.as_deref(),
            Some("submit")
        );
    }

    #[test]
    fn test_reconstruct_action_with_precondition() {
        let content = r#"
ServiceDef::new("order")
    .action(ActionDef::new("approve").precondition("is_manager"))
        "#;

        let service = reconstruct_service_def("order", &None, content).unwrap();
        assert_eq!(service.actions.len(), 1);
        assert_eq!(service.actions[0].preconditions, vec!["is_manager"]);
    }

    #[test]
    fn test_reconstruct_action_with_multiple_preconditions() {
        let content = r#"
ServiceDef::new("order")
    .action(ActionDef::new("process").precondition("has_items").precondition("payment_valid"))
        "#;

        let service = reconstruct_service_def("order", &None, content).unwrap();
        assert_eq!(service.actions.len(), 1);
        assert_eq!(
            service.actions[0].preconditions,
            vec!["has_items", "payment_valid"]
        );
    }

    #[test]
    fn test_reconstruct_action_with_inputs() {
        let content = r#"
ServiceDef::new("feedback")
    .action(
        ActionDef::new("submit_feedback")
            .input(InputDef::new("subject", DataType::String, FieldMeaning::EntityName))
            .input(InputDef::new("rating", DataType::Integer, FieldMeaning::Quantity))
    )
        "#;

        let service = reconstruct_service_def("feedback", &None, content).unwrap();
        assert_eq!(service.actions.len(), 1);
        assert_eq!(service.actions[0].inputs.len(), 2);
        assert_eq!(service.actions[0].inputs[0].name, "subject");
        assert_eq!(service.actions[0].inputs[0].data_type, DataType::String);
        assert_eq!(
            service.actions[0].inputs[0].meaning,
            FieldMeaning::EntityName
        );
        assert_eq!(service.actions[0].inputs[1].name, "rating");
    }

    #[test]
    fn test_reconstruct_action_with_display_name() {
        let content = r#"
ServiceDef::new("order")
    .action(ActionDef::new("approve").display_name("Approve Order"))
        "#;

        let service = reconstruct_service_def("order", &None, content).unwrap();
        assert_eq!(service.actions.len(), 1);
        assert_eq!(
            service.actions[0].display_name.as_deref(),
            Some("Approve Order")
        );
    }

    #[test]
    fn test_reconstruct_action_full_chain() {
        let content = r#"
ServiceDef::new("order")
    .action(
        ActionDef::new("approve")
            .display_name("Approve Order")
            .transition_trigger("approve")
            .precondition("is_manager")
            .input(InputDef::new("notes", DataType::String, FieldMeaning::FreeText))
    )
        "#;

        let service = reconstruct_service_def("order", &None, content).unwrap();
        assert_eq!(service.actions.len(), 1);
        let action = &service.actions[0];
        assert_eq!(action.name, "approve");
        assert_eq!(action.display_name.as_deref(), Some("Approve Order"));
        assert_eq!(action.transition_trigger.as_deref(), Some("approve"));
        assert_eq!(action.preconditions, vec!["is_manager"]);
        assert_eq!(action.inputs.len(), 1);
        assert_eq!(action.inputs[0].name, "notes");
    }

    #[test]
    fn test_reconstruct_guarded_transitions() {
        let content = r#"
ServiceDef::new("order")
    .state_machine(
        StateMachine::new("lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("approved"))
            .transition(Transition::new("draft", "approve", "approved").guard("is_manager"))
    )
        "#;

        let service = reconstruct_service_def("order", &None, content).unwrap();
        let sm = service.state_machine.as_ref().unwrap();
        assert_eq!(sm.transitions.len(), 1);
        assert_eq!(sm.transitions[0].guard.as_deref(), Some("is_manager"));
    }

    #[test]
    fn test_reconstruct_guard_defs() {
        let content = r#"
ServiceDef::new("order")
    .guard(GuardDef::new("is_manager").display_name("Manager Approval Required"))
    .guard(GuardDef::new("has_items"))
        "#;

        let service = reconstruct_service_def("order", &None, content).unwrap();
        assert_eq!(service.guards.len(), 2);
        assert_eq!(service.guards[0].name, "is_manager");
        assert_eq!(
            service.guards[0].display_name.as_deref(),
            Some("Manager Approval Required")
        );
        assert_eq!(service.guards[1].name, "has_items");
        assert!(service.guards[1].display_name.is_none());
    }

    #[test]
    fn test_reconstruct_full_order_service() {
        let content = r#"
ServiceDef::new("order")
    .display_name("Order")
    .field("id", DataType::Integer, FieldMeaning::Identifier)
    .field("customer_name", DataType::String, FieldMeaning::EntityName)
    .field("total", DataType::Float, FieldMeaning::Money)
    .field("status", DataType::String, FieldMeaning::Status)
    .state_machine(
        StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("submitted"))
            .state(StateDef::new("approved"))
            .state(StateDef::new("delivered").final_state())
            .state(StateDef::new("cancelled").final_state())
            .transition(Transition::new("draft", "submit", "submitted"))
            .transition(Transition::new("submitted", "approve", "approved").guard("is_manager"))
            .transition(Transition::new("submitted", "reject", "cancelled"))
            .transition(Transition::new("approved", "ship", "delivered"))
    )
    .guard(GuardDef::new("is_manager"))
    .action(ActionDef::new("submit").transition_trigger("submit"))
    .action(ActionDef::new("approve").transition_trigger("approve").precondition("is_manager"))
    .belongs_to("customer", "user")
    .has_many("line_items", "line_item")
        "#;

        let service =
            reconstruct_service_def("order", &Some("Order".to_string()), content).unwrap();

        // Fields
        assert_eq!(service.fields.len(), 4);

        // State machine with guarded transition
        let sm = service.state_machine.as_ref().unwrap();
        assert_eq!(sm.transitions.len(), 4);
        let guarded: Vec<_> = sm
            .transitions
            .iter()
            .filter(|t| t.guard.is_some())
            .collect();
        assert_eq!(guarded.len(), 1);
        assert_eq!(guarded[0].guard.as_deref(), Some("is_manager"));

        // Guards
        assert_eq!(service.guards.len(), 1);
        assert_eq!(service.guards[0].name, "is_manager");

        // Actions with transition triggers and preconditions
        assert_eq!(service.actions.len(), 2);
        assert_eq!(
            service.actions[0].transition_trigger.as_deref(),
            Some("submit")
        );
        assert_eq!(
            service.actions[1].transition_trigger.as_deref(),
            Some("approve")
        );
        assert_eq!(service.actions[1].preconditions, vec!["is_manager"]);

        // Relationships
        assert_eq!(service.relationships.len(), 2);

        // Derive intents and verify Process is primary
        let intents = derive_intents(&service);
        assert!(!intents.is_empty(), "Should derive at least one intent");
        assert_eq!(
            intents[0].intent,
            ferro_projections::Intent::Process,
            "Order with guarded state machine should derive Process intent, got {:?}",
            intents[0].intent
        );
    }

    #[test]
    fn test_extract_action_blocks_multiple() {
        let content = r#"
    .action(ActionDef::new("submit").transition_trigger("submit"))
    .action(ActionDef::new("approve").precondition("is_manager"))
        "#;

        let blocks = extract_action_blocks(content);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].contains("submit"));
        assert!(blocks[1].contains("approve"));
    }

    #[test]
    fn test_extract_action_blocks_nested_parens() {
        let content = r#"
    .action(
        ActionDef::new("submit_feedback")
            .input(InputDef::new("name", DataType::String, FieldMeaning::EntityName))
            .input(InputDef::new("rating", DataType::Integer, FieldMeaning::Quantity))
    )
        "#;

        let blocks = extract_action_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].contains("submit_feedback"));
        assert!(blocks[0].contains("InputDef::new"));
    }

    // --- Integration tests against real projection files ---

    /// Helper to read a projection source file from the sample app.
    fn read_projection_source(name: &str) -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest_dir)
            .join("../app/src/projections")
            .join(format!("{name}.rs"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
    }

    /// Reconstruct a ServiceDef from a real projection file and validate it.
    fn reconstruct_and_validate(
        name: &str,
        display: &str,
    ) -> (ServiceDef, Vec<ferro_projections::IntentScore>) {
        let content = read_projection_source(name);
        let service = reconstruct_service_def(name, &Some(display.to_string()), &content).unwrap();

        // Validate — no errors allowed, warnings are OK
        let warnings = service.validate().unwrap_or_else(|e| {
            panic!("ServiceDef::validate() returned error for {name}: {e}");
        });
        let _ = warnings; // warnings are acceptable

        let intents = derive_intents(&service);
        assert!(
            !intents.is_empty(),
            "{name}: should derive at least one intent"
        );
        (service, intents)
    }

    #[test]
    fn test_integration_user_projection() {
        let (service, intents) = reconstruct_and_validate("user", "User");
        assert_eq!(service.fields.len(), 5);
        // Model-based: accept any reasonable intent
        let _ = intents[0].intent.clone();
    }

    #[test]
    fn test_integration_todo_projection() {
        let (service, intents) = reconstruct_and_validate("todo", "Todo");
        assert_eq!(service.fields.len(), 5);
        // Model-based: accept any reasonable intent
        let _ = intents[0].intent.clone();
    }

    #[test]
    fn test_integration_api_key_projection() {
        let (service, intents) = reconstruct_and_validate("api_key", "Api Key");
        assert_eq!(service.fields.len(), 8);
        // Model-based: accept any reasonable intent
        let _ = intents[0].intent.clone();
    }

    #[test]
    fn test_integration_order_projection() {
        let (service, intents) = reconstruct_and_validate("order", "Order");
        use ferro_projections::Intent;

        // Verify reconstruction completeness
        assert_eq!(service.fields.len(), 5);
        assert!(service.state_machine.is_some());
        let sm = service.state_machine.as_ref().unwrap();
        assert_eq!(sm.states.len(), 6);
        assert_eq!(sm.transitions.len(), 6);
        // Check guarded transition was parsed
        let guarded_count = sm.transitions.iter().filter(|t| t.guard.is_some()).count();
        assert!(
            guarded_count >= 1,
            "order: expected at least 1 guarded transition, got {guarded_count}"
        );
        // Check guards parsed
        assert!(
            !service.guards.is_empty(),
            "order: expected guard definitions"
        );
        // Check action details
        assert!(
            service.actions.len() >= 3,
            "order: expected at least 3 actions, got {}",
            service.actions.len()
        );
        let trigger_count = service
            .actions
            .iter()
            .filter(|a| a.transition_trigger.is_some())
            .count();
        assert!(
            trigger_count >= 2,
            "order: expected at least 2 actions with transition_trigger, got {trigger_count}"
        );
        let precondition_count = service
            .actions
            .iter()
            .filter(|a| !a.preconditions.is_empty())
            .count();
        assert!(
            precondition_count >= 1,
            "order: expected at least 1 action with preconditions, got {precondition_count}"
        );
        assert_eq!(service.relationships.len(), 2);

        // Assert exact primary intent
        assert_eq!(
            intents[0].intent,
            Intent::Process,
            "order: expected Process intent, got {:?} (confidence: {}, signals: {:?})",
            intents[0].intent,
            intents[0].confidence,
            intents[0].matching_signals
        );
    }

    #[test]
    fn test_integration_product_projection() {
        let (service, intents) = reconstruct_and_validate("product", "Product");
        use ferro_projections::Intent;

        assert_eq!(service.fields.len(), 6);
        assert_eq!(service.relationships.len(), 3);

        assert_eq!(
            intents[0].intent,
            Intent::Browse,
            "product: expected Browse intent, got {:?} (confidence: {}, signals: {:?})",
            intents[0].intent,
            intents[0].confidence,
            intents[0].matching_signals
        );
    }

    #[test]
    fn test_integration_revenue_dashboard_projection() {
        let (service, intents) = reconstruct_and_validate("revenue_dashboard", "Revenue Dashboard");
        use ferro_projections::Intent;

        assert_eq!(service.fields.len(), 6);
        // All non-id fields should be read-only
        let read_only_count = service.fields.iter().filter(|f| !f.writable).count();
        assert!(
            read_only_count >= 5,
            "revenue_dashboard: expected at least 5 read-only fields, got {read_only_count}"
        );

        assert_eq!(
            intents[0].intent,
            Intent::Summarize,
            "revenue_dashboard: expected Summarize intent, got {:?} (confidence: {}, signals: {:?})",
            intents[0].intent,
            intents[0].confidence,
            intents[0].matching_signals
        );
    }

    #[test]
    fn test_integration_sales_analytics_projection() {
        let (service, intents) = reconstruct_and_validate("sales_analytics", "Sales Analytics");
        use ferro_projections::Intent;

        assert_eq!(service.fields.len(), 5);

        assert_eq!(
            intents[0].intent,
            Intent::Analyze,
            "sales_analytics: expected Analyze intent, got {:?} (confidence: {}, signals: {:?})",
            intents[0].intent,
            intents[0].confidence,
            intents[0].matching_signals
        );
    }

    #[test]
    fn test_integration_feedback_form_projection() {
        let (service, intents) = reconstruct_and_validate("feedback_form", "Feedback Form");
        use ferro_projections::Intent;

        assert_eq!(service.fields.len(), 6);
        // Check write-only fields were parsed
        let write_only_count = service
            .fields
            .iter()
            .filter(|f| f.writable && !f.readable)
            .count();
        assert!(
            write_only_count >= 2,
            "feedback_form: expected at least 2 write-only fields, got {write_only_count}"
        );
        // Check action inputs were parsed
        assert!(
            !service.actions.is_empty(),
            "feedback_form: expected at least 1 action"
        );
        let input_count: usize = service.actions.iter().map(|a| a.inputs.len()).sum();
        assert!(
            input_count >= 3,
            "feedback_form: expected at least 3 action inputs, got {input_count}"
        );

        assert_eq!(
            intents[0].intent,
            Intent::Collect,
            "feedback_form: expected Collect intent, got {:?} (confidence: {}, signals: {:?})",
            intents[0].intent,
            intents[0].confidence,
            intents[0].matching_signals
        );
    }

    #[test]
    fn test_integration_all_projections_validate() {
        let projections = [
            ("user", "User"),
            ("todo", "Todo"),
            ("api_key", "Api Key"),
            ("order", "Order"),
            ("product", "Product"),
            ("revenue_dashboard", "Revenue Dashboard"),
            ("sales_analytics", "Sales Analytics"),
            ("feedback_form", "Feedback Form"),
        ];

        for (name, display) in &projections {
            let content = read_projection_source(name);
            let service =
                reconstruct_service_def(name, &Some(display.to_string()), &content).unwrap();
            service.validate().unwrap_or_else(|e| {
                panic!("ServiceDef::validate() error for {name}: {e}");
            });
        }
    }
}
