//! CLI command: `ferro projection:check`
//!
//! Scans `src/projections/` for ServiceDef functions, reconstructs them
//! via regex-based source parsing, and validates structural correctness.

use ferro_projections::{
    ActionDef, Cardinality, DataType, FieldMeaning, IntentHint, ServiceDef, StateDef, StateMachine,
    Transition,
};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process;

/// Result of validating a single projection.
struct CheckResult {
    fn_name: String,
    file: String,
    warnings: Vec<String>,
    errors: Vec<String>,
}

/// Execute the projection:check command.
pub fn execute(name: Option<&str>) {
    let project_root = std::env::current_dir().unwrap_or_else(|_| {
        eprintln!("Error: could not determine current directory");
        process::exit(1);
    });

    let projections_dir = project_root.join("src/projections");
    if !projections_dir.exists() {
        println!("No projections directory found at src/projections/");
        return;
    }

    let discovered = discover_projections(&project_root);
    if discovered.is_empty() {
        println!("No projections found in src/projections/");
        return;
    }

    // Filter by name if provided
    let targets: Vec<_> = if let Some(filter) = name {
        discovered
            .into_iter()
            .filter(|(fn_name, _)| fn_name == filter)
            .collect()
    } else {
        discovered
    };

    if targets.is_empty() {
        if let Some(filter) = name {
            eprintln!("Projection '{filter}' not found in src/projections/");
            process::exit(1);
        }
        println!("No projections found in src/projections/");
        return;
    }

    println!("Checking projections...");

    let mut results = Vec::new();
    for (fn_name, file) in &targets {
        let result = check_projection(&project_root, fn_name, file);
        results.push(result);
    }

    // Print results
    let mut total_warnings = 0usize;
    let mut total_errors = 0usize;
    let mut projections_with_issues = 0usize;

    for result in &results {
        let issue_count = result.warnings.len() + result.errors.len();
        if issue_count == 0 {
            println!(
                "  \u{2713} {} ({}) \u{2014} 0 warnings",
                result.fn_name, result.file
            );
        } else {
            projections_with_issues += 1;
            if !result.errors.is_empty() {
                println!(
                    "  \u{2717} {} ({}) \u{2014} {} error(s), {} warning(s)",
                    result.fn_name,
                    result.file,
                    result.errors.len(),
                    result.warnings.len()
                );
            } else {
                println!(
                    "  \u{26a0} {} ({}) \u{2014} {} warning(s)",
                    result.fn_name,
                    result.file,
                    result.warnings.len()
                );
            }
            for err in &result.errors {
                println!("    - Error: {err}");
            }
            for warn in &result.warnings {
                println!("    - {warn}");
            }
        }
        total_warnings += result.warnings.len();
        total_errors += result.errors.len();
    }

    println!();
    println!(
        "{} projection(s) checked, {} warning(s), {} error(s) in {} projection(s)",
        results.len(),
        total_warnings,
        total_errors,
        projections_with_issues
    );

    if total_errors > 0 {
        process::exit(1);
    }
}

/// Discover all projection functions in src/projections/.
fn discover_projections(project_root: &Path) -> Vec<(String, String)> {
    let projections_dir = project_root.join("src/projections");
    let fn_re = Regex::new(r"pub\s+fn\s+(\w+)\s*\(.*\).*->\s*ServiceDef").unwrap();

    let mut result = Vec::new();

    let entries: Vec<_> = match fs::read_dir(&projections_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return result,
    };

    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        if path.file_name().is_some_and(|n| n == "mod.rs") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let relative = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        for cap in fn_re.captures_iter(&content) {
            result.push((cap[1].to_string(), relative.clone()));
        }
    }

    result
}

/// Check a single projection: read source, reconstruct ServiceDef, validate.
fn check_projection(project_root: &Path, fn_name: &str, file: &str) -> CheckResult {
    let file_path = project_root.join(file);
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            return CheckResult {
                fn_name: fn_name.to_string(),
                file: file.to_string(),
                warnings: Vec::new(),
                errors: vec![format!("could not read file: {}", e)],
            }
        }
    };

    // Extract service name and display name
    let service_name_re = Regex::new(r#"ServiceDef::new\("([^"]+)"\)"#).unwrap();
    let display_name_re = Regex::new(r#"\.display_name\("([^"]+)"\)"#).unwrap();

    let service_name = service_name_re
        .captures(&content)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| fn_name.to_string());
    let display_name = display_name_re.captures(&content).map(|c| c[1].to_string());

    let service = match reconstruct_service_def(&service_name, &display_name, &content) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult {
                fn_name: fn_name.to_string(),
                file: file.to_string(),
                warnings: Vec::new(),
                errors: vec![format!("reconstruction failed: {}", e)],
            }
        }
    };

    match service.validate() {
        Ok(warnings) => CheckResult {
            fn_name: fn_name.to_string(),
            file: file.to_string(),
            warnings: warnings.iter().map(|w| format!("{w:?}")).collect(),
            errors: Vec::new(),
        },
        Err(e) => CheckResult {
            fn_name: fn_name.to_string(),
            file: file.to_string(),
            warnings: Vec::new(),
            errors: vec![e.to_string()],
        },
    }
}

/// Reconstruct a ServiceDef from source code using regex parsing.
///
/// Replicates the reconstruction logic from ferro-mcp's render_projection tool.
fn reconstruct_service_def(
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

fn parse_and_add_fields(mut service: ServiceDef, content: &str) -> ServiceDef {
    let field_re =
        Regex::new(r#"\.field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#).unwrap();
    for cap in field_re.captures_iter(content) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            service = service.field(&cap[1], dt, fm);
        }
    }

    let opt_re =
        Regex::new(r#"\.optional_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in opt_re.captures_iter(content) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            service = service.optional_field(&cap[1], dt, fm);
        }
    }

    let ro_re =
        Regex::new(r#"\.read_only_field\("([^"]+)",\s*DataType::(\w+),\s*FieldMeaning::(\w+)\)"#)
            .unwrap();
    for cap in ro_re.captures_iter(content) {
        if let (Some(dt), Some(fm)) = (parse_data_type(&cap[2]), parse_field_meaning(&cap[3])) {
            service = service.read_only_field(&cap[1], dt, fm);
        }
    }

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

fn parse_and_add_relationships(mut service: ServiceDef, content: &str) -> ServiceDef {
    let hm_re = Regex::new(r#"\.has_many\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in hm_re.captures_iter(content) {
        service = service.has_many(&cap[1], &cap[2]);
    }

    let bt_re = Regex::new(r#"\.belongs_to\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in bt_re.captures_iter(content) {
        service = service.belongs_to(&cap[1], &cap[2]);
    }

    let ho_re = Regex::new(r#"\.has_one\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in ho_re.captures_iter(content) {
        service = service.has_one(&cap[1], &cap[2]);
    }

    let btm_re = Regex::new(r#"\.belongs_to_many\("([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in btm_re.captures_iter(content) {
        service = service.belongs_to_many(&cap[1], &cap[2]);
    }

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

fn parse_and_add_actions(mut service: ServiceDef, content: &str) -> ServiceDef {
    let action_re = Regex::new(r#"\.action\(ActionDef::new\("([^"]+)"\)"#).unwrap();
    for cap in action_re.captures_iter(content) {
        let action = ActionDef::new(&cap[1]);
        service = service.action(action);
    }
    service
}

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

fn parse_state_machine(content: &str) -> Option<StateMachine> {
    let name_re = Regex::new(r#"StateMachine::new\("([^"]+)"\)"#).unwrap();
    let name = name_re.captures(content).map(|c| c[1].to_string())?;

    let initial_re = Regex::new(r#"\.initial\("([^"]+)"\)"#).unwrap();
    let initial = initial_re
        .captures(content)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "initial".to_string());

    let mut machine = StateMachine::new(&name).initial(&initial);

    let final_state_re = Regex::new(r#"StateDef::new\("([^"]+)"\)[^;]*\.final_state\(\)"#).unwrap();
    let final_states: HashSet<String> = final_state_re
        .captures_iter(content)
        .map(|c| c[1].to_string())
        .collect();

    let state_re = Regex::new(r#"StateDef::new\("([^"]+)"\)"#).unwrap();
    for cap in state_re.captures_iter(content) {
        let state_name = cap[1].to_string();
        let mut state = StateDef::new(&state_name);
        if final_states.contains(&state_name) {
            state = state.final_state();
        }
        machine = machine.state(state);
    }

    let trans_re = Regex::new(r#"Transition::new\("([^"]+)",\s*"([^"]+)",\s*"([^"]+)"\)"#).unwrap();
    for cap in trans_re.captures_iter(content) {
        machine = machine.transition(Transition::new(&cap[1], &cap[2], &cap[3]));
    }

    Some(machine)
}

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

fn parse_cardinality(s: &str) -> Option<Cardinality> {
    match s {
        "OneToOne" => Some(Cardinality::OneToOne),
        "OneToMany" => Some(Cardinality::OneToMany),
        "ManyToOne" => Some(Cardinality::ManyToOne),
        "ManyToMany" => Some(Cardinality::ManyToMany),
        _ => None,
    }
}

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
    fn test_discover_empty_project() {
        let tmp = std::path::PathBuf::from("/tmp/ferro_projection_check_test_empty");
        let result = discover_projections(&tmp);
        assert!(result.is_empty());
    }

    #[test]
    fn test_reconstruct_minimal() {
        let content = r#"
pub fn user_service() -> ServiceDef {
    ServiceDef::new("user")
        .display_name("User")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
}
        "#;

        let service = reconstruct_service_def("user", &Some("User".to_string()), content);
        assert!(service.is_ok());
        let svc = service.unwrap();
        assert_eq!(svc.name, "user");
        assert_eq!(svc.fields.len(), 2);
    }

    #[test]
    fn test_check_valid_projection() {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        fs::create_dir_all(&proj_dir).unwrap();

        let content = r#"
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("total", DataType::Float, FieldMeaning::Money)
}
        "#;
        fs::write(proj_dir.join("order.rs"), content).unwrap();

        let result = check_projection(tmp.path(), "order_service", "src/projections/order.rs");
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_check_projection_with_orphan_state() {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        fs::create_dir_all(&proj_dir).unwrap();

        // Create a projection with an orphan state (unreachable + dead end)
        let content = r#"
use ferro::{ServiceDef, DataType, FieldMeaning, StateMachine, StateDef, Transition};

pub fn broken_service() -> ServiceDef {
    ServiceDef::new("broken")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .state_machine(
            StateMachine::new("lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("published").final_state())
                .state(StateDef::new("orphan"))
                .transition(Transition::new("draft", "publish", "published"))
        )
}
        "#;
        fs::write(proj_dir.join("broken.rs"), content).unwrap();

        let result = check_projection(tmp.path(), "broken_service", "src/projections/broken.rs");
        assert!(result.errors.is_empty());
        assert!(
            !result.warnings.is_empty(),
            "Should have warnings for orphan state"
        );
    }

    #[test]
    fn test_discover_projections() {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        fs::create_dir_all(&proj_dir).unwrap();

        fs::write(
            proj_dir.join("user.rs"),
            r#"pub fn user_service() -> ServiceDef { ServiceDef::new("user") }"#,
        )
        .unwrap();

        // mod.rs should be skipped
        fs::write(proj_dir.join("mod.rs"), "pub mod user;").unwrap();

        let discovered = discover_projections(tmp.path());
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].0, "user_service");
    }
}
