//! List models tool - returns detailed model/entity information

use crate::error::{McpError, Result};
use quote::ToTokens;
use serde::Serialize;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, Fields, ItemStruct, Type};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct ModelDetails {
    pub name: String,
    pub table: Option<String>,
    pub path: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Serialize)]
pub struct FieldInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub is_primary_key: bool,
    pub is_nullable: bool,
}

struct ModelVisitor {
    models: Vec<ModelDetails>,
    current_path: String,
}

impl ModelVisitor {
    fn new(path: String) -> Self {
        Self {
            models: Vec::new(),
            current_path: path,
        }
    }

    fn has_model_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(
                            ident.as_deref(),
                            Some("DeriveEntityModel") | Some("Model") | Some("Entity")
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

    fn extract_fields(&self, fields: &Fields, attrs: &[Attribute]) -> Vec<FieldInfo> {
        let mut field_infos = Vec::new();

        // Check if there's a primary_key attribute at struct level
        let struct_pk = self.find_primary_key_attr(attrs);

        if let Fields::Named(named) = fields {
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    let name = ident.to_string();
                    let field_type = self.type_to_string(&field.ty);
                    let is_nullable = field_type.starts_with("Option<");
                    let is_primary_key = self.is_field_primary_key(&field.attrs)
                        || struct_pk.as_ref() == Some(&name);

                    field_infos.push(FieldInfo {
                        name,
                        field_type,
                        is_primary_key,
                        is_nullable,
                    });
                }
            }
        }

        field_infos
    }

    fn is_field_primary_key(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("sea_orm") {
                let tokens = attr.meta.to_token_stream().to_string();
                if tokens.contains("primary_key") {
                    return true;
                }
            }
        }
        false
    }

    fn find_primary_key_attr(&self, attrs: &[Attribute]) -> Option<String> {
        for attr in attrs {
            if attr.path().is_ident("sea_orm") {
                let tokens = attr.meta.to_token_stream().to_string();
                // Look for primary_key = "field_name"
                if let Some(start) = tokens.find("primary_key") {
                    let after = &tokens[start..];
                    if let Some(quote_start) = after.find('"') {
                        let after_quote = &after[quote_start + 1..];
                        if let Some(quote_end) = after_quote.find('"') {
                            return Some(after_quote[..quote_end].to_string());
                        }
                    }
                }
            }
        }
        None
    }

    fn type_to_string(&self, ty: &Type) -> String {
        use quote::ToTokens;
        ty.to_token_stream().to_string().replace(' ', "")
    }
}

impl<'ast> Visit<'ast> for ModelVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_model_derive(&node.attrs) {
            let name = node.ident.to_string();
            let table = self.extract_table_name(&node.attrs);
            let fields = self.extract_fields(&node.fields, &node.attrs);

            self.models.push(ModelDetails {
                name,
                table,
                path: self.current_path.clone(),
                fields,
            });
        }
        syn::visit::visit_item_struct(self, node);
    }
}

pub fn execute(project_root: &Path) -> Result<Vec<ModelDetails>> {
    let src_path = project_root.join("src");
    let models_path = src_path.join("models");
    let entities_path = src_path.join("entities");

    let mut all_models = Vec::new();

    // Scan models directory
    if models_path.exists() {
        scan_directory(&models_path, &mut all_models, project_root)?;
    }

    // Scan entities directory
    if entities_path.exists() {
        scan_directory(&entities_path, &mut all_models, project_root)?;
    }

    if all_models.is_empty() {
        return Err(McpError::NotFound(
            "No models found. Check src/models/ or src/entities/ directories.".to_string(),
        ));
    }

    Ok(all_models)
}

fn scan_directory(dir: &Path, models: &mut Vec<ModelDetails>, project_root: &Path) -> Result<()> {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let content = fs::read_to_string(entry.path())
            .map_err(|e| McpError::FileReadError(format!("{}: {}", entry.path().display(), e)))?;

        if let Ok(syntax) = syn::parse_file(&content) {
            let relative_path = entry
                .path()
                .strip_prefix(project_root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .to_string();

            let mut visitor = ModelVisitor::new(relative_path);
            visitor.visit_file(&syntax);

            models.extend(visitor.models);
        }
    }

    Ok(())
}
