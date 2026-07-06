//! Derive macro for generating Resource trait implementations from annotated structs.
//!
//! Provides `#[derive(ApiResource)]` with field-level `#[resource(...)]` attributes
//! for controlling JSON output shape and optional `From<Model>` generation.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit, Meta, Path};

/// Parse struct-level `#[resource(model = "...")]` attribute.
/// Returns the model path if present.
fn parse_model_attr(attrs: &[syn::Attribute]) -> Result<Option<Path>, syn::Error> {
    let mut model_path: Option<Path> = None;

    for attr in attrs {
        if !attr.path().is_ident("resource") {
            continue;
        }

        let nested = attr.parse_args_with(
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
        )?;

        for meta in &nested {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("model") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        model_path = Some(lit_str.parse()?);
                    } else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "expected string literal for `model`",
                        ));
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unknown struct-level resource attribute; expected `model = \"...\"`",
                    ));
                }
            }
        }
    }

    Ok(model_path)
}

/// Field-level attribute config.
struct FieldConfig {
    skip: bool,
    rename: Option<String>,
}

/// Parse field-level `#[resource(...)]` attributes.
fn parse_field_attrs(attrs: &[syn::Attribute]) -> Result<FieldConfig, syn::Error> {
    let mut config = FieldConfig {
        skip: false,
        rename: None,
    };

    for attr in attrs {
        if !attr.path().is_ident("resource") {
            continue;
        }

        let nested = attr.parse_args_with(
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
        )?;

        for meta in &nested {
            match meta {
                Meta::Path(p) if p.is_ident("skip") => {
                    config.skip = true;
                }
                Meta::NameValue(nv) if nv.path.is_ident("rename") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        config.rename = Some(lit_str.value());
                    } else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "expected string literal for `rename`",
                        ));
                    }
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unknown field-level resource attribute; expected `skip` or `rename = \"...\"`",
                    ));
                }
            }
        }
    }

    Ok(config)
}

/// Entry point for the ApiResource derive macro.
pub fn api_resource_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_api_resource(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_api_resource(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let name = &input.ident;

    // Only named structs are supported
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "ApiResource only supports structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "ApiResource can only be derived for structs",
            ));
        }
    };

    let model_path = parse_model_attr(&input.attrs)?;

    // Build ResourceMap field chain and From impl field mappings
    let mut resource_fields = Vec::new();
    let mut from_fields = Vec::new();

    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let config = parse_field_attrs(&field.attrs)?;

        // From<Model> maps ALL fields regardless of skip
        from_fields.push(quote! {
            #field_ident: model.#field_ident
        });

        if config.skip {
            continue;
        }

        let key = config.rename.unwrap_or_else(|| field_ident.to_string());

        resource_fields.push(quote! {
            .field(#key, ferro::serde_json::json!(self.#field_ident))
        });
    }

    let resource_impl = quote! {
        impl ferro::Resource for #name {
            fn to_resource(&self, _req: &ferro::Request) -> ferro::serde_json::Value {
                ferro::ResourceMap::new()
                    #(#resource_fields)*
                    .build()
            }
        }
    };

    let from_impl = if let Some(model) = model_path {
        quote! {
            impl From<#model> for #name {
                fn from(model: #model) -> Self {
                    Self {
                        #(#from_fields),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #resource_impl
        #from_impl
    })
}
