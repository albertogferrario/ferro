//! `#[memoize]` attribute macro implementation.
//!
//! Rewrites an `async fn` (free or impl method) into a wrapper that looks up
//! the ambient per-request [`::ferro::memo::MemoStore`] via
//! [`::ferro::memo::current_memo_store()`].  On a cache miss the original body
//! is executed once; its result is stored in a
//! [`futures::future::Shared`] slot so concurrent callers coalesce onto one
//! computation.  Outside a request context (no `MEMO_STORE` task-local scope)
//! the body runs un-memoized — no panic (D-02).
//!
//! # Key design points
//!
//! - A unique zero-sized marker type (`__FerroMemoMarker{n}`) is emitted at
//!   each macro expansion site.  Its [`std::any::TypeId`] forms the callsite
//!   component of [`::ferro::memo::MemoKey`], so two distinct call sites can
//!   never share a cache slot even when their argument hashes collide (D-03).
//! - `&self` / `&mut self` receivers are excluded from the argument hash
//!   because `#[service]` singletons are stateless (D-03).
//! - All value-argument types must implement [`std::hash::Hash`]; a `where`
//!   clause is emitted to enforce this at the call site.
//! - The return type must implement `Clone + Send + Sync + 'static`; a
//!   `where` clause is emitted to enforce this at the call site.
//! - Only `async fn` is accepted.  A clear `compile_error!` is emitted for
//!   synchronous functions (Pitfall 4).
//! - Only `Pat::Ident` argument patterns are supported in v17.0; destructuring
//!   patterns emit a `compile_error!` naming the offending argument.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, ReturnType};

use crate::utils::ferro;

/// Global expansion counter — each macro invocation gets a unique integer
/// used to mint the per-callsite zero-sized marker type name.
static MEMO_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Implementation of the `#[memoize]` attribute macro.
///
/// Called from the proc-macro entry point in `lib.rs`.
pub fn memoize_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Unused attribute arguments are silently ignored for forward-compat.
    let _ = attr;

    let input_fn = parse_macro_input!(input as ItemFn);
    let ferro = ferro();

    // ── async-only guard ──────────────────────────────────────────────────────
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[memoize] can only be applied to `async fn`",
        )
        .to_compile_error()
        .into();
    }

    // ── Unique per-expansion marker ───────────────────────────────────────────
    let n = MEMO_COUNTER.fetch_add(1, Ordering::Relaxed);
    let marker_name = format_ident!("__FerroMemoMarker{n}");

    // ── Function components ───────────────────────────────────────────────────
    let fn_vis = &input_fn.vis;
    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_block = &input_fn.block;
    let fn_attrs = &input_fn.attrs;
    let fn_output = &input_fn.sig.output;

    // Collect ALL inputs (for the re-emitted signature).
    let all_inputs: Vec<_> = input_fn.sig.inputs.iter().collect();

    // ── Split receiver from value args ────────────────────────────────────────
    // Receivers (&self / &mut self) are excluded from the key (D-03).
    let value_inputs: Vec<_> = input_fn
        .sig
        .inputs
        .iter()
        .filter(|a| !matches!(a, FnArg::Receiver(_)))
        .collect();

    // ── Extract binding idents for the args tuple ─────────────────────────────
    // v17.0 restriction: only Pat::Ident patterns are supported.
    let mut value_arg_names: Vec<proc_macro2::Ident> = Vec::new();
    let mut value_arg_types: Vec<&syn::Type> = Vec::new();

    for arg in &value_inputs {
        match arg {
            FnArg::Typed(pat_type) => {
                let ty = &*pat_type.ty;
                match &*pat_type.pat {
                    Pat::Ident(pat_ident) => {
                        value_arg_names.push(pat_ident.ident.clone());
                        value_arg_types.push(ty);
                    }
                    other => {
                        return syn::Error::new_spanned(
                            other,
                            "#[memoize] arguments must be simple identifiers in v17.0; \
                             got a destructuring pattern",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
            FnArg::Receiver(_) => {
                // Already filtered out above — unreachable.
            }
        }
    }

    // ── Return type ───────────────────────────────────────────────────────────
    let return_ty: proc_macro2::TokenStream = match fn_output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    // ── Generate wrapper ──────────────────────────────────────────────────────
    //
    // The args tuple uses a trailing comma so that both the zero-arg case
    // `&( , )` and the one-arg case `&(x, )` produce a true tuple rather than
    // parenthesised expressions, ensuring uniform Hash semantics.
    let output = quote! {
        #(#fn_attrs)*
        #fn_vis async fn #fn_name #fn_generics(#(#all_inputs),*) #fn_output
        where
            #( #value_arg_types: ::std::hash::Hash, )*
            #return_ty: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            // Zero-sized marker type unique to this macro expansion site.
            struct #marker_name;

            let __ferro_memo_key = #ferro::memo::MemoKey::new::<#marker_name, _>(
                &( #( #value_arg_names, )* ),
            );

            if let ::std::option::Option::Some(__ferro_store) =
                #ferro::memo::current_memo_store()
            {
                let __ferro_slot = __ferro_store.get_or_insert(
                    __ferro_memo_key,
                    move || {
                        ::std::boxed::Box::pin(async move {
                            let __ferro_result: #return_ty = { #fn_block };
                            ::std::sync::Arc::new(__ferro_result)
                                as ::std::sync::Arc<
                                    dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync,
                                >
                        })
                    },
                );
                let __ferro_arc = __ferro_slot.await;
                return ::std::clone::Clone::clone(
                    __ferro_arc
                        .downcast_ref::<#return_ty>()
                        .expect("MemoStore type invariant violated"),
                );
            }

            // D-02: no store in scope — run body un-memoized, no panic.
            { #fn_block }
        }
    };

    output.into()
}
