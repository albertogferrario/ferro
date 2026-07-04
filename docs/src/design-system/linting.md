# Linting Guide

The `ferro design:lint` command and the `design_lint` MCP tool run the same rule engine against JSON-UI specs. Neither rejects a spec at parse time nor affects rendering — findings are diagnostics only.

## CLI Usage

```bash
# Lint all JSON-UI specs under src/views/ (default path)
ferro design:lint

# Lint a specific file
ferro design:lint src/views/orders/index.json

# Machine-readable output (JSON array of findings)
ferro design:lint --json

# Non-zero exit on any Warning findings (CI gate)
ferro design:lint --deny

# Combine: JSON output and CI gate
ferro design:lint --json --deny
```

| Flag | Meaning |
|------|---------|
| `[path]` | File or directory to scan; defaults to `src/views/` |
| `--json` | Emit findings as a JSON array instead of human-readable text |
| `--deny` | Exit with code 1 if any Warning-severity finding is present |

## MCP Tool

Use `design_lint` inside an agent session to validate a spec before saving it. Provide exactly one of `spec_json` (inline JSON string) or `path` (path to a single file) — not both and not neither.

**Inline spec:**

```json
{
  "spec_json": "{\"$schema\":\"ferro-json-ui/v2\", ... }"
}
```

**File path:**

```json
{
  "path": "src/views/orders/index.json"
}
```

Combining `spec_lint` with `json_ui_validate_spec` gives full coverage: structural and catalog validity first, then design-pattern conformance.

## Output Shape

Both the CLI (`--json`) and the MCP tool return the same array of finding objects:

```json
[
  {
    "file": "src/views/orders/index.json",
    "rule": "page-header",
    "element_id": null,
    "severity": "Warning",
    "message": "Dashboard-family layout has no PageHeader element.",
    "suggestion": "Add a PageHeader element (with a `title` prop) as the first child of root."
  }
]
```

| Field | Type | Description |
|-------|------|-------------|
| `file` | string | File path, or `"<inline>"` for inline spec input |
| `rule` | string | Rule id (stable; use in `allow` lists) |
| `element_id` | string or null | Element that triggered the finding, if applicable |
| `severity` | `"Warning"` or `"Info"` | Warning trips `--deny`; Info is advisory only |
| `message` | string | Human-readable description of the finding |
| `suggestion` | string | Concrete remediation hint |

An empty array means the spec conforms to all applicable rules.

## Allowing a Rule

To suppress a finding for a specific spec, add the rule id to the `design.allow` array:

```json
{
  "$schema": "ferro-json-ui/v2",
  "root": "form",
  "layout": "auth",
  "elements": { "form": { "type": "Form" } },
  "design": {
    "intent": "collect",
    "allow": ["breadcrumb-on-subpages", "page-header"]
  }
}
```

See [Pattern Catalog](patterns.md) for the `allow` value for each rule.
