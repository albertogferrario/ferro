use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::path::{Path, PathBuf};
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, LitStr};

use crate::utils::levenshtein_distance;

/// Custom parser for redirect! macro
pub struct RedirectInput {
    pub route_name: LitStr,
}

impl Parse for RedirectInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(RedirectInput {
            route_name: input.parse()?,
        })
    }
}

/// Implementation for the redirect! macro
///
/// Supports both path redirects and named route redirects:
/// - Path (starts with /): `redirect!("/dashboard")` → `Redirect::to("/dashboard")`
/// - Named route: `redirect!("users.index")` → `Redirect::route("users.index")`
pub fn redirect_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as RedirectInput);

    let ferro = quote!(::ferro);

    let route_name = input.route_name.value();
    let route_lit = &input.route_name;

    // Check if this is a path (starts with /) or a named route
    if route_name.starts_with('/') {
        // Path redirect - use Redirect::to() directly
        let expanded = quote! {
            #ferro::Redirect::to(#route_lit)
        };
        return expanded.into();
    }

    // Named route - validate it exists at compile time
    if let Err(err) = validate_route_exists(&route_name, route_lit.span()) {
        return err.to_compile_error().into();
    }

    // Generate the redirect builder for named routes
    let expanded = quote! {
        #ferro::Redirect::route(#route_lit)
    };

    expanded.into()
}

fn validate_route_exists(route_name: &str, span: Span) -> Result<(), syn::Error> {
    // Get the manifest directory
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => return Ok(()), // Skip validation if env not available
    };

    let project_root = PathBuf::from(&manifest_dir);

    // Scan main.rs for route definitions
    let available_routes = extract_route_names(&project_root);

    if available_routes.is_empty() {
        // No routes found, skip validation (might be running in different context)
        return Ok(());
    }

    if !available_routes.contains(&route_name.to_string()) {
        let mut error_msg = format!("Route '{route_name}' not found.");

        error_msg.push_str("\n\nAvailable routes:");
        for route in &available_routes {
            error_msg.push_str(&format!("\n  - {route}"));
        }

        // Suggest similar route names
        if let Some(suggestion) = find_similar_route(route_name, &available_routes) {
            error_msg.push_str(&format!("\n\nDid you mean '{suggestion}'?"));
        }

        return Err(syn::Error::new(span, error_msg));
    }

    Ok(())
}

fn extract_route_names(project_root: &Path) -> Vec<String> {
    // Try routes.rs first, fall back to main.rs
    let routes_rs = project_root.join("src").join("routes.rs");
    let main_rs = project_root.join("src").join("main.rs");

    let content = std::fs::read_to_string(&routes_rs)
        .or_else(|_| std::fs::read_to_string(&main_rs))
        .unwrap_or_default();

    if content.is_empty() {
        return Vec::new();
    }

    let mut routes = Vec::new();

    // Use regex to find .name("...") patterns
    let name_re = regex::Regex::new(r#"\.name\s*\(\s*"([^"]+)"\s*\)"#).unwrap();
    for cap in name_re.captures_iter(&content) {
        if let Some(m) = cap.get(1) {
            routes.push(m.as_str().to_string());
        }
    }

    // Find resource! macros and generate their route names
    // Pattern: resource!("/path", ...) - extract path and generate 7 standard route names
    // Also handles: resource!("/path", ..., only: [...])
    let resource_re = regex::Regex::new(r#"resource!\s*\(\s*"(/[^"]*)"#).unwrap();
    let actions_re = regex::Regex::new(r"\[([^\]]+)\]").unwrap();

    for cap in resource_re.captures_iter(&content) {
        if let Some(m) = cap.get(1) {
            let path = m.as_str();
            // Derive name prefix from path: "/users" -> "users", "/api/users" -> "api.users"
            let name_prefix = path.trim_start_matches('/').replace('/', ".");

            // Check if this resource uses "only:" to limit actions
            // Find the full resource! call to check for only:
            let start = m.start();
            let remaining = &content[start..];
            // Find the closing parenthesis or the only: keyword
            if let Some(resource_call_end) = remaining.find("),") {
                let resource_call = &remaining[..resource_call_end + 1];

                if let Some(only_start) = resource_call.find("only:") {
                    // Extract the only: [...] list
                    let only_section = &resource_call[only_start..];
                    if let Some(actions_cap) = actions_re.captures(only_section) {
                        if let Some(actions_str) = actions_cap.get(1) {
                            // Parse individual actions
                            for action in actions_str.as_str().split(',') {
                                let action = action.trim();
                                if !action.is_empty() {
                                    routes.push(format!("{name_prefix}.{action}"));
                                }
                            }
                        }
                    }
                } else {
                    // No "only:" - add all 7 standard routes
                    for action in &[
                        "index", "create", "store", "show", "edit", "update", "destroy",
                    ] {
                        routes.push(format!("{name_prefix}.{action}"));
                    }
                }
            } else {
                // Fallback: if we can't find the end, assume full resource
                for action in &[
                    "index", "create", "store", "show", "edit", "update", "destroy",
                ] {
                    routes.push(format!("{name_prefix}.{action}"));
                }
            }
        }
    }

    routes
}

fn find_similar_route(target: &str, available: &[String]) -> Option<String> {
    let target_lower = target.to_lowercase();

    // Check for case-insensitive exact match first
    for route in available {
        if route.to_lowercase() == target_lower {
            return Some(route.clone());
        }
    }

    // Find closest match using Levenshtein distance
    let mut best_match: Option<(String, usize)> = None;
    let threshold = std::cmp::max(2, target.len() / 3);
    for route in available {
        let distance = levenshtein_distance(&target_lower, &route.to_lowercase());
        if distance <= threshold
            && (best_match.is_none() || distance < best_match.as_ref().unwrap().1)
        {
            best_match = Some((route.clone(), distance));
        }
    }

    best_match.map(|(name, _)| name)
}
