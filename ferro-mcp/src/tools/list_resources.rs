//! List API resources tool - scan for #[derive(ApiResource)] structs

use crate::error::Result;
use quote::ToTokens;
use serde::Serialize;
use std::fs;
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, Fields, ItemStruct, Type};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct ResourcesInfo {
    pub resources: Vec<ResourceInfo>,
}

#[derive(Debug, Serialize)]
pub struct ResourceInfo {
    pub name: String,
    pub path: String,
    pub field_count: usize,
    pub fields: Vec<ResourceFieldInfo>,
}

#[derive(Debug, Serialize)]
pub struct ResourceFieldInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<String>,
}

struct ResourceVisitor {
    resources: Vec<ResourceInfo>,
    current_path: String,
}

impl ResourceVisitor {
    fn new(path: String) -> Self {
        Self {
            resources: Vec::new(),
            current_path: path,
        }
    }

    fn has_api_resource_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                if let Ok(nested) = attr.parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                ) {
                    for path in nested {
                        let ident = path.segments.last().map(|s| s.ident.to_string());
                        if matches!(ident.as_deref(), Some("ApiResource")) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn extract_fields(&self, fields: &Fields) -> Vec<ResourceFieldInfo> {
        let mut field_infos = Vec::new();

        if let Fields::Named(named) = fields {
            for field in &named.named {
                if let Some(ident) = &field.ident {
                    let name = ident.to_string();
                    let field_type = type_to_string(&field.ty);
                    let attributes = extract_resource_attributes(&field.attrs);

                    field_infos.push(ResourceFieldInfo {
                        name,
                        field_type,
                        attributes,
                    });
                }
            }
        }

        field_infos
    }
}

impl<'ast> Visit<'ast> for ResourceVisitor {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        if self.has_api_resource_derive(&node.attrs) {
            let name = node.ident.to_string();
            let fields = self.extract_fields(&node.fields);
            let field_count = fields.len();

            self.resources.push(ResourceInfo {
                name,
                path: self.current_path.clone(),
                field_count,
                fields,
            });
        }
        syn::visit::visit_item_struct(self, node);
    }
}

fn type_to_string(ty: &Type) -> String {
    ty.to_token_stream().to_string().replace(' ', "")
}

/// Extract #[resource(skip)], #[resource(rename = "...")] attributes from fields.
fn extract_resource_attributes(attrs: &[Attribute]) -> Vec<String> {
    let mut result = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("resource") {
            let tokens = attr.meta.to_token_stream().to_string();
            // Extract the inner content: resource(skip) -> "skip", resource(rename = "foo") -> "rename = \"foo\""
            if tokens.contains("skip") {
                result.push("skip".to_string());
            }
            if tokens.contains("rename") {
                // Extract rename value
                if let Some(start) = tokens.find("rename") {
                    let after = &tokens[start..];
                    if let Some(q1) = after.find('"') {
                        let rest = &after[q1 + 1..];
                        if let Some(q2) = rest.find('"') {
                            result.push(format!("rename={}", &rest[..q2]));
                        }
                    }
                }
            }
        }
    }

    result
}

pub fn execute(project_root: &Path) -> Result<ResourcesInfo> {
    let src_path = project_root.join("src");
    let mut all_resources = Vec::new();

    if src_path.exists() {
        scan_directory(&src_path, &mut all_resources, project_root);
    }

    Ok(ResourcesInfo {
        resources: all_resources,
    })
}

fn scan_directory(dir: &Path, resources: &mut Vec<ResourceInfo>, project_root: &Path) {
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(syntax) = syn::parse_file(&content) {
                let relative_path = entry
                    .path()
                    .strip_prefix(project_root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();

                let mut visitor = ResourceVisitor::new(relative_path);
                visitor.visit_file(&syntax);
                resources.extend(visitor.resources);
            }
        }
    }
}
