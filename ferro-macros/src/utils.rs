//! Shared helpers for the `ferro-macros` proc-macro crate.
//!
//! Hosts:
//! - `levenshtein_distance` — fuzzy matching for error suggestions
//! - `ferro()` — the `::ferro` token path emitted by every macro
//! - Parameter-extraction scaffolding (`ParamKind`, `classify_param_type`,
//!   `generate_extraction`, `extract_param_name`, `is_primitive_type_name`)
//!   used by `#[handler]` and `#[action]`.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Pat, Type};

/// Calculate Levenshtein distance between two strings.
/// Used for fuzzy matching suggestions in error messages.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix: Vec<Vec<usize>> = vec![vec![0; len_b + 1]; len_a + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len_a + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate() {
        *cell = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len_a][len_b]
}

/// Returns the token stream for the ferro crate path: `::ferro`.
/// Emitted as the absolute path so generated code resolves correctly in
/// consumer crates that depend on `ferro-rs` under the `ferro` alias.
pub(crate) fn ferro() -> TokenStream2 {
    quote!(::ferro)
}

/// Parameter classification for handler / action extraction strategy.
///
/// Shared by `#[handler]` and `#[action]`; the choice of `Request` vs
/// `FormRequest` vs `Primitive` vs `Model` determines which trait is used
/// at the call site.
pub(crate) enum ParamKind {
    /// Request type — pass through unchanged.
    Request,
    /// Primitive type (i32, String, ...) — extract from path params via `FromParam`.
    Primitive,
    /// Model type (`*::Model`) — extract via `AutoRouteBinding`.
    Model,
    /// Other types — extract via `FromRequest` (FormRequest, etc.).
    FormRequest,
}

/// Extract the parameter name as a string from the pattern.
pub(crate) fn extract_param_name(pat: &Pat) -> String {
    match pat {
        Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
        Pat::Wild(_) => "_".to_string(),
        _ => "param".to_string(),
    }
}

/// Classify the parameter type to determine extraction strategy.
pub(crate) fn classify_param_type(ty: &Type) -> ParamKind {
    match ty {
        Type::Path(type_path) => {
            let segments = &type_path.path.segments;

            // Check for Request type
            if segments.len() == 1 && segments[0].ident == "Request" {
                return ParamKind::Request;
            }
            if segments.len() == 2 && segments[0].ident == "ferro" && segments[1].ident == "Request"
            {
                return ParamKind::Request;
            }

            // Check for primitive types
            if segments.len() == 1 {
                let ident = segments[0].ident.to_string();
                if is_primitive_type_name(&ident) {
                    return ParamKind::Primitive;
                }
            }

            // Check for Model type (path ends with ::Model)
            if let Some(last_segment) = segments.last() {
                if last_segment.ident == "Model" && segments.len() >= 2 {
                    return ParamKind::Model;
                }
            }

            // Default to FormRequest for other types
            ParamKind::FormRequest
        }
        _ => ParamKind::FormRequest,
    }
}

/// Check if a type name is a primitive that should use `FromParam`.
pub(crate) fn is_primitive_type_name(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "isize"
            | "String"
    )
}

/// Generate extraction code for a parameter based on its classification.
///
/// Returns a `let <pat>: <ty> = ...;` statement that the macro slots into
/// the generated handler body before the user's original code runs.
pub(crate) fn generate_extraction(
    ferro: &TokenStream2,
    pat: &Pat,
    ty: &Type,
    param_name: &str,
    kind: &ParamKind,
    has_consumer: &mut bool,
    has_request: &mut bool,
) -> TokenStream2 {
    match kind {
        ParamKind::Request => {
            *has_request = true;
            *has_consumer = true;
            quote! {
                let #pat: #ty = __ferro_req;
            }
        }
        ParamKind::Primitive => {
            quote! {
                let #pat: #ty = {
                    let __value = __ferro_params.get(#param_name)
                        .ok_or_else(|| #ferro::FrameworkError::param(#param_name))?;
                    <#ty as #ferro::FromParam>::from_param(__value)?
                };
            }
        }
        ParamKind::Model => {
            quote! {
                let #pat: #ty = {
                    let __value = __ferro_params.get(#param_name)
                        .ok_or_else(|| #ferro::FrameworkError::param(#param_name))?;
                    <#ty as #ferro::AutoRouteBinding>::from_route_param(__value).await?
                };
            }
        }
        ParamKind::FormRequest => {
            *has_consumer = true;
            quote! {
                let #pat: #ty = <#ty as #ferro::FromRequest>::from_request(__ferro_req).await?;
            }
        }
    }
}
