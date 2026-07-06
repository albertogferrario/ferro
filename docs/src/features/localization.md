# Localization

Ferro provides JSON-based localization via the `ferro-lang` crate with per-request locale detection, parameter interpolation, pluralization, and automatic validation message translation.

## Configuration

### Environment Variables

Configure localization in your `.env` file:

| Variable | Default | Description |
|----------|---------|-------------|
| `APP_LOCALE` | `en` | Default locale |
| `APP_FALLBACK_LOCALE` | `en` | Fallback when key missing in requested locale |
| `LANG_PATH` | `lang` | Directory containing translation files |

### Programmatic Configuration

Override defaults in your `config_fn()` before `Application::run()`:

```rust
use ferro::{Config, LangConfig};

pub fn config_fn() {
    Config::register(LangConfig::builder()
        .locale("es")
        .fallback_locale("en")
        .path("resources/lang")
        .build());
}
```

Unset builder fields fall back to environment variables automatically.

## Directory Structure

Translation files are organized by locale, with each locale in its own subdirectory:

```
lang/
  en/
    app.json
    validation.json
  es/
    app.json
    validation.json
```

JSON files support nested objects, flattened via dot notation:

```json
{
  "auth": {
    "login": "Log in",
    "register": "Create account"
  }
}
```

Accessed as `t("auth.login", &[])`.

Multiple JSON files per locale are merged. File names are arbitrary -- use them to organize by domain (e.g., `app.json`, `validation.json`, `auth.json`).

## Translation Helpers

### `t()` / `trans()`

Basic translation with parameter interpolation:

```rust
use ferro::t;

// Simple translation
let msg = t("welcome", &[]);

// With parameters -- :param syntax
let msg = t("greeting", &[("name", "Alice")]);
// "Hello, :name!" becomes "Hello, Alice!"
```

`trans()` is an alias for `t()` for those who prefer the longer name.

### Parameter Interpolation

Parameters use the `:param` placeholder syntax with three case variants:

- `:name` -- value as-is
- `:Name` -- first character uppercased
- `:NAME` -- entire value uppercased

```rust
// Given translation: ":name :Name :NAME"
let msg = t("example", &[("name", "alice")]);
// Result: "alice Alice ALICE"
```

Parameters are processed longest-key-first to avoid partial replacement (e.g., `:username` is replaced before `:user`). Missing placeholders are left as-is.

### `lang_choice()`

Pluralized translation:

```rust
use ferro::lang_choice;

let msg = lang_choice("items.count", 1, &[]);  // "One item"
let msg = lang_choice("items.count", 5, &[]);  // "5 items"
```

A `:count` parameter is added automatically.

## Pluralization

### Simple Forms

Pipe-separated values where count of 1 selects the first form, everything else selects the second:

```json
{
  "items.count": "One item|:count items"
}
```

### Explicit Ranges

For finer control, use range syntax:

```json
{
  "cart.summary": "{0} Your cart is empty|{1} One item|[2,*] :count items"
}
```

Range syntax:

| Syntax | Meaning |
|--------|---------|
| `{N}` | Exact match for count N |
| `[N,M]` | Inclusive range N through M |
| `[N,*]` | N or more |
| Plain pipe `\|` | First form for count=1, second for everything else |

### Example

```json
{
  "apples": "{0} No apples|{1} One apple|[2,5] A few apples|[6,*] :count apples"
}
```

```rust
lang_choice("apples", 0, &[]);   // "No apples"
lang_choice("apples", 1, &[]);   // "One apple"
lang_choice("apples", 3, &[]);   // "A few apples"
lang_choice("apples", 10, &[]);  // "10 apples"
```

## Locale Detection

The `LangMiddleware` detects locale per-request with this priority:

1. `?locale=xx` query parameter (explicit override)
2. `Accept-Language` header (first language tag)
3. `APP_LOCALE` default from config

### Setup

Register the middleware globally:

```rust
use ferro::{global_middleware, LangMiddleware};

pub fn register() {
    global_middleware!(LangMiddleware);
}
```

### Manual Override in Handlers

```rust
use ferro::{locale, set_locale};

#[handler]
pub async fn show(req: Request) -> Response {
    let current = locale();  // e.g. "en"
    set_locale("fr");        // override for this request
    // subsequent t() calls use "fr"
}
```

### Locale Normalization

Locale identifiers are normalized to lowercase with hyphens: `en_US`, `EN-US`, and `en-us` all resolve to `en-us`. This applies to directory names, query parameters, and `Accept-Language` headers.

## Validation Messages

Validation error messages are automatically localized when translations are loaded. The framework registers a validation bridge at boot that routes message lookups through the Translator.

Translation keys follow the pattern `validation.{rule_name}`:

```json
{
  "validation": {
    "required": "The :attribute field is required.",
    "email": "The :attribute field must be a valid email address.",
    "min": {
      "string": "The :attribute field must be at least :min characters.",
      "numeric": "The :attribute field must be at least :min.",
      "array": "The :attribute field must have at least :min items."
    }
  }
}
```

Default English validation messages are bundled with the framework. Custom messages in your translation files override them per-locale.

Size rules (`min`, `max`, `between`) use nested keys for type-specific messages: `validation.min.string`, `validation.min.numeric`, `validation.min.array`.

## CLI Commands

### Generate Translation Files

Create translation files for a new locale:

```bash
ferro make:lang es
```

This creates `lang/es/` with `app.json` and `validation.json` starter templates.

### New Project Scaffolding

`ferro new` includes `lang/en/` by default with English starter translations and the locale environment variables in `.env.example`.

## Fallback Chain

When looking up a translation key:

1. Check the requested locale
2. If not found, check the fallback locale (`APP_FALLBACK_LOCALE`)

Fallback keys are pre-merged into each locale at load time, so runtime lookup is a single hash map access with no fallback chain traversal.

## Graceful Degradation

If no `lang/` directory exists or translations fail to load:

- `t()` and `trans()` return the key as-is (e.g., `"welcome"`)
- `lang_choice()` returns the key as-is
- Validation rules fall back to hardcoded English messages
- No panics, no errors -- the application runs normally without localization

This means localization is entirely opt-in. Applications work without any translation files present.

## MCP Tools

Use `list_lang_files` to discover all translation files in the project.

### `list_lang_files`

Returns all JSON translation files found under the configured `LANG_PATH` directory, organized by locale. For each file, shows the locale, file name, and the top-level translation keys it defines. Use this to audit translation coverage across locales before adding new keys or supporting a new language.
