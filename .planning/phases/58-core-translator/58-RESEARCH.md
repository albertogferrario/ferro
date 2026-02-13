# Phase 58: Core Translator - Research

**Researched:** 2026-02-13
**Domain:** Rust localization — JSON translation loading, interpolation, pluralization
**Confidence:** HIGH

<research_summary>
## Summary

Researched the Rust i18n ecosystem and common web framework localization patterns to inform the ferro-lang crate design. The domain is well-understood with established patterns.

The v6.0 milestone already locks key decisions (JSON files, `:param` interpolation, `|` plural separator). Research confirms these are sound and identifies one key tension: the `|` pipe separator follows Laravel's simple approach, which works for Western European languages but breaks down for Slavic/Arabic. The recommendation is to start with pipe syntax as planned but design the internal API around CLDR plural categories so a suffix-based format can be added later.

The main "don't hand-roll" finding: **CLDR plural rules**. The `icu_plurals` crate (Unicode Consortium, v2.0 May 2025) provides correct plural category resolution for 280+ locales. Several ICU4X crates are already transitive dependencies in the workspace. For Phase 58, a simple `n == 1 → one, else → other` is sufficient since only the pipe format is in scope, but the Translator API should accept `PluralCategory` to enable proper CLDR support when needed.

**Primary recommendation:** Build ferro-lang following ferro-cache/ferro-events crate patterns. Hand-roll JSON loading + interpolation (trivial with serde). Use simple `one|other` pluralization for Phase 58. Design the internal plural resolution as a trait so `icu_plurals` can be plugged in later via feature flag.
</research_summary>

<standard_stack>
## Standard Stack

### Core (Phase 58 dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.x | JSON deserialization | Already in workspace, standard for JSON loading |
| serde_json | 1.x | JSON parsing | Already in workspace, needed for translation files |
| thiserror | 1.x/2.x | Error types | Ferro crate convention |
| tracing | 0.1 | Debug logging | Ferro crate convention |

### Future (Phase 59+ or feature-gated)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| icu_plurals | 2.x | CLDR plural rules | When proper multilingual pluralization needed |
| icu_locale_core | 2.x | Locale parsing | Already transitive dep; use for locale ID validation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom JSON loading | rust-i18n | rust-i18n uses compile-time codegen from YAML; too opinionated, doesn't match ferro patterns |
| Custom JSON loading | fluent-rs | Fluent uses `.ftl` format, not JSON; would override the JSON decision |
| Simple plural rules | icu_plurals | icu_plurals is correct for all locales; adds dependency. Best as opt-in feature flag |
| `:param` interpolation | `{{param}}` (i18next) | Both work; `:param` is simpler and matches Laravel convention already decided |

**Installation (Phase 58):**
```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
```
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Recommended Crate Structure
```
ferro-lang/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API: Translator, translate(), translate_choice()
│   ├── translator.rs    # Core Translator struct: load, get, interpolate, pluralize
│   ├── loader.rs        # JSON file loading: read lang/{locale}/*.json
│   ├── interpolation.rs # :param replacement logic
│   ├── pluralization.rs # Plural form selection (simple or CLDR)
│   └── error.rs         # LangError enum (thiserror)
```

### Pattern 1: Translator as Shared State
**What:** Single `Translator` instance loaded at startup, shared via `Arc`
**When to use:** Always — translations are read-heavy, write-never after init
**Rationale:** Matches ferro-cache pattern (shared `CacheManager`). No per-request allocation.

### Pattern 2: Fallback Chain
**What:** Look up key in requested locale → fallback locale → return key as-is
**When to use:** Every translation lookup
**Rationale:** Prevents missing translations from crashing; graceful degradation. Laravel, Rails, i18next all do this.

### Pattern 3: Flat JSON with Dot-Notation Keys
**What:** Translation files are flat key-value JSON. Nested access via `"auth.login.title"` dot keys maps to either flat keys or nested JSON objects.
**When to use:** All translation files
**Example:**
```json
{
    "auth.login.title": "Sign In",
    "auth.login.email": "Email Address",
    "validation.required": "The :attribute field is required."
}
```

### Pattern 4: Pipe-Separated Plural Forms
**What:** `"one form|other form"` in translation value, selected by count
**When to use:** Phase 58 pluralization (as decided in v6.0 milestone)
**Example:**
```json
{
    "items.count": "One item|:count items",
    "cart.items": "{0} Your cart is empty|{1} One item in cart|[2,*] :count items in cart"
}
```

### Anti-Patterns to Avoid
- **Loading translations per-request:** Load once at startup, share via Arc. Translation files don't change at runtime.
- **Returning Option<String> from translate():** Return the key itself as fallback (like Laravel `__()`) — never None.
- **Hardcoding locale list:** Discover available locales from filesystem (`lang/` directory contents).
- **Nested HashMap<String, HashMap<String, String>>:** Use `HashMap<String, String>` with dot-notation keys. Simpler API, easier merging.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Full CLDR plural rules | Custom rule functions for 280+ locales | `icu_plurals` crate (feature-gated) | CLDR rules are complex (Arabic has 6 forms with modulo conditions), updated regularly |
| Locale ID parsing/validation | Custom regex for "en", "en-US", "pt-BR" | `icu_locale_core` (already a transitive dep) | BCP 47 locale tags have edge cases |

**What IS safe to hand-roll (Phase 58):**

| Problem | Hand-Roll OK | Why |
|---------|-------------|-----|
| JSON file loading | `serde_json::from_str` + file reading | Trivial — read files, deserialize to HashMap |
| `:param` interpolation | String replacement loop | Simple find-and-replace; no edge cases worth a library |
| Simple `one\|other` pluralization | `if count == 1 { forms[0] } else { forms[1] }` | Two-form selection is a one-liner |
| Fallback chain | Check locale → check fallback → return key | Simple HashMap lookups |

**Key insight:** The only "don't hand-roll" item for Phase 58 is the internal API design — make the pluralization trait-based so `icu_plurals` can be swapped in later. The actual Phase 58 implementation is entirely hand-rollable because it only needs `one|other`.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Eager File Loading Blocks Startup
**What goes wrong:** Reading 50+ JSON files synchronously delays server start
**Why it happens:** Translation directories grow as locales are added
**How to avoid:** Use `std::fs::read_to_string` (sync is fine for startup), but load all files in a single pass with clear timing logs via `tracing::info!`
**Warning signs:** Startup time increasing as locales are added

### Pitfall 2: Missing Interpolation Parameters Produce Broken Output
**What goes wrong:** `"Hello :name"` with no `:name` parameter → user sees literal `:name`
**Why it happens:** Caller forgets to pass parameter; no validation
**How to avoid:** Log a warning via `tracing::warn!` when a `:param` placeholder has no matching replacement, but still return the string (don't error). This matches Laravel behavior.
**Warning signs:** UI showing `:attribute` or `:count` literally

### Pitfall 3: Plural Form Count Mismatch
**What goes wrong:** Translation has 3 pipe-separated forms but locale only expects 2 (or vice versa)
**Why it happens:** Translator providing wrong number of forms
**How to avoid:** Always fall back to the last form if index is out of range. Log warning if form count doesn't match expected. Never panic.
**Warning signs:** Empty strings in UI where counts should appear

### Pitfall 4: Case-Sensitive Locale Keys
**What goes wrong:** `"en-US"` vs `"en-us"` vs `"en_US"` all treated as different locales
**Why it happens:** No locale key normalization
**How to avoid:** Normalize all locale identifiers to lowercase with hyphens on load: `"en-us"`, `"pt-br"`. Accept any case/separator as input.
**Warning signs:** Translations "missing" despite files existing

### Pitfall 5: Not Pre-merging Fallback at Load Time
**What goes wrong:** Every `translate()` call does a fallback lookup chain at runtime
**Why it happens:** Lazy fallback resolution
**How to avoid:** At load time, merge fallback translations into each locale's map (locale entries override fallback entries). Then runtime lookup is a single HashMap::get.
**Warning signs:** Unnecessary complexity in hot path
</common_pitfalls>

<code_examples>
## Code Examples

### Translation File Format
```json
// lang/en/messages.json
{
    "welcome": "Welcome, :name!",
    "items.count": "One item|:count items",
    "cart.summary": "{0} Your cart is empty|{1} :count item in your cart|[2,*] :count items in your cart",
    "validation.required": "The :attribute field is required.",
    "validation.email": "The :attribute field must be a valid email address.",
    "validation.min.string": "The :attribute field must be at least :min characters."
}
```

### Translator Public API (Target Design)
```rust
// Source: Designed from ferro-cache pattern + Laravel API
use std::collections::HashMap;
use std::sync::Arc;

pub struct Translator {
    translations: HashMap<String, HashMap<String, String>>,  // locale -> key -> value
    fallback: String,
}

impl Translator {
    /// Load all translation files from a directory.
    /// Expects: {path}/{locale}/*.json
    pub fn load(path: &str, fallback: &str) -> Result<Self, LangError> { ... }

    /// Get a translated string with parameter replacement.
    pub fn get(&self, locale: &str, key: &str, params: &[(&str, &str)]) -> String { ... }

    /// Get a pluralized translated string.
    pub fn choice(&self, locale: &str, key: &str, count: i64, params: &[(&str, &str)]) -> String { ... }
}
```

### Interpolation Logic
```rust
// Source: Laravel :param convention
fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        // :key → value as-is
        result = result.replace(&format!(":{}", key), value);
    }
    result
}
```

### Simple Plural Selection (Phase 58)
```rust
// Source: Laravel pipe-separator convention
fn select_plural_form(value: &str, count: i64) -> &str {
    let forms: Vec<&str> = value.split('|').collect();
    match forms.len() {
        0 => value,
        1 => forms[0],
        _ => {
            if count == 1 {
                forms[0]
            } else {
                forms.last().unwrap_or(&forms[0])
            }
        }
    }
}
```

### Ferro Crate Pattern (from ferro-cache)
```rust
// Source: ferro-cache/Cargo.toml pattern
// ferro-lang/Cargo.toml
[package]
name = "ferro-lang"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Localization for the Ferro web framework"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"

[features]
default = []
icu = ["icu_plurals", "icu_locale_core"]

[dependencies.icu_plurals]
version = "2"
optional = true

[dependencies.icu_locale_core]
version = "2"
optional = true
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `intl_pluralrules` crate | `icu_plurals` (ICU4X) | 2023+ | ICU4X is the official Unicode successor; same author |
| ICU4X 1.x | ICU4X 2.0 | May 2025 | Major release with better modularity, smaller binaries |
| Custom locale parsing | `icu_locale_core` | 2024+ | Part of ICU4X, handles BCP 47 correctly |

**New tools/patterns:**
- **ICU4X 2.0 modular crates:** Can depend on just `icu_plurals` without the full `icu` meta-crate. Good for feature-gated optional dependency.
- **`icu_locale_core` already in workspace:** Available as transitive dependency; free to use for locale validation.

**Still valid:**
- **Laravel's `:param` interpolation** remains the simplest, most readable approach for web frameworks
- **JSON translation files** remain the standard portable format for web i18n
- **Pipe-separated plurals** (Laravel) are adequate for most Western language apps
</sota_updates>

<open_questions>
## Open Questions

1. **Range syntax in pipe plurals: support in Phase 58?**
   - What we know: Laravel supports `{0}|{1}|[2,*]` range syntax alongside simple pipe
   - What's unclear: Whether the v6.0 milestone expects range syntax or just simple `one|other`
   - Recommendation: Implement simple `one|other` first. Add range syntax parsing only if needed — it adds complexity without clear immediate value.

2. **ICU feature flag: when to add?**
   - What we know: `icu_plurals` would give correct pluralization for all CLDR locales
   - What's unclear: Whether this should be a Phase 58 feature flag or deferred to a later phase
   - Recommendation: Define the `icu` feature in Cargo.toml in Phase 58 but implement behind the flag in a later phase. The trait-based plural resolver makes this additive.

3. **Translation file discovery: glob vs explicit config?**
   - What we know: Need to load `lang/{locale}/*.json` files
   - What's unclear: Whether to use filesystem glob or require explicit locale listing in config
   - Recommendation: Filesystem discovery (read `lang/` subdirectory names). Simpler, matches Laravel convention, no config needed.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Ferro codebase inspection — ferro-cache, ferro-events crate patterns, validation/rules.rs message format
- Ferro v6.0 milestone doc — key decisions on format, interpolation, plural syntax
- ICU4X 2.0 release blog (Unicode Consortium, May 2025) — icu_plurals crate status
- CLDR Language Plural Rules (unicode.org/cldr) — plural category definitions

### Secondary (MEDIUM confidence)
- Laravel 12.x Localization docs — `:param` interpolation, pipe plural syntax, `__()` function behavior
- i18next v4 JSON format docs — suffix-based plural keys as alternative pattern
- Rails i18n guide — CLDR category approach comparison

### Tertiary (LOW confidence - needs validation)
- rust-i18n crate (GitHub) — feature comparison, not tested
- fluent-rs crate (crates.io) — feature comparison, not tested
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust i18n, JSON translation files, string interpolation
- Ecosystem: icu_plurals, icu_locale_core, rust-i18n, fluent-rs (evaluated, not adopted)
- Patterns: Translator singleton, fallback chain, pipe pluralization, `:param` interpolation
- Pitfalls: File loading performance, interpolation parameter validation, plural form mismatches, locale normalization

**Confidence breakdown:**
- Standard stack: HIGH — minimal dependencies, all already in workspace
- Architecture: HIGH — follows established ferro crate patterns exactly
- Pitfalls: HIGH — well-known issues from Laravel/Rails ecosystems
- Code examples: HIGH — derived from ferro codebase patterns and Laravel conventions

**Research date:** 2026-02-13
**Valid until:** 2026-03-15 (30 days — stable domain, no fast-moving ecosystem)
</metadata>

---

*Phase: 58-core-translator*
*Research completed: 2026-02-13*
*Ready for planning: yes*
