# ferro-lang

Localization crate for the [Ferro](https://ferro-rs.dev) web framework.

Provides JSON-based translation loading with per-request locale detection, string interpolation,
and pluralization. Integrates directly with Ferro's request lifecycle via middleware.

## Features

- **JSON translation files** — one file per locale (`en.json`, `fr.json`, etc.)
- **Per-request locale detection** — `Accept-Language` header, query param (`?lang=fr`), or session value
- **String interpolation** — `:name` syntax for named placeholders
- **Pluralization** — locale-aware plural forms (`one | other` rules)
- **Validation message localization** — translate Ferro validation error messages automatically
- **Middleware** — `LocaleMiddleware` sets the active locale on every request

## Usage

```rust
use ferro_lang::{Lang, LocaleMiddleware};

// Load translations at startup
let lang = Lang::from_dir("lang/")   // loads en.json, fr.json, etc.
    .default_locale("en")
    .build()?;

// In middleware or handler
let greeting = lang.t("messages.welcome", &[("name", "World")]);
// en.json: { "messages": { "welcome": "Hello, :name!" } }
// => "Hello, World!"

// Pluralization
let message = lang.t_count("items.count", count, &[]);
// en.json: { "items": { "count": "one item | :count items" } }
// => "1 item" or "5 items"
```

## Documentation

Full documentation at [docs.ferro-rs.dev](https://docs.ferro-rs.dev).

## License

MIT
