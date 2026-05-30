//! `#[action]` attribute macro implementation.
//!
//! Transforms an async handler that returns `ActionResult` into a
//! `ferro::Response`-returning handler. The macro:
//!
//! - Parses required `redirect_to = "..."` and optional `method = "..."`.
//! - Extracts parameters using the shared scaffold in `crate::utils`
//!   (same as `#[handler]`), with an action-specific variant for `Request`
//!   that binds `&mut Request` instead of consuming the request.
//! - Wraps the user body in an `ActionResult`-typed scope so `?` works.
//! - Dispatches the result to `ferro::http::action::handle_action_result`,
//!   which emits the 303 redirect, writes session flash, and logs.
//!
//! # Generated handler shape
//!
//! ```ignore
//! pub async fn h(__ferro_req: ::ferro::Request) -> ::ferro::Response {
//!     let mut __ferro_req = __ferro_req;
//!     let __ferro_params = __ferro_req.params().clone();
//!     /* extractions for user-declared params */
//!     let __action_result: ::ferro::ActionResult = { /* user body */ };
//!     ::ferro::http::action::handle_action_result(
//!         __action_result,
//!         "<redirect_to literal>",
//!         concat!(module_path!(), "::", stringify!(h)),
//!         &mut __ferro_req,
//!     )
//! }
//! ```
//!
//! # Request binding ownership
//!
//! `#[handler]`'s `generate_extraction` for `ParamKind::Request` emits
//! `let #pat: #ty = __ferro_req;` which moves the request. Since
//! `handle_action_result` needs `&mut __ferro_req` AFTER the user body runs,
//! `#[action]` uses `generate_action_extraction` instead, which binds the
//! `Request` param as `&mut ::ferro::Request` — leaving `__ferro_req` alive
//! for the final dispatch call.
//!
//! # FormRequest limitation (Phase 180)
//!
//! `FromRequest::from_request` takes `Request` by move (see
//! `framework/src/http/extract.rs`). Consuming `__ferro_req` inside the
//! extraction would make the subsequent `&mut __ferro_req` in the dispatch
//! call a use-after-move error. Until `FromRequest` gains a `&mut Request`
//! variant, `#[action]` emits a `compile_error!` for `FormRequest` params.
//! Extract the form from `req` inside the body via
//! `let form: MyForm = req.form().await?;` instead.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Expr, FnArg, ItemFn, Lit, Meta};

use crate::utils::{classify_param_type, extract_param_name, ferro, ParamKind};

/// Parsed attributes from `#[action(redirect_to = "...", method = "POST")]`.
struct ActionAttrs {
    redirect_to: String,
    /// HTTP method; default `"POST"`. Currently unused in the generated handler
    /// body — the attribute is parsed so the surface matches CONTEXT D-05 and
    /// future routing integration can read it.
    #[allow(dead_code)]
    method: String,
}

/// Parse `#[action(...)]` attribute arguments.
///
/// Required: `redirect_to = "<path>"`.
/// Optional: `method = "<METHOD>"` (default `"POST"`).
/// Any other key emits a `compile_error!`.
fn parse_action_attrs(attr: TokenStream) -> Result<ActionAttrs, syn::Error> {
    let mut redirect_to: Option<String> = None;
    let mut method: String = "POST".to_string();

    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let metas = syn::parse::Parser::parse(parser, attr).map_err(|e| {
        syn::Error::new(
            e.span(),
            format!("#[action]: invalid attribute syntax: {e}"),
        )
    })?;

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
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                redirect_to = Some(lit_str.value());
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "#[action]: `redirect_to` must be a string literal",
                        ));
                    }
                    "method" => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                method = lit_str.value();
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "#[action]: `method` must be a string literal (default \"POST\")",
                        ));
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            format!(
                                "#[action]: unknown attribute `{other}` — supported keys: `redirect_to`, `method`"
                            ),
                        ));
                    }
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "#[action]: only `key = \"value\"` attributes are supported",
                ));
            }
        }
    }

    let redirect_to = redirect_to.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "#[action]: `redirect_to` is required, e.g. #[action(redirect_to = \"/dashboard/foo\")]",
        )
    })?;

    Ok(ActionAttrs {
        redirect_to,
        method,
    })
}

/// Generate extraction code for an action handler parameter.
///
/// Differs from `crate::utils::generate_extraction` in the `Request` case:
/// emits `let #pat: &mut ::ferro::Request = &mut __ferro_req;` so that
/// `__ferro_req` remains alive (un-moved) for the final dispatch call to
/// `handle_action_result(&mut __ferro_req)`.
///
/// For `FormRequest` params, emits `compile_error!` — see module-level note.
fn generate_action_extraction(
    ferro: &proc_macro2::TokenStream,
    pat: &syn::Pat,
    ty: &syn::Type,
    param_name: &str,
    kind: &ParamKind,
) -> proc_macro2::TokenStream {
    match kind {
        ParamKind::Request => {
            // Bind as &mut Request so the borrow is released at the end of
            // { #fn_block } before handle_action_result borrows __ferro_req.
            quote! {
                let #pat: &mut #ferro::Request = &mut __ferro_req;
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
            // FromRequest::from_request takes Request by move (see
            // framework/src/http/extract.rs:45). Consuming __ferro_req here
            // would make the &mut __ferro_req in the dispatch call a
            // use-after-move error. Emit a compile_error! so the user knows
            // to extract the form inside the body instead.
            quote! {
                compile_error!("#[action] does not yet support FormRequest parameters. Extract the form from `req` inside the body, e.g. `let form: MyForm = req.form().await?;`");
            }
        }
    }
}

/// Implementation of the `#[action]` attribute macro.
pub fn action_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = match parse_action_attrs(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let input_fn = parse_macro_input!(input as ItemFn);

    let ferro = ferro();

    let fn_vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    // fn_output is intentionally discarded — replaced by `-> #ferro::Response`.

    let params: Vec<_> = input_fn.sig.inputs.iter().collect();
    let mut extractions = Vec::new();

    for param in &params {
        match param {
            FnArg::Typed(pat_type) => {
                let param_pat = &pat_type.pat;
                let param_type = &pat_type.ty;
                let param_name = extract_param_name(param_pat);
                let kind = classify_param_type(param_type);
                let extraction =
                    generate_action_extraction(&ferro, param_pat, param_type, &param_name, &kind);
                extractions.push(extraction);
            }
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    param,
                    "#[action] does not support methods with `self` receiver",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let redirect_to_lit = &attrs.redirect_to;

    // The `let mut __ferro_req = __ferro_req;` re-binding is intentional:
    // it makes __ferro_req mutable so that `&mut __ferro_req` compiles for
    // both the Request param extraction and the final dispatch call.
    //
    // The borrow lifetime of the `req: &mut Request` user binding ends at
    // the closing `}` of `{ #fn_block }`, which is BEFORE
    // `handle_action_result` borrows `__ferro_req` again — so the borrow
    // checker is satisfied.
    let output = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name #fn_generics(__ferro_req: #ferro::Request) -> #ferro::Response {
            let mut __ferro_req = __ferro_req;
            let __ferro_params = __ferro_req.params().clone();
            #(#extractions)*
            // The body runs inside an `async move` block so the `?` operator
            // propagates to ActionResult (the block's inferred Output), not to
            // the outer Response-returning fn. Without this wrapping `?` would
            // try to convert errors via `From<E> for HttpResponse`, defeating
            // the killer-feature ergonomics of `?` on `String`, `FrameworkError`,
            // and `sea_orm::DbErr`.
            let __action_result: #ferro::ActionResult = async move { #fn_block }.await;
            #ferro::http::action::handle_action_result(
                __action_result,
                #redirect_to_lit,
                concat!(module_path!(), "::", stringify!(#fn_name)),
                &mut __ferro_req,
            )
        }
    };

    output.into()
}
