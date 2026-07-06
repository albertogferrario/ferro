---
phase: 151
plan: 151-02
slug: subject-trait
wave: 2
depends_on: [151-01]
files_modified:
  - ferro-wallet/src/subject.rs
  - ferro-wallet/src/lib.rs
autonomous: true
requirements: [ACC-1e, ACC-1f]
must_haves:
  truths:
    - "Downstream models can implement `WalletSubject` for their domain object and pass it to either builder"
    - "Value types serialise/derive cleanly (`Debug, Clone` on every type; closed enums implement `PartialEq, Eq`)"
    - "`RgbColor::from_hex` parses `#RRGGBB` strings deterministically"
    - "`TextColorMode::Auto` derives foreground colour from background via BT.601 luminance (D-06)"
  artifacts:
    - path: "ferro-wallet/src/subject.rs"
      provides: "WalletSubject trait + Field/Branding/PassKind/GeoPoint/RgbColor/TextColorMode/FieldAlignment + auto_foreground helper"
      contains: "pub trait WalletSubject"
      min_lines: 200
    - path: "ferro-wallet/src/lib.rs"
      provides: "Restored `pub use subject::{...}` re-export block"
      contains: "pub use subject::"
  key_links:
    - from: "ferro-wallet/src/lib.rs"
      to: "ferro-wallet/src/subject.rs"
      via: "pub use subject::{WalletSubject, …}"
      pattern: "pub use subject::"
    - from: "TextColorMode::Auto"
      to: "auto_foreground(bg: RgbColor) -> RgbColor"
      via: "BT.601 luminance threshold at 0.5 (D-06)"
      pattern: "fn auto_foreground"
---

<objective>
Land the `WalletSubject` trait and its supporting value types (`Field`, `Branding`, `PassKind`, `GeoPoint`, `RgbColor`, `TextColorMode`, `FieldAlignment`), plus the BT.601 `auto_foreground` helper and `RgbColor::from_hex` constructor. Restore the `subject::*` re-export block in `lib.rs`. This unblocks PLAN-05 (apple) and PLAN-07 (google).
</objective>

<context>
@.planning/phases/151-ferro-wallet-crate/151-CONTEXT.md
@.planning/phases/151-ferro-wallet-crate/151-PATTERNS.md
@.planning/phases/151-ferro-wallet-crate/151-RESEARCH.md
@.planning/phases/151-ferro-wallet-crate/151-VALIDATION.md
@docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md
@ferro-wallet/src/lib.rs
@ferro-wallet/src/error.rs

<interfaces>
The trait and value types are the contract every consumer (gestiscilo-it, downstream apps) implements. They are also the input shape both `ApplePassBuilder::build` (PLAN-05) and `GoogleWalletBuilder::save_jwt` (PLAN-07) accept.

From spec §3.1 — authoritative public API surface:
```rust
pub trait WalletSubject {
    fn pass_kind(&self) -> PassKind;
    fn serial(&self) -> String;
    fn primary(&self) -> Field;
    fn secondary(&self) -> Vec<Field>;
    fn auxiliary(&self) -> Vec<Field>;
    fn back(&self) -> Vec<Field>;
    fn barcode_token(&self) -> String;
    fn relevant_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    fn expires_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    fn locations(&self) -> Vec<GeoPoint>;
    fn branding(&self) -> Branding;
}

pub enum PassKind { EventTicket, Generic, Coupon }
pub enum FieldAlignment { Left, Center, Right, Natural }
pub enum TextColorMode { Auto, Light, Dark }

pub struct Field {
    pub key: String,
    pub label: String,
    pub value: String,
    pub alignment: FieldAlignment,
}

pub struct Branding {
    pub organization_name: Option<String>,
    pub logo_text: Option<String>,
    pub background_color: RgbColor,
    pub text_color_mode: TextColorMode,
    pub logo_png_bytes: Vec<u8>,
    pub icon_png_bytes: Option<Vec<u8>>,
    pub hero_png_bytes: Option<Vec<u8>>,
}

pub struct RgbColor { pub r: u8, pub g: u8, pub b: u8 }
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub relevant_text: Option<String>,
}
```
</interfaces>
</context>

<must_haves>
- `WalletSubject` trait exists with all 11 methods from spec §3.1.
- `PassKind`, `FieldAlignment`, `TextColorMode` derive `Debug, Clone, PartialEq, Eq`.
- `RgbColor` derives `Debug, Clone, Copy, PartialEq, Eq`.
- `Field`, `Branding`, `GeoPoint` derive `Debug, Clone`.
- `RgbColor::from_hex(&str) -> Result<RgbColor, WalletError>` parses `#RRGGBB` and `RRGGBB`; rejects malformed input with `WalletError::InvalidInput`.
- `auto_foreground(bg: RgbColor) -> RgbColor` implements BT.601 luminance threshold per D-06: `< 0.5` → `RgbColor { r: 255, g: 255, b: 255 }`; `>= 0.5` → `RgbColor { r: 17, g: 24, b: 39 }`.
- Unit tests cover ACC-1e (`rgb_from_hex`) and ACC-1f (`auto_foreground_dark_bg_is_white`).
- `lib.rs` `pub use subject::{...}` block restored (D-11).
</must_haves>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement WalletSubject trait + value types + BT.601 helper + tests</name>
  <files>ferro-wallet/src/subject.rs</files>
  <read_first>
    - docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md §3.1 (full trait + value type definitions)
    - 151-PATTERNS.md §"ferro-wallet/src/subject.rs (WalletSubject trait + value types) — NEW PATTERN"
    - 151-CONTEXT.md D-06 (BT.601 luminance threshold, white vs `rgb(17,24,39)`)
    - 151-RESEARCH.md §"Component Responsibilities" (subject.rs row)
    - 151-VALIDATION.md ACC-1e and ACC-1f rows (test names + commands)
    - ferro-wallet/src/error.rs (for `WalletError::InvalidInput`)
  </read_first>
  <behavior>
    - `RgbColor::from_hex("#ffffff").unwrap() == RgbColor { r: 255, g: 255, b: 255 }`
    - `RgbColor::from_hex("000000").unwrap() == RgbColor { r: 0, g: 0, b: 0 }`
    - `RgbColor::from_hex("#FF8000").unwrap() == RgbColor { r: 255, g: 128, b: 0 }`
    - `RgbColor::from_hex("not-a-color").is_err()`  (returns `WalletError::InvalidInput`)
    - `RgbColor::from_hex("#fff").is_err()`  (3-digit short form not supported; rejects)
    - `auto_foreground(RgbColor { r: 0, g: 0, b: 0 }) == RgbColor { r: 255, g: 255, b: 255 }` (dark bg → white)
    - `auto_foreground(RgbColor { r: 255, g: 255, b: 255 }) == RgbColor { r: 17, g: 24, b: 39 }` (light bg → dark slate)
    - `auto_foreground(RgbColor { r: 128, g: 128, b: 128 })` deterministic — choose either branch; document which.
  </behavior>
  <action>
    Replace the `// placeholder` line in `ferro-wallet/src/subject.rs`. Implement:

    1. Imports: `use chrono::{DateTime, Utc}; use crate::WalletError;`

    2. The `WalletSubject` trait exactly per spec §3.1 (signature above, 11 methods).

    3. Closed enums with full derives:
       ```rust
       #[derive(Debug, Clone, PartialEq, Eq)]
       pub enum PassKind { EventTicket, Generic, Coupon }

       #[derive(Debug, Clone, PartialEq, Eq)]
       pub enum FieldAlignment { Left, Center, Right, Natural }

       #[derive(Debug, Clone, PartialEq, Eq)]
       pub enum TextColorMode { Auto, Light, Dark }
       ```

    4. Value structs:
       ```rust
       #[derive(Debug, Clone)]
       pub struct Field {
           pub key: String,
           pub label: String,
           pub value: String,
           pub alignment: FieldAlignment,
       }

       #[derive(Debug, Clone)]
       pub struct Branding {
           pub organization_name: Option<String>,
           pub logo_text: Option<String>,
           pub background_color: RgbColor,
           pub text_color_mode: TextColorMode,
           pub logo_png_bytes: Vec<u8>,
           pub icon_png_bytes: Option<Vec<u8>>,
           pub hero_png_bytes: Option<Vec<u8>>,
       }

       #[derive(Debug, Clone)]
       pub struct GeoPoint {
           pub latitude: f64,
           pub longitude: f64,
           pub relevant_text: Option<String>,
       }
       ```

    5. `RgbColor` with `Copy`:
       ```rust
       #[derive(Debug, Clone, Copy, PartialEq, Eq)]
       pub struct RgbColor { pub r: u8, pub g: u8, pub b: u8 }

       impl RgbColor {
           pub fn from_hex(s: &str) -> Result<Self, WalletError> {
               let hex = s.strip_prefix('#').unwrap_or(s);
               if hex.len() != 6 {
                   return Err(WalletError::InvalidInput(format!(
                       "rgb hex must be 6 chars (with optional leading '#'): got {s:?}"
                   )));
               }
               let parse = |range: std::ops::Range<usize>| -> Result<u8, WalletError> {
                   u8::from_str_radix(&hex[range], 16).map_err(|e| {
                       WalletError::InvalidInput(format!("rgb hex parse: {e}"))
                   })
               };
               Ok(RgbColor { r: parse(0..2)?, g: parse(2..4)?, b: parse(4..6)? })
           }

           /// CSS-style `rgb(r,g,b)` literal — used by Apple pass.json.
           pub fn css_rgb(&self) -> String {
               format!("rgb({},{},{})", self.r, self.g, self.b)
           }
       }
       ```

    6. BT.601 luminance helper (D-06). Implement as a free function `pub fn auto_foreground(bg: RgbColor) -> RgbColor`. The BT.601 normalised luminance formula is `(0.299*r + 0.587*g + 0.114*b) / 255.0`. Threshold at `0.5`:
       ```rust
       /// Derives a readable foreground colour from a background using BT.601 luminance (D-06).
       ///
       /// Background luminance `< 0.5` → white. `>= 0.5` → dark slate `rgb(17, 24, 39)`.
       pub fn auto_foreground(bg: RgbColor) -> RgbColor {
           let r = bg.r as f64 / 255.0;
           let g = bg.g as f64 / 255.0;
           let b = bg.b as f64 / 255.0;
           let lum = 0.299 * r + 0.587 * g + 0.114 * b;
           if lum < 0.5 {
               RgbColor { r: 255, g: 255, b: 255 }
           } else {
               RgbColor { r: 17, g: 24, b: 39 }
           }
       }
       ```

    7. `#[cfg(test)] mod tests` block with at minimum:
       - `rgb_from_hex` — covers `#ffffff`, `000000`, `#FF8000`, mixed case, success path (ACC-1e).
       - `rgb_from_hex_rejects_malformed` — rejects `"not-a-color"`, `"#fff"` (short form), `"#fffffff"` (7 chars), `"#zzzzzz"` (non-hex). Each returns `Err(WalletError::InvalidInput(_))`.
       - `auto_foreground_dark_bg_is_white` — asserts `auto_foreground(RgbColor { r: 0, g: 0, b: 0 }) == RgbColor { r: 255, g: 255, b: 255 }` (ACC-1f).
       - `auto_foreground_light_bg_is_dark_slate` — asserts `auto_foreground(RgbColor { r: 255, g: 255, b: 255 }) == RgbColor { r: 17, g: 24, b: 39 }`.
       - `rgb_css_rgb_format` — asserts `RgbColor { r: 17, g: 24, b: 39 }.css_rgb() == "rgb(17,24,39)"`.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib subject::tests::rgb_from_hex &amp;&amp; cargo test -p ferro-wallet --lib subject::tests::auto_foreground_dark_bg_is_white &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub trait WalletSubject' ferro-wallet/src/subject.rs &amp;&amp; grep -F 'pub fn auto_foreground' ferro-wallet/src/subject.rs &amp;&amp; grep -F 'pub fn from_hex' ferro-wallet/src/subject.rs</automated>
  </verify>
  <done>`WalletSubject` + 7 value types + `RgbColor::from_hex` + `auto_foreground` all land. ACC-1e and ACC-1f test names exist and pass. Clippy + fmt clean.</done>
</task>

<task type="auto">
  <name>Task 2: Restore `subject::*` re-exports in lib.rs</name>
  <files>ferro-wallet/src/lib.rs</files>
  <read_first>
    - ferro-wallet/src/lib.rs (the commented-out re-export block from PLAN-01 Task 1)
    - 151-CONTEXT.md D-11 (re-export restoration timing)
  </read_first>
  <action>
    Uncomment the `pub use subject::{ ... };` block. Verify the symbol list matches `subject.rs` exports exactly:

    ```rust
    pub use subject::{
        auto_foreground, Branding, Field, FieldAlignment, GeoPoint, PassKind, RgbColor,
        TextColorMode, WalletSubject,
    };
    ```

    Leave `apple::ApplePassBuilder`, `google::GoogleWalletBuilder`, and `config::{...}` re-exports commented out — those restore in PLAN-03 / PLAN-05 / PLAN-07 respectively.
  </action>
  <verify>
    <automated>cargo build -p ferro-wallet &amp;&amp; cargo test -p ferro-wallet --lib &amp;&amp; cargo clippy -p ferro-wallet --all-targets -- -D warnings &amp;&amp; cargo fmt -p ferro-wallet -- --check &amp;&amp; grep -F 'pub use subject::' ferro-wallet/src/lib.rs &amp;&amp; grep -F 'WalletSubject' ferro-wallet/src/lib.rs</automated>
  </verify>
  <done>`lib.rs` re-exports all `subject` types. Downstream callers can `use ferro_wallet::WalletSubject;`. Build + tests + clippy + fmt green.</done>
</task>

</tasks>

<threat_model>
This plan introduces pure-data domain types and one deterministic pure function (`auto_foreground`). No crypto, no secrets, no external service interaction.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-151-Apple-COLOR | D | `auto_foreground` (subject.rs) | mitigate | ACC-1f unit test pins the dark-bg→white branch; ACC-1f-complement (light-bg → dark slate) added in this plan as a sister test. Mismatched-luminance branches cause usability bugs, not security issues. |

No other STRIDE entries apply.
</threat_model>

<verification>
- `cargo test -p ferro-wallet --lib subject::tests` runs ≥5 tests, all pass (covers ACC-1e + ACC-1f + supporting cases).
- `cargo build -p ferro-wallet` exits 0.
- `cargo clippy -p ferro-wallet --all-targets -- -D warnings` exits 0.
- `cargo fmt -p ferro-wallet -- --check` exits 0.
- `grep -F 'pub use subject::' ferro-wallet/src/lib.rs` returns one match.
</verification>

<success_criteria>
PLAN-05 (apple builder) and PLAN-07 (google builder) can consume `WalletSubject` via `<S: WalletSubject>` bounds. `RgbColor::from_hex` and `auto_foreground` are available for `apple/manifest.rs` (`build_pass_json`) to derive foreground/label colours per D-06.
</success_criteria>

<output>
After completion, create `.planning/phases/151-ferro-wallet-crate/151-02-SUMMARY.md` listing the trait method signatures and a one-line note on each value type's derived traits, so PLAN-05 and PLAN-07 executors can see the contract at a glance.
</output>

## PLANNING COMPLETE
