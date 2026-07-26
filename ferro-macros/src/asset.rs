//! The `asset!("path")` function-like proc-macro.
//!
//! Collapses the boot-time `Bundle::new(name, bytes).content_type(ct).hashed_url()`
//! chain into a single expression at the use site. Embeds the file via
//! `include_bytes!` (call-site-source-relative), lazily registers a content-hashed
//! `ferro::bundle::Bundle` once per call site via a `static OnceLock<String>`, and
//! returns the hashed URL as `&'static str`.

use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use syn::{parse_macro_input, LitStr};

use crate::utils::ferro;

/// Implementation of the `asset!("path")` function-like proc-macro.
///
/// Called from the proc-macro entry point in `lib.rs`.
pub fn asset_impl(input: TokenStream) -> TokenStream {
    let path_lit = parse_macro_input!(input as LitStr);
    let path_str = path_lit.value();
    let ferro = ferro();

    // Extension for MIME inference (D-05). Lowercased so ".JS" → "js".
    let ext = Path::new(&path_str)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();

    // Sanitize the path → bundle name (D-04): keep [a-z0-9-], map the rest to '_'.
    // Uppercase ASCII letters are lowercased so the name is strictly [a-z0-9_-].
    let bundle_name: String = path_str
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    let output = quote! {
        {
            static __FERRO_ASSET_URL: ::std::sync::OnceLock<::std::string::String>
                = ::std::sync::OnceLock::new();
            __FERRO_ASSET_URL
                .get_or_init(|| {
                    static __FERRO_ASSET_BYTES: &[u8] = include_bytes!(#path_lit);
                    #ferro::bundle::Bundle::new(#bundle_name, __FERRO_ASSET_BYTES)
                        .content_type(#ferro::bundle::mime_from_ext(#ext))
                        .hashed_url()
                })
                .as_str()
        }
    };
    output.into()
}
