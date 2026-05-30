//! `#[action]` attribute macro implementation
//!
//! Transforms an `async fn handler(...) -> ActionResult` into a standard
//! ferro handler `async fn handler(__ferro_req: ::ferro::Request) ->
//! ::ferro::Response`. The user's body is wrapped so that any `Err`
//! becomes a 303 redirect with a structured flash payload. See
//! `framework/src/http/action.rs` for the runtime contract.
//!
//! Parameter extraction is shared with `#[handler]` via
//! `crate::utils::{classify_param_type, generate_extraction, ...}`.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, Expr, FnArg, ItemFn, Lit, Meta, Token};

use crate::utils::{classify_param_type, extract_param_name, ferro, generate_extraction};

/// Parsed attributes for `#[action(...)]`.
#[derive(Debug)]
struct ActionAttrs {
    /// Required: target URL for the success-side 303 redirect.
    redirect_to: String,
    /// Optional: HTTP method this action handles. Defaults to `"POST"`.
    /// Currently informational — not used at runtime; reserved for future
    /// per-method dispatch wiring without breaking the surface (D-05).
    #[allow(dead_code)]
    method: String,
}

/// Parse `#[action(redirect_to = "...", method = "...")]`.
///
/// Accepts `proc_macro2::TokenStream` so this function is unit-testable
/// outside the proc-macro compilation environment.
fn parse_action_attrs(attr: TokenStream2) -> Result<ActionAttrs, syn::Error> {
    let mut redirect_to: Option<String> = None;
    let mut method: Option<String> = None;

    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = syn::parse::Parser::parse2(parser, attr)?;

    for meta in metas {
        match meta {
            Meta::NameValue(nv) => {
                let key = nv
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();

                match key.as_str() {
                    "redirect_to" => {
                        redirect_to = Some(expect_str_lit(&nv.value, "redirect_to")?);
                    }
                    "method" => {
                        method = Some(expect_str_lit(&nv.value, "method")?);
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            &nv.path,
                            format!(
                                "#[action] unknown key '{other}' — supported keys are \
                                 `redirect_to` (required) and `method` (optional)",
                            ),
                        ));
                    }
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "#[action] arguments must be of the form `key = \"value\"`",
                ));
            }
        }
    }

    let redirect_to = redirect_to.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "#[action] requires `redirect_to = \"/path\"` — the success-side 303 target",
        )
    })?;

    Ok(ActionAttrs {
        redirect_to,
        method: method.unwrap_or_else(|| "POST".to_string()),
    })
}

fn expect_str_lit(expr: &Expr, key: &str) -> Result<String, syn::Error> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(s) => Ok(s.value()),
            _ => Err(syn::Error::new_spanned(
                expr,
                format!("#[action] `{key}` must be a string literal"),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            expr,
            format!("#[action] `{key}` must be a string literal"),
        )),
    }
}

/// Implementation of the `#[action]` attribute macro.
pub fn action_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr2 = TokenStream2::from(attr);
    let attrs = match parse_action_attrs(attr2) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };
    let redirect_to = attrs.redirect_to;

    let input_fn = parse_macro_input!(input as ItemFn);
    let ferro = ferro();

    let fn_vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    // Param extraction reuses Plan 02's helpers.
    let params: Vec<_> = input_fn.sig.inputs.iter().collect();

    let mut extractions = Vec::new();
    let mut has_request_consumer = false;
    let mut has_request_param = false;

    for param in &params {
        match param {
            FnArg::Typed(pat_type) => {
                let param_pat = &pat_type.pat;
                let param_type = &pat_type.ty;
                let param_name = extract_param_name(param_pat);
                let kind = classify_param_type(param_type);
                let extraction = generate_extraction(
                    &ferro,
                    param_pat,
                    param_type,
                    &param_name,
                    &kind,
                    &mut has_request_consumer,
                    &mut has_request_param,
                );
                extractions.push(extraction);
            }
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    param,
                    "#[action] does not support methods with self receiver",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    // Suppress unused-variable warnings for the tracking booleans — they are
    // set by generate_extraction but the generated output doesn't branch on them
    // (unlike #[handler] which has two identical branches).
    let _ = has_request_consumer;
    let _ = has_request_param;

    let output = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name #fn_generics(__ferro_req: #ferro::Request) -> #ferro::Response {
            let __ferro_params = __ferro_req.params().clone();
            #(#extractions)*

            let __result: #ferro::ActionResult = {
                #fn_block
            };

            #ferro::http::handle_action_result(
                __result,
                #redirect_to,
                concat!(module_path!(), "::", stringify!(#fn_name)),
            )
        }
    };

    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(src: &str) -> TokenStream2 {
        src.parse().expect("parse attr src")
    }

    #[test]
    fn parses_only_redirect_to() {
        let a = parse_action_attrs(attr(r#"redirect_to = "/x""#)).unwrap();
        assert_eq!(a.redirect_to, "/x");
        assert_eq!(a.method, "POST");
    }

    #[test]
    fn parses_redirect_to_and_method() {
        let a = parse_action_attrs(attr(r#"redirect_to = "/x", method = "PATCH""#)).unwrap();
        assert_eq!(a.redirect_to, "/x");
        assert_eq!(a.method, "PATCH");
    }

    #[test]
    fn rejects_missing_redirect_to_empty_attr() {
        let e = parse_action_attrs(attr("")).unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("requires `redirect_to"), "got: {msg}");
    }

    #[test]
    fn rejects_missing_redirect_to_method_only() {
        let e = parse_action_attrs(attr(r#"method = "POST""#)).unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("requires `redirect_to"), "got: {msg}");
    }

    #[test]
    fn rejects_unknown_key() {
        let e = parse_action_attrs(attr(r#"redirect_to = "/x", bogus = "y""#)).unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("unknown key 'bogus'"), "got: {msg}");
    }

    #[test]
    fn rejects_non_string_value() {
        let e = parse_action_attrs(attr(r#"redirect_to = 42"#)).unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("must be a string literal"), "got: {msg}");
    }
}
