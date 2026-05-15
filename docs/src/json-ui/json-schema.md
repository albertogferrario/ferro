# JSON Schema Export

Ferro can export a JSON Schema document describing the full v2 spec format, including all built-in components and any registered custom plugins. Use it to enable IDE validation and autocompletion for spec files.

## Generate the Schema

Run from your project root:

```bash
# Export full schema (all components + custom plugins)
ferro json-ui:schema --output schema.json --pretty

# Export schema for a single component
ferro json-ui:schema --component DataTable --pretty
```

**Flags:**

| Flag | Description |
|------|-------------|
| `--output <file>` | Write schema to a file (defaults to stdout) |
| `--pretty` | Pretty-print the JSON output |
| `--component <name>` | Export schema for a single component only |

The command requires a compiled Ferro project — it runs your app binary to include custom plugins. The first run may be slow due to compilation.

## VS Code Integration

Add to `.vscode/settings.json` to enable schema validation and autocompletion in spec files:

```jsonc
{
  "json.schemas": [
    {
      "fileMatch": ["src/views/*.json"],
      "url": "./schema.json"
    }
  ]
}
```

After this setup, VS Code validates spec files against the schema and provides autocompletion for component types and their props.

## Other Editors

The generated `schema.json` is standard JSON Schema draft-07, compatible with any editor that supports JSON Schema validation — IntelliJ IDEA, Neovim with an LSP client, and others.

Configure your editor to associate `src/views/*.json` with the local `schema.json` file using its JSON Schema settings.

## What the Schema Covers

- Full `Spec` object shape (`$schema`, `title`, `layout`, `root`, `elements`)
- All element fields (`type`, `props`, `children`, `action`, `visible`)
- Per-component props definitions (required vs optional fields, allowed values)
- Expression objects (`$data` and `$template` shapes)
- Custom plugin components registered in the app

## Schema Structure

Partial output to illustrate the shape:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Ferro JSON-UI Spec",
  "type": "object",
  "required": ["root", "elements"],
  "properties": {
    "$schema": { "type": "string" },
    "title": { "type": "string" },
    "layout": { "type": "string" },
    "root": { "type": "string" },
    "elements": {
      "type": "object",
      "additionalProperties": { "$ref": "#/definitions/Element" }
    }
  }
}
```

## Keeping the Schema Up to Date

Re-run `ferro json-ui:schema --output schema.json --pretty` when you:

- Add new custom plugins to the app
- Upgrade Ferro to a new version

The schema reflects the component catalog at build time, so it must be regenerated whenever the catalog changes.
