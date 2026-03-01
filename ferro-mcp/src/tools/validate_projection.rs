//! Validate a service projection for structural issues.
//!
//! Reconstructs a ServiceDef from source, runs `validate()`, and returns
//! structured results with warnings and errors.

use serde::Serialize;
use std::fs;
use std::path::Path;

use super::inspect_projection::InspectResult;
use super::render_projection::reconstruct_service_def;

/// Validation result for a single projection.
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub service_name: String,
    pub file: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub valid: bool,
}

/// Summary when validating all projections.
#[derive(Debug, Serialize)]
pub struct ValidationSummary {
    pub results: Vec<ValidationResult>,
    pub total: usize,
    pub valid_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

/// Validate a single named projection.
pub fn execute_single(project_root: &Path, name: &str) -> Result<ValidationResult, String> {
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

    let file_path = project_root.join(&detail.file);
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("failed to read {}: {e}", detail.file))?;

    let service = reconstruct_service_def(&detail.service_name, &detail.display_name, &content)?;

    match service.validate() {
        Ok(warnings) => {
            let warning_strs: Vec<String> = warnings.iter().map(|w| format!("{w:?}")).collect();
            Ok(ValidationResult {
                service_name: detail.service_name,
                file: detail.file,
                warnings: warning_strs,
                errors: Vec::new(),
                valid: true,
            })
        }
        Err(e) => Ok(ValidationResult {
            service_name: detail.service_name,
            file: detail.file,
            warnings: Vec::new(),
            errors: vec![e.to_string()],
            valid: false,
        }),
    }
}

/// Validate all discovered projections.
pub fn execute_all(project_root: &Path) -> ValidationSummary {
    let list = super::list_projections::execute(project_root, None);

    let mut results = Vec::new();
    let mut warning_count = 0usize;
    let mut error_count = 0usize;

    for info in &list.projections {
        let result = execute_single(project_root, &info.name);
        match result {
            Ok(vr) => {
                warning_count += vr.warnings.len();
                error_count += vr.errors.len();
                results.push(vr);
            }
            Err(e) => {
                error_count += 1;
                results.push(ValidationResult {
                    service_name: info.service_name.clone().unwrap_or_default(),
                    file: info.file.clone(),
                    warnings: Vec::new(),
                    errors: vec![e],
                    valid: false,
                });
            }
        }
    }

    let valid_count = results.iter().filter(|r| r.valid).count();
    let total = results.len();

    ValidationSummary {
        results,
        total,
        valid_count,
        warning_count,
        error_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            service_name: "user".to_string(),
            file: "src/projections/user.rs".to_string(),
            warnings: vec!["UnusedGuard(\"is_admin\")".to_string()],
            errors: Vec::new(),
            valid: true,
        };

        let json = serde_json::to_string(&result);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("user"));
        assert!(json_str.contains("UnusedGuard"));
        assert!(json_str.contains("\"valid\":true"));
    }

    #[test]
    fn test_validation_summary_serialization() {
        let summary = ValidationSummary {
            results: vec![ValidationResult {
                service_name: "order".to_string(),
                file: "src/projections/order.rs".to_string(),
                warnings: Vec::new(),
                errors: Vec::new(),
                valid: true,
            }],
            total: 1,
            valid_count: 1,
            warning_count: 0,
            error_count: 0,
        };

        let json = serde_json::to_string(&summary);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("order"));
        assert!(json_str.contains("\"total\":1"));
        assert!(json_str.contains("\"valid_count\":1"));
    }

    #[test]
    fn test_validate_valid_projection() {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let content = r#"
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn order_service() -> ServiceDef {
    ServiceDef::new("order")
        .display_name("Order")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("total", DataType::Float, FieldMeaning::Money)
}
        "#;
        std::fs::write(proj_dir.join("order.rs"), content).unwrap();

        let result = execute_single(tmp.path(), "order_service");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert!(vr.valid);
        assert!(vr.warnings.is_empty());
        assert!(vr.errors.is_empty());
        assert_eq!(vr.service_name, "order");
    }

    #[test]
    fn test_validate_projection_with_orphan_state() {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();

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
        std::fs::write(proj_dir.join("broken.rs"), content).unwrap();

        let result = execute_single(tmp.path(), "broken_service");
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert!(vr.valid); // Warnings don't make it invalid
        assert!(
            !vr.warnings.is_empty(),
            "Should have warnings for orphan state"
        );
    }

    #[test]
    fn test_validate_all_projections() {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();

        std::fs::write(
            proj_dir.join("user.rs"),
            r#"
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn user_service() -> ServiceDef {
    ServiceDef::new("user")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
}
            "#,
        )
        .unwrap();

        std::fs::write(
            proj_dir.join("product.rs"),
            r#"
use ferro::{ServiceDef, DataType, FieldMeaning};

pub fn product_service() -> ServiceDef {
    ServiceDef::new("product")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
}
            "#,
        )
        .unwrap();

        let summary = execute_all(tmp.path());
        assert_eq!(summary.total, 2);
        assert_eq!(summary.valid_count, 2);
        assert_eq!(summary.warning_count, 0);
        assert_eq!(summary.error_count, 0);
    }

    #[test]
    fn test_validate_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();

        let result = execute_single(tmp.path(), "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
