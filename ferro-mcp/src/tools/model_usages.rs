//! Model usages tool - find all routes/handlers that reference a given model
//!
//! Inverse of route_dependencies - answers "where is this model used?"
//! Scans all handler files for model references.

use crate::error::{McpError, Result};
use crate::tools::{list_routes, route_dependencies};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ModelUsages {
    /// Model name that was searched
    pub model: String,
    /// Routes that use this model
    pub routes: Vec<RouteUsage>,
    /// Summary statistics
    pub summary: UsageSummary,
}

#[derive(Debug, Serialize)]
pub struct RouteUsage {
    /// Route path
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Handler function path
    pub handler: String,
    /// How the model is used in this route
    pub usage_types: Vec<String>,
    /// Inertia component (if any)
    pub component: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    /// Total routes using this model
    pub total_routes: usize,
    /// Routes that query this model (Entity::find, etc.)
    pub query_routes: usize,
    /// Routes that modify this model (ActiveModel)
    pub mutation_routes: usize,
    /// Routes that filter by this model's columns
    pub filter_routes: usize,
}

pub async fn execute(project_root: &Path, model_name: &str) -> Result<ModelUsages> {
    // Get all routes
    let routes_info = list_routes::execute(project_root).await?;

    let mut route_usages = Vec::new();
    let mut query_count = 0;
    let mut mutation_count = 0;
    let mut filter_count = 0;

    // Check each route for model usage
    for route in &routes_info.routes {
        // Skip routes without handlers (runtime-only routes)
        if route.handler.is_empty() {
            continue;
        }

        // Get dependencies for this route
        let deps = match route_dependencies::execute(project_root, &route.path).await {
            Ok(d) => d,
            Err(_) => continue, // Skip routes we can't analyze
        };

        // Check if this route uses the target model
        let usages: Vec<_> = deps
            .models_used
            .iter()
            .filter(|u| u.model.eq_ignore_ascii_case(model_name))
            .collect();

        if !usages.is_empty() {
            let usage_types: Vec<String> = usages
                .iter()
                .map(|u| format!("{:?}", u.usage_type).to_lowercase())
                .collect();

            // Track usage type counts
            for usage in &usages {
                match usage.usage_type {
                    route_dependencies::ModelUsageType::EntityQuery => query_count += 1,
                    route_dependencies::ModelUsageType::ActiveModel => mutation_count += 1,
                    route_dependencies::ModelUsageType::ColumnFilter => filter_count += 1,
                    _ => {}
                }
            }

            route_usages.push(RouteUsage {
                path: route.path.clone(),
                method: route.method.clone(),
                handler: route.handler.clone(),
                usage_types,
                component: deps.inertia_component,
            });
        }
    }

    if route_usages.is_empty() {
        return Err(McpError::NotFound(format!(
            "No routes found using model '{model_name}'"
        )));
    }

    let summary = UsageSummary {
        total_routes: route_usages.len(),
        query_routes: query_count,
        mutation_routes: mutation_count,
        filter_routes: filter_count,
    };

    Ok(ModelUsages {
        model: model_name.to_string(),
        routes: route_usages,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_usage_serialization() {
        let usage = RouteUsage {
            path: "/users/{id}".to_string(),
            method: "GET".to_string(),
            handler: "controllers::users::show".to_string(),
            usage_types: vec!["entity_query".to_string()],
            component: Some("Users/Show".to_string()),
        };

        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("/users/{id}"));
        assert!(json.contains("GET"));
        assert!(json.contains("entity_query"));
    }

    #[test]
    fn test_model_usages_serialization() {
        let usages = ModelUsages {
            model: "User".to_string(),
            routes: vec![RouteUsage {
                path: "/users".to_string(),
                method: "GET".to_string(),
                handler: "controllers::users::index".to_string(),
                usage_types: vec!["entity_query".to_string()],
                component: None,
            }],
            summary: UsageSummary {
                total_routes: 1,
                query_routes: 1,
                mutation_routes: 0,
                filter_routes: 0,
            },
        };

        let json = serde_json::to_string_pretty(&usages).unwrap();
        assert!(json.contains("\"model\": \"User\""));
        assert!(json.contains("\"total_routes\": 1"));
    }
}
