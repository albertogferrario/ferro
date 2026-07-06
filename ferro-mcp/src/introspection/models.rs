//! Model/entity introspection

use crate::tools::application_info::ModelInfo;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, ItemStruct};
use walkdir::WalkDir;

struct ModelVisitor {
    models: Vec<(String, Option<String>)>,
}

impl ModelVisitor {
    fn new() -> Self {
        Self { models: Vec::new() }
    }

    fn has_model_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        // Check for common ORM derives
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(
                            ident.as_deref(),
                            Some("DeriveEntityModel")
                                | Some("Model")
                                | Some("Entity")
                                | Some("ActiveModel")
                        ) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn extract_table_name(&self, attrs: &[Attribute]) -> Option<String> {
        for attr in attrs {
            if attr.path().is_ident("sea_orm") {
                // Try to extract table_name from #[sea_orm(table_name = "...")]
                if let Ok(syn::Meta::NameValue(nv)) = attr.parse_args::<syn::Meta>() {
                    if nv.path.is_ident("table_name") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }) = nv.value
                        {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
        None
    }
}

impl<'ast> Visit<'ast> for ModelVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_model_derive(&node.attrs) {
            let name = node.ident.to_string();
            let table = self.extract_table_name(&node.attrs);
            self.models.push((name, table));
        }
        syn::visit::visit_item_struct(self, node);
    }
}

pub fn scan_models(project_root: &Path) -> Vec<ModelInfo> {
    let src_path = project_root.join("src");
    let models_path = src_path.join("models");
    let entities_path = src_path.join("entities");

    let mut all_models = Vec::new();

    // Scan models directory
    if models_path.exists() {
        scan_directory(&models_path, &mut all_models, project_root);
    }

    // Scan entities directory
    if entities_path.exists() {
        scan_directory(&entities_path, &mut all_models, project_root);
    }

    all_models
}

fn scan_directory(dir: &Path, models: &mut Vec<ModelInfo>, project_root: &Path) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(syntax) = syn::parse_file(&content) {
                let mut visitor = ModelVisitor::new();
                visitor.visit_file(&syntax);

                let relative_path = entry
                    .path()
                    .strip_prefix(project_root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();

                for (name, table) in visitor.models {
                    models.push(ModelInfo {
                        name,
                        table,
                        path: relative_path.clone(),
                    });
                }
            }
        }
    }
}
