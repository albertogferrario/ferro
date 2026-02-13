# Phase 61: Validation Bridge - Research

**Researched:** 2026-02-13
**Domain:** Rust OnceLock callback pattern for decoupled validation message translation
**Confidence:** HIGH

<research_summary>
## Summary

Researched the internal codebase patterns and Rust standard library mechanisms for implementing a translation callback in the validation module without introducing a dependency on ferro-lang.

The validation module currently has 21 rules (required, email, min, max, etc.) that return hardcoded English error messages via `format!()`. Phase 61 bridges validation to ferro-lang by registering a translator callback at startup, which rules call to resolve localized messages. The validation crate never imports ferro-lang — it only holds a `fn` pointer set externally.

The established Ferro pattern for this is `OnceLock<T>` (used in config/repository.rs, container/mod.rs, routing/router.rs, middleware/registry.rs, metrics/mod.rs). The validation bridge follows the same pattern: `static TRANSLATOR: OnceLock<TranslatorFn>` where `TranslatorFn = fn(&str, &[(&str, &str)]) -> Option<String>`.

**Primary recommendation:** Add a single `OnceLock<TranslatorFn>` in `validation/mod.rs`. Rules call a `translate_validation(key, params)` helper that checks the OnceLock — if set, returns the translated message; if not, returns `None` and rules fall back to their current hardcoded English. Framework integration (Phase 63) registers the callback at boot.
</research_summary>

<standard_stack>
## Standard Stack

### Core (Phase 61 dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::sync::OnceLock | std | One-time callback registration | Already used in 6+ framework modules |

No new dependencies required. This phase only uses std types already present in the workspace.

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none) | - | - | Phase 61 adds zero new dependencies |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `OnceLock<fn>` | `OnceLock<Box<dyn Fn>>` | Box<dyn Fn> allows closures that capture state but requires `Send + Sync` bounds, heap allocation. `fn` pointer is simpler — the translator function is a static function, not a closure |
| `OnceLock<fn>` | Trait object via `OnceLock<Arc<dyn Translator>>` | Over-engineered for a single function call. A bare `fn` is simpler and matches the single-purpose bridge |
| Global OnceLock | Passing translator to Validator constructor | Would break the existing `Validator::new(&data)` API. Rules inside the validator can't access constructor params. OnceLock is the only viable pattern |
| `OnceLock<fn>` per rule | Single global `OnceLock` | Per-rule callbacks add 21 registration calls. Single global callback with key-based dispatch is simpler |
</standard_stack>

<architecture_patterns>
## Architecture Patterns

### Pattern 1: OnceLock Callback Registration (Established Ferro Pattern)
**What:** Global static `OnceLock<T>` initialized once at app boot, read at runtime
**When to use:** Any cross-module communication without direct dependency
**Existing usage in Ferro:**
```
framework/src/config/repository.rs     → OnceLock<RwLock<ConfigRepository>>
framework/src/container/mod.rs         → OnceLock<RwLock<Container>>
framework/src/routing/router.rs        → OnceLock<RwLock<HashMap<...>>>
framework/src/middleware/registry.rs   → OnceLock<RwLock<Vec<BoxedMiddleware>>>
framework/src/middleware/rate_limit.rs → OnceLock<DashMap<...>>
framework/src/metrics/mod.rs          → OnceLock<RwLock<MetricsStore>>
```

### Pattern 2: Translation Key Convention
**What:** Each validation rule maps to a translation key like `validation.{rule_name}`
**When to use:** All 21 validation rules
**Convention (matches Laravel):**
```
validation.required     → "The :attribute field is required."
validation.email        → "The :attribute field must be a valid email address."
validation.min.string   → "The :attribute field must be at least :min characters."
validation.min.numeric  → "The :attribute field must be at least :min."
validation.max.string   → "The :attribute field must not be greater than :max characters."
validation.between      → "The :attribute field must be between :min and :max."
validation.confirmed    → "The :attribute confirmation does not match."
validation.in           → "The selected :attribute is invalid."
```

### Pattern 3: Fallback to Hardcoded English
**What:** If no translator callback is registered (OnceLock empty), rules return their current hardcoded English messages unchanged
**When to use:** Always — validation must work without ferro-lang
**Rationale:** Zero-breaking-change introduction. Existing apps without localization see no difference.

### Translation Function Signature
```rust
/// Callback signature for validation message translation.
///
/// - `key`: Translation key (e.g., "validation.required")
/// - `params`: Interpolation parameters (e.g., [("attribute", "email"), ("min", "8")])
/// - Returns: Translated message, or None if key not found (falls back to English)
type TranslatorFn = fn(&str, &[(&str, &str)]) -> Option<String>;
```

### Anti-Patterns to Avoid
- **Making validation depend on ferro-lang:** The entire point is decoupling. Validation must compile and work without ferro-lang.
- **Using RwLock for the callback:** The translator function is set once at boot and never changes. OnceLock alone is sufficient — no RwLock needed.
- **Changing Rule trait signature:** Don't add `translate` to the Rule trait. Keep rules returning `String`. The translation happens inside each rule's `validate()` method before returning `Err(message)`.
- **Requiring translator registration:** Validation must work identically if no translator is ever registered. The OnceLock being empty is the expected state for apps without localization.
</architecture_patterns>

<dont_hand_roll>
## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| (none applicable) | - | - | This phase is pure internal wiring with no external solutions to use |

**What IS safe to hand-roll (Phase 61):**

| Problem | Hand-Roll OK | Why |
|---------|-------------|-----|
| OnceLock callback | `static TRANSLATOR: OnceLock<TranslatorFn>` | Standard Rust pattern, 5 lines of code |
| `translate_validation()` helper | Function checking OnceLock + calling callback | Trivial wrapper, 8 lines |
| `register_validation_translator()` | Public function setting the OnceLock | One-liner `OnceLock::set()` |
| Translation key mapping | Match rule name to `"validation.{name}"` | Simple string formatting |

**Key insight:** Phase 61 is pure internal wiring. There are no external libraries, patterns, or solutions to adopt. The entire implementation is approximately 30 lines of Rust — a static OnceLock, a registration function, and a lookup helper. The complexity is in updating 21 rules to call the helper, not in the bridge mechanism itself.
</dont_hand_roll>

<common_pitfalls>
## Common Pitfalls

### Pitfall 1: Breaking Existing Validation Messages
**What goes wrong:** Localized message format differs from hardcoded English, causing test failures or unexpected output
**Why it happens:** Translation keys use `:attribute` but hardcoded messages use `field` variable directly
**How to avoid:** The `translate_validation()` helper should receive the same field name and parameters. If translation returns None, fall back to the exact current hardcoded message. Add tests verifying identical output with and without translator.
**Warning signs:** Test failures in existing validation tests after Phase 61 changes

### Pitfall 2: Parameter Name Mismatch Between Rules and Translation Files
**What goes wrong:** Rule passes `("field", "email")` but translation expects `("attribute", "email")`
**Why it happens:** Inconsistent parameter naming between rule implementations and Laravel-style translation strings
**How to avoid:** Standardize on Laravel's `:attribute` convention. The translation params should always include `("attribute", field_name)`. Additional params like `("min", "8")` or `("max", "255")` use the rule's specific parameter names.
**Warning signs:** Translated messages showing literal `:attribute` instead of the field name

### Pitfall 3: OnceLock Initialization Order
**What goes wrong:** Translator not available when first validation runs during app boot
**Why it happens:** Registration function called after first validation attempt
**How to avoid:** This is a non-issue for Phase 61. The OnceLock simply returns None if not set yet, falling back to English. Phase 63 (framework integration) ensures registration happens in the init sequence before any request handling.
**Warning signs:** First validation returning English while subsequent ones are localized (would indicate a timing issue in Phase 63, not Phase 61)

### Pitfall 4: Size-Dependent Rules Need Type-Aware Translation Keys
**What goes wrong:** `min(8)` on a string says "8 characters" but on a number says "8"
**Why it happens:** Same rule, different message based on value type
**How to avoid:** Use compound keys: `validation.min.string`, `validation.min.numeric`, `validation.min.array`. The rule determines the value type and picks the appropriate key.
**Warning signs:** "The age field must be at least 18 characters" (wrong unit for numeric)

### Pitfall 5: Not Passing Rule-Specific Parameters
**What goes wrong:** Translation string has `:min` placeholder but params only contain `:attribute`
**Why it happens:** Rule doesn't include its constraint value in the params
**How to avoid:** Each rule must include its specific parameters: Min adds `("min", &self.min.to_string())`, Max adds `("max", ...)`, Between adds both. Audit all 21 rules.
**Warning signs:** Translated messages showing literal `:min`, `:max`, `:other`
</common_pitfalls>

<code_examples>
## Code Examples

### Bridge Registration (validation/mod.rs)
```rust
// Source: Ferro OnceLock pattern from config/repository.rs
use std::sync::OnceLock;

/// Callback for translating validation messages.
type TranslatorFn = fn(&str, &[(&str, &str)]) -> Option<String>;

static VALIDATION_TRANSLATOR: OnceLock<TranslatorFn> = OnceLock::new();

/// Register a translation function for validation messages.
///
/// Called once at app boot by the framework integration layer.
/// If not called, all rules return hardcoded English messages.
pub fn register_validation_translator(f: TranslatorFn) {
    let _ = VALIDATION_TRANSLATOR.set(f);
}

/// Attempt to translate a validation message.
///
/// Returns the translated message if a translator is registered and
/// the key exists, otherwise returns None (caller falls back to English).
pub(crate) fn translate_validation(key: &str, params: &[(&str, &str)]) -> Option<String> {
    VALIDATION_TRANSLATOR.get().and_then(|f| f(key, params))
}
```

### Rule Using the Bridge (before/after)
```rust
// BEFORE (current — hardcoded English)
impl Rule for Required {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if is_empty(value) {
            Err(format!("The {} field is required.", field))
        } else {
            Ok(())
        }
    }
}

// AFTER (Phase 61 — translation with fallback)
impl Rule for Required {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if is_empty(value) {
            let msg = translate_validation(
                "validation.required",
                &[("attribute", field)],
            ).unwrap_or_else(|| format!("The {} field is required.", field));
            Err(msg)
        } else {
            Ok(())
        }
    }
}
```

### Size-Aware Rule Translation
```rust
// Min rule needs type-aware key selection
impl Rule for Min {
    fn validate(&self, field: &str, value: &Value, _data: &Value) -> Result<(), String> {
        if value.is_null() {
            return Ok(());
        }
        let size = get_size(value);
        if size < self.min {
            let min_str = (self.min as i64).to_string();
            let key = match value {
                Value::String(_) => "validation.min.string",
                Value::Array(_) => "validation.min.array",
                _ => "validation.min.numeric",
            };
            let msg = translate_validation(key, &[("attribute", field), ("min", &min_str)])
                .unwrap_or_else(|| {
                    let unit = get_size_unit(value);
                    format!("The {} field must be at least {} {}.", field, self.min as i64, unit)
                });
            Err(msg)
        } else {
            Ok(())
        }
    }
}
```

### Framework Registration (Phase 63 preview)
```rust
// This is Phase 63 work, shown for context on how the bridge gets connected
// framework/src/lang/mod.rs or framework init
fn register_lang_validation_bridge() {
    fn translate(key: &str, params: &[(&str, &str)]) -> Option<String> {
        // Uses the global Translator instance and current locale
        let translator = get_translator()?;
        let locale = locale();
        let result = translator.get(&locale, key, params);
        // If translator returns the key itself, treat as "not found"
        if result == key { None } else { Some(result) }
    }
    register_validation_translator(translate);
}
```
</code_examples>

<sota_updates>
## State of the Art (2025-2026)

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `lazy_static!` | `std::sync::OnceLock` | Rust 1.80 (stable) | No external dependency needed for one-time init |
| `once_cell::sync::OnceCell` | `std::sync::OnceLock` | Rust 1.80 | once_cell adopted into std as OnceLock |

**Relevant Rust std evolution:**
- `OnceLock` stabilized in Rust 1.80 — the framework already uses it in 6+ locations
- No newer patterns supersede the OnceLock callback approach for this use case

**Still valid:**
- `fn` pointer for simple callbacks remains the lightest-weight option when no state capture is needed
- The callback approach (vs trait objects) is standard for decoupled module communication in Rust
</sota_updates>

<open_questions>
## Open Questions

1. **Should Phase 61 update all 21 rules or just add the bridge mechanism?**
   - What we know: The roadmap says "OnceLock callback in validation/mod.rs, decouple from ferro-lang"
   - What's unclear: Whether the 21 rules get updated in Phase 61 or Phase 62 ("Update 21 rules to use translate_validation()")
   - Recommendation: Phase 61 adds the bridge mechanism + updates a few representative rules (required, email, min/max) as proof. Phase 62 updates the remaining rules and adds default English JSON.

2. **Translation key naming: `validation.required` vs `validation.required.message`?**
   - What we know: Laravel uses `validation.required` (flat). Some frameworks use nested.
   - What's unclear: Whether Phase 62's English JSON matches Laravel exactly or uses a Ferro-specific convention
   - Recommendation: Use Laravel convention (`validation.required`, `validation.min.string`, `validation.min.numeric`). No reason to deviate.
</open_questions>

<sources>
## Sources

### Primary (HIGH confidence)
- Ferro codebase: `framework/src/config/repository.rs` — OnceLock pattern reference
- Ferro codebase: `framework/src/validation/rules.rs` — 21 rules with hardcoded messages (current state)
- Ferro codebase: `framework/src/validation/rule.rs` — Rule trait interface
- Ferro codebase: `framework/src/validation/validator.rs` — How rules are invoked
- Ferro codebase: `framework/src/lang/mod.rs` — locale() and set_locale() already available
- Ferro codebase: `ferro-lang/src/translator.rs` — Translator API that the bridge will call
- Phase 58 RESEARCH.md — Translation key convention from Laravel
- Rust std docs — `std::sync::OnceLock` API

### Secondary (MEDIUM confidence)
- Laravel 12.x validation localization — key naming convention (validation.required, validation.min.string)
- Laravel validation translation file format — parameter naming (:attribute, :min, :max, :other)
</sources>

<metadata>
## Metadata

**Research scope:**
- Core technology: Rust std::sync::OnceLock, fn pointer callbacks
- Ecosystem: None (internal patterns only)
- Patterns: OnceLock registration, translation key convention, fallback strategy
- Pitfalls: Parameter naming, type-aware keys, initialization order

**Confidence breakdown:**
- Standard stack: HIGH — uses only std, zero new dependencies
- Architecture: HIGH — follows 6 existing OnceLock usages in the framework
- Pitfalls: HIGH — derived from code inspection of all 21 rules
- Code examples: HIGH — adapted from existing ferro codebase patterns

**Research date:** 2026-02-13
**Valid until:** 2026-03-15 (30 days — stable internal patterns, no external dependencies)
</metadata>

---

*Phase: 61-validation-bridge*
*Research completed: 2026-02-13*
*Ready for planning: yes*
