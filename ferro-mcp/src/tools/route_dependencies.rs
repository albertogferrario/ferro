//! Route dependencies tool - analyze handler source to detect model usage
//!
//! Identifies which models/entities a handler uses by analyzing:
//! - Entity type annotations (e.g., `User::Entity`)
//! - Model imports and usage patterns
//! - Database query patterns with model names

use crate::error::Result;
use crate::tools::{get_handler, list_models};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;

/// Cached regex patterns for detecting model usage
static ENTITY_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w+)::Entity").expect("Invalid regex"));

static MODEL_USE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:use\s+)?(?:\w+::)*models::(\w+)").expect("Invalid regex"));

static FIND_BY_ID_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w+)::find_by_id").expect("Invalid regex"));

static ACTIVE_MODEL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w+)::ActiveModel").expect("Invalid regex"));

static COLUMN_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\w+)::Column::").expect("Invalid regex"));

static SERVICE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?:App::resolve|inject)\s*::<(\w+)>").expect("Invalid regex"));

#[derive(Debug, Serialize)]
pub struct RouteDependencies {
    /// Route path
    pub route: String,
    /// HTTP method
    pub method: String,
    /// Handler function path
    pub handler: String,
    /// Models used by this handler
    pub models_used: Vec<ModelUsage>,
    /// Inertia component rendered (if any)
    pub inertia_component: Option<String>,
    /// Services injected/resolved
    pub services_used: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ModelUsage {
    /// Model name (e.g., "User", "Animal")
    pub model: String,
    /// How the model is used
    pub usage_type: ModelUsageType,
    /// Line where usage was detected (approximate)
    pub context: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ModelUsageType {
    /// Direct entity access (Entity::find_by_id, etc.)
    EntityQuery,
    /// ActiveModel for insert/update
    ActiveModel,
    /// Column filter
    ColumnFilter,
    /// Import/use statement
    Import,
    /// Type annotation
    TypeAnnotation,
}

pub async fn execute(project_root: &Path, route_path: &str) -> Result<RouteDependencies> {
    // Get handler source code
    let handler_info = get_handler::execute(project_root, route_path).await?;

    // Get list of known models for validation
    let known_models = list_models::execute(project_root).unwrap_or_default();
    let known_model_names: HashSet<String> = known_models.iter().map(|m| m.name.clone()).collect();

    // Analyze source code for model usage
    let models_used = extract_model_usage(&handler_info.source_code, &known_model_names);

    // Extract services
    let services_used = extract_services(&handler_info.source_code);

    Ok(RouteDependencies {
        route: handler_info.path,
        method: handler_info.method,
        handler: handler_info.handler,
        models_used,
        inertia_component: handler_info.component,
        services_used,
    })
}

/// Extract model usage from handler source code
fn extract_model_usage(source: &str, known_models: &HashSet<String>) -> Vec<ModelUsage> {
    let mut usages = Vec::new();
    let mut seen = HashSet::new();

    // Detect Entity::find_by_id and similar patterns
    for cap in ENTITY_PATTERN.captures_iter(source) {
        if let Some(model_match) = cap.get(1) {
            let model = model_match.as_str().to_string();
            let key = (model.clone(), ModelUsageType::EntityQuery);
            if known_models.contains(&model) && !seen.contains(&key) {
                seen.insert(key);
                usages.push(ModelUsage {
                    model,
                    usage_type: ModelUsageType::EntityQuery,
                    context: extract_context(source, model_match.start()),
                });
            }
        }
    }

    // Detect ActiveModel usage
    for cap in ACTIVE_MODEL_PATTERN.captures_iter(source) {
        if let Some(model_match) = cap.get(1) {
            let model = model_match.as_str().to_string();
            let key = (model.clone(), ModelUsageType::ActiveModel);
            if known_models.contains(&model) && !seen.contains(&key) {
                seen.insert(key);
                usages.push(ModelUsage {
                    model,
                    usage_type: ModelUsageType::ActiveModel,
                    context: extract_context(source, model_match.start()),
                });
            }
        }
    }

    // Detect Column:: usage
    for cap in COLUMN_PATTERN.captures_iter(source) {
        if let Some(model_match) = cap.get(1) {
            let model = model_match.as_str().to_string();
            let key = (model.clone(), ModelUsageType::ColumnFilter);
            if known_models.contains(&model) && !seen.contains(&key) {
                seen.insert(key);
                usages.push(ModelUsage {
                    model,
                    usage_type: ModelUsageType::ColumnFilter,
                    context: extract_context(source, model_match.start()),
                });
            }
        }
    }

    // Detect find_by_id pattern
    for cap in FIND_BY_ID_PATTERN.captures_iter(source) {
        if let Some(model_match) = cap.get(1) {
            let model = model_match.as_str().to_string();
            let key = (model.clone(), ModelUsageType::EntityQuery);
            if known_models.contains(&model) && !seen.contains(&key) {
                seen.insert(key);
                usages.push(ModelUsage {
                    model,
                    usage_type: ModelUsageType::EntityQuery,
                    context: extract_context(source, model_match.start()),
                });
            }
        }
    }

    // Detect model imports
    for cap in MODEL_USE_PATTERN.captures_iter(source) {
        if let Some(model_match) = cap.get(1) {
            let model = model_match.as_str().to_string();
            let key = (model.clone(), ModelUsageType::Import);
            if known_models.contains(&model) && !seen.contains(&key) {
                seen.insert(key);
                usages.push(ModelUsage {
                    model,
                    usage_type: ModelUsageType::Import,
                    context: extract_context(source, model_match.start()),
                });
            }
        }
    }

    usages
}

/// Extract services from handler source code
fn extract_services(source: &str) -> Vec<String> {
    let mut services = Vec::new();
    let mut seen = HashSet::new();

    for cap in SERVICE_PATTERN.captures_iter(source) {
        if let Some(service_match) = cap.get(1) {
            let service = service_match.as_str().to_string();
            if !seen.contains(&service) {
                seen.insert(service.clone());
                services.push(service);
            }
        }
    }

    services
}

/// Extract context line around a match position
fn extract_context(source: &str, pos: usize) -> String {
    // Find the line containing this position
    let line_start = source[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_end = source[pos..]
        .find('\n')
        .map(|p| pos + p)
        .unwrap_or(source.len());

    source[line_start..line_end].trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_model_usage_entity() {
        let source = r#"
            async fn show(req: Request) -> Response {
                let animal = Animal::Entity::find_by_id(id).one(&db).await?;
                Ok(json!(animal))
            }
        "#;

        let known_models: HashSet<String> = ["Animal".to_string()].into_iter().collect();
        let usages = extract_model_usage(source, &known_models);

        assert!(!usages.is_empty());
        assert!(usages.iter().any(|u| u.model == "Animal"));
        assert!(usages
            .iter()
            .any(|u| u.usage_type == ModelUsageType::EntityQuery));
    }

    #[test]
    fn test_extract_model_usage_active_model() {
        let source = r#"
            async fn store(req: Request) -> Response {
                let user = User::ActiveModel {
                    name: Set(form.name),
                    email: Set(form.email),
                    ..Default::default()
                };
                user.insert(&db).await?;
            }
        "#;

        let known_models: HashSet<String> = ["User".to_string()].into_iter().collect();
        let usages = extract_model_usage(source, &known_models);

        assert!(!usages.is_empty());
        assert!(usages.iter().any(|u| u.model == "User"));
        assert!(usages
            .iter()
            .any(|u| u.usage_type == ModelUsageType::ActiveModel));
    }

    #[test]
    fn test_extract_model_usage_column_filter() {
        let source = r#"
            async fn index(req: Request) -> Response {
                let animals = Animal::Entity::find()
                    .filter(Animal::Column::species.eq("dog"))
                    .all(&db).await?;
            }
        "#;

        let known_models: HashSet<String> = ["Animal".to_string()].into_iter().collect();
        let usages = extract_model_usage(source, &known_models);

        assert!(!usages.is_empty());
        assert!(usages
            .iter()
            .any(|u| u.usage_type == ModelUsageType::ColumnFilter));
    }

    #[test]
    fn test_extract_services() {
        let source = r#"
            async fn store(req: Request) -> Response {
                let mailer = App::resolve::<MailService>();
                let storage = App::resolve::<StorageService>();
                mailer.send(&email)?;
            }
        "#;

        let services = extract_services(source);

        assert_eq!(services.len(), 2);
        assert!(services.contains(&"MailService".to_string()));
        assert!(services.contains(&"StorageService".to_string()));
    }

    #[test]
    fn test_ignores_unknown_models() {
        let source = r#"
            async fn show(req: Request) -> Response {
                let thing = Unknown::Entity::find_by_id(id).one(&db).await?;
            }
        "#;

        let known_models: HashSet<String> = ["Animal".to_string()].into_iter().collect();
        let usages = extract_model_usage(source, &known_models);

        assert!(usages.is_empty());
    }
}
