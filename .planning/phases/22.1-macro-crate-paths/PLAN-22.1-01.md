# Plan 22.1-01: Add proc-macro-crate Dependency and Helper

```yaml
phase: 22.1
plan: 01
type: implementation
wave: 1
depends_on: []
files_modified:
  - ferro-macros/Cargo.toml
  - ferro-macros/src/lib.rs
  - ferro-macros/src/crate_path.rs
autonomous: true
```

## Objective

Add the `proc-macro-crate` dependency and create a helper function that dynamically resolves the ferro crate path. This enables all macros to use the actual crate name from the user's Cargo.toml instead of hardcoded `::ferro_rs::`.

## Context

Currently, all proc macros generate code with hardcoded `::ferro_rs::` paths:
```rust
quote! {
    ::ferro_rs::testing::set_current_test_name(...)
}
```

This forces users to name their dependency exactly `ferro_rs` in Cargo.toml. With `proc-macro-crate`, we can detect the actual name:
```rust
// If user has: ferro = { ... }
// We generate: ::ferro::testing::...
```

## Tasks

### 1. Add proc-macro-crate dependency

**File:** `ferro-macros/Cargo.toml`

Add to `[dependencies]`:
```toml
proc-macro-crate = "3"
```

### 2. Create crate path helper module

**File:** `ferro-macros/src/crate_path.rs` (new file)

```rust
//! Dynamic crate path resolution for proc macros
//!
//! Resolves the actual ferro crate name from user's Cargo.toml,
//! allowing `ferro = ...` instead of requiring `ferro_rs = ...`.

use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

/// Returns a TokenStream for the ferro crate path.
///
/// Attempts to find "ferro" first (the published name), then falls back
/// to "ferro_rs" for backwards compatibility, then defaults to "ferro_rs".
///
/// # Example
///
/// ```ignore
/// let ferro = ferro_crate();
/// quote! { #ferro::Response }
/// // Generates: ::ferro::Response (or ::my_ferro::Response if renamed)
/// ```
pub fn ferro_crate() -> TokenStream {
    // Try "ferro" first (the crates.io published name)
    if let Ok(found) = crate_name("ferro") {
        return match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                quote!(::#ident)
            }
        };
    }

    // Fall back to "ferro_rs" for backwards compatibility
    if let Ok(found) = crate_name("ferro_rs") {
        return match found {
            FoundCrate::Itself => quote!(crate),
            FoundCrate::Name(name) => {
                let ident = proc_macro2::Ident::new(&name, proc_macro2::Span::call_site());
                quote!(::#ident)
            }
        };
    }

    // Default fallback
    quote!(::ferro_rs)
}
```

### 3. Export the helper module

**File:** `ferro-macros/src/lib.rs`

Add module declaration near other module declarations:
```rust
mod crate_path;
```

Make it available internally:
```rust
pub(crate) use crate_path::ferro_crate;
```

## Verification

```bash
# Build the macro crate to verify syntax
cd /Users/alberto/repositories/albertogferrario/ferro
cargo build -p ferro-macros

# Verify the helper function is accessible internally
cargo check -p ferro-macros
```

## Success Criteria

- [ ] `proc-macro-crate = "3"` added to ferro-macros/Cargo.toml
- [ ] `crate_path.rs` created with `ferro_crate()` function
- [ ] Module exported in lib.rs
- [ ] `cargo build -p ferro-macros` succeeds
