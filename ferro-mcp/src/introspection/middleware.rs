//! Middleware introspection

use serde::Serialize;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, ItemStruct};
use walkdir::WalkDir;

/// Middleware item from static analysis
#[derive(Debug, Serialize, Clone)]
pub struct MiddlewareItem {
    pub name: String,
    pub path: String,
    pub global: bool,
}

struct MiddlewareVisitor {
    middleware: Vec<String>,
}

impl MiddlewareVisitor {
    fn new() -> Self {
        Self {
            middleware: Vec::new(),
        }
    }

    fn has_middleware_impl(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(ident.as_deref(), Some("Middleware")) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for MiddlewareVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_middleware_impl(&node.attrs) {
            self.middleware.push(node.ident.to_string());
        }
        syn::visit::visit_item_struct(self, node);
    }
}

pub fn scan_middleware(project_root: &Path) -> Vec<MiddlewareItem> {
    let middleware_dir = project_root.join("src/middleware");
    let bootstrap_file = project_root.join("src/bootstrap.rs");

    let mut all_middleware = Vec::new();
    let mut global_middleware = Vec::new();

    // Check bootstrap.rs for global middleware registration
    if bootstrap_file.exists() {
        if let Ok(content) = fs::read_to_string(&bootstrap_file) {
            global_middleware = extract_global_middleware(&content);
        }
    }

    // Scan middleware directory
    if !middleware_dir.exists() {
        return all_middleware;
    }

    for entry in WalkDir::new(&middleware_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(syntax) = syn::parse_file(&content) {
                let mut visitor = MiddlewareVisitor::new();
                visitor.visit_file(&syntax);

                let relative_path = entry
                    .path()
                    .strip_prefix(project_root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();

                for middleware_name in visitor.middleware {
                    let is_global = global_middleware.contains(&middleware_name);
                    all_middleware.push(MiddlewareItem {
                        name: middleware_name,
                        path: relative_path.clone(),
                        global: is_global,
                    });
                }
            }
        }
    }

    // Also check for struct definitions that implement Middleware trait
    if all_middleware.is_empty() {
        // Fallback: scan for common middleware patterns
        for entry in WalkDir::new(&middleware_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if content.contains("impl Middleware") || content.contains("#[async_trait]") {
                    // Extract struct names
                    for line in content.lines() {
                        if line.trim().starts_with("pub struct ") {
                            if let Some(name) = line
                                .trim()
                                .strip_prefix("pub struct ")
                                .and_then(|s| s.split(|c: char| !c.is_alphanumeric()).next())
                            {
                                let relative_path = entry
                                    .path()
                                    .strip_prefix(project_root)
                                    .unwrap_or(entry.path())
                                    .to_string_lossy()
                                    .to_string();

                                let is_global = global_middleware.contains(&name.to_string());
                                all_middleware.push(MiddlewareItem {
                                    name: name.to_string(),
                                    path: relative_path,
                                    global: is_global,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    all_middleware
}

fn extract_global_middleware(content: &str) -> Vec<String> {
    let mut global = Vec::new();

    // Look for global_middleware! or register_global_middleware! macro calls
    for line in content.lines() {
        if line.contains("global_middleware!") || line.contains("register_global_middleware!") {
            // Extract middleware names from the macro
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    let middleware_list = &line[start + 1..end];
                    for item in middleware_list.split(',') {
                        let name = item
                            .trim()
                            .trim_matches(|c| c == '"' || c == '\'' || c == '<' || c == '>');
                        if !name.is_empty() {
                            global.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    global
}
