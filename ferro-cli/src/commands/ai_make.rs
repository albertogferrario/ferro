//! `ferro ai:make <description>` — AI-powered ServiceDef generator.
//!
//! Thin CLI wrapper over `ferro_mcp::tools::ai_scaffold::scaffold_core`. All
//! introspection, relevance filtering, prompt assembly, LLM completion, and
//! validation run inside the core. This module owns the tokio runtime bridge,
//! console output, file write, and process exit — writing exactly one
//! `src/projections/<snake>.rs` file on success.

#[cfg(feature = "projections")]
use ferro_projections::{
    ActionDef, Cardinality, DataType, FieldDef, FieldMeaning, Intent, IntentHint, RelationshipDef,
    ServiceDef, StateMachine,
};

#[cfg(feature = "projections")]
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// ServiceDef → Rust builder source emitter
// ---------------------------------------------------------------------------

/// Emit a `ServiceDef` as idiomatic Rust builder source.
///
/// Produces a `pub fn <snake_name>_service() -> ServiceDef { ... }` function
/// ready to drop into `src/projections/<snake_name>.rs`.
///
/// The function name is derived from the snake_case-validated identifier (same
/// as the file/module name) so the generated code is always valid Rust that
/// passes `clippy -D warnings` without a `non_snake_case` warning.
#[cfg(feature = "projections")]
pub(crate) fn emit_service_def_source(service: &ServiceDef) -> String {
    let name = &service.name;
    // Use the same snake_case name that resolve_projection_path uses for the
    // file name — ensures the function identifier is always valid snake_case.
    let snake_name = crate::naming::to_snake_case(name);
    let fn_name = format!("{snake_name}_service");

    // Collect which types are actually used so the `use ferro::{ ... }` header
    // imports only what is needed.
    let mut uses_action = false;
    let mut uses_guard = false;
    let mut uses_relationship = false;
    let mut uses_state_machine = false;
    let mut uses_intent_hint = false;
    let mut uses_intent = false;
    let mut uses_cardinality = false;

    if !service.actions.is_empty() {
        uses_action = true;
    }
    if !service.guards.is_empty() {
        uses_guard = true;
    }
    if !service.relationships.is_empty() {
        uses_relationship = true;
        uses_cardinality = true;
    }
    if service.state_machine.is_some() {
        uses_state_machine = true;
    }
    if !service.intent_hints.is_empty() {
        uses_intent_hint = true;
        uses_intent = true;
    }

    // Build the `use ferro::{...}` imports line
    let mut use_items: Vec<&str> = vec!["DataType", "FieldMeaning", "ServiceDef"];
    if uses_action {
        use_items.push("ActionDef");
    }
    if uses_guard {
        use_items.push("GuardDef");
    }
    if uses_relationship {
        use_items.push("RelationshipDef");
    }
    if uses_cardinality {
        use_items.push("Cardinality");
    }
    if uses_state_machine {
        use_items.push("StateDef");
        use_items.push("StateMachine");
        use_items.push("Transition");
    }
    if uses_intent_hint {
        use_items.push("IntentHint");
    }
    if uses_intent {
        use_items.push("Intent");
    }
    use_items.sort_unstable();
    use_items.dedup();

    let use_line = format!("use ferro::{{{}}};\n", use_items.join(", "));

    // Build the builder chain lines
    let mut chain: Vec<String> = Vec::new();
    chain.push(format!("    ServiceDef::new({name:?})"));

    if let Some(ref dn) = service.display_name {
        chain.push(format!("        .display_name({dn:?})"));
    }
    if let Some(ref desc) = service.description {
        chain.push(format!("        .description({desc:?})"));
    }

    for field in &service.fields {
        let builder_method = field_builder_method(field);
        let dt = emit_data_type(&field.data_type);
        let meaning = emit_field_meaning(&field.meaning);
        chain.push(format!(
            "        .{builder_method}({:?}, {dt}, {meaning})",
            field.name
        ));
    }

    for guard in &service.guards {
        chain.push(format!("        .guard(GuardDef::new({:?}))", guard.name));
    }

    for action in &service.actions {
        chain.push(emit_action_def(action));
    }

    for rel in &service.relationships {
        chain.push(emit_relationship_def(rel));
    }

    for hint in &service.intent_hints {
        chain.push(emit_intent_hint(hint));
    }

    if let Some(ref sm) = service.state_machine {
        chain.push(emit_state_machine(sm));
    }

    let builder_body = chain.join("\n");

    format!(
        "{use_line}\n/// Build the {name} service projection.\npub fn {fn_name}() -> ServiceDef {{\n{builder_body}\n}}\n"
    )
}

/// Select the builder method name based on FieldDef flags.
///
/// Priority order (first match wins):
/// - `!readable` → write_only_field
/// - `!writable` → read_only_field
/// - `is_list`   → list_field
/// - `!required` → optional_field
/// - default     → field
#[cfg(feature = "projections")]
fn field_builder_method(field: &FieldDef) -> &'static str {
    if !field.readable {
        "write_only_field"
    } else if !field.writable {
        "read_only_field"
    } else if field.is_list {
        "list_field"
    } else if !field.required {
        "optional_field"
    } else {
        "field"
    }
}

/// Emit the Rust identifier for a DataType variant.
///
/// REQUIRED: explicit match — DataType has `#[serde(rename_all = "snake_case")]`
/// so `serde_json::to_string(&DataType::DateTime)` → `"date_time"`, not `"DateTime"`.
#[cfg(feature = "projections")]
fn emit_data_type(dt: &DataType) -> &'static str {
    match dt {
        DataType::String => "DataType::String",
        DataType::Integer => "DataType::Integer",
        DataType::Float => "DataType::Float",
        DataType::Boolean => "DataType::Boolean",
        DataType::DateTime => "DataType::DateTime",
        DataType::Date => "DataType::Date",
        DataType::Json => "DataType::Json",
        DataType::Binary => "DataType::Binary",
        DataType::Uuid => "DataType::Uuid",
        DataType::Enum => "DataType::Enum",
    }
}

/// Emit the Rust expression for a FieldMeaning value.
///
/// REQUIRED: explicit match for all 18 known variants — `Custom(String)` uses
/// `#[serde(untagged)]` so serde cannot distinguish it from known variants at
/// the JSON level. We must check all known variants first.
#[cfg(feature = "projections")]
fn emit_field_meaning(m: &FieldMeaning) -> String {
    match m {
        FieldMeaning::Identifier => "FieldMeaning::Identifier".into(),
        FieldMeaning::ForeignKey => "FieldMeaning::ForeignKey".into(),
        FieldMeaning::EntityName => "FieldMeaning::EntityName".into(),
        FieldMeaning::Email => "FieldMeaning::Email".into(),
        FieldMeaning::Phone => "FieldMeaning::Phone".into(),
        FieldMeaning::Url => "FieldMeaning::Url".into(),
        FieldMeaning::ImageUrl => "FieldMeaning::ImageUrl".into(),
        FieldMeaning::Money => "FieldMeaning::Money".into(),
        FieldMeaning::Percentage => "FieldMeaning::Percentage".into(),
        FieldMeaning::Quantity => "FieldMeaning::Quantity".into(),
        FieldMeaning::Status => "FieldMeaning::Status".into(),
        FieldMeaning::Category => "FieldMeaning::Category".into(),
        FieldMeaning::Boolean => "FieldMeaning::Boolean".into(),
        FieldMeaning::FreeText => "FieldMeaning::FreeText".into(),
        FieldMeaning::CreatedAt => "FieldMeaning::CreatedAt".into(),
        FieldMeaning::UpdatedAt => "FieldMeaning::UpdatedAt".into(),
        FieldMeaning::DateTime => "FieldMeaning::DateTime".into(),
        FieldMeaning::Sensitive => "FieldMeaning::Sensitive".into(),
        FieldMeaning::Custom(s) => format!(r#"FieldMeaning::Custom({s:?}.into())"#),
    }
}

/// Emit an ActionDef builder chain line.
#[cfg(feature = "projections")]
fn emit_action_def(action: &ActionDef) -> String {
    let mut parts = vec![format!("ActionDef::new({:?})", action.name)];
    if let Some(ref dn) = action.display_name {
        parts.push(format!(".display_name({dn:?})"));
    }
    if let Some(ref desc) = action.description {
        parts.push(format!(".description({desc:?})"));
    }
    for pre in &action.preconditions {
        parts.push(format!(".precondition({pre:?})"));
    }
    for eff in &action.effects {
        parts.push(format!(".effect({eff:?})"));
    }
    if let Some(ref trigger) = action.transition_trigger {
        parts.push(format!(".transition_trigger({trigger:?})"));
    }
    format!("        .action({})", parts.join(""))
}

/// Emit a RelationshipDef builder chain line.
#[cfg(feature = "projections")]
fn emit_relationship_def(rel: &RelationshipDef) -> String {
    let card = emit_cardinality(&rel.cardinality);
    let mut parts = vec![format!(
        "RelationshipDef::new({:?}, {:?}, {card})",
        rel.name, rel.target
    )];
    if let Some(ref fk) = rel.foreign_key {
        parts.push(format!(".foreign_key({fk:?})"));
    }
    if let Some(ref inv) = rel.inverse {
        parts.push(format!(".inverse({inv:?})"));
    }
    format!("        .relationship({})", parts.join(""))
}

/// Emit the Rust identifier for a Cardinality variant.
///
/// REQUIRED: explicit match — Cardinality has `#[serde(rename_all = "snake_case")]`.
#[cfg(feature = "projections")]
fn emit_cardinality(card: &Cardinality) -> &'static str {
    match card {
        Cardinality::OneToOne => "Cardinality::OneToOne",
        Cardinality::OneToMany => "Cardinality::OneToMany",
        Cardinality::ManyToOne => "Cardinality::ManyToOne",
        Cardinality::ManyToMany => "Cardinality::ManyToMany",
    }
}

/// Emit an IntentHint builder line.
///
/// IntentHint is an externally-tagged enum: Primary(Intent) | Exclude(Intent).
#[cfg(feature = "projections")]
fn emit_intent_hint(hint: &IntentHint) -> String {
    match hint {
        IntentHint::Primary(intent) => {
            format!(
                "        .intent_hint(IntentHint::Primary({}))",
                emit_intent(intent)
            )
        }
        IntentHint::Exclude(intent) => {
            format!(
                "        .intent_hint(IntentHint::Exclude({}))",
                emit_intent(intent)
            )
        }
    }
}

/// Emit the Rust expression for an Intent value.
///
/// REQUIRED: explicit match — Intent has `#[serde(rename_all = "snake_case")]`
/// and `Custom(String)` uses `#[serde(untagged)]`.
#[cfg(feature = "projections")]
fn emit_intent(intent: &Intent) -> String {
    match intent {
        Intent::Browse => "Intent::Browse".into(),
        Intent::Focus => "Intent::Focus".into(),
        Intent::Collect => "Intent::Collect".into(),
        Intent::Process => "Intent::Process".into(),
        Intent::Summarize => "Intent::Summarize".into(),
        Intent::Analyze => "Intent::Analyze".into(),
        Intent::Track => "Intent::Track".into(),
        Intent::Custom(s) => format!(r#"Intent::Custom({s:?}.into())"#),
    }
}

/// Emit a StateMachine builder chain as a multi-line block for `.state_machine(...)`.
#[cfg(feature = "projections")]
fn emit_state_machine(sm: &StateMachine) -> String {
    let mut lines = vec![format!(
        "        .state_machine(StateMachine::new({:?})",
        sm.name
    )];

    if let Some(ref dn) = sm.display_name {
        lines.push(format!("            .display_name({dn:?})"));
    }
    if !sm.initial_state.is_empty() {
        lines.push(format!("            .initial({:?})", sm.initial_state));
    }
    for state in &sm.states {
        let mut s = format!("            .state(StateDef::new({:?})", state.name);
        if let Some(ref dn) = state.display_name {
            s.push_str(&format!(".display_name({dn:?})"));
        }
        if state.is_final {
            s.push_str(".final_state()");
        }
        s.push(')');
        lines.push(s);
    }
    for t in &sm.transitions {
        let mut tr = format!(
            "            .transition(Transition::new({:?}, {:?}, {:?})",
            t.from, t.event, t.to
        );
        if let Some(ref g) = t.guard {
            tr.push_str(&format!(".guard({g:?})"));
        }
        tr.push(')');
        lines.push(tr);
    }
    lines.push("        )".to_string());
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Path sanitization helpers
// ---------------------------------------------------------------------------

/// Resolve and sanitize a projection file path from a raw service name.
///
/// Converts the name to snake_case, validates it is a safe Rust identifier
/// (rejects path traversal, absolute paths, non-identifier chars), then
/// joins it under the fixed `src/projections/` base.
#[cfg(feature = "projections")]
pub(crate) fn resolve_projection_path(raw: &str) -> Result<PathBuf, String> {
    let snake = crate::naming::to_snake_case(raw);
    if !crate::naming::is_valid_identifier(&snake) {
        return Err(format!(
            "'{raw}' is not a valid projection name (must be a Rust identifier after snake_case conversion)"
        ));
    }
    Ok(Path::new("src/projections").join(format!("{snake}.rs")))
}

// ---------------------------------------------------------------------------
// Output result type for dry-run / file-write abstraction
// ---------------------------------------------------------------------------

#[cfg(feature = "projections")]
pub(crate) enum OutputResult {
    /// Dry-run: pretty-printed ServiceDef JSON, no files written.
    DryRun(String),
    /// File written successfully at the given path.
    Written(PathBuf),
    /// Projection file already existed; skipped.
    AlreadyExists(PathBuf),
}

/// Render the ServiceDef output: either pretty JSON (dry_run) or write to disk.
///
/// `out_dir` is the project root; files are written under `out_dir/src/projections/`.
#[cfg(feature = "projections")]
pub(crate) fn render_output(
    service: &ServiceDef,
    dry_run: bool,
    out_dir: &Path,
) -> Result<OutputResult, String> {
    if dry_run {
        let json = serde_json::to_string_pretty(service)
            .map_err(|e| format!("Failed to serialize ServiceDef: {e}"))?;
        return Ok(OutputResult::DryRun(json));
    }

    // Resolve the path relative to out_dir
    let rel = resolve_projection_path(&service.name)?;
    let projection_file = out_dir.join(&rel);

    if projection_file.exists() {
        return Ok(OutputResult::AlreadyExists(projection_file));
    }

    let projections_dir = projection_file
        .parent()
        .ok_or_else(|| "cannot determine projections directory".to_string())?;

    std::fs::create_dir_all(projections_dir)
        .map_err(|e| format!("Failed to create projections directory: {e}"))?;

    let content = emit_service_def_source(service);
    std::fs::write(&projection_file, &content)
        .map_err(|e| format!("Failed to write projection file: {e}"))?;

    // Register in mod.rs
    let file_stem = projection_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&service.name);
    let mod_file = projections_dir.join("mod.rs");

    if mod_file.exists() {
        let mod_content = std::fs::read_to_string(&mod_file).unwrap_or_default();
        let pub_mod_decl = format!("pub mod {file_stem};");
        if !mod_content.contains(&pub_mod_decl) {
            crate::commands::make_projection::update_mod_file(&mod_file, file_stem)
                .map_err(|e| format!("Failed to update mod.rs: {e}"))?;
        }
    } else {
        std::fs::write(&mod_file, format!("pub mod {file_stem};\n"))
            .map_err(|e| format!("Failed to create mod.rs: {e}"))?;
    }

    Ok(OutputResult::Written(projection_file))
}

// ---------------------------------------------------------------------------
// Command entry point
// ---------------------------------------------------------------------------

/// Run the `ferro ai:make <description>` command.
///
/// Thin wrapper over `ferro_mcp::tools::ai_scaffold::scaffold_core`. All
/// introspection, relevance filtering, prompt assembly, LLM completion, and
/// validation run inside the core. The CLI bridge (tokio runtime, console
/// output, file write, process exit) stays here.
/// `--dry-run` prints the ServiceDef as pretty JSON and writes nothing.
#[cfg(feature = "projections")]
pub fn run(description: String, dry_run: bool) {
    use console::style;

    // Tokio runtime bridge (ferro-cli main is sync; the core is async).
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "{} Failed to create tokio runtime: {e}",
                style("Error:").red().bold()
            );
            std::process::exit(1);
        }
    };

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    println!("{} Generating ServiceDef via AI...", style("⏳").cyan());

    let service = match rt.block_on(ferro_mcp::tools::ai_scaffold::scaffold_core(
        &description,
        &cwd,
    )) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} {e}", style("Error:").red().bold());
            std::process::exit(1);
        }
    };

    match render_output(&service, dry_run, &cwd) {
        Ok(OutputResult::DryRun(json)) => {
            println!("{json}");
        }
        Ok(OutputResult::Written(path)) => {
            println!("{} Created {}", style("✓").green(), path.display());
        }
        Ok(OutputResult::AlreadyExists(path)) => {
            eprintln!(
                "{} Projection already exists at {}. Delete it first or use a different name.",
                style("Info:").yellow().bold(),
                path.display()
            );
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("{} {e}", style("Error:").red().bold());
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "projections"))]
mod tests {
    use super::*;
    use ferro_projections::{
        ActionDef, Cardinality, DataType, FieldMeaning, GuardDef, Intent, IntentHint,
        RelationshipDef, ServiceDef, StateDef, StateMachine, Transition,
    };
    use tempfile::TempDir;

    // ---- Emitter unit tests ----

    #[test]
    fn emit_data_type_datetime_is_not_snake_case() {
        assert_eq!(emit_data_type(&DataType::DateTime), "DataType::DateTime");
        // Must NOT produce "DataType::date_time" (what serde would give)
        assert_ne!(emit_data_type(&DataType::DateTime), "DataType::date_time");
    }

    #[test]
    fn emit_field_meaning_known_variant() {
        assert_eq!(
            emit_field_meaning(&FieldMeaning::Money),
            "FieldMeaning::Money"
        );
    }

    #[test]
    fn emit_field_meaning_custom_variant() {
        assert_eq!(
            emit_field_meaning(&FieldMeaning::Custom("sku".into())),
            r#"FieldMeaning::Custom("sku".into())"#
        );
    }

    #[test]
    fn emit_field_meaning_description_escaping() {
        // A custom meaning whose value contains a double-quote
        let m = FieldMeaning::Custom(r#"has "quotes""#.into());
        let emitted = emit_field_meaning(&m);
        // {:?} debug-formats with escaped quotes
        assert!(emitted.contains(r#"\"quotes\""#), "got: {emitted}");
    }

    #[test]
    fn emitter_round_trip() {
        // Build a representative ServiceDef
        let service = ServiceDef::new("test_service")
            .display_name("Test Service")
            .description("A test service with \"quoted\" description")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .optional_field("note", DataType::String, FieldMeaning::FreeText)
            .field("sku", DataType::String, FieldMeaning::Custom("sku".into()))
            .guard(GuardDef::new("authenticated"))
            .action(ActionDef::new("create"))
            .relationship(RelationshipDef::new(
                "customer",
                "customer",
                Cardinality::ManyToOne,
            ))
            .intent_hint(IntentHint::Primary(Intent::Browse))
            .state_machine(
                StateMachine::new("lifecycle")
                    .initial("active")
                    .state(StateDef::new("active"))
                    .state(StateDef::new("closed").final_state())
                    .transition(Transition::new("active", "close", "closed")),
            );

        let source = emit_service_def_source(&service);

        // Check function signature
        assert!(
            source.contains("pub fn test_service_service() -> ServiceDef"),
            "missing function signature\nsource:\n{source}"
        );
        // Check ServiceDef::new call
        assert!(
            source.contains(r#"ServiceDef::new("test_service")"#),
            "source:\n{source}"
        );
        // Check data type (must NOT be snake_case)
        assert!(source.contains("DataType::Integer"), "source:\n{source}");
        // Check field meaning
        assert!(
            source.contains("FieldMeaning::Identifier"),
            "source:\n{source}"
        );
        // Check custom meaning
        assert!(
            source.contains(r#"FieldMeaning::Custom("sku".into())"#),
            "source:\n{source}"
        );
        // Check guard
        assert!(source.contains("GuardDef::new("), "source:\n{source}");
        // Check action
        assert!(source.contains("ActionDef::new("), "source:\n{source}");
        // Check relationship
        assert!(
            source.contains("RelationshipDef::new("),
            "source:\n{source}"
        );
        // Check intent hint
        assert!(source.contains("IntentHint"), "source:\n{source}");
        // Check state machine
        assert!(source.contains("StateMachine"), "source:\n{source}");
    }

    #[test]
    fn emitter_pascal_case_name_produces_snake_case_function() {
        // WR-01: LLM may return PascalCase service names. The emitter must use
        // snake_case for the function name to avoid clippy non_snake_case warnings.
        let service =
            ServiceDef::new("OrderItem").field("id", DataType::Integer, FieldMeaning::Identifier);
        let source = emit_service_def_source(&service);
        // Function name must be snake_case (order_item_service), NOT PascalCase
        assert!(
            source.contains("pub fn order_item_service() -> ServiceDef"),
            "function name must be snake_case\nsource:\n{source}"
        );
        assert!(
            !source.contains("pub fn OrderItem_service"),
            "function name must NOT be PascalCase\nsource:\n{source}"
        );
        // ServiceDef::new preserves the original name (runtime identity, not identifier)
        assert!(
            source.contains(r#"ServiceDef::new("OrderItem")"#),
            "ServiceDef::new must use the original name\nsource:\n{source}"
        );
    }

    // ---- Path sanitization tests ----

    #[test]
    fn ai_make_rejects_path_traversal() {
        assert!(
            resolve_projection_path("../../etc/passwd").is_err(),
            "path traversal should be rejected"
        );
    }

    #[test]
    fn ai_make_accepts_valid_name() {
        let path = resolve_projection_path("Order").expect("valid name should succeed");
        assert!(path.ends_with("src/projections/order.rs"), "got: {path:?}");
    }

    // ---- Dry-run test ----

    #[test]
    fn dry_run_no_file_write() {
        let dir = TempDir::new().expect("tempdir");
        let service =
            ServiceDef::new("preview").field("id", DataType::Integer, FieldMeaning::Identifier);

        let result = render_output(&service, true, dir.path()).expect("dry-run should not error");

        match result {
            OutputResult::DryRun(json) => {
                assert!(json.contains("preview"), "JSON should contain service name");
                // No files written
                let proj_file = dir.path().join("src/projections/preview.rs");
                assert!(!proj_file.exists(), "dry-run must not write files");
            }
            _ => panic!("expected DryRun result"),
        }
    }

}
