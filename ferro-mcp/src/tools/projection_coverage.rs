//! Coverage report showing which models have service projections and which need them.
//!
//! Cross-references models from `src/models/` with projections from `src/projections/`
//! to identify coverage gaps and derive primary intents for existing projections.

use serde::Serialize;
use std::fs;
use std::path::Path;

use super::render_projection::reconstruct_service_def;
use ferro_projections::derive_intents;

/// Full coverage report: per-model coverage and aggregate summary.
#[derive(Debug, Serialize)]
pub struct CoverageReport {
    pub models: Vec<ModelCoverage>,
    pub coverage: CoverageSummary,
}

/// Coverage status for a single model.
#[derive(Debug, Serialize)]
pub struct ModelCoverage {
    /// PascalCase model name (e.g., "User").
    pub model_name: String,
    /// Whether a matching projection exists.
    pub has_projection: bool,
    /// Projection function name if matched (e.g., "user_service").
    pub projection_name: Option<String>,
    /// Relative path to the projection file.
    pub projection_file: Option<String>,
    /// Primary derived intent (e.g., "Browse").
    pub primary_intent: Option<String>,
    /// Confidence score for the primary intent.
    pub intent_confidence: Option<f64>,
    /// Suggested CLI command to create a missing projection.
    pub suggestion: Option<String>,
    /// Checkpoint status read from the cache file, stale-ok.
    /// `"clean"` | `"failing"` | `"unverified"` (file absent or projection not found).
    pub checkpoint_status: String,
}

/// Aggregate coverage statistics.
#[derive(Debug, Serialize)]
pub struct CoverageSummary {
    pub total_models: usize,
    pub with_projections: usize,
    pub without_projections: usize,
    pub percentage: f64,
}

/// Generate a coverage report cross-referencing models and projections.
pub fn execute(project_root: &Path) -> CoverageReport {
    // Get models — returns Err if none found, which we treat as empty
    let models = match super::list_models::execute(project_root) {
        Ok(m) => m,
        Err(_) => {
            return CoverageReport {
                models: Vec::new(),
                coverage: CoverageSummary {
                    total_models: 0,
                    with_projections: 0,
                    without_projections: 0,
                    percentage: 0.0,
                },
            }
        }
    };

    // Get projections
    let projection_list = super::list_projections::execute(project_root, None);

    let mut coverages = Vec::new();
    let mut with_count = 0usize;

    for model in &models {
        let model_lower = model.name.to_lowercase();

        // Match: projection service_name (lowercase) == model name (lowercase)
        let matched = projection_list.projections.iter().find(|p| {
            p.service_name
                .as_ref()
                .is_some_and(|sn| sn.to_lowercase() == model_lower)
        });

        if let Some(proj) = matched {
            with_count += 1;

            // Try to derive primary intent
            let (primary_intent, intent_confidence) = derive_primary_intent(
                project_root,
                &proj.file,
                proj.service_name.as_deref(),
                &proj.name,
            );

            coverages.push(ModelCoverage {
                model_name: model.name.clone(),
                has_projection: true,
                projection_name: Some(proj.name.clone()),
                projection_file: Some(proj.file.clone()),
                primary_intent,
                intent_confidence,
                suggestion: None,
                checkpoint_status: crate::tools::checkpoint_projection::read_ambient_status(
                    project_root,
                    &proj.name, // FUNCTION name e.g. "booking_service" — NOT model.name
                )
                .to_string(),
            });
        } else {
            let snake = to_snake_case(&model.name);
            coverages.push(ModelCoverage {
                model_name: model.name.clone(),
                has_projection: false,
                projection_name: None,
                projection_file: None,
                primary_intent: None,
                intent_confidence: None,
                suggestion: Some(format!("ferro make:projection {snake} --from-model")),
                checkpoint_status: "unverified".to_string(),
            });
        }
    }

    let total = models.len();
    let without = total - with_count;
    let percentage = if total > 0 {
        (with_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    CoverageReport {
        models: coverages,
        coverage: CoverageSummary {
            total_models: total,
            with_projections: with_count,
            without_projections: without,
            percentage,
        },
    }
}

/// Derive the primary intent for a projection by reconstructing its ServiceDef.
fn derive_primary_intent(
    project_root: &Path,
    file: &str,
    service_name: Option<&str>,
    function_name: &str,
) -> (Option<String>, Option<f64>) {
    let file_path = project_root.join(file);
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(_) => return (None, None),
    };

    let sn = service_name.unwrap_or(function_name);

    // Extract display_name from source
    let display_name = regex::Regex::new(r#"\.display_name\("([^"]+)"\)"#)
        .ok()
        .and_then(|re| re.captures(&content))
        .map(|c| c[1].to_string());

    let service = match reconstruct_service_def(sn, &display_name, &content) {
        Ok(s) => s,
        Err(_) => return (None, None),
    };

    let intents = derive_intents(&service);
    if let Some(primary) = intents.first() {
        (
            Some(format!("{:?}", primary.intent)),
            Some(primary.confidence),
        )
    } else {
        (None, None)
    }
}

/// Convert PascalCase to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_report_serialization() {
        let report = CoverageReport {
            models: vec![
                ModelCoverage {
                    model_name: "User".to_string(),
                    has_projection: true,
                    projection_name: Some("user_service".to_string()),
                    projection_file: Some("src/projections/user.rs".to_string()),
                    primary_intent: Some("Browse".to_string()),
                    intent_confidence: Some(0.85),
                    suggestion: None,
                    checkpoint_status: "unverified".to_string(),
                },
                ModelCoverage {
                    model_name: "Product".to_string(),
                    has_projection: false,
                    projection_name: None,
                    projection_file: None,
                    primary_intent: None,
                    intent_confidence: None,
                    suggestion: Some("ferro make:projection product --from-model".to_string()),
                    checkpoint_status: "unverified".to_string(),
                },
            ],
            coverage: CoverageSummary {
                total_models: 2,
                with_projections: 1,
                without_projections: 1,
                percentage: 50.0,
            },
        };

        let json = serde_json::to_string_pretty(&report);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("User"));
        assert!(json_str.contains("user_service"));
        assert!(json_str.contains("Browse"));
        assert!(json_str.contains("0.85"));
        assert!(json_str.contains("Product"));
        assert!(json_str.contains("ferro make:projection product --from-model"));
        assert!(json_str.contains("\"total_models\": 2"));
        assert!(json_str.contains("\"percentage\": 50.0"));
    }

    #[test]
    fn test_empty_project() {
        let non_existent =
            std::path::PathBuf::from("/tmp/non_existent_ferro_projection_coverage_test");
        let report = execute(&non_existent);
        assert_eq!(report.coverage.total_models, 0);
        assert_eq!(report.coverage.with_projections, 0);
        assert_eq!(report.coverage.without_projections, 0);
        assert_eq!(report.coverage.percentage, 0.0);
        assert!(report.models.is_empty());
    }

    #[test]
    fn test_suggestion_format() {
        let coverage = ModelCoverage {
            model_name: "OrderItem".to_string(),
            has_projection: false,
            projection_name: None,
            projection_file: None,
            primary_intent: None,
            intent_confidence: None,
            suggestion: Some("ferro make:projection order_item --from-model".to_string()),
            checkpoint_status: "unverified".to_string(),
        };

        assert_eq!(
            coverage.suggestion.as_deref(),
            Some("ferro make:projection order_item --from-model")
        );
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("OrderItem"), "order_item");
        assert_eq!(to_snake_case("HTMLParser"), "h_t_m_l_parser");
        assert_eq!(to_snake_case("simple"), "simple");
    }

    #[test]
    fn test_coverage_summary_percentages() {
        let summary = CoverageSummary {
            total_models: 3,
            with_projections: 2,
            without_projections: 1,
            percentage: 66.66666666666667,
        };

        let json = serde_json::to_string_pretty(&summary);
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"total_models\": 3"));
        assert!(json_str.contains("\"with_projections\": 2"));
        assert!(json_str.contains("\"without_projections\": 1"));
    }

    // ------------------------------------------------------------------
    // CHK-08 tests: checkpoint_status populated via read_ambient_status
    // ------------------------------------------------------------------

    /// Write a checkpoint cache file for `proj_name` under `project_root`.
    fn write_checkpoint_cache(project_root: &std::path::Path, proj_name: &str, status: &str) {
        let cache_dir = project_root.join(".ferro").join("checkpoints");
        std::fs::create_dir_all(&cache_dir).unwrap();
        std::fs::write(
            cache_dir.join(format!("{proj_name}.json")),
            format!(r#"{{"ambient_status":"{status}"}}"#),
        )
        .unwrap();
    }

    /// Build a minimal SeaORM model source for list_models to parse.
    fn model_src(struct_name: &str) -> String {
        let table = struct_name.to_lowercase() + "s";
        format!(
            r#"use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "{table}")]
pub struct {struct_name} {{
    pub id: i64,
}}
"#
        )
    }

    /// Build a minimal projection source for list_projections to discover.
    fn projection_src(fn_name: &str, service_name: &str) -> String {
        format!(
            r#"use ferro::{{ServiceDef, DataType, FieldMeaning}};
pub fn {fn_name}() -> ServiceDef {{
    ServiceDef::new("{service_name}")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
}}
"#
        )
    }

    #[test]
    fn checkpoint_status_from_cache_failing() {
        // CHK-08: a model with a projection whose cache says "failing" →
        // ModelCoverage.checkpoint_status == "failing".
        let tmp = tempfile::TempDir::new().unwrap();

        // Write model.
        let models_dir = tmp.path().join("src/models");
        std::fs::create_dir_all(&models_dir).unwrap();
        std::fs::write(models_dir.join("booking.rs"), model_src("Booking")).unwrap();

        // Write projection (list_projections scans src/projections/).
        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();
        std::fs::write(
            proj_dir.join("booking_service.rs"),
            projection_src("booking_service", "booking"),
        )
        .unwrap();

        // Write cache with "failing" status keyed on function name.
        write_checkpoint_cache(tmp.path(), "booking_service", "failing");

        let report = execute(tmp.path());
        let booking = report
            .models
            .iter()
            .find(|m| m.model_name == "Booking")
            .expect("Booking model must appear in coverage report");

        assert!(
            booking.has_projection,
            "Booking must be matched to booking_service"
        );
        assert_eq!(
            booking.checkpoint_status, "failing",
            "checkpoint_status must reflect the cache value"
        );
    }

    #[test]
    fn checkpoint_status_unverified_no_cache() {
        // CHK-08: a model with a projection but no cache file → "unverified".
        let tmp = tempfile::TempDir::new().unwrap();

        let models_dir = tmp.path().join("src/models");
        std::fs::create_dir_all(&models_dir).unwrap();
        std::fs::write(models_dir.join("booking.rs"), model_src("Booking")).unwrap();

        let proj_dir = tmp.path().join("src/projections");
        std::fs::create_dir_all(&proj_dir).unwrap();
        std::fs::write(
            proj_dir.join("booking_service.rs"),
            projection_src("booking_service", "booking"),
        )
        .unwrap();

        // No cache file written.

        let report = execute(tmp.path());
        let booking = report
            .models
            .iter()
            .find(|m| m.model_name == "Booking")
            .expect("Booking model must appear in coverage report");

        assert_eq!(
            booking.checkpoint_status, "unverified",
            "missing cache → unverified"
        );
    }

    #[test]
    fn checkpoint_status_unverified_no_projection() {
        // CHK-08: a model with no projection at all → checkpoint_status == "unverified".
        let tmp = tempfile::TempDir::new().unwrap();

        let models_dir = tmp.path().join("src/models");
        std::fs::create_dir_all(&models_dir).unwrap();
        std::fs::write(models_dir.join("widget.rs"), model_src("Widget")).unwrap();

        // No projection directory or file for Widget.

        let report = execute(tmp.path());
        let widget = report
            .models
            .iter()
            .find(|m| m.model_name == "Widget")
            .expect("Widget model must appear in coverage report");

        assert!(!widget.has_projection, "Widget has no projection");
        assert_eq!(
            widget.checkpoint_status, "unverified",
            "no projection → checkpoint_status unverified"
        );
    }
}
