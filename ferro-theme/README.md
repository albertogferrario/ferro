# ferro-theme

Semantic theme tokens and intent template schema for Ferro.

Provides the `Theme` struct which holds an embedded or filesystem-loaded CSS file (using Tailwind v4 `@theme` syntax) and optional `ThemeTemplates` that configure how each of the seven structural intents renders its slot layout. Themes are pure CSS plus a small JSON schema, requiring no Rust code from theme authors.

Status: part of the [ferro](https://github.com/albertogferrario/ferro) framework workspace.

Documentation: https://docs.rs/ferro-theme

License: MIT
