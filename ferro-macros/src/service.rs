//! Service trait macro for the Ferro framework
//!
//! Provides the `#[service]` attribute macro that:
//! 1. Adds `Send + Sync + 'static` bounds to trait definitions
//! 2. Optionally auto-registers a concrete implementation with the container
//! 3. Optionally generates a `fake()` method for testing

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, ItemTrait, Path, Token};

/// Parsed arguments from the service attribute
struct ServiceArgs {
    impl_type: Option<Path>,
    fake_type: Option<Path>,
}

impl Parse for ServiceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut impl_type = None;
        let mut fake_type = None;

        if input.is_empty() {
            return Ok(ServiceArgs {
                impl_type: None,
                fake_type: None,
            });
        }

        // Check if this is named parameters (contains '=') or positional (old syntax)
        let fork = input.fork();
        let is_named = if fork.parse::<Ident>().is_ok() {
            fork.peek(Token![=])
        } else {
            false
        };

        if is_named {
            // Parse named parameters: impl = Type, fake = Type
            while !input.is_empty() {
                let name: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let path: Path = input.parse()?;

                match name.to_string().as_str() {
                    "impl" => impl_type = Some(path),
                    "fake" => fake_type = Some(path),
                    _ => {
                        return Err(syn::Error::new(
                            name.span(),
                            format!("unknown parameter '{name}', expected 'impl' or 'fake'"),
                        ))
                    }
                }

                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }
        } else {
            // Backwards compatible: positional argument is the impl type
            impl_type = Some(input.parse()?);
        }

        Ok(ServiceArgs {
            impl_type,
            fake_type,
        })
    }
}

/// Implements the `#[service]` attribute macro
///
/// This macro transforms a trait definition to add `Send + Sync + 'static` bounds,
/// making it suitable for use with the App container.
///
/// # Without arguments (just adds bounds)
///
/// ```rust,ignore
/// #[service]
/// pub trait HttpClient {
///     async fn get(&self, url: &str) -> Result<String, Error>;
/// }
/// ```
///
/// # With impl type (auto-registration, backwards compatible)
///
/// ```rust,ignore
/// #[service(RedisCache)]  // or #[service(impl = RedisCache)]
/// pub trait CacheStore {
///     fn get(&self, key: &str) -> Option<String>;
/// }
/// ```
///
/// # With fake type (generates fake() method for testing)
///
/// ```rust,ignore
/// #[service(impl = RealCache, fake = FakeCache)]
/// pub trait CacheStore {
///     fn get(&self, key: &str) -> Option<String>;
/// }
///
/// // In tests:
/// let _guard = <dyn CacheStore>::fake();  // Binds FakeCache, returns TestContainerGuard
/// ```
///
/// # Attribute ordering with `#[offload]`
///
/// `#[service]` must be the **outermost** attribute — listed first in source order,
/// above `#[async_trait]` — so it parses raw `async fn` signatures for `#[offload]`
/// derivation. If `#[async_trait]` is listed first, `#[service]` sees the
/// desugared boxed-future signature instead of the original `async fn` form.
///
/// Correct order:
/// ```rust,ignore
/// #[service(impl = ReportBuilder)]
/// #[async_trait]
/// pub trait Reports {
///     #[offload]
///     async fn build_monthly(&self, month: Month);
/// }
/// ```
pub fn service_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ServiceArgs);
    let mut item_trait = parse_macro_input!(input as ItemTrait);

    let ferro = quote!(::ferro);

    // Add Send + Sync + 'static to the trait's supertraits
    let send_bound: syn::TypeParamBound = syn::parse_quote!(Send);
    let sync_bound: syn::TypeParamBound = syn::parse_quote!(Sync);
    let static_bound: syn::TypeParamBound = syn::parse_quote!('static);

    // Check if bounds already exist to avoid duplicates
    let has_send = item_trait.supertraits.iter().any(|bound| {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            trait_bound
                .path
                .segments
                .last()
                .map(|s| s.ident == "Send")
                .unwrap_or(false)
        } else {
            false
        }
    });

    let has_sync = item_trait.supertraits.iter().any(|bound| {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            trait_bound
                .path
                .segments
                .last()
                .map(|s| s.ident == "Sync")
                .unwrap_or(false)
        } else {
            false
        }
    });

    let has_static = item_trait
        .supertraits
        .iter()
        .any(|bound| matches!(bound, syn::TypeParamBound::Lifetime(lt) if lt.ident == "static"));

    // Add missing bounds
    if !has_send {
        item_trait.supertraits.push(send_bound);
    }
    if !has_sync {
        item_trait.supertraits.push(sync_bound);
    }
    if !has_static {
        item_trait.supertraits.push(static_bound);
    }

    // Clone the trait ident before the mutable borrow of item_trait.items below.
    let trait_ident = item_trait.ident.clone();
    let trait_name = &item_trait.ident;
    let trait_name_str = trait_name.to_string();

    // Collect #[offload] methods and strip the attribute so rustc never sees it (RQ-1).
    // The strip MUST happen before #item_trait is re-emitted (Pitfall 1).
    let mut offload_infos: Vec<crate::offload::OffloadMethodInfo> = Vec::new();
    for item in &mut item_trait.items {
        if let syn::TraitItem::Fn(method) = item {
            if let Some(pos) = method
                .attrs
                .iter()
                .position(|a| a.path().is_ident("offload"))
            {
                method.attrs.remove(pos);
                match crate::offload::collect_info(&trait_ident, method) {
                    Ok(info) => offload_infos.push(info),
                    Err(e) => return e.to_compile_error().into(),
                }
            }
        }
    }

    // Generate impl registration if impl_type is specified
    let impl_registration = args.impl_type.as_ref().map(|concrete_type| {
        quote! {
            // Auto-register this service binding at startup
            #ferro::inventory::submit! {
                #ferro::container::provider::ServiceBindingEntry {
                    register: || {
                        #ferro::App::bind::<dyn #trait_name>(
                            ::std::sync::Arc::new(<#concrete_type as ::std::default::Default>::default())
                        );
                    },
                    name: #trait_name_str,
                }
            }
        }
    });

    // Generate fake() method if fake_type is specified
    let fake_impl = args.fake_type.as_ref().map(|fake_type| {
        quote! {
            impl dyn #trait_name {
                /// Create a test container with the fake implementation bound.
                ///
                /// Returns a guard that clears the test container when dropped.
                ///
                /// # Example
                /// ```rust,ignore
                /// #[test]
                /// fn test_something() {
                ///     let _guard = <dyn MyService>::fake();
                ///     // App::make::<dyn MyService>() now returns the fake
                /// }
                /// ```
                pub fn fake() -> #ferro::container::testing::TestContainerGuard {
                    let guard = #ferro::container::testing::TestContainer::fake();
                    #ferro::container::testing::TestContainer::bind::<dyn #trait_name>(
                        ::std::sync::Arc::new(<#fake_type as ::std::default::Default>::default())
                    );
                    guard
                }
            }
        }
    });

    let offload_items = offload_infos
        .iter()
        .map(|info| crate::offload::emit_job_items(&trait_ident, info));

    let expanded = quote! {
        #item_trait
        #impl_registration
        #fake_impl
        #( #offload_items )*
    };

    TokenStream::from(expanded)
}
