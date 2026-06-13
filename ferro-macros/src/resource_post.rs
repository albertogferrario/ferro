//! `#[resource_post]` attribute macro implementation.
//!
//! Folds the recurring tenant-scoped POST handler prelude + validation-failure
//! redirect envelope into a single attribute:
//!
//! 1. Extract the resource id from the `"id"` path param as `<R as TenantScoped>::Id`.
//! 2. Resolve the tenant via `::ferro::current_tenant()` (or a caller-supplied expr).
//! 3. Look up the resource via `<R as TenantScoped>::find_for_tenant(id, tenant.id)`.
//! 4. On miss: 303 redirect (if `on_miss` given) or return a 404 `HttpResponse`.
//! 5. Synthesize `__form_url` from `form_url` template (if given).
//! 6. Delegate to `__<name>_inner` with typed params + `__form_url: &str`.
//! 7. Dispatch the `ActionResult` via `handle_action_result`.
//!
//! # Generated shape
//!
//! ```ignore
//! pub async fn save(__ferro_req: ::ferro::Request) -> ::ferro::Response {
//!     let mut __ferro_req = __ferro_req;
//!     let __ferro_params = __ferro_req.params().clone();
//!     // id extraction, tenant resolution, lookup, miss arm, form_url synthesis ...
//!     let __action_result: ::ferro::ActionResult = async move {
//!         __save_inner(&mut __ferro_req, &__tenant, &__resource, &__form_url).await
//!     }.await;
//!     ::ferro::http::action::handle_action_result(
//!         __action_result, "/redirect", "module::save", &mut __ferro_req,
//!     )
//! }
//!
//! async fn __save_inner(
//!     req: &mut ::ferro::Request,
//!     tenant: &::ferro::TenantContext,
//!     model: &Model,
//!     __form_url: &str,
//! ) -> ::ferro::ActionResult { /* user body */ }
//! ```
//!
//! # Security
//!
//! The generated lookup always calls `find_for_tenant(__resource_id, __tenant.id)` —
//! there is no code path that emits an un-scoped `find_by_id`. T-212-01 mitigation.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Expr, FnArg, ItemFn, Lit, Meta, Pat};

use crate::utils::ferro;

/// Parsed attributes from `#[resource_post(ResourceType, redirect_to = "...", form_url = "...", ...)]`.
struct ResourcePostAttrs {
    /// The resource type token stream (first positional arg).
    resource_ty: proc_macro2::TokenStream,
    /// Required: default 303 redirect on success.
    redirect_to: String,
    /// Optional edit-form URL template; `{id}` placeholder supported.
    form_url: Option<String>,
    /// Optional miss redirect URL; omitted → `ActionError::not_found`.
    on_miss: Option<String>,
    /// Optional escape-hatch tenant expression (D-02).
    tenant_expr: Option<String>,
    /// Optional lookup function override (D-04).
    find_fn: Option<String>,
}

/// Parse `#[resource_post(...)]` attribute arguments.
fn parse_resource_post_attrs(attr: TokenStream) -> Result<ResourcePostAttrs, syn::Error> {
    let mut resource_ty: Option<proc_macro2::TokenStream> = None;
    let mut redirect_to: Option<String> = None;
    let mut form_url: Option<String> = None;
    let mut on_miss: Option<String> = None;
    let mut tenant_expr: Option<String> = None;
    let mut find_fn: Option<String> = None;

    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let metas = syn::parse::Parser::parse(parser, attr).map_err(|e| {
        syn::Error::new(
            e.span(),
            format!("#[resource_post]: invalid attribute syntax: {e}"),
        )
    })?;

    for meta in metas {
        match meta {
            // First positional arg: the resource type
            Meta::Path(p) => {
                if resource_ty.is_none() {
                    resource_ty = Some(quote! { #p });
                } else {
                    return Err(syn::Error::new_spanned(
                        p,
                        "#[resource_post]: unexpected extra positional type argument",
                    ));
                }
            }
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
                            "#[resource_post]: `redirect_to` must be a string literal",
                        ));
                    }
                    "form_url" => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                form_url = Some(lit_str.value());
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "#[resource_post]: `form_url` must be a string literal",
                        ));
                    }
                    "on_miss" => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                on_miss = Some(lit_str.value());
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "#[resource_post]: `on_miss` must be a string literal",
                        ));
                    }
                    "tenant" => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                tenant_expr = Some(lit_str.value());
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "#[resource_post]: `tenant` must be a string literal expression",
                        ));
                    }
                    "find" => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                find_fn = Some(lit_str.value());
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "#[resource_post]: `find` must be a string literal path",
                        ));
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            format!(
                                "#[resource_post]: unknown attribute `{other}` — supported keys: `redirect_to`, `form_url`, `on_miss`, `tenant`, `find`"
                            ),
                        ));
                    }
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "#[resource_post]: only a positional Type or `key = \"value\"` attributes are supported",
                ));
            }
        }
    }

    let resource_ty = resource_ty.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "#[resource_post]: a resource type is required as the first argument, e.g. #[resource_post(Customer, redirect_to = \"/list\")]",
        )
    })?;

    let redirect_to = redirect_to.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "#[resource_post]: `redirect_to` is required, e.g. #[resource_post(Customer, redirect_to = \"/dashboard/foo\")]",
        )
    })?;

    Ok(ResourcePostAttrs {
        resource_ty,
        redirect_to,
        form_url,
        on_miss,
        tenant_expr,
        find_fn,
    })
}

/// Validate `{placeholder}` tokens in a URL string.
///
/// Recognized names are `"id"` and the resource param name. An unterminated `{`
/// with no closing `}` is rejected — it would produce a malformed `format!` string
/// in the generated code if accepted silently.
fn validate_url_placeholders(
    url: &str,
    resource_param_name: &str,
    context: &str,
) -> Result<(), String> {
    let mut i = 0;
    let bytes = url.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let start = i + 1;
            if let Some(end_off) = bytes[start..].iter().position(|&b| b == b'}') {
                let name = &url[start..start + end_off];
                if name != "id" && name != resource_param_name {
                    return Err(format!(
                        "#[resource_post]: unknown path param `{{{name}}}` in `{context}` — \
                         declared params are: id, {resource_param_name}"
                    ));
                }
                i = start + end_off + 1;
                continue;
            } else {
                // `{` with no matching `}` — unterminated placeholder.
                return Err(format!(
                    "#[resource_post]: unterminated `{{` placeholder in `{context}` — \
                     missing closing `}}`"
                ));
            }
        }
        i += 1;
    }
    Ok(())
}

/// Build a `format!` call that substitutes `{id}` / `{resource}` in a URL template.
fn build_url_format(
    url: &str,
    resource_param_name: &str,
) -> (String, Vec<proc_macro2::TokenStream>) {
    let mut fmt = String::new();
    let mut args: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut i = 0;
    let bytes = url.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'{' {
            let start = i + 1;
            if let Some(end_off) = bytes[start..].iter().position(|&b| b == b'}') {
                let name = &url[start..start + end_off];
                fmt.push_str("{}");
                if name == "id" || name == resource_param_name {
                    args.push(quote! { __resource_id });
                }
                i = start + end_off + 1;
                continue;
            }
        }
        fmt.push(bytes[i] as char);
        i += 1;
    }
    (fmt, args)
}

/// Parsed inner params from a resource fn signature.
type InnerParams = (
    Box<syn::Pat>,
    Box<syn::Type>,
    Box<syn::Pat>,
    Box<syn::Type>,
    String,
);

/// Extract the tenant and resource params from the user's fn signature.
fn extract_inner_params(input_fn: &ItemFn) -> Result<InnerParams, syn::Error> {
    let params: Vec<_> = input_fn.sig.inputs.iter().collect();

    if params.len() < 3 {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "#[resource_post]: function must have at least 3 params: `req`, `tenant`, and the resource",
        ));
    }

    let tenant_param = match params[1] {
        FnArg::Typed(pt) => pt,
        FnArg::Receiver(r) => {
            return Err(syn::Error::new_spanned(
                r,
                "#[resource_post]: does not support `self` receiver",
            ))
        }
    };

    let resource_param = match params[2] {
        FnArg::Typed(pt) => pt,
        FnArg::Receiver(r) => {
            return Err(syn::Error::new_spanned(
                r,
                "#[resource_post]: does not support `self` receiver",
            ))
        }
    };

    let tenant_ty = tenant_param.ty.clone();
    let tenant_pat = tenant_param.pat.clone();
    let resource_ty = resource_param.ty.clone();
    let resource_pat = resource_param.pat.clone();

    let resource_name = match resource_param.pat.as_ref() {
        Pat::Ident(pi) => pi.ident.to_string(),
        _ => "resource".to_string(),
    };

    Ok((
        tenant_pat,
        tenant_ty,
        resource_pat,
        resource_ty,
        resource_name,
    ))
}

/// Implementation of the `#[resource_post]` attribute macro.
pub fn resource_post_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = match parse_resource_post_attrs(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let input_fn = parse_macro_input!(input as ItemFn);

    // Must be async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input_fn.sig.fn_token,
            "#[resource_post] requires an async fn",
        )
        .to_compile_error()
        .into();
    }

    let ferro = ferro();
    let fn_vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    let resource_ty = &attrs.resource_ty;
    let redirect_to_lit = &attrs.redirect_to;

    let (tenant_pat, tenant_ty, resource_pat, resource_ty_param, resource_name) =
        match extract_inner_params(&input_fn) {
            Ok(p) => p,
            Err(e) => return e.to_compile_error().into(),
        };

    // Validate placeholders
    if let Some(ref url) = attrs.form_url {
        if let Err(msg) = validate_url_placeholders(url, &resource_name, "form_url") {
            return syn::Error::new(Span::call_site(), msg)
                .to_compile_error()
                .into();
        }
    }
    if let Some(ref url) = attrs.on_miss {
        if let Err(msg) = validate_url_placeholders(url, &resource_name, "on_miss") {
            return syn::Error::new(Span::call_site(), msg)
                .to_compile_error()
                .into();
        }
    }

    let inner_fn_name = format_ident!("__{}_inner", fn_name);

    // Build tenant resolution code
    let tenant_resolution = if let Some(ref expr_str) = attrs.tenant_expr {
        // Escape hatch: bind WITHOUT an explicit type annotation so type inference yields
        // the owned TenantContext the caller's expression returns — mirroring the default arm.
        // The delegation site does `&__tenant`, which then correctly produces `&TenantContext`.
        let expr: proc_macro2::TokenStream = match expr_str.parse() {
            Ok(ts) => ts,
            Err(_) => {
                return syn::Error::new(
                    Span::call_site(),
                    "#[resource_post]: `tenant` expression failed to parse",
                )
                .to_compile_error()
                .into();
            }
        };
        quote! {
            let __tenant = { #expr };
        }
    } else {
        quote! {
            let __tenant: #ferro::TenantContext = ::ferro::current_tenant()
                .ok_or_else(|| #ferro::HttpResponse::new().status(400).set_body("No tenant context"))?;
        }
    };

    // Build tenant-scoped lookup (load-bearing: always passes tenant.id — T-212-01)
    let lookup = if let Some(ref find_path_str) = attrs.find_fn {
        let find_path: proc_macro2::TokenStream = match find_path_str.parse() {
            Ok(ts) => ts,
            Err(_) => {
                return syn::Error::new(
                    Span::call_site(),
                    "#[resource_post]: `find` path failed to parse",
                )
                .to_compile_error()
                .into();
            }
        };
        quote! {
            let __resource_opt = #find_path(__resource_id, __tenant.id).await
                .map_err(|_| #ferro::HttpResponse::new().status(500))?;
        }
    } else {
        quote! {
            let __resource_opt = <#resource_ty as #ferro::TenantScoped>::find_for_tenant(__resource_id, __tenant.id).await
                .map_err(|_| #ferro::HttpResponse::new().status(500))?;
        }
    };

    // Build POST miss arm (D-05).
    // The outer wrapper returns Response = Result<HttpResponse, HttpResponse>, so the miss arm
    // must return Err(HttpResponse). ActionError is only valid inside an ActionResult body;
    // at the prelude level we emit an HttpResponse directly (same shape as resource_get).
    let miss_arm = if let Some(ref url) = attrs.on_miss {
        let (fmt, args) = build_url_format(url, &resource_name);
        if args.is_empty() {
            quote! {
                None => {
                    return Err(#ferro::HttpResponse::new()
                        .status(303)
                        .header("Location", #url));
                }
            }
        } else {
            quote! {
                None => {
                    let __miss_url = format!(#fmt, #(#args),*);
                    return Err(#ferro::HttpResponse::new()
                        .status(303)
                        .header("Location", &__miss_url));
                }
            }
        }
    } else {
        // No on_miss: generic 404
        quote! {
            None => {
                return Err(#ferro::HttpResponse::new().status(404));
            }
        }
    };

    // Build form_url synthesis
    let form_url_init = if let Some(ref url) = attrs.form_url {
        let (fmt, args) = build_url_format(url, &resource_name);
        if args.is_empty() {
            quote! {
                let __form_url: &str = #url;
            }
        } else {
            quote! {
                let __form_url_owned = format!(#fmt, #(#args),*);
                let __form_url: &str = &__form_url_owned;
            }
        }
    } else {
        quote! {
            let __form_url: &str = "";
        }
    };

    // Replicate the action.rs wrapper shape exactly (D-06 / action.rs:256-278)
    let output = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name #fn_generics(__ferro_req: #ferro::Request) -> #ferro::Response {
            let mut __ferro_req = __ferro_req;
            let __ferro_params = __ferro_req.params().clone();

            // Step 1: extract resource id from the "id" path param
            let __resource_id: <#resource_ty as #ferro::TenantScoped>::Id =
                __ferro_req.param_as("id")
                    .map_err(|_| #ferro::HttpResponse::new().status(400))?;

            // Step 2: resolve tenant (D-01 / D-02)
            #tenant_resolution

            // Step 3: tenant-scoped lookup — always passes tenant.id (T-212-01)
            #lookup

            // Step 4: miss handling (D-05) — emits HttpResponse (303 redirect or 404)
            let __resource = match __resource_opt {
                Some(r) => r,
                #miss_arm
            };

            // Step 5: synthesize form_url from template (D-08)
            #form_url_init

            // Step 6: delegate to named inner fn.
            // The inner fn borrows &mut __ferro_req; the borrow ends when the inner fn
            // returns, before handle_action_result borrows __ferro_req again (Pitfall 3).
            let __action_result: #ferro::ActionResult =
                #inner_fn_name(&mut __ferro_req, &__tenant, &__resource, __form_url).await;

            // Step 7: dispatch result (mirrors action.rs:269-274)
            #ferro::http::action::handle_action_result(
                __action_result,
                #redirect_to_lit,
                concat!(module_path!(), "::", stringify!(#fn_name)),
                &mut __ferro_req,
            )
        }

        async fn #inner_fn_name #fn_generics(
            req: &mut #ferro::Request,
            #tenant_pat: #tenant_ty,
            #resource_pat: #resource_ty_param,
            __form_url: &str,
        ) -> #ferro::ActionResult {
            #fn_block
        }
    };

    output.into()
}
