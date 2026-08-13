//! Helper module for `#[offload]` — consumed by the `#[service]` macro.
//!
//! The `#[offload]` marker on a `#[service]` trait method is an *inert helper
//! attribute*; it is stripped by `service_impl` before re-emitting the trait
//! and used here to derive:
//!
//! 1. A `pub struct <Trait><Method>Job { … }` carrying the non-`self`
//!    parameters as owned, serializable fields.
//! 2. A `#[::ferro::async_trait] impl ::ferro::queue::Job for <..>Job` whose
//!    `handle()` resolves the concrete service from the container and calls the
//!    original method body.
//! 3. An `::ferro::inventory::submit! { … }` self-registration entry so
//!    `WorkerLoop::from_registry` picks the job up automatically — no manual
//!    `Queue::register` call.
//!
//! # Attribute ordering caveat (Open Question 3)
//!
//! `#[service]` must be the **outermost** attribute (listed first in source order,
//! above `#[async_trait]`) so that it receives un-desugared `async fn` signatures.
//! If `#[async_trait]` is listed first, `#[service]` sees the desugared
//! boxed-future signature and `#[offload]` derivation will not parse correctly.
//!
//! # Dispatch key caveat (Pitfall 2)
//!
//! `WorkerLoop::register::<J>()` stores the handler keyed by
//! `std::any::type_name::<J>()`, which includes the fully-qualified module path.
//! The derived struct overrides `fn name()` to return a stable plain-name literal
//! (e.g. `"ReportsBuildMonthlyJob"`) for human-readable logging, but the DB
//! `job_type` column is populated by `type_name` at enqueue time. Moving the
//! enclosing trait to a different module changes `type_name` and silently breaks
//! dispatch for jobs already in the queue. Rename modules with care.

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{FnArg, Pat, ReturnType, TraitItemFn, Type};

/// Metadata collected from one `#[offload]`-marked trait method.
pub(crate) struct OffloadMethodInfo {
    /// The derived Job struct ident, e.g. `ReportsBuildMonthlyJob`.
    pub job_ident: proc_macro2::Ident,
    /// The original method ident, e.g. `build_monthly`.
    pub method_ident: proc_macro2::Ident,
    /// Field names for the derived struct (non-`self` param idents).
    pub field_names: Vec<proc_macro2::Ident>,
    /// Owned field types (after `owned_type` substitution).
    pub field_types: Vec<TokenStream2>,
    /// Whether the original method is `async fn`.
    pub is_async: bool,
    /// Whether the original return type is `Result<_, _>`.
    pub returns_result: bool,
}

/// Map a (possibly borrowed) parameter type to an owned, serializable type.
///
/// | Source type | Derived field type |
/// |-------------|-------------------|
/// | `&str`      | `String`           |
/// | `&[T]`      | `Vec<T>`           |
/// | `&T`        | `T`                |
/// | `&mut T`    | compile error      |
/// | `T`         | `T`                |
pub(crate) fn owned_type(ty: &Type) -> syn::Result<TokenStream2> {
    match ty {
        Type::Reference(r) => {
            if r.mutability.is_some() {
                return Err(syn::Error::new_spanned(
                    ty,
                    "#[offload] parameters may not be &mut references — \
                     Job payloads must be owned and serializable",
                ));
            }
            match r.elem.as_ref() {
                Type::Path(p) if p.path.is_ident("str") => Ok(quote! { String }),
                Type::Slice(s) => {
                    let inner = &s.elem;
                    Ok(quote! { Vec<#inner> })
                }
                other => Ok(quote! { #other }),
            }
        }
        other => Ok(quote! { #other }),
    }
}

/// Convert a snake_case identifier to PascalCase.
///
/// E.g. `build_monthly` → `BuildMonthly`.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|seg| !seg.is_empty())
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Collect derivation metadata from a `#[offload]`-marked trait method.
///
/// The caller must have already stripped the `#[offload]` attribute from
/// `method.attrs` before calling this function.
pub(crate) fn collect_info(
    trait_ident: &proc_macro2::Ident,
    method: &TraitItemFn,
) -> syn::Result<OffloadMethodInfo> {
    let method_ident = method.sig.ident.clone();

    // Build the Job ident: <TraitPascalCase><MethodPascalCase>Job.
    // The trait ident is already PascalCase; the method ident is snake_case.
    let method_pascal = to_pascal_case(&method_ident.to_string());
    let job_ident = format_ident!("{}{}Job", trait_ident, method_pascal);

    // Collect non-self parameters.
    let mut field_names: Vec<proc_macro2::Ident> = Vec::new();
    let mut field_types: Vec<TokenStream2> = Vec::new();

    for arg in method.sig.inputs.iter() {
        match arg {
            FnArg::Receiver(_) => {
                // &self — excluded from the payload (D-11).
            }
            FnArg::Typed(pat_type) => {
                match &*pat_type.pat {
                    Pat::Ident(pat_ident) => {
                        let owned = owned_type(&pat_type.ty)?;
                        field_names.push(pat_ident.ident.clone());
                        field_types.push(owned);
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            other,
                            "#[offload] parameters must be simple identifiers",
                        ));
                    }
                }
            }
        }
    }

    let is_async = method.sig.asyncness.is_some();

    // Detect `Result<_, _>` return type by matching the last path segment.
    let returns_result = match &method.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => {
            if let Type::Path(type_path) = ty.as_ref() {
                type_path
                    .path
                    .segments
                    .last()
                    .map(|seg| seg.ident == "Result")
                    .unwrap_or(false)
            } else {
                false
            }
        }
    };

    Ok(OffloadMethodInfo {
        job_ident,
        method_ident,
        field_names,
        field_types,
        is_async,
        returns_result,
    })
}

/// Emit the derived Job struct, `impl Job`, and `inventory::submit!` for one
/// offloaded method.
///
/// All emitted paths go through `::ferro::*` so the generated code resolves in
/// any crate that depends only on `ferro-rs` — direct ferro-queue paths are
/// never emitted.
pub(crate) fn emit_job_items(
    trait_ident: &proc_macro2::Ident,
    info: &OffloadMethodInfo,
) -> TokenStream2 {
    let job_ident = &info.job_ident;
    let method_ident = &info.method_ident;
    let field_names = &info.field_names;
    let field_types = &info.field_types;

    let job_ident_str = job_ident.to_string();
    let trait_ident_str = trait_ident.to_string();

    // Build the `handle()` call expression based on sync/async and Result/non-Result.
    let call_expr: TokenStream2 = match (info.is_async, info.returns_result) {
        (true, false) => quote! {
            let _ = svc.#method_ident( #( self.#field_names.clone() ),* ).await;
            Ok(())
        },
        (true, true) => quote! {
            svc.#method_ident( #( self.#field_names.clone() ),* ).await
                .map(|_| ())
                .map_err(|e| ::ferro::queue::Error::job_failed(
                    #job_ident_str,
                    format!("{e}"),
                ))
        },
        (false, false) => quote! {
            let _ = svc.#method_ident( #( self.#field_names.clone() ),* );
            Ok(())
        },
        (false, true) => quote! {
            svc.#method_ident( #( self.#field_names.clone() ),* )
                .map(|_| ())
                .map_err(|e| ::ferro::queue::Error::job_failed(
                    #job_ident_str,
                    format!("{e}"),
                ))
        },
    };

    let expect_msg = format!(
        "{trait_ident_str} is not registered in the App container. \
         Did you annotate the impl with #[service(impl = …)]?"
    );

    quote! {
        /// Derived job payload for the `#[offload]`-marked `#[method_ident]` method.
        ///
        /// # Dispatch key stability
        ///
        /// `WorkerLoop` stores the handler keyed by `std::any::type_name::<Self>()`,
        /// which includes the full module path. Moving the enclosing trait to a
        /// different module changes this key and silently breaks dispatch for jobs
        /// already in the queue. Rename modules with care.
        #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #job_ident {
            #( pub #field_names: #field_types, )*
        }

        #[::ferro::async_trait]
        impl ::ferro::queue::Job for #job_ident {
            fn name(&self) -> &'static str {
                #job_ident_str
            }

            async fn handle(&self) -> ::std::result::Result<(), ::ferro::queue::Error> {
                let svc = ::ferro::App::make::<dyn #trait_ident>()
                    .expect(#expect_msg);
                #call_expr
            }
        }

        ::ferro::inventory::submit! {
            ::ferro::queue::JobRegistrarEntry {
                register: |w: &mut ::ferro::queue::WorkerLoop| {
                    w.register::<#job_ident>();
                },
                name: #job_ident_str,
            }
        }
    }
}
