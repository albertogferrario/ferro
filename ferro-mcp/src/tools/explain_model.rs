//! explain_model tool - Explains what a model represents in the business domain
//!
//! Returns semantic information about a model to help agents understand
//! the domain meaning and relationships of database entities.

use crate::error::{McpError, Result};
use crate::tools::list_models::{execute as list_models, FieldInfo, ModelDetails};
use crate::tools::list_routes::{execute as list_routes, RouteInfo};
use serde::Serialize;
use std::path::Path;

/// Explanation of a model's domain meaning and context
#[derive(Debug, Serialize)]
pub struct ModelExplanation {
    /// Model name
    pub model: String,
    /// Domain meaning - what this model represents in the business
    pub domain_meaning: String,
    /// Table name if different from model name
    pub table: Option<String>,
    /// Field explanations
    pub fields: Vec<FieldExplanation>,
    /// Inferred relationships to other models
    pub relationships: Vec<String>,
    /// Routes that work with this model
    pub related_routes: Vec<String>,
    /// Common query patterns for this model
    pub common_queries: Vec<String>,
    /// File path where model is defined
    pub path: String,
}

/// Explanation of a model field
#[derive(Debug, Serialize)]
pub struct FieldExplanation {
    /// Field name
    pub name: String,
    /// Field type
    #[serde(rename = "type")]
    pub field_type: String,
    /// What this field represents
    pub meaning: String,
    /// Whether this is the primary key
    pub is_primary_key: bool,
    /// Whether this field is optional
    pub is_optional: bool,
}

/// Execute the explain_model tool
pub async fn execute(project_root: &Path, model_name: &str) -> Result<ModelExplanation> {
    // Get all models
    let models = list_models(project_root)?;

    // Find the matching model (case-insensitive)
    let model = models
        .iter()
        .find(|m| m.name.to_lowercase() == model_name.to_lowercase())
        .ok_or_else(|| McpError::NotFound(format!("Model '{model_name}' not found")))?;

    // Get routes for finding related routes
    let routes = list_routes(project_root)
        .await
        .map(|r| r.routes)
        .unwrap_or_default();

    // Generate explanation
    Ok(ModelExplanation {
        model: model.name.clone(),
        domain_meaning: infer_domain_meaning(&model.name, &model.fields),
        table: model.table.clone(),
        fields: explain_fields(&model.fields),
        relationships: infer_relationships(&model.fields, &models),
        related_routes: find_related_routes(&model.name, &routes),
        common_queries: generate_common_queries(&model.name, &model.fields),
        path: model.path.clone(),
    })
}

/// Infer the domain meaning of a model from its name and fields
fn infer_domain_meaning(name: &str, fields: &[FieldInfo]) -> String {
    let name_lower = name.to_lowercase();

    // Check for common domain patterns
    let base_meaning = if name_lower.contains("user") {
        "Application user account with authentication credentials"
    } else if name_lower.contains("todo") || name_lower.contains("task") {
        "Task or to-do item for tracking work and productivity"
    } else if name_lower.contains("rifugio") {
        "Mountain shelter or refuge for hikers and visitors"
    } else if name_lower.contains("prenotazione")
        || name_lower.contains("booking")
        || name_lower.contains("reservation")
    {
        "Reservation or booking record representing user intent to reserve a resource"
    } else if name_lower.contains("session") {
        "User session for tracking authentication state"
    } else if name_lower.contains("token") {
        "Authentication or API access token"
    } else if name_lower.contains("comment") {
        "User-generated comment or feedback on content"
    } else if name_lower.contains("post") || name_lower.contains("article") {
        "Content post or article for publishing"
    } else if name_lower.contains("category") {
        "Classification category for organizing content"
    } else if name_lower.contains("tag") {
        "Label or tag for content classification and filtering"
    } else if name_lower.contains("image")
        || name_lower.contains("photo")
        || name_lower.contains("media")
    {
        "Image or media asset for content"
    } else if name_lower.contains("file")
        || name_lower.contains("document")
        || name_lower.contains("attachment")
    {
        "File or document attachment"
    } else if name_lower.contains("notification") || name_lower.contains("alert") {
        "User notification or alert message"
    } else if name_lower.contains("setting")
        || name_lower.contains("config")
        || name_lower.contains("preference")
    {
        "Configuration or settings record"
    } else if name_lower.contains("role") || name_lower.contains("permission") {
        "Authorization role or permission for access control"
    } else if name_lower.contains("log") || name_lower.contains("audit") {
        "Audit log or activity record"
    } else if name_lower.contains("order") || name_lower.contains("purchase") {
        "Order or purchase transaction"
    } else if name_lower.contains("payment") || name_lower.contains("transaction") {
        "Payment or financial transaction"
    } else if name_lower.contains("product") || name_lower.contains("item") {
        "Product or inventory item"
    } else {
        "Domain entity"
    };

    // Enhance with field-based context
    let has_email = fields
        .iter()
        .any(|f| f.name.to_lowercase().contains("email"));
    let has_password = fields
        .iter()
        .any(|f| f.name.to_lowercase().contains("password"));
    let has_status = fields
        .iter()
        .any(|f| f.name.to_lowercase().contains("status"));
    let has_amount = fields.iter().any(|f| {
        f.name.to_lowercase().contains("amount") || f.name.to_lowercase().contains("price")
    });

    let mut meaning = base_meaning.to_string();

    if has_email && has_password && !name_lower.contains("user") {
        meaning.push_str(". Contains authentication credentials");
    }
    if has_status {
        meaning.push_str(". Tracks lifecycle state via status field");
    }
    if has_amount && !name_lower.contains("payment") && !name_lower.contains("order") {
        meaning.push_str(". Contains financial/quantity data");
    }

    meaning
}

/// Generate explanations for model fields
fn explain_fields(fields: &[FieldInfo]) -> Vec<FieldExplanation> {
    fields
        .iter()
        .map(|f| {
            let meaning = infer_field_meaning(&f.name, &f.field_type);
            FieldExplanation {
                name: f.name.clone(),
                field_type: f.field_type.clone(),
                meaning,
                is_primary_key: f.is_primary_key,
                is_optional: f.is_nullable,
            }
        })
        .collect()
}

/// Infer the meaning of a field from its name and type
fn infer_field_meaning(name: &str, field_type: &str) -> String {
    let name_lower = name.to_lowercase();

    // Common field patterns
    if name_lower == "id" {
        "Unique identifier (primary key)".to_string()
    } else if name_lower == "uuid" {
        "Universally unique identifier".to_string()
    } else if name_lower.contains("email") {
        "Email address for contact or authentication".to_string()
    } else if name_lower.contains("password") {
        "Hashed password for authentication".to_string()
    } else if name_lower == "name" || name_lower == "title" {
        "Display name or title".to_string()
    } else if name_lower.contains("first_name") || name_lower.contains("firstname") {
        "User's first/given name".to_string()
    } else if name_lower.contains("last_name") || name_lower.contains("lastname") {
        "User's last/family name".to_string()
    } else if name_lower.contains("description") || name_lower.contains("bio") {
        "Descriptive text or biography".to_string()
    } else if name_lower.contains("content") || name_lower.contains("body") {
        "Main content or body text".to_string()
    } else if name_lower.contains("status") {
        "Current state or status in workflow".to_string()
    } else if name_lower.contains("active") || name_lower.contains("enabled") {
        "Whether the record is active/enabled".to_string()
    } else if name_lower.contains("deleted") || name_lower.contains("archived") {
        "Soft deletion or archive flag".to_string()
    } else if name_lower.contains("created_at") || name_lower.contains("createdat") {
        "Timestamp when record was created".to_string()
    } else if name_lower.contains("updated_at") || name_lower.contains("updatedat") {
        "Timestamp when record was last updated".to_string()
    } else if name_lower.contains("deleted_at") || name_lower.contains("deletedat") {
        "Timestamp of soft deletion (null if not deleted)".to_string()
    } else if name_lower.ends_with("_id") || name_lower.ends_with("id") && name_lower != "id" {
        // Foreign key pattern
        let related = name_lower.trim_end_matches("_id").trim_end_matches("id");
        format!("Foreign key reference to {related} record")
    } else if name_lower.contains("count") || name_lower.contains("quantity") {
        "Numeric count or quantity".to_string()
    } else if name_lower.contains("amount")
        || name_lower.contains("price")
        || name_lower.contains("cost")
    {
        "Monetary amount or price".to_string()
    } else if name_lower.contains("url") || name_lower.contains("link") {
        "URL or web link".to_string()
    } else if name_lower.contains("path") || name_lower.contains("file") {
        "File path or storage location".to_string()
    } else if name_lower.contains("token") {
        "Security token or access key".to_string()
    } else if name_lower.contains("date")
        || name_lower.contains("time")
        || field_type.contains("DateTime")
        || field_type.contains("NaiveDateTime")
    {
        "Date/time value".to_string()
    } else if field_type.contains("bool") || field_type.contains("Bool") {
        format!("Boolean flag for {}", name.replace('_', " "))
    } else if field_type.contains("i32") || field_type.contains("i64") || field_type.contains("u32")
    {
        format!("Numeric value for {}", name.replace('_', " "))
    } else {
        format!("Value for {}", name.replace('_', " "))
    }
}

/// Infer relationships from foreign key fields
fn infer_relationships(fields: &[FieldInfo], all_models: &[ModelDetails]) -> Vec<String> {
    let mut relationships = Vec::new();

    for field in fields {
        let name_lower = field.name.to_lowercase();

        // Check for foreign key pattern
        if name_lower.ends_with("_id") && name_lower != "id" {
            let related_name = name_lower.trim_end_matches("_id");

            // Try to find matching model
            let related_model = all_models.iter().find(|m| {
                m.name.to_lowercase() == related_name
                    || m.name.to_lowercase() == format!("{related_name}s")
            });

            if let Some(model) = related_model {
                relationships.push(format!("belongs_to: {}", model.name));
            } else {
                // Capitalize the inferred name
                let capitalized = related_name
                    .split('_')
                    .map(|s| {
                        let mut c = s.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("");
                relationships.push(format!("belongs_to: {capitalized} (inferred)"));
            }
        }
    }

    relationships
}

/// Find routes related to this model
fn find_related_routes(model_name: &str, routes: &[RouteInfo]) -> Vec<String> {
    let snake_name = to_snake_case(model_name);
    let plural = format!("{snake_name}s");

    routes
        .iter()
        .filter(|r| {
            let path_lower = r.path.to_lowercase();
            path_lower.contains(&snake_name) || path_lower.contains(&plural)
        })
        .map(|r| format!("{} {}", r.method, r.path))
        .collect()
}

/// Generate common query patterns for the model
fn generate_common_queries(_model_name: &str, fields: &[FieldInfo]) -> Vec<String> {
    let mut queries = vec!["find_by_id".to_string()];

    for field in fields {
        let name_lower = field.name.to_lowercase();

        // Add common query patterns based on field names
        if name_lower.contains("email") {
            queries.push("find_by_email".to_string());
        }
        if name_lower == "name" || name_lower == "username" {
            queries.push(format!("find_by_{name_lower}"));
        }
        if name_lower.contains("status") {
            queries.push("find_by_status".to_string());
            queries.push("find_active".to_string());
        }
        if name_lower.ends_with("_id") && name_lower != "id" {
            let related = name_lower.trim_end_matches("_id");
            queries.push(format!("find_by_{related}"));
        }
        if name_lower.contains("slug") {
            queries.push("find_by_slug".to_string());
        }
    }

    // Add common collection queries
    queries.push("find_all".to_string());
    queries.push("count".to_string());

    queries
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_field_meaning() {
        assert!(infer_field_meaning("id", "i32").contains("primary key"));
        assert!(infer_field_meaning("email", "String").contains("Email"));
        assert!(infer_field_meaning("user_id", "i32").contains("Foreign key"));
        assert!(infer_field_meaning("created_at", "DateTime").contains("created"));
    }

    #[test]
    fn test_infer_domain_meaning() {
        let fields = vec![FieldInfo {
            name: "email".to_string(),
            field_type: "String".to_string(),
            is_primary_key: false,
            is_nullable: false,
        }];
        let meaning = infer_domain_meaning("User", &fields);
        assert!(meaning.contains("user account"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("TodoItem"), "todo_item");
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
    }

    #[test]
    fn test_infer_relationships() {
        let fields = vec![FieldInfo {
            name: "user_id".to_string(),
            field_type: "i32".to_string(),
            is_primary_key: false,
            is_nullable: false,
        }];
        let models = vec![ModelDetails {
            name: "User".to_string(),
            table: Some("users".to_string()),
            path: "src/models/user.rs".to_string(),
            fields: vec![],
        }];
        let relationships = infer_relationships(&fields, &models);
        assert_eq!(relationships.len(), 1);
        assert!(relationships[0].contains("User"));
    }

    #[test]
    fn test_generate_common_queries() {
        let fields = vec![
            FieldInfo {
                name: "email".to_string(),
                field_type: "String".to_string(),
                is_primary_key: false,
                is_nullable: false,
            },
            FieldInfo {
                name: "status".to_string(),
                field_type: "String".to_string(),
                is_primary_key: false,
                is_nullable: false,
            },
        ];
        let queries = generate_common_queries("User", &fields);
        assert!(queries.contains(&"find_by_id".to_string()));
        assert!(queries.contains(&"find_by_email".to_string()));
        assert!(queries.contains(&"find_by_status".to_string()));
    }
}
