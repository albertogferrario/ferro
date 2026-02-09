//! Derive macro for reducing SeaORM model boilerplate
//!
//! Generates builder, update builder, and trait implementations for Ferro models.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

/// Returns the token stream for the ferro crate path: `::ferro`
fn ferro() -> TokenStream2 {
    quote!(::ferro)
}

/// Generate model boilerplate from a SeaORM Model struct
pub fn ferro_model_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ferro = ferro();

    let name = &input.ident;

    // Extract fields from struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(&input, "FerroModel only supports named structs")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "FerroModel only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Generate builder name
    let builder_name = format_ident!("{}Builder", name);
    let update_builder_name = format_ident!("{}UpdateBuilder", name);

    // Generate field data for various uses
    let mut builder_fields = Vec::new();
    let mut builder_default_fields = Vec::new();
    let mut builder_setters = Vec::new();
    let mut build_fields = Vec::new();

    // UpdateBuilder data
    let mut update_builder_fields = Vec::new();
    let mut update_builder_defaults = Vec::new();
    let mut update_builder_setters = Vec::new();
    let mut update_save_fields = Vec::new();

    // Track id type and updated_at info
    let mut id_type: Option<Type> = None;
    let mut has_updated_at = false;
    let mut updated_at_is_string = false;

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let is_id = field_name == "id";
        let is_timestamps = field_name == "created_at" || field_name == "updated_at";
        let is_option = is_option_type(field_ty);

        // Track id type
        if is_id {
            id_type = Some(field_ty.clone());
        }

        // Track updated_at
        if field_name == "updated_at" {
            has_updated_at = true;
            updated_at_is_string = is_string_like(field_ty);
        }

        // === Create Builder fields ===
        if !is_id {
            builder_fields.push(quote! {
                #field_name: Option<#field_ty>
            });

            builder_default_fields.push(quote! {
                #field_name: None
            });
        }

        // Builder setter method (for create builder)
        if !is_id && !is_timestamps {
            let setter_name = format_ident!("set_{}", field_name);

            if is_option {
                let inner_ty = get_option_inner_type(field_ty);
                if is_string_like(&inner_ty) {
                    builder_setters.push(quote! {
                        /// Set the #field_name field
                        pub fn #setter_name(mut self, value: impl Into<String>) -> Self {
                            self.#field_name = Some(Some(value.into()));
                            self
                        }
                    });
                } else {
                    builder_setters.push(quote! {
                        /// Set the #field_name field
                        pub fn #setter_name(mut self, value: #inner_ty) -> Self {
                            self.#field_name = Some(Some(value));
                            self
                        }
                    });
                }
            } else if is_string_like(field_ty) {
                builder_setters.push(quote! {
                    /// Set the #field_name field
                    pub fn #setter_name(mut self, value: impl Into<String>) -> Self {
                        self.#field_name = Some(value.into());
                        self
                    }
                });
            } else {
                builder_setters.push(quote! {
                    /// Set the #field_name field
                    pub fn #setter_name(mut self, value: #field_ty) -> Self {
                        self.#field_name = Some(value);
                        self
                    }
                });
            }
        }

        // Build field (for converting create builder to ActiveModel)
        if is_id {
            build_fields.push(quote! {
                #field_name: sea_orm::ActiveValue::NotSet
            });
        } else {
            build_fields.push(quote! {
                #field_name: self.#field_name.map(sea_orm::Set).unwrap_or(sea_orm::ActiveValue::NotSet)
            });
        }

        // === UpdateBuilder fields ===
        if !is_id && !is_timestamps {
            if is_option {
                // Option<T> -> Option<Option<T>> for tracking: None=not modified, Some(None)=clear, Some(Some(v))=set
                update_builder_fields.push(quote! {
                    #field_name: Option<#field_ty>
                });
                update_builder_defaults.push(quote! {
                    #field_name: None
                });

                // set_field: accepts inner type
                let setter_name = format_ident!("set_{}", field_name);
                let clear_name = format_ident!("clear_{}", field_name);
                let inner_ty = get_option_inner_type(field_ty);

                if is_string_like(&inner_ty) {
                    update_builder_setters.push(quote! {
                        /// Set the #field_name field to a value
                        pub fn #setter_name(mut self, value: impl Into<String>) -> Self {
                            self.#field_name = Some(Some(value.into()));
                            self
                        }

                        /// Clear the #field_name field (set to NULL)
                        pub fn #clear_name(mut self) -> Self {
                            self.#field_name = Some(None);
                            self
                        }
                    });
                } else {
                    update_builder_setters.push(quote! {
                        /// Set the #field_name field to a value
                        pub fn #setter_name(mut self, value: #inner_ty) -> Self {
                            self.#field_name = Some(Some(value));
                            self
                        }

                        /// Clear the #field_name field (set to NULL)
                        pub fn #clear_name(mut self) -> Self {
                            self.#field_name = Some(None);
                            self
                        }
                    });
                }

                // save field: Option<Option<T>> -> Set(v) or NotSet
                update_save_fields.push(quote! {
                    #field_name: match self.#field_name {
                        Some(v) => sea_orm::Set(v),
                        None => sea_orm::ActiveValue::NotSet,
                    }
                });
            } else {
                // Required field: Option<T> for tracking: None=not modified, Some(v)=set
                update_builder_fields.push(quote! {
                    #field_name: Option<#field_ty>
                });
                update_builder_defaults.push(quote! {
                    #field_name: None
                });

                let setter_name = format_ident!("set_{}", field_name);

                if is_string_like(field_ty) {
                    update_builder_setters.push(quote! {
                        /// Set the #field_name field
                        pub fn #setter_name(mut self, value: impl Into<String>) -> Self {
                            self.#field_name = Some(value.into());
                            self
                        }
                    });
                } else {
                    update_builder_setters.push(quote! {
                        /// Set the #field_name field
                        pub fn #setter_name(mut self, value: #field_ty) -> Self {
                            self.#field_name = Some(value);
                            self
                        }
                    });
                }

                // save field: Option<T> -> Set(v) or NotSet
                update_save_fields.push(quote! {
                    #field_name: self.#field_name.map(sea_orm::Set).unwrap_or(sea_orm::ActiveValue::NotSet)
                });
            }
        }
    }

    let id_ty = id_type.unwrap_or_else(|| syn::parse_str::<Type>("i32").unwrap());

    // Generate updated_at field for save method
    let updated_at_save = if has_updated_at {
        if updated_at_is_string {
            quote! { updated_at: sea_orm::Set(chrono::Utc::now().to_string()) }
        } else {
            quote! { updated_at: sea_orm::Set(chrono::Utc::now()) }
        }
    } else {
        quote! {}
    };

    // Generate created_at and updated_at for save ActiveModel
    // We need these fields in the ActiveModel even if they're timestamps
    let mut update_timestamp_fields = Vec::new();
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        if field_name == "created_at" {
            update_timestamp_fields.push(quote! {
                created_at: sea_orm::ActiveValue::NotSet
            });
        }
    }

    let expanded = quote! {
        impl #name {
            /// Start a new query builder
            pub fn query() -> #ferro::database::QueryBuilder<Entity> {
                #ferro::database::QueryBuilder::new()
            }

            /// Create a new record builder
            pub fn create() -> #builder_name {
                #builder_name::default()
            }

            /// Start an update builder for selective field updates
            pub fn update(self) -> #update_builder_name {
                #update_builder_name {
                    id: self.id,
                    #(#update_builder_defaults),*
                }
            }

            /// Delete this record from the database
            pub async fn delete(self) -> Result<u64, #ferro::FrameworkError> {
                <Entity as #ferro::database::ModelMut>::delete_by_pk(self.id).await
            }
        }

        /// Builder for creating new #name records
        #[derive(Default)]
        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            #(#builder_setters)*

            /// Insert the record into the database
            pub async fn insert(self) -> Result<#name, #ferro::FrameworkError> {
                let active = self.build();
                <Entity as #ferro::database::ModelMut>::insert_one(active).await
            }

            fn build(self) -> ActiveModel {
                ActiveModel {
                    #(#build_fields),*
                }
            }
        }

        /// Builder for updating existing #name records with selective field tracking
        pub struct #update_builder_name {
            id: #id_ty,
            #(#update_builder_fields),*
        }

        impl #update_builder_name {
            #(#update_builder_setters)*

            /// Save only the modified fields to the database
            pub async fn save(self) -> Result<#name, #ferro::FrameworkError> {
                let active = ActiveModel {
                    id: sea_orm::Unchanged(self.id),
                    #(#update_save_fields,)*
                    #(#update_timestamp_fields,)*
                    #updated_at_save
                };
                <Entity as #ferro::database::ModelMut>::update_one(active).await
            }
        }

        // Implement SeaORM ActiveModelBehavior for lifecycle hooks
        impl sea_orm::ActiveModelBehavior for ActiveModel {}

        // Implement Ferro's Model trait for convenient read operations
        impl #ferro::database::Model for Entity {}

        // Implement Ferro's ModelMut trait for write operations
        impl #ferro::database::ModelMut for Entity {}
    };

    TokenStream::from(expanded)
}

/// Check if a type is Option<T>
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Get the inner type of Option<T>
fn get_option_inner_type(ty: &Type) -> Type {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                        return inner.clone();
                    }
                }
            }
        }
    }
    ty.clone()
}

/// Check if type is String or string-like
fn is_string_like(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }
    false
}
