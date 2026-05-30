//! Handler attribute macro implementation
//!
//! Transforms controller functions to automatically extract typed parameters
//! from HTTP requests, including path parameters and route model binding.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn};

use crate::utils::{classify_param_type, extract_param_name, ferro, generate_extraction};

/// Implementation of the `#[handler]` attribute macro
///
/// Supports multiple parameter extraction:
///
/// - `Request` - passes through unchanged
/// - Primitives (`i32`, `String`, etc.) - extracted from path params via `FromParam`
/// - Model types (`user::Model`) - extracted via `RouteBinding` (auto 404 if not found)
/// - Other types - extracted via `FromRequest` (FormRequest validation)
///
/// # Examples
///
/// ```rust,ignore
/// // No parameters
/// #[handler]
/// pub async fn index() -> Response { ... }
///
/// // Request passthrough
/// #[handler]
/// pub async fn show(req: Request) -> Response { ... }
///
/// // Path parameter extraction
/// #[handler]
/// pub async fn show(id: i32) -> Response { ... }
///
/// // Route model binding
/// #[handler]
/// pub async fn show(user: user::Model) -> Response { ... }
///
/// // FormRequest validation
/// #[handler]
/// pub async fn store(form: CreateUserRequest) -> Response { ... }
///
/// // Mixed parameters
/// #[handler]
/// pub async fn update(user: user::Model, form: UpdateUserRequest) -> Response { ... }
/// ```
pub fn handler_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let ferro = ferro();

    let fn_vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_output = &input_fn.sig.output;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;

    let is_async = input_fn.sig.asyncness.is_some();
    let async_token = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    // Collect all parameters
    let params: Vec<_> = input_fn.sig.inputs.iter().collect();

    // Handle no parameters case
    if params.is_empty() {
        let output = quote! {
            #(#fn_attrs)*
            #fn_vis #async_token fn #fn_name #fn_generics(_: #ferro::Request) #fn_output {
                #fn_block
            }
        };
        return output.into();
    }

    // Process parameters and generate extraction code
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
                    "#[handler] does not support methods with self receiver",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    // Generate the transformed function
    let output = if has_request_param {
        // If we have a Request param, we need to handle it specially
        quote! {
            #(#fn_attrs)*
            #fn_vis #async_token fn #fn_name #fn_generics(__ferro_req: #ferro::Request) #fn_output {
                let __ferro_params = __ferro_req.params().clone();
                #(#extractions)*
                #fn_block
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #fn_vis #async_token fn #fn_name #fn_generics(__ferro_req: #ferro::Request) #fn_output {
                let __ferro_params = __ferro_req.params().clone();
                #(#extractions)*
                #fn_block
            }
        }
    };

    output.into()
}
