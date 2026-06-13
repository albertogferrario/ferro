//! `#[resource_get]` attribute macro implementation.
//!
//! Folds the recurring tenant-scoped GET handler prelude into a single attribute:
//!
//! 1. Extract the resource id from the `"id"` path param as `<R as TenantScoped>::Id`.
//! 2. Resolve the tenant via `::ferro::current_tenant()` (or a caller-supplied expr).
//! 3. Look up the resource via `<R as TenantScoped>::find_for_tenant(id, tenant.id)`.
//! 4. On miss: redirect to `on_miss` (if given) or return a 404.
//! 5. Delegate to the named inner fn `__<name>_inner` with real typed params.
//!
//! # Generated shape
//!
//! ```ignore
//! pub async fn edit(__ferro_req: ::ferro::Request) -> ::ferro::Response {
//!     let mut __ferro_req = __ferro_req;
//!     let __ferro_params = __ferro_req.params().clone();
//!     // id extraction, tenant resolution, lookup, miss arm ...
//!     __edit_inner(&mut __ferro_req, &__tenant, &__resource).await
//! }
//!
//! async fn __edit_inner(
//!     req: &mut ::ferro::Request,
//!     tenant: &::ferro::TenantContext,
//!     model: &Model,
//! ) -> ::ferro::Response { /* user body */ }
//! ```
//!
//! # Security
//!
//! The generated lookup always calls `find_for_tenant(__resource_id, __tenant.id)` —
//! there is no code path that emits an un-scoped `find_by_id`. T-212-01 mitigation.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Expr, FnArg, ItemFn, Lit, Meta, Pat, Type};

use crate::utils::ferro;

/// Parsed attributes from `#[resource_get(ResourceType, on_miss = "...", tenant = "...", find = "...")]`.
struct ResourceGetAttrs {
    /// The resource type token stream (first positional arg, e.g. `Customer`).
    resource_ty: proc_macro2::TokenStream,
    /// URL to redirect to on lookup miss. If `None`, emits a 404.
    on_miss: Option<String>,
    /// Optional escape-hatch expression for tenant resolution (D-02).
    tenant_expr: Option<String>,
    /// Optional override for the lookup function (D-04).
    find_fn: Option<String>,
}

/// Parse `#[resource_get(...)]` attribute arguments.
fn parse_resource_get_attrs(attr: TokenStream) -> Result<ResourceGetAttrs, syn::Error> {
    let mut resource_ty: Option<proc_macro2::TokenStream> = None;
    let mut on_miss: Option<String> = None;
    let mut tenant_expr: Option<String> = None;
    let mut find_fn: Option<String> = None;

    let parser = syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated;
    let metas = syn::parse::Parser::parse(parser, attr).map_err(|e| {
        syn::Error::new(
            e.span(),
            format!("#[resource_get]: invalid attribute syntax: {e}"),
        )
    })?;

    for meta in metas {
        match meta {
            // First positional arg: the resource type (e.g. `Customer`)
            Meta::Path(p) => {
                if resource_ty.is_none() {
                    resource_ty = Some(quote! { #p });
                } else {
                    return Err(syn::Error::new_spanned(
                        p,
                        "#[resource_get]: unexpected extra positional type argument",
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
                    "on_miss" => {
                        if let Expr::Lit(expr_lit) = &nv.value {
                            if let Lit::Str(lit_str) = &expr_lit.lit {
                                on_miss = Some(lit_str.value());
                                continue;
                            }
                        }
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "#[resource_get]: `on_miss` must be a string literal",
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
                            "#[resource_get]: `tenant` must be a string literal expression",
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
                            "#[resource_get]: `find` must be a string literal path",
                        ));
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            nv.path,
                            format!(
                                "#[resource_get]: unknown attribute `{other}` — supported keys: `on_miss`, `tenant`, `find`"
                            ),
                        ));
                    }
                }
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "#[resource_get]: only a positional Type or `key = \"value\"` attributes are supported",
                ));
            }
        }
    }

    let resource_ty = resource_ty.ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            "#[resource_get]: a resource type is required as the first argument, e.g. #[resource_get(Customer, ...)]",
        )
    })?;

    Ok(ResourceGetAttrs {
        resource_ty,
        on_miss,
        tenant_expr,
        find_fn,
    })
}

/// Validate `{placeholder}` tokens in a URL string.
///
/// Recognized names are `"id"` and the resource param name. Any unknown
/// placeholder emits `compile_error!`.
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
                        "#[resource_get]: unknown path param `{{{name}}}` in `{context}` — \
                         declared params are: id, {resource_param_name}"
                    ));
                }
                i = start + end_off + 1;
                continue;
            }
        }
        i += 1;
    }
    Ok(())
}

/// Build a `format!` call that substitutes `{id}` / `{resource}` in a URL template.
///
/// Returns `(format_string, args)` where the combined expression is
/// `format!(format_string, args...)`.
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
type InnerParams = (Box<Pat>, Box<Type>, Box<Pat>, Box<Type>, String);

/// Extract the resource param name and type from the user's fn signature.
///
/// The function signature is expected to have params in the order:
/// `req: &mut Request, tenant: &TenantContext, resource: &ResourceType`
///
/// Returns `(tenant_pat, tenant_ty, resource_pat, resource_ty, resource_name)`.
fn extract_inner_params(input_fn: &ItemFn) -> Result<InnerParams, syn::Error> {
    let params: Vec<_> = input_fn.sig.inputs.iter().collect();

    if params.len() < 3 {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "#[resource_get]: function must have at least 3 params: `req`, `tenant`, and the resource",
        ));
    }

    // param[1] = tenant
    let tenant_param = match params[1] {
        FnArg::Typed(pt) => pt,
        FnArg::Receiver(r) => {
            return Err(syn::Error::new_spanned(
                r,
                "#[resource_get]: does not support `self` receiver",
            ))
        }
    };

    // param[2] = resource
    let resource_param = match params[2] {
        FnArg::Typed(pt) => pt,
        FnArg::Receiver(r) => {
            return Err(syn::Error::new_spanned(
                r,
                "#[resource_get]: does not support `self` receiver",
            ))
        }
    };

    // Extract the inner type from `&TenantContext` → `TenantContext`
    let tenant_ty = tenant_param.ty.clone();
    let tenant_pat = tenant_param.pat.clone();

    let resource_ty = resource_param.ty.clone();
    let resource_pat = resource_param.pat.clone();

    // Extract resource param name string for placeholder validation
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

/// Implementation of the `#[resource_get]` attribute macro.
pub fn resource_get_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = match parse_resource_get_attrs(attr) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error().into(),
    };

    let input_fn = parse_macro_input!(input as ItemFn);

    // Must be async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input_fn.sig.fn_token,
            "#[resource_get] requires an async fn",
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

    // Extract inner params from signature
    let (tenant_pat, tenant_ty, resource_pat, resource_ty_param, resource_name) =
        match extract_inner_params(&input_fn) {
            Ok(p) => p,
            Err(e) => return e.to_compile_error().into(),
        };

    // Validate placeholders in on_miss URL
    if let Some(ref url) = attrs.on_miss {
        if let Err(msg) = validate_url_placeholders(url, &resource_name, "on_miss") {
            return syn::Error::new(Span::call_site(), msg)
                .to_compile_error()
                .into();
        }
    }

    // Build the inner fn name: __edit_inner
    let inner_fn_name = format_ident!("__{}_inner", fn_name);

    // Build tenant resolution code
    let tenant_resolution = if let Some(ref expr_str) = attrs.tenant_expr {
        // Escape hatch: emit the expression verbatim in a block
        let expr: proc_macro2::TokenStream = expr_str.parse().unwrap_or_else(|_| {
            quote! { compile_error!("#[resource_get]: `tenant` expression failed to parse") }
        });
        quote! {
            let __tenant: #tenant_ty = { #expr };
        }
    } else {
        // Default: current_tenant() (D-01)
        quote! {
            let __tenant: #ferro::TenantContext = ::ferro::current_tenant()
                .ok_or_else(|| #ferro::HttpResponse::new().status(400).set_body("No tenant context"))?;
        }
    };

    // Build tenant-scoped lookup code (load-bearing: always passes tenant.id)
    let lookup = if let Some(ref find_path_str) = attrs.find_fn {
        let find_path: proc_macro2::TokenStream = find_path_str.parse().unwrap_or_else(|_| {
            quote! { compile_error!("#[resource_get]: `find` path failed to parse") }
        });
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

    // Build miss arm
    let miss_arm = if let Some(ref url) = attrs.on_miss {
        let (fmt, args) = build_url_format(url, &resource_name);
        if args.is_empty() {
            // Static URL, no interpolation needed
            quote! {
                None => {
                    return Err(#ferro::HttpResponse::new()
                        .status(302)
                        .header("Location", #url));
                }
            }
        } else {
            quote! {
                None => {
                    let __miss_url = format!(#fmt, #(#args),*);
                    return Err(#ferro::HttpResponse::new()
                        .status(302)
                        .header("Location", &__miss_url));
                }
            }
        }
    } else {
        // No on_miss: generic 404 (D-05)
        quote! {
            None => {
                return Err(#ferro::HttpResponse::new().status(404));
            }
        }
    };

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

            // Step 4: miss handling (D-05)
            let __resource = match __resource_opt {
                Some(r) => r,
                #miss_arm
            };

            // Step 5: delegate to named inner fn with typed params (D-09 / CRUD-05)
            #inner_fn_name(&mut __ferro_req, &__tenant, &__resource).await
        }

        #(#fn_attrs)*
        async fn #inner_fn_name #fn_generics(
            req: &mut #ferro::Request,
            #tenant_pat: #tenant_ty,
            #resource_pat: #resource_ty_param,
        ) -> #ferro::Response {
            #fn_block
        }
    };

    output.into()
}
