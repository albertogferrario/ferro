# `ferro generate-routes --json` — Stable JSON Schema

**Status:** Stable contract (Phase 124, D-10..D-12). Consumed by `ferro-mcp`
and external agents. Additive changes only; renames or removals are breaking
changes that require a major version bump.

## Command

```bash
ferro generate-routes --json
```

Without `--json`, `ferro generate-routes` writes a TypeScript route helper
file (default behavior, unchanged).

With `--json`, the command prints a single JSON document to **stdout** and
exits non-zero on error. No files are written.

## Schema

Expressed as TypeScript declarations matching the Rust types in
`ferro-cli/src/commands/generate_routes.rs` (`RoutesJson` / `RouteJson`):

```ts
interface RoutesJson {
  routes: RouteJson[];
}

interface RouteJson {
  /** Uppercase HTTP verb. */
  method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

  /** Path with `{param}` placeholders, e.g. "/users/{id}". */
  path: string;

  /** Fully-qualified handler, e.g. "controllers::user::show". */
  handler: string;

  /** Optional named route, e.g. "users.show". `null` if unnamed. */
  name: string | null;

  /**
   * Middleware names attached to this route. Always present.
   *
   * Phase 124 always emits `[]` — middleware parsing from `routes.rs` is
   * future work. The field is part of the stable contract so consumers can
   * rely on its presence today.
   */
  middleware: string[];
}
```

### Field stability

| Field        | Stability       | Notes                                         |
| ------------ | --------------- | --------------------------------------------- |
| `routes`     | stable          | Top-level array.                              |
| `method`     | stable          | Always uppercase.                             |
| `path`       | stable          | `{param}` placeholders preserved verbatim.    |
| `handler`    | stable          | `module::fn` joined with `::`.                |
| `name`       | stable          | `null` when no `.name(...)` call on route.    |
| `middleware` | stable, partial | Currently always `[]`; populated in a later phase. |

Path parameters are intentionally **not** emitted as a separate field —
consumers parse them from `path` themselves. Form-request bodies and
TypeScript-only artifacts are out of scope for the JSON contract.

## Example

A project with two routes:

```rust
get!("/users", controllers::user::index).name("users.index");
get!("/users/{id}", controllers::user::show).name("users.show");
```

Produces:

```json
{
  "routes": [
    {
      "method": "GET",
      "path": "/users",
      "handler": "controllers::user::index",
      "name": "users.index",
      "middleware": []
    },
    {
      "method": "GET",
      "path": "/users/{id}",
      "handler": "controllers::user::show",
      "name": "users.show",
      "middleware": []
    }
  ]
}
```

## Consumer hint: ferro-mcp (D-12)

`ferro-mcp` introspection tools should shell out to `ferro generate-routes
--json` and deserialize the result rather than parsing the human
pretty-printed output of `ferro routes`. The schema above is the contract.
