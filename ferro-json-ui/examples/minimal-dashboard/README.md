# minimal-dashboard

DX-01 pit-of-success proof for ferro-json-ui v2.

## What This Demonstrates

A complete ferro app that composes:

- `DashboardLayout` — persistent sidebar + header shell
- `PageHeader` — title with action button
- `StatCard` — three metric cards (clients, orders, revenue)
- `DataTable` — tabular list with status column and per-row actions
- `Card` + `Form` — a creation form with three inputs

All rendered with **zero custom CSS**. No `tokens.css` override, no
inline styles, no `.css` files of any kind. The default ferro-json-ui
tokens produce Linear/Attio-quality output out of the box.

## How to Run

```bash
cargo run -p minimal-dashboard
```

Then open <http://localhost:8099> in your browser.

Port 8099 is the default. Override with `SERVER_PORT=<port>`.

## Why This Exists

This is the DX-01 requirement from the v7.3 milestone: json-ui v2 *is* the
product. Any developer who picks up ferro and registers a layout should get
premium dashboard quality without designing anything. This example is both
the proof and the durable documentation of that guarantee.

## Zero Custom CSS

```
find ferro-json-ui/examples/minimal-dashboard -name "*.css"
# (no output — no CSS files exist)
```

The only styling comes from:
1. `ThemeMiddleware::default_theme()` — injects the default token set as a
   `<style>` block at render time.
2. `ferro-base.css` — served by the framework at `/_ferro/ferro-base.css`,
   containing the `@layer components` skin built entirely from token
   variables.

No application-level CSS is written or loaded.
