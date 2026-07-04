//! Generation context tool - returns framework conventions and patterns for code generation

use serde::Serialize;

/// Comprehensive framework conventions for code generation
#[derive(Debug, Serialize)]
pub struct GenerationContext {
    pub naming_conventions: NamingConventions,
    pub file_structure: FileStructure,
    pub common_patterns: CommonPatterns,
    pub avoid: Vec<String>,
    pub imports: ImportTemplates,
    /// Design system summary for JSON-UI spec authoring (D-06).
    pub design_system: DesignSystemSummary,
}

/// Naming conventions for different framework artifacts
#[derive(Debug, Serialize)]
pub struct NamingConventions {
    pub models: String,
    pub tables: String,
    pub handlers: String,
    pub routes: String,
    pub middleware: String,
    pub services: String,
    pub views: String,
}

/// Expected file locations for different artifact types
#[derive(Debug, Serialize)]
pub struct FileStructure {
    pub handlers: String,
    pub models: String,
    pub entities: String,
    pub migrations: String,
    pub middleware: String,
    pub services: String,
    pub views: String,
}

/// Common code patterns with template snippets
#[derive(Debug, Serialize)]
pub struct CommonPatterns {
    pub crud_handler: String,
    pub validation: String,
    pub error_handling: String,
    pub inertia_render: String,
    pub json_ui_view: String,
}

/// Common import blocks for different contexts
#[derive(Debug, Serialize)]
pub struct ImportTemplates {
    pub handler: String,
    pub model: String,
    pub validation: String,
    pub json_ui_view: String,
}

/// Design system summary for agent-authoring context (D-06).
#[derive(Debug, Serialize)]
pub struct DesignSystemSummary {
    /// Semantic token vocabulary (30 slots). Each entry: CSS variable name + one-line purpose.
    pub tokens: &'static [TokenInfo],
    /// Design rules grouped by intent key: rule id + title + rationale.
    pub intent_patterns: std::collections::HashMap<String, Vec<IntentPattern>>,
    /// Canonical variant/tone/size value lists.
    pub canonical_variants: CanonicalVariants,
    /// Pointer to full design system documentation.
    pub docs: &'static str,
}

/// One semantic token slot: CSS variable name + one-line purpose.
#[derive(Debug, Serialize)]
pub struct TokenInfo {
    pub name: &'static str,
    pub purpose: &'static str,
}

/// Rule metadata for a specific intent, derived from the rule registry.
#[derive(Debug, Serialize)]
pub struct IntentPattern {
    pub rule_id: &'static str,
    pub title: &'static str,
    pub rationale: &'static str,
}

/// Canonical shared enum values across JSON-UI components.
#[derive(Debug, Serialize)]
pub struct CanonicalVariants {
    pub variant: Vec<String>,
    pub tone: Vec<String>,
    pub size: Vec<String>,
}

/// Semantic token descriptions — maintained in parallel with `ferro_theme::token::ALL_TOKENS`.
///
/// Order and count MUST match ALL_TOKENS exactly. The count drift guard test
/// (`token_description_count_matches_all_tokens`) enforces this at CI time.
static DESIGN_TOKEN_DESCRIPTIONS: &[TokenInfo] = &[
    TokenInfo {
        name: "--color-background",
        purpose: "Page/canvas background",
    },
    TokenInfo {
        name: "--color-surface",
        purpose: "Component surface (cards, panels)",
    },
    TokenInfo {
        name: "--color-card",
        purpose: "Card background (may differ from surface)",
    },
    TokenInfo {
        name: "--color-border",
        purpose: "Dividers, input borders, separators",
    },
    TokenInfo {
        name: "--color-text",
        purpose: "Primary text",
    },
    TokenInfo {
        name: "--color-text-muted",
        purpose: "Secondary/muted text, placeholders",
    },
    TokenInfo {
        name: "--color-primary",
        purpose: "Primary action color (buttons, links)",
    },
    TokenInfo {
        name: "--color-primary-foreground",
        purpose: "Text on primary-colored surfaces",
    },
    TokenInfo {
        name: "--color-secondary",
        purpose: "Secondary action / subdued UI elements",
    },
    TokenInfo {
        name: "--color-secondary-foreground",
        purpose: "Text on secondary-colored surfaces",
    },
    TokenInfo {
        name: "--color-accent",
        purpose: "Accent highlight (hover, selection)",
    },
    TokenInfo {
        name: "--color-destructive",
        purpose: "Destructive actions and danger states",
    },
    TokenInfo {
        name: "--color-success",
        purpose: "Success / positive states",
    },
    TokenInfo {
        name: "--color-warning",
        purpose: "Warning / caution states",
    },
    TokenInfo {
        name: "--radius-sm",
        purpose: "Small corner radius (badges, chips)",
    },
    TokenInfo {
        name: "--radius-md",
        purpose: "Medium corner radius (buttons, inputs)",
    },
    TokenInfo {
        name: "--radius-lg",
        purpose: "Large corner radius (cards, modals)",
    },
    TokenInfo {
        name: "--radius-full",
        purpose: "Full / pill corner radius (avatars)",
    },
    TokenInfo {
        name: "--shadow-sm",
        purpose: "Small elevation shadow",
    },
    TokenInfo {
        name: "--shadow-md",
        purpose: "Medium elevation shadow (dropdowns)",
    },
    TokenInfo {
        name: "--shadow-lg",
        purpose: "Large elevation shadow (modals)",
    },
    TokenInfo {
        name: "--font-sans",
        purpose: "Body / UI sans-serif font stack",
    },
    TokenInfo {
        name: "--font-mono",
        purpose: "Monospace font stack (code, IDs)",
    },
    TokenInfo {
        name: "--spacing",
        purpose: "Base spacing unit (density scale)",
    },
    TokenInfo {
        name: "--motion-duration-fast",
        purpose: "Fast transitions (100-150 ms)",
    },
    TokenInfo {
        name: "--motion-duration-base",
        purpose: "Standard transitions (200-250 ms)",
    },
    TokenInfo {
        name: "--motion-duration-slow",
        purpose: "Slow transitions (300-400 ms)",
    },
    TokenInfo {
        name: "--motion-ease",
        purpose: "Default easing curve",
    },
    TokenInfo {
        name: "--color-ring",
        purpose: "Focus ring / outline color",
    },
    TokenInfo {
        name: "--font-display",
        purpose: "Display/heading font (defaults to --font-sans)",
    },
];

/// Execute the generation context tool - returns comprehensive framework conventions
pub fn execute() -> GenerationContext {
    // ── Design system summary (D-06) ─────────────────────────────────────────
    use ferro_json_ui::component::{Size, Tone, Variant};
    use ferro_json_ui::design::rules as design_rules;
    use strum::VariantArray;

    let variant_values: Vec<String> = Variant::VARIANTS
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect();
    let tone_values: Vec<String> = Tone::VARIANTS
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect();
    let size_values: Vec<String> = Size::VARIANTS
        .iter()
        .map(|v| v.as_ref().to_string())
        .collect();

    // Group rules by intent; rules with empty intents go into an "all" bucket.
    let mut intent_patterns: std::collections::HashMap<String, Vec<IntentPattern>> =
        std::collections::HashMap::new();
    for rule in design_rules() {
        if rule.intents.is_empty() {
            intent_patterns
                .entry("all".to_string())
                .or_default()
                .push(IntentPattern {
                    rule_id: rule.id,
                    title: rule.title,
                    rationale: rule.rationale,
                });
        } else {
            for &intent in rule.intents {
                intent_patterns
                    .entry(intent.to_string())
                    .or_default()
                    .push(IntentPattern {
                        rule_id: rule.id,
                        title: rule.title,
                        rationale: rule.rationale,
                    });
            }
        }
    }

    let design_system = DesignSystemSummary {
        tokens: DESIGN_TOKEN_DESCRIPTIONS,
        intent_patterns,
        canonical_variants: CanonicalVariants {
            variant: variant_values,
            tone: tone_values,
            size: size_values,
        },
        docs: "See docs/src/design-system/ for the full design system \
               (principles, tokens, variants, patterns, linting).",
    };

    GenerationContext {
        naming_conventions: NamingConventions {
            models: "PascalCase singular (User, BlogPost, Animal)".to_string(),
            tables: "snake_case plural (users, blog_posts, animals)".to_string(),
            handlers: "snake_case verb (show, create, update, destroy, index)".to_string(),
            routes: "RESTful lowercase (GET /users, POST /users, GET /users/{id}, PUT /users/{id}, DELETE /users/{id})".to_string(),
            middleware: "PascalCase (AuthMiddleware, RateLimitMiddleware, CorsMiddleware)".to_string(),
            services: "PascalCase with trait+impl (UserService trait, PostgresUserService impl)".to_string(),
            views: "snake_case singular function (user_list, user_form, dashboard)".to_string(),
        },
        file_structure: FileStructure {
            handlers: "src/controllers/{resource}.rs or src/handlers/{resource}.rs".to_string(),
            models: "src/models/{resource}.rs".to_string(),
            entities: "src/entities/{resource}.rs (SeaORM generated)".to_string(),
            migrations: "migration/src/m{timestamp}_{name}.rs".to_string(),
            middleware: "src/middleware/{name}.rs".to_string(),
            services: "src/services/{name}.rs".to_string(),
            views: "src/views/{name}.json (v2 flat spec) + handler in src/controllers/{name}.rs".to_string(),
        },
        common_patterns: CommonPatterns {
            crud_handler: r#"// Option A: Traditional REST handler (web surface)
#[handler]
pub async fn show(req: Request, id: Path<i32>) -> Response {
    let db = req.db();
    let entity = Entity::find_by_id(*id)
        .one(db)
        .await?
        .ok_or_else(|| not_found("Resource not found"))?;
    Ok(json!(entity))
}

// Option B: Projection-derived MCP CRUD tools (agent surface)
// Add to your ServiceDef in src/projections/<service>.rs:
//   .mcp_write_ability("manage-<service>s")  // write gate
//   .creatable(true)   // derives create_<service> MCP tool
//   .updatable(true)   // derives update_<service> MCP tool
//   .deletable(true)   // derives delete_<service> MCP tool (soft-delete via deleted_at)
//   .soft_delete_column("deleted_at") // required: list_<svc> excludes soft-deleted rows
// Requires: deleted_at column in migration, tenant_column set, mcp_ability for reads.
// Derived tools: create_<svc>, update_<svc>, delete_<svc>, list_<svc> with query polish.
// status is excluded from write inputs when a StateMachine is defined (set server-side)."#
                .to_string(),
            validation: r#"let data = req.input::<CreateRequest>().await?;
Validator::new(&data)
    .rules("email", rules![required(), email()])
    .rules("password", rules![required(), min(8.0)])
    .validate()?;"#.to_string(),
            error_handling: r#"// Return Result<HttpResponse, HttpResponse> (aliased as Response)
// Use ? operator for automatic error conversion
let user = User::find_by_id(id)
    .one(db)
    .await
    .map_err(|e| internal_error(format!("Database error: {}", e)))?
    .ok_or_else(|| not_found("User not found"))?;

// Or use the error helpers directly:
// not_found("message") - 404 response
// bad_request("message") - 400 response
// internal_error("message") - 500 response
// unauthorized() - 401 response"#.to_string(),
            inertia_render: r#"// Basic Inertia render
Inertia::render(&req, "Dashboard/Index", DashboardProps { users })

// When consuming request before render (e.g., form input)
// IMPORTANT: Save context first, then consume request
let ctx = SavedInertiaContext::from(&req);
let form = req.input::<CreateForm>().await?;  // Consumes req
// ... process form ...
Inertia::render_ctx(&ctx, "Users/Show", UserProps { user })"#.to_string(),
            json_ui_view: r#"// src/views/user_list.json (v2 flat spec)
{
  "$schema": "ferro-json-ui/v2",
  "title": "Users",
  "layout": "dashboard",
  "root": "root",
  "elements": {
    "root": {
      "type": "DataTable",
      "props": { "data_path": "/data/users" }
    }
  }
}

// Paired Rust handler
#[handler]
pub async fn user_list(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/user_list.json", data)
}"#
            .to_string(),
        },
        avoid: vec![
            "Don't use unwrap() in handlers - return proper Response errors".to_string(),
            "Don't skip validation for POST/PUT requests - use Validator".to_string(),
            "Don't hardcode configuration values - use Config".to_string(),
            "Don't use raw SQL when SeaORM queries work - prefer type-safe queries".to_string(),
            "Don't expose password_hash or sensitive fields in JSON responses".to_string(),
            "Don't forget SavedInertiaContext when consuming request before Inertia::render".to_string(),
            "Don't use Entity::find() without pagination for large tables".to_string(),
            "Don't use panic! or expect() in request handlers - return errors".to_string(),
            "Don't block async runtime with sync operations - use spawn_blocking if needed".to_string(),
            "Don't store sensitive data in session without encryption".to_string(),
            "Don't create JSON-UI views as .rs files - views are .json spec files loaded by JsonUi::render_file".to_string(),
            "Don't omit the layout field in JSON-UI specs - views without layout render as raw HTML".to_string(),
        ],
        imports: ImportTemplates {
            handler: r#"use ferro::{handler, Request, Response, HttpResponse, ResponseExt};
use serde::Deserialize;"#.to_string(),
            model: r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};"#.to_string(),
            validation: r#"use ferro::{Validator, required, email, min, max, string, rules};"#.to_string(),
            json_ui_view: r#"use ferro::{JsonUi, Response};"#.to_string(),
        },
        design_system,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_context_has_all_sections() {
        let context = execute();

        // Verify naming conventions populated
        assert!(!context.naming_conventions.models.is_empty());
        assert!(!context.naming_conventions.tables.is_empty());
        assert!(!context.naming_conventions.handlers.is_empty());
        assert!(!context.naming_conventions.routes.is_empty());
        assert!(!context.naming_conventions.middleware.is_empty());
        assert!(!context.naming_conventions.services.is_empty());
        assert!(!context.naming_conventions.views.is_empty());

        // Verify file structure populated
        assert!(!context.file_structure.handlers.is_empty());
        assert!(!context.file_structure.models.is_empty());
        assert!(!context.file_structure.entities.is_empty());
        assert!(!context.file_structure.migrations.is_empty());
        assert!(!context.file_structure.middleware.is_empty());
        assert!(!context.file_structure.services.is_empty());
        assert!(!context.file_structure.views.is_empty());

        // Verify common patterns populated
        assert!(!context.common_patterns.crud_handler.is_empty());
        assert!(!context.common_patterns.validation.is_empty());
        assert!(!context.common_patterns.error_handling.is_empty());
        assert!(!context.common_patterns.inertia_render.is_empty());
        assert!(!context.common_patterns.json_ui_view.is_empty());

        // Verify imports populated
        assert!(!context.imports.handler.is_empty());
        assert!(!context.imports.model.is_empty());
        assert!(!context.imports.validation.is_empty());
        assert!(!context.imports.json_ui_view.is_empty());

        // Verify design system summary populated (D-06)
        assert_eq!(context.design_system.tokens.len(), 30);
        assert!(!context.design_system.intent_patterns.is_empty());
        assert!(!context.design_system.canonical_variants.variant.is_empty());
        assert!(!context.design_system.docs.is_empty());
    }

    #[test]
    fn token_description_count_matches_all_tokens() {
        assert_eq!(
            DESIGN_TOKEN_DESCRIPTIONS.len(),
            ferro_theme::token::ALL_TOKENS.len(),
            "DESIGN_TOKEN_DESCRIPTIONS must have one entry per ALL_TOKENS slot (D-06 drift guard)"
        );
    }

    #[test]
    fn test_naming_conventions_complete() {
        let context = execute();

        // Check naming conventions contain expected terms
        assert!(context.naming_conventions.models.contains("PascalCase"));
        assert!(context.naming_conventions.tables.contains("snake_case"));
        assert!(context.naming_conventions.handlers.contains("snake_case"));
        assert!(context.naming_conventions.routes.contains("RESTful"));
        assert!(context.naming_conventions.middleware.contains("PascalCase"));
        assert!(context.naming_conventions.services.contains("trait"));
        assert!(context.naming_conventions.views.contains("snake_case"));
    }

    #[test]
    fn test_avoid_list_not_empty() {
        let context = execute();

        assert!(!context.avoid.is_empty());
        assert!(
            context.avoid.len() >= 5,
            "Should have at least 5 anti-patterns"
        );

        // Verify key anti-patterns are present
        let avoid_text = context.avoid.join(" ");
        assert!(avoid_text.contains("unwrap"));
        assert!(avoid_text.contains("validation"));
        assert!(avoid_text.contains("password"));
    }

    #[test]
    fn test_serialization() {
        let context = execute();
        let json = serde_json::to_string(&context);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        assert!(json_str.contains("naming_conventions"));
        assert!(json_str.contains("file_structure"));
        assert!(json_str.contains("common_patterns"));
        assert!(json_str.contains("avoid"));
        assert!(json_str.contains("imports"));
    }
}
