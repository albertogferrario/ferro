# ferro-text

Conversational-text renderer for Ferro service projections.

`ferro-text` provides `TextRenderer`, a [`Renderer`] implementation that projects a
[`ServiceDef`] to deterministic plain text. It is the text-channel output crate in the
same renderer-per-output-crate pattern as `ferro-json-ui` (`JsonUiRenderer`) and
`ferro-mcp-server` (`McpRenderer`).

## Supported intents

| Intent | Output shape |
|--------|-------------|
| Browse | Entity name + domain field labels |
| Collect | "Fields to provide" with required markers |
| Process | Current state + guard-passing actions |
| Summarize | Entity + key metric/status fields |
| Track | Current state + terminal status |
| Focus | Field rendering with `render_hint` applied (degraded fallback) |
| Analyze | Entity + field set (degraded fallback, no fabricated statistics) |

Output is guard-filtered via `BaseContext::evaluated_guards` and verbosity-aware
(`Verbosity::Full` / `Verbosity::Brief`).

## Usage

```rust
use ferro_projections::{derive_intents, render::BaseContext, ServiceDef};
use ferro_text::TextRenderer;
use ferro_projections::render::Renderer;

let service = ServiceDef::new("order")
    .display_name("Order")
    .field("title", ferro_projections::DataType::String, ferro_projections::FieldMeaning::EntityName)
    .field("status", ferro_projections::DataType::String, ferro_projections::FieldMeaning::Status);

let intents = derive_intents(&service);
let ctx = BaseContext::default();
let text = TextRenderer.render(&service, &intents, &ctx).unwrap();
```
