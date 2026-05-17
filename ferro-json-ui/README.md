# ferro-json-ui

JSON-based server-driven UI schema types for the [Ferro](https://ferro-rs.dev) web framework.

Define UI as a JSON spec and have Ferro render it to HTML on the server — no
frontend build step required.

## Features

- 41 built-in components (Card, Form, DataTable, KanbanBoard, Modal, Tabs,
  Alert, Badge, Button, ...) and a plugin system for custom components
- Layout system (`dashboard`, `app`, `auth`) and ID-keyed element graph with
  parse-time structural validation
- Action system: navigate, submit form, call API, open modal
- Data binding via JSON Pointer (`{"$data": "/path"}`) and iteration directives
  (`$each`, `$if`)
- Compile-time schema validation via `schemars` on every typed `*Props`

## Usage

Serve a spec from a Rust handler:

```rust
use ferro::{handler, JsonUi, Request, Response};

#[handler]
pub async fn dashboard(req: Request) -> Response {
    let data = serde_json::json!({});
    JsonUi::render_file("views/dashboard.json", data)
}
```

Or construct a spec in Rust:

```rust
use ferro_json_ui::{Spec, Element};

let spec = Spec::builder()
    .title("Demo")
    .element("root", Element::new("Text").prop("content", "Hi"))
    .build()
    .unwrap();
```

## Documentation

Full documentation at [docs.ferro-rs.dev](https://docs.ferro-rs.dev).

## License

MIT
