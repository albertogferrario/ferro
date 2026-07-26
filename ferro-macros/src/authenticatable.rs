//! Derive macro for the `Authenticatable` trait.
//!
//! Generates the ~17 lines of boilerplate every user model otherwise writes by
//! hand so `Auth::user_as::<T>()` works. Use on a struct with an integer `id`
//! field; override the field with `#[auth(id = "user_id")]`.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, LitStr};

pub fn derive_authenticatable_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Optional `#[auth(id = "field")]` to pick a non-`id` identifier field.
    let mut id_field: Option<String> = None;
    for attr in &input.attrs {
        if attr.path().is_ident("auth") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("id") {
                    let val: LitStr = meta.value()?.parse()?;
                    id_field = Some(val.value());
                }
                Ok(())
            });
        }
    }

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Authenticatable derive requires a struct with named fields",
                )
                .to_compile_error()
                .into()
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Authenticatable derive supports structs only")
                .to_compile_error()
                .into()
        }
    };

    let id_name = id_field.unwrap_or_else(|| "id".to_string());
    let has_field = fields
        .iter()
        .any(|f| f.ident.as_ref().is_some_and(|i| *i == id_name));
    if !has_field {
        return syn::Error::new_spanned(
            &input,
            format!(
                "Authenticatable derive: no `{id_name}` field found. Add \
                 #[auth(id = \"…\")] to name the identifier field, or implement \
                 Authenticatable by hand."
            ),
        )
        .to_compile_error()
        .into();
    }

    let id_ident = syn::Ident::new(&id_name, Span::call_site());
    let id_name_lit = LitStr::new(&id_name, Span::call_site());
    let (impl_g, ty_g, where_g) = input.generics.split_for_impl();

    quote! {
        impl #impl_g ::ferro::Authenticatable for #name #ty_g #where_g {
            fn auth_identifier(&self) -> i64 {
                self.#id_ident as i64
            }
            fn auth_identifier_name(&self) -> &'static str {
                #id_name_lit
            }
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
        }
    }
    .into()
}
