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
    /// Register composition guidance for POS-style sale screens (D-03). Everything derivable is
    /// derived; prose is drift-guarded by `register_composition_drift_guard`.
    pub register_composition: RegisterCompositionGuidance,
    /// Live projection surface guidance for v17.0 capabilities (D-03). Prose drift-guarded
    /// by `live_projection_drift_guard`.
    pub live_projection: LiveProjectionGuidance,
    /// Work distribution: offloadable service methods and the deployable worker model.
    /// Read-only summary; see docs/src/features/offload.md for the full authoring surface.
    pub offload: &'static str,
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

/// Register composition guidance for POS-style sale screens (D-03). Everything derivable is derived;
/// prose is drift-guarded by `register_composition_drift_guard`.
#[derive(Debug, Serialize)]
pub struct RegisterCompositionGuidance {
    /// (a) When to use Register layout template vs. a form-only Collect spec. Also states that Numpad
    /// and a standalone FilterTabs are author-composable additions, not part of the v1 register
    /// template (D-06 / 257 D-07).
    pub when_to_use: &'static str,
    /// (b) Form-state selection contract: hidden-input qty accumulation (`data-qty-input`), ONE confirm
    /// POST, single Form common ancestor, TileGrid.form_id == SelectionPanel.form_id, SelectionPanel is
    /// a live client-side view of form state — never a second source of truth.
    pub form_state_contract: &'static str,
    /// (c) Runtime data attributes for filter + numpad + qty wiring (format: `"attr — role"`).
    pub data_attributes: &'static [&'static str],
    /// (d) fill_viewport requirement: required when a spec has TileGrid/SelectionPanel/Numpad; root Grid
    /// needs `fill: true`; supported shell layouts are "app" and "dashboard" ONLY.
    pub fill_viewport_requirement: &'static str,
    /// (e) The four register-composition lint rule ids (three `register-*` rules plus
    /// `fill-viewport-layout-unknown`) to check via design_lint, derived from design::rules().
    pub lint_rules: Vec<RegisterRuleRef>,
    /// (f) Pointer to register_template() (ferro-json-ui/src/projection/intent_layout.rs) — the
    /// one-call Collect->Register override; the projection-derived /cassa sample is the reference.
    pub template_helper: &'static str,
}

/// Register lint-rule reference derived from the rule registry.
#[derive(Debug, Serialize)]
pub struct RegisterRuleRef {
    pub id: &'static str,
    pub title: &'static str,
    pub rationale: &'static str,
}

/// Live projection surface guidance for the v17.0 capabilities (D-03). Everything derivable
/// is derived from a registry; prose is drift-guarded by `live_projection_drift_guard`.
#[derive(Debug, Serialize)]
pub struct LiveProjectionGuidance {
    /// (a) LiveFragment — when to use, the projection/key/template contract, first-paint
    /// behavior with an absent snapshot, and the one-binding-pattern limitation.
    pub live_fragment: &'static str,
    /// (b) Container + channel contract — the `data-live-fragment` marker and the
    /// `data-channel="projection.{name}.{key}"` value format the server emits.
    pub container_contract: &'static str,
    /// (c) `#[memoize]` — when to annotate, request-scoped dedup, coalescing, error caching,
    /// graceful no-op outside request scope, complement to eager_loading/BatchLoad.
    pub memoize: &'static str,
    /// (d) asset!() — one-line embed, content-hashed &'static str URL, lazy register-once,
    /// ferro::bundle mount required, `ferro assets fetch` CLI.
    pub asset_macro: &'static str,
    /// Pointer to docs/src for depth (D-04 compact style).
    pub docs: &'static str,
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
    // Type-scale tokens (v2 — Phase 246)
    TokenInfo {
        name: "--text-display-size",
        purpose: "Font size for display/hero headings",
    },
    TokenInfo {
        name: "--text-display-weight",
        purpose: "Font weight for display/hero headings",
    },
    TokenInfo {
        name: "--text-section-size",
        purpose: "Font size for section headings (h2-level)",
    },
    TokenInfo {
        name: "--text-section-weight",
        purpose: "Font weight for section headings",
    },
    TokenInfo {
        name: "--text-body-size",
        purpose: "Font size for body / paragraph text",
    },
    TokenInfo {
        name: "--text-body-weight",
        purpose: "Font weight for body text",
    },
    TokenInfo {
        name: "--text-meta-size",
        purpose: "Font size for meta / caption text",
    },
    TokenInfo {
        name: "--text-meta-weight",
        purpose: "Font weight for meta / caption text",
    },
    TokenInfo {
        name: "--text-micro-size",
        purpose: "Font size for micro / legal text",
    },
    TokenInfo {
        name: "--text-micro-weight",
        purpose: "Font weight for micro / legal text",
    },
];

/// Runtime data attributes for the register composition (filter, tile-qty, numpad, form-guard).
/// Drift-guarded by `register_composition_drift_guard` — each attribute must appear in FERRO_RUNTIME_JS.
static REGISTER_DATA_ATTRIBUTES: &[&str] = &[
    "data-filter-scope — scoping container for a filter group (TileGrid root)",
    "data-filter-tab=\"<token>\" — filter tab button; empty value = 'All' (FilterTabs)",
    "data-filter-search — optional text search input inside TileGrid",
    "data-filter-text=\"<name>\" — search source on Tile root (emitted from Tile.name)",
    "data-filter-tokens=\"t1 t2 ...\" — space-separated category tokens on Tile root",
    "data-qty-inc=\"{field}\" — Tile tap button; increments the named hidden input",
    "data-qty-dec=\"{field}\" — QuantityStepper − button; decrements the named hidden input",
    "data-qty-input=\"{field}\" — hidden input the runtime writes qty into",
    "data-qty-display=\"{field}\" — display element updated on qty change",
    "data-unit-price — integer cents on Tile root for SelectionPanel running total",
    "data-numpad-target=\"<field>\" — names the hidden input the numpad updates",
    "data-numpad-mode=\"price|quantity\" — entry mode (default: quantity)",
    "data-disable-on-submit — disables non-qty buttons on form submit (double-submit guard)",
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

    // ── Register composition guidance (D-03) ─────────────────────────────────
    let register_rule_ids = [
        "register-fill-viewport",
        "register-grid-fill",
        "register-selection-present",
        "fill-viewport-layout-unknown",
    ];
    let lint_rules: Vec<RegisterRuleRef> = design_rules()
        .iter()
        .filter(|r| register_rule_ids.contains(&r.id))
        .map(|r| RegisterRuleRef {
            id: r.id,
            title: r.title,
            rationale: r.rationale,
        })
        .collect();

    let register_composition = RegisterCompositionGuidance {
        when_to_use: "Use the Register layout template when the screen has BOTH a browseable \
            items pane (TileGrid) and a running-selection pane (SelectionPanel) that pins and \
            scrolls internally; use a plain Collect (Form) spec for a standard create/edit form \
            without an adjacent selection pane. Numpad and a standalone FilterTabs (outside the \
            TileGrid integrated strip) are author-composable additions, NOT part of the v1 \
            register template.",
        form_state_contract: "A single Form element (HTML id, e.g. 'sale_form') is the common \
            ancestor of both the TileGrid pane and the SelectionPanel pane. \
            TileGrid.form_id and SelectionPanel.form_id must both equal the Form's id. \
            Hidden inputs (data-qty-input=\"{field}\") accumulate per-tile quantity; \
            SelectionPanel is a live client-side view of that form state — it is NOT a second \
            source of truth. ONE confirm POST button submits the entire Form; \
            disable_on_submit: true on the confirm Button prevents double-submission. \
            Data contract: handler-supplied rows include price_cents (integer cents, never float) \
            and the fields the Tile $each template binds.",
        data_attributes: REGISTER_DATA_ATTRIBUTES,
        fill_viewport_requirement: "fill_viewport: true at the Spec level is required when a \
            spec contains TileGrid, SelectionPanel, or Numpad (lint rule register-fill-viewport \
            fires otherwise). The root Grid must have fill: true (lint rule register-grid-fill). \
            Supported shell layouts for fill_viewport are 'app' and 'dashboard' ONLY — using any \
            other causes silent whole-page scroll (lint rule fill-viewport-layout-unknown). \
            register_template() emits layout 'dashboard' and fill_viewport: true automatically.",
        lint_rules,
        template_helper: "register_template() at ferro-json-ui/src/projection/intent_layout.rs \
            — pass via VisualContext { templates: Some(register_template()), .. } to override \
            Collect -> Register; see docs/src/json-ui/layouts.md#register-layout-template and \
            the projection-derived /cassa sample.",
    };

    // ── Live projection surface guidance (D-03) ──────────────────────────────
    let live_projection = LiveProjectionGuidance {
        live_fragment: "Use a LiveFragment builtin when a page element must reflect a \
            ferro-projection per-key snapshot in real time without a page reload or client \
            state. Props: projection (the ferro-projection NAME / Projection::NAME const), key \
            (the per-key channel selector), and template (the child JSON-UI spec rendered \
            against the snapshot as its data scope). At first paint with no snapshot the \
            container renders empty (child receives {}); it binds ONE per-key snapshot only — \
            list/collection reconciliation is an explicit non-goal.",
        container_contract: "The server wraps the rendered child in \
            <div data-live-fragment data-channel=\"projection.{name}.{key}\">...</div>; both \
            channel segments are HTML-escaped server-side (server-controlled, not \
            user-injectable). A no-WASM client runtime opens one socket to /_ferro/ws, \
            subscribes per channel, and swaps the container innerHTML on each `fragment` event. \
            See docs/src/json-ui/runtime-primitives.md.",
        memoize: "Annotate an async fn or #[service] method with #[memoize] (use ferro::memoize) \
            when N intents over one key call it during a render pass: it runs the body at most \
            once per (callsite, args) per request and coalesces concurrent callers onto one \
            shared computation (errors cached). It is request-scoped (dropped with the request), \
            a graceful no-op outside request scope, and COMPLEMENTS eager_loading/BatchLoad — it \
            is NOT cross-request caching (that stays ferro-cache).",
        asset_macro:
            "asset!(\"path\") (use ferro::asset!) embeds a file at the call-site-relative \
            path via include_bytes!, registers it once (OnceLock), and returns a content-hashed \
            &'static str URL (e.g. \"/bundles/app.a1b2c3d4.js\") with MIME inferred from the \
            extension. The app must mount ferro::bundle serving for the URL to resolve; \
            `ferro assets fetch iconify|fontsource` downloads third-party assets at author time.",
        docs: "See docs/src/json-ui/components.md, docs/src/json-ui/runtime-primitives.md, \
            docs/src/features/ferro-assets.md, and docs/src/features/projections.md for usage \
            examples.",
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
        register_composition,
        live_projection,
        offload: "Mark a #[service] trait method with #[offload] to derive a ferro-queue Job \
            from its signature — the trait method keeps its in-process contract; #[offload] adds \
            an offload enqueue entrypoint returning a typed result handle. Queue defaults to \
            \"default\"; override with #[offload(queue = \"name\")]. Deploy workers as \
            `<app-bin> worker --queue <name>` at N replicas. See docs/src/features/offload.md.",
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
        // 30 base tokens + 10 type-scale tokens (Phase 246) = 40 total
        assert_eq!(context.design_system.tokens.len(), 40);
        assert!(!context.design_system.intent_patterns.is_empty());
        assert!(!context.design_system.canonical_variants.variant.is_empty());
        assert!(!context.design_system.docs.is_empty());

        // Verify register composition guidance populated (D-03)
        assert!(
            !context.register_composition.when_to_use.is_empty(),
            "register_composition.when_to_use must be non-empty"
        );
        assert_eq!(
            context.register_composition.lint_rules.len(),
            4,
            "must derive all four register-composition rules from design::rules()"
        );

        // Verify live projection guidance populated (D-03)
        assert!(
            !context.live_projection.live_fragment.is_empty(),
            "live_projection.live_fragment must be non-empty"
        );
        assert!(
            !context.live_projection.container_contract.is_empty(),
            "live_projection.container_contract must be non-empty"
        );
        assert!(
            !context.live_projection.memoize.is_empty(),
            "live_projection.memoize must be non-empty"
        );
        assert!(
            !context.live_projection.asset_macro.is_empty(),
            "live_projection.asset_macro must be non-empty"
        );
    }

    #[test]
    fn register_composition_drift_guard() {
        use std::collections::HashSet;
        let ctx = execute();

        // 1. Component names mentioned in the guidance exist as builtins, AND the
        // guidance prose actually mentions them — renaming a component in the prose
        // alone (e.g. SelectionPanel -> CartPanel) fails here, not just registry drift.
        let builtins: HashSet<String> = ferro_json_ui::global_catalog()
            .components_sorted()
            .map(|c| c.name.clone())
            .collect();
        for name in [
            "TileGrid",
            "SelectionPanel",
            "FilterTabs",
            "QuantityStepper",
            "Numpad",
            "Tile",
        ] {
            assert!(
                builtins.contains(name as &str),
                "register guidance names non-builtin `{name}`"
            );
        }
        let prose = format!(
            "{} {} {}",
            ctx.register_composition.when_to_use,
            ctx.register_composition.form_state_contract,
            ctx.register_composition.fill_viewport_requirement
        );
        // QuantityStepper is register vocabulary but intentionally absent from the
        // prose (panel steppers are described by behavior, not component name).
        for name in ["TileGrid", "SelectionPanel", "FilterTabs", "Numpad", "Tile"] {
            assert!(
                prose.contains(name),
                "register guidance prose no longer mentions `{name}`"
            );
        }

        // 2. Every id the guidance hardcodes exists in the rule registry, and is
        // derived into `lint_rules`. The expected list is duplicated here on
        // purpose — asserting against the derived output alone would be vacuous
        // (it is filtered from the registry by construction).
        let rule_ids: HashSet<&str> = ferro_json_ui::design::rules()
            .iter()
            .map(|r| r.id)
            .collect();
        let derived: HashSet<&str> = ctx
            .register_composition
            .lint_rules
            .iter()
            .map(|r| r.id)
            .collect();
        for id in [
            "register-fill-viewport",
            "register-grid-fill",
            "register-selection-present",
            "fill-viewport-layout-unknown",
        ] {
            assert!(rule_ids.contains(id), "registry lost rule `{id}`");
            assert!(
                derived.contains(id),
                "guidance failed to derive rule `{id}`"
            );
        }

        // 3. EVERY published attribute appears in the assembled runtime bundle.
        // Each entry is `"attr — role"` or `"attr=\"...\" — role"`; the attribute
        // name is the token before the first `=` or space.
        for entry in ctx.register_composition.data_attributes {
            let name = entry
                .split([' ', '='])
                .next()
                .expect("attribute entry is non-empty");
            assert!(
                ferro_json_ui::FERRO_RUNTIME_JS.contains(name),
                "runtime bundle missing `{name}` — register guidance is stale"
            );
        }
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
    fn live_projection_drift_guard() {
        use std::collections::HashSet;
        let ctx = execute();

        // 1. LiveFragment is a builtin and the guidance prose mentions it.
        let builtins: HashSet<String> = ferro_json_ui::global_catalog()
            .components_sorted()
            .map(|c| c.name.clone())
            .collect();
        assert!(
            builtins.contains("LiveFragment"),
            "live_projection guidance names non-builtin `LiveFragment`"
        );
        let prose = format!(
            "{} {}",
            ctx.live_projection.live_fragment, ctx.live_projection.container_contract
        );
        assert!(
            prose.contains("LiveFragment"),
            "live_projection prose no longer mentions `LiveFragment`"
        );

        // 2. Data attributes appear in the assembled runtime bundle AND in the prose.
        for attr in ["data-live-fragment", "data-channel"] {
            assert!(
                ferro_json_ui::FERRO_RUNTIME_JS.contains(attr),
                "runtime bundle missing `{attr}` — live_projection guidance is stale"
            );
            assert!(
                prose.contains(attr),
                "live_projection prose no longer mentions `{attr}`"
            );
        }

        // 3. Macro names mentioned in the guidance exist as framework re-exports (checked structurally).
        let macro_prose = format!(
            "{} {} {}",
            ctx.live_projection.memoize, ctx.live_projection.asset_macro, ctx.live_projection.docs
        );
        for name in ["memoize", "asset!"] {
            assert!(
                macro_prose.contains(name),
                "live_projection prose no longer mentions `{name}`"
            );
        }
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
