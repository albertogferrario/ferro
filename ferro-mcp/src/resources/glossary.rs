//! Domain glossary resource for MCP introspection
//!
//! Extracts business domain terms from models and routes to help agents
//! understand the semantic meaning of application entities.

use crate::tools::list_models::ModelDetails;
use crate::tools::list_routes::RouteInfo;
use serde::Serialize;
use std::collections::HashMap;

/// A glossary entry describing a domain term
#[derive(Debug, Serialize)]
pub struct GlossaryEntry {
    /// Definition of what this term means in the business domain
    pub definition: String,
    /// Model names related to this term
    pub models: Vec<String>,
    /// Route paths related to this term
    pub routes: Vec<String>,
    /// Business intent/context for this term
    pub intent: String,
}

/// Domain glossary containing business terms and their meanings
#[derive(Debug, Serialize)]
pub struct DomainGlossary {
    /// Map of term name to glossary entry
    pub terms: HashMap<String, GlossaryEntry>,
}

/// Generate a domain glossary from models and routes
///
/// Extracts domain terms by analyzing model names and route paths,
/// then generates definitions and context based on patterns.
pub fn generate_glossary(models: &[ModelDetails], routes: &[RouteInfo]) -> DomainGlossary {
    let mut terms: HashMap<String, GlossaryEntry> = HashMap::new();

    // Extract terms from models
    for model in models {
        let term = to_snake_case(&model.name);
        let entry = terms.entry(term.clone()).or_insert_with(|| GlossaryEntry {
            definition: generate_model_definition(&model.name, &model.fields),
            models: Vec::new(),
            routes: Vec::new(),
            intent: generate_model_intent(&model.name),
        });
        entry.models.push(model.name.clone());

        // Find related routes
        for route in routes {
            if route_relates_to_model(&route.path, &model.name)
                && !entry.routes.contains(&route.path)
            {
                entry.routes.push(route.path.clone());
            }
        }
    }

    // Extract additional terms from routes that don't have models
    for route in routes {
        let route_terms = extract_route_terms(&route.path);
        for route_term in route_terms {
            if !terms.contains_key(&route_term) {
                terms.insert(
                    route_term.clone(),
                    GlossaryEntry {
                        definition: generate_route_term_definition(&route_term),
                        models: Vec::new(),
                        routes: vec![route.path.clone()],
                        intent: generate_route_term_intent(&route_term),
                    },
                );
            }
        }
    }

    DomainGlossary { terms }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// Generate a definition from model name and fields
fn generate_model_definition(
    name: &str,
    fields: &[crate::tools::list_models::FieldInfo],
) -> String {
    let field_names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();

    // Common domain patterns
    let definition = match name.to_lowercase().as_str() {
        s if s.contains("user") => "Application user account with authentication credentials",
        s if s.contains("todo") => "Task or to-do item for tracking work",
        s if s.contains("rifugio") => "Mountain shelter or refuge for hikers and visitors",
        s if s.contains("prenotazione") || s.contains("booking") || s.contains("reservation") => {
            "Reservation or booking record"
        }
        s if s.contains("session") => "User session for authentication state",
        s if s.contains("token") => "Authentication or API access token",
        s if s.contains("comment") => "User-generated comment or feedback",
        s if s.contains("post") => "Content post or article",
        s if s.contains("category") => "Classification category for organizing content",
        s if s.contains("tag") => "Label or tag for content classification",
        s if s.contains("image") || s.contains("photo") => "Image or photo media asset",
        s if s.contains("file") || s.contains("document") => "File or document attachment",
        s if s.contains("notification") => "User notification or alert",
        s if s.contains("setting") || s.contains("config") => "Configuration or settings record",
        _ => "Domain entity",
    };

    // Enhance with field information
    let key_fields: Vec<&str> = field_names
        .iter()
        .filter(|f| {
            !["id", "created_at", "updated_at", "deleted_at"].contains(&f.to_lowercase().as_str())
        })
        .take(3)
        .copied()
        .collect();

    if key_fields.is_empty() {
        definition.to_string()
    } else {
        format!("{}. Key attributes: {}", definition, key_fields.join(", "))
    }
}

/// Generate business intent from model name
fn generate_model_intent(name: &str) -> String {
    match name.to_lowercase().as_str() {
        s if s.contains("user") => "Core entity for user management and authentication".to_string(),
        s if s.contains("todo") => "Task tracking for productivity features".to_string(),
        s if s.contains("rifugio") => "Core entity users discover, view, and book".to_string(),
        s if s.contains("prenotazione") || s.contains("booking") => {
            "Transaction representing user intent to reserve".to_string()
        }
        s if s.contains("session") => "Authentication state management".to_string(),
        _ => format!("Domain data for {} features", to_snake_case(name)),
    }
}

/// Check if a route path relates to a model
fn route_relates_to_model(path: &str, model_name: &str) -> bool {
    let snake = to_snake_case(model_name);
    let plural = format!("{snake}s");

    // Check path segments
    let path_lower = path.to_lowercase();
    path_lower.contains(&snake) || path_lower.contains(&plural)
}

/// Extract domain terms from route path
fn extract_route_terms(path: &str) -> Vec<String> {
    let mut terms = Vec::new();

    // Split path and extract meaningful segments
    for segment in path.split('/') {
        let segment = segment.trim();
        if segment.is_empty() || segment.starts_with('{') || segment.starts_with(':') {
            continue;
        }

        // Skip common non-domain segments
        if ["api", "v1", "v2", "admin", "auth", "public"].contains(&segment.to_lowercase().as_str())
        {
            continue;
        }

        terms.push(segment.to_lowercase());
    }

    terms
}

/// Generate definition for route-derived term
fn generate_route_term_definition(term: &str) -> String {
    match term {
        "login" => "User authentication endpoint".to_string(),
        "logout" => "Session termination endpoint".to_string(),
        "register" => "New user registration endpoint".to_string(),
        "dashboard" => "Main user dashboard view".to_string(),
        "profile" => "User profile management".to_string(),
        "settings" => "Application settings management".to_string(),
        "search" => "Search functionality endpoint".to_string(),
        "health" => "Health check endpoint for monitoring".to_string(),
        _ => format!("Feature related to {term}"),
    }
}

/// Generate intent for route-derived term
fn generate_route_term_intent(term: &str) -> String {
    match term {
        "login" | "logout" | "register" => "Authentication workflow".to_string(),
        "dashboard" => "Primary user interface entry point".to_string(),
        "profile" | "settings" => "User customization features".to_string(),
        "search" => "Content discovery".to_string(),
        "health" => "System monitoring and operations".to_string(),
        _ => format!("Functionality for {term} feature"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("TodoItem"), "todo_item");
        assert_eq!(to_snake_case("HTTPRequest"), "h_t_t_p_request");
    }

    #[test]
    fn test_generate_glossary_empty() {
        let glossary = generate_glossary(&[], &[]);
        assert!(glossary.terms.is_empty());
    }

    #[test]
    fn test_generate_glossary_with_model() {
        let models = vec![ModelDetails {
            name: "User".to_string(),
            table: Some("users".to_string()),
            path: "src/models/user.rs".to_string(),
            fields: vec![
                crate::tools::list_models::FieldInfo {
                    name: "id".to_string(),
                    field_type: "i32".to_string(),
                    is_primary_key: true,
                    is_nullable: false,
                },
                crate::tools::list_models::FieldInfo {
                    name: "email".to_string(),
                    field_type: "String".to_string(),
                    is_primary_key: false,
                    is_nullable: false,
                },
            ],
        }];

        let routes = vec![RouteInfo {
            method: "GET".to_string(),
            path: "/users".to_string(),
            handler: "controllers::user::index".to_string(),
            name: Some("users.index".to_string()),
            middleware: vec![],
        }];

        let glossary = generate_glossary(&models, &routes);

        assert!(glossary.terms.contains_key("user"));
        let user_entry = glossary.terms.get("user").unwrap();
        assert!(user_entry.models.contains(&"User".to_string()));
        assert!(user_entry.routes.contains(&"/users".to_string()));
    }

    #[test]
    fn test_route_relates_to_model() {
        assert!(route_relates_to_model("/users", "User"));
        assert!(route_relates_to_model("/users/{id}", "User"));
        assert!(route_relates_to_model("/api/todos", "Todo"));
        assert!(!route_relates_to_model("/posts", "User"));
    }

    #[test]
    fn test_extract_route_terms() {
        let terms = extract_route_terms("/api/v1/users/{id}/posts");
        assert!(terms.contains(&"users".to_string()));
        assert!(terms.contains(&"posts".to_string()));
        assert!(!terms.contains(&"api".to_string()));
        assert!(!terms.contains(&"v1".to_string()));
    }
}
