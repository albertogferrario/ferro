//! v1 → v2 JSON-UI controller migration codemod.
//!
//! Reads a Rust controller file using the v1 `make_node` / `JsonUiView::new`
//! builder pattern, emits a flat JSON spec under `src/views/{module}/{handler}.json`,
//! and rewrites the controller body to call `JsonUi::render_file(...)`.
//!
//! Phase 163 D-09 / D-10 / D-11:
//!   - AST-based (`syn` 2), not regex (D-11).
//!   - Single file per invocation (D-10) — no directory recursion.
//!   - Idempotent — re-running on already-migrated source emits a warning
//!     and exits 0 (D-11).
//!   - `--dry-run` prints proposed output without writing (D-11).
//!   - Cases the codemod cannot translate produce a `// TODO: …` marker
//!     above the handler signature; the handler body is left intact (D-09).
//!
//! The output spec path is `src/views/{module}/{handler}.json`, where `{module}`
//! is the file stem of the controller (e.g. `src/controllers/auth.rs` →
//! `src/views/auth/login_form.json`).

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};
use syn::visit::{self, Visit};
use syn::{ExprCall, File, ImplItemFn, ItemFn};

/// Entry point invoked by the `json-ui:migrate-v1` subcommand.
pub fn run(file: String, dry_run: bool) -> Result<()> {
    let path = PathBuf::from(&file);
    if !path.exists() {
        return Err(anyhow!("controller file not found: {file}"));
    }
    let src = std::fs::read_to_string(&path)?;
    let parsed: File =
        syn::parse_file(&src).map_err(|e| anyhow!("failed to parse {file} as Rust source: {e}"))?;

    if file_already_migrated(&parsed) {
        eprintln!(
            "warning: {file} appears already migrated (uses JsonUi::render_file). \
             No changes made."
        );
        return Ok(());
    }

    let mut visitor = MigrationVisitor::new(&path);
    visitor.visit_file(&parsed);

    if visitor.specs.is_empty() && visitor.todo_handlers.is_empty() {
        eprintln!(
            "warning: {file} contains no recognizable v1 patterns \
             (make_node / JsonUiView::new). No changes made."
        );
        return Ok(());
    }

    let outputs = visitor.finalize(&src)?;

    if dry_run {
        print_dry_run(&outputs);
        return Ok(());
    }

    for SpecOutput {
        path: spec_path,
        json,
    } in &outputs.specs
    {
        if let Some(parent) = spec_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(spec_path, json)?;
        eprintln!("wrote {}", spec_path.display());
    }
    std::fs::write(&path, &outputs.rewritten_controller)?;
    eprintln!("rewrote {}", path.display());
    Ok(())
}

/// Detect "already migrated" by checking for any `JsonUi::render_file(...)`
/// call inside any function body in the file.
fn file_already_migrated(parsed: &File) -> bool {
    struct Detector(bool);
    impl<'ast> Visit<'ast> for Detector {
        fn visit_expr_call(&mut self, call: &'ast ExprCall) {
            let s = quote::quote!(#call).to_string();
            if s.contains("JsonUi :: render_file") || s.contains("JsonUi::render_file") {
                self.0 = true;
            }
            visit::visit_expr_call(self, call);
        }
    }
    let mut d = Detector(false);
    d.visit_file(parsed);
    d.0
}

/// Collected output of a migration run.
struct MigrationOutputs {
    specs: Vec<SpecOutput>,
    rewritten_controller: String,
}

struct SpecOutput {
    path: PathBuf,
    json: String,
}

fn print_dry_run(out: &MigrationOutputs) {
    println!("=== dry-run: proposed JSON specs ===");
    for spec in &out.specs {
        println!("--- {} ---", spec.path.display());
        println!("{}", spec.json);
    }
    println!("=== dry-run: rewritten controller ===");
    println!("{}", out.rewritten_controller);
}

/// AST visitor that scans top-level `ItemFn`s and impl-method `ImplItemFn`s
/// for v1 `JsonUiView::new` / `make_node` patterns.
struct MigrationVisitor {
    /// Successful per-handler migrations (one spec per handler).
    specs: Vec<HandlerMigration>,
    /// Handlers that could not be migrated; rewritten file gets a TODO comment.
    todo_handlers: Vec<String>,
    /// Source path; used to compute the `src/views/{module}` prefix.
    controller_path: PathBuf,
}

#[derive(Debug)]
struct HandlerMigration {
    handler_fn_name: String,
    spec_path: PathBuf,
    spec_json: Value,
}

impl MigrationVisitor {
    fn new(controller_path: &Path) -> Self {
        Self {
            specs: Vec::new(),
            todo_handlers: Vec::new(),
            controller_path: controller_path.to_path_buf(),
        }
    }

    fn finalize(self, original_src: &str) -> Result<MigrationOutputs> {
        let mut rewritten = String::from(original_src);
        // Inject TODO markers first so subsequent rewrites do not shift the
        // anchor strings the marker injector matches against.
        for handler in &self.todo_handlers {
            let marker =
                "// TODO: ferro json-ui:migrate-v1 could not auto-translate this handler\n";
            rewritten = inject_todo_above_handler(&rewritten, handler, marker);
        }
        for migration in &self.specs {
            rewritten = rewrite_handler_body(
                &rewritten,
                &migration.handler_fn_name,
                migration.spec_path.to_string_lossy().as_ref(),
            )?;
        }

        let specs: Vec<SpecOutput> = self
            .specs
            .into_iter()
            .map(|m| SpecOutput {
                path: m.spec_path,
                json: serde_json::to_string_pretty(&m.spec_json).unwrap(),
            })
            .collect();
        Ok(MigrationOutputs {
            specs,
            rewritten_controller: rewritten,
        })
    }
}

impl<'ast> Visit<'ast> for MigrationVisitor {
    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let handler_name = item.sig.ident.to_string();
        match try_migrate_handler(&item.block, &handler_name, &self.controller_path) {
            HandlerResult::Migrated(spec_path, spec_json) => {
                self.specs.push(HandlerMigration {
                    handler_fn_name: handler_name,
                    spec_path,
                    spec_json,
                });
            }
            HandlerResult::Unsupported => {
                self.todo_handlers.push(handler_name);
            }
            HandlerResult::NotAHandler => {}
        }
        visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        // For Plan 07 scope, we only auto-translate free functions.
        // impl-block methods that mention JsonUiView get a TODO marker.
        let handler_name = item.sig.ident.to_string();
        if contains_jsonuiview_new(&item.block) {
            self.todo_handlers.push(handler_name);
        }
        visit::visit_impl_item_fn(self, item);
    }
}

#[derive(Debug)]
enum HandlerResult {
    Migrated(PathBuf, Value),
    Unsupported,
    NotAHandler,
}

fn try_migrate_handler(
    block: &syn::Block,
    handler_name: &str,
    controller_path: &Path,
) -> HandlerResult {
    let body_tokens = quote::quote!(#block).to_string();
    if !body_tokens.contains("JsonUiView") {
        return HandlerResult::NotAHandler;
    }
    // Known un-translatable patterns. Spec::builder fall-through is not a v1
    // shape we can translate flatly; dynamic-key format!() defeats id stability.
    if body_tokens.contains("Spec :: builder")
        || body_tokens.contains("Spec::builder")
        || has_dynamic_key(&body_tokens)
    {
        return HandlerResult::Unsupported;
    }
    if has_runtime_branch(block) {
        return HandlerResult::Unsupported;
    }

    let Some(view) = extract_view_call(block) else {
        return HandlerResult::Unsupported;
    };

    // Build the flat element map. Root is the first top-level node.
    let mut elements = Map::<String, Value>::new();

    let top_ids = flatten_nodes(view.nodes, &mut elements);
    let Some(root) = top_ids.first().cloned() else {
        return HandlerResult::Unsupported;
    };

    let mut spec = Map::new();
    spec.insert(
        "$schema".to_string(),
        Value::String("ferro-json-ui/v2".to_string()),
    );
    if let Some(title) = view.title {
        spec.insert("title".to_string(), Value::String(title));
    }
    if let Some(layout) = view.layout {
        spec.insert("layout".to_string(), Value::String(layout));
    }
    spec.insert("root".to_string(), Value::String(root));
    spec.insert("elements".to_string(), Value::Object(elements));

    let module_name = controller_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    let spec_path: PathBuf = [
        "src",
        "views",
        &module_name,
        &format!("{handler_name}.json"),
    ]
    .iter()
    .collect();

    HandlerResult::Migrated(spec_path, Value::Object(spec))
}

fn flatten_nodes(nodes: Vec<ExtractedNode>, elements: &mut Map<String, Value>) -> Vec<String> {
    let mut ids = Vec::with_capacity(nodes.len());
    for node in nodes {
        let child_ids = flatten_nodes(node.children_nodes, elements);
        let mut el_obj = Map::new();
        el_obj.insert(
            "type".to_string(),
            Value::String(node.component_type.clone()),
        );
        if !node.props.is_empty() {
            el_obj.insert("props".to_string(), Value::Object(node.props));
        }
        if !child_ids.is_empty() {
            el_obj.insert(
                "children".to_string(),
                Value::Array(child_ids.into_iter().map(Value::String).collect()),
            );
        }
        if let Some(action) = node.action {
            el_obj.insert("action".to_string(), action);
        }
        ids.push(node.id.clone());
        elements.insert(node.id, Value::Object(el_obj));
    }
    ids
}

// ---- Extraction types ----

#[derive(Debug, Default)]
struct ExtractedView {
    title: Option<String>,
    layout: Option<String>,
    nodes: Vec<ExtractedNode>,
}

#[derive(Debug)]
struct ExtractedNode {
    id: String,
    component_type: String,
    props: Map<String, Value>,
    /// Children parsed from `fields: vec![...]` / `buttons: vec![...]` /
    /// `children: vec![...]` fields nested inside the component's struct
    /// literal. The caller flattens these into the top-level elements map.
    children_nodes: Vec<ExtractedNode>,
    action: Option<Value>,
}

/// Plan 07 Task 2: full AST walk of a `JsonUiView::new(...).layout(...)` chain.
///
/// Filled in alongside the fixture-driven tests (Task 2).
fn extract_view_call(_block: &syn::Block) -> Option<ExtractedView> {
    None
}

// ---- Heuristics for un-translatable patterns ----

fn has_runtime_branch(block: &syn::Block) -> bool {
    // True if any `if cond { JsonUiView::new(...) } else { JsonUiView::new(...) }`
    // or `match cond { _ => JsonUiView::new(...) }` appears in the block.
    struct Detector(bool);
    impl<'ast> Visit<'ast> for Detector {
        fn visit_expr_if(&mut self, expr: &'ast syn::ExprIf) {
            if contains_jsonuiview_new(&expr.then_branch) {
                self.0 = true;
                return;
            }
            if let Some((_, else_branch)) = &expr.else_branch {
                let s = quote::quote!(#else_branch).to_string();
                if s.contains("JsonUiView") {
                    self.0 = true;
                    return;
                }
            }
            visit::visit_expr_if(self, expr);
        }
        fn visit_expr_match(&mut self, expr: &'ast syn::ExprMatch) {
            for arm in &expr.arms {
                let s = quote::quote!(#arm).to_string();
                if s.contains("JsonUiView") {
                    self.0 = true;
                    return;
                }
            }
            visit::visit_expr_match(self, expr);
        }
    }
    let mut d = Detector(false);
    d.visit_block(block);
    d.0
}

fn has_dynamic_key(body_tokens: &str) -> bool {
    // Detect `make_node(format!("..", ...), ...)` patterns. The token stream
    // serialization spaces `!` apart from `format`, so we look for
    // `format !` near `make_node`.
    if !body_tokens.contains("format !") {
        return false;
    }
    body_tokens.contains("make_node")
}

fn contains_jsonuiview_new(block: &syn::Block) -> bool {
    quote::quote!(#block).to_string().contains("JsonUiView")
}

// ---- Source rewriting helpers ----

fn inject_todo_above_handler(src: &str, handler_name: &str, marker: &str) -> String {
    // Find `fn <handler_name>(` and insert `marker` above the line that
    // contains the matched `fn`.
    let needle = format!("fn {handler_name}(");
    let Some(fn_pos) = src.find(&needle) else {
        return src.to_string();
    };
    let line_start = src[..fn_pos].rfind('\n').map(|n| n + 1).unwrap_or(0);
    let mut out = String::with_capacity(src.len() + marker.len());
    out.push_str(&src[..line_start]);
    out.push_str(marker);
    out.push_str(&src[line_start..]);
    out
}

fn rewrite_handler_body(src: &str, handler_name: &str, spec_path: &str) -> Result<String> {
    let needle = format!("fn {handler_name}(");
    let Some(start) = src.find(&needle) else {
        eprintln!("warning: handler {handler_name} not found at rewrite time");
        return Ok(src.to_string());
    };
    let Some(brace_off) = src[start..].find('{') else {
        return Ok(src.to_string());
    };
    let body_start = start + brace_off;
    let mut depth = 0i32;
    let mut body_end = body_start;
    for (i, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    body_end = body_start + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if body_end == body_start {
        return Ok(src.to_string());
    }
    let new_body =
        format!("{{\n    JsonUi::render_file(\"{spec_path}\", serde_json::json!({{}}))\n}}");
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..body_start]);
    out.push_str(&new_body);
    out.push_str(&src[body_end..]);
    Ok(out)
}
