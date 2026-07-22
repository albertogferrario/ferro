# Nearly — Architecture

Nearly is a standard Ferro application. This document maps the code to the
framework's patterns so a new contributor (human or agent) can navigate it.

## Module map

```
nearly/src
├── main.rs            entry point: serve + db:* subcommands
├── bootstrap.rs       DB init, global middleware, app-shell layout, demo seed
├── config/            registers DatabaseConfig from env
├── middleware/        logging + auth/guest helpers
├── models/            SeaORM entities + query helpers (user, profile, presence, trillo, place)
├── migrations/        one migration per table (+ sessions)
├── projections/       ServiceDef per domain service — the core abstraction
├── controllers/       one module per screen; assemble data, render a view
├── views/*.json       declarative JSON-UI specs (the entire UI)
├── routes.rs          route table with guest/auth middleware groups
└── tests/             design-lint gate, projection intents, no-chat guard
```

## Request lifecycle

1. A route in `routes.rs` maps a method + path to a controller handler.
   `SessionMiddleware` (global) wraps every request; guest/auth groups gate
   access and redirect.
2. The handler queries models (`Model::query().filter(...).all()`), shapes a
   small `serde_json` payload, and calls `JsonUi::render_file("src/views/X.json", data)`.
3. The framework loads the cached spec, expands directives, resolves `$data`
   expressions (injecting the handler's data into element props), resolves
   action handlers to URLs, applies the active theme, and renders HTML inside
   the declared layout (`auth` for sign-in, `dashboard` for the app shell).

## Projection / intent

`src/projections/*.rs` each return a `ServiceDef` describing a service's fields
and their `FieldMeaning` (e.g. `EntityName`, `Status`, `latitude`). This is the
framework's central abstraction: `ferro_projections::derive_intents(&svc)`
turns a service into scored intents. The seven intents (`browse`, `focus`,
`collect`, `process`, `summarize`, `analyze`, `track`) are the same archetypes
each view declares in `design.intent`, so a view's structure and its service's
derived intent stay aligned (guarded by the design linter).

## The map data flow

`GET /map` (`controllers/home.rs`):

1. Load visible profiles, all presences, all places.
2. Join profiles↔presences by `user_id` in memory.
3. Emit a `markers` array: people (blue) with a popup linking to
   `/utenti/:id`; places (gold if premium, else green) with a category popup.
4. Render `views/map.json`, whose `Map` element pulls `center` and `markers`
   from handler data via `{"$data": "/…"}`. Ferro's Leaflet `Map` plugin
   renders the interactive map and injects the Leaflet CSS/JS assets.

## The trillo (no-chat) design

A `trillo` row is `{from_user_id, to_user_id, status, created_at}` — **no
message column**. Sending is a POST with only a hidden `to_user_id`; responding
is `accept`/`decline`. The `no_chat_surface` test fails the build if any view
introduces a `Chat` component or a `message` field, keeping the product
principle enforceable rather than aspirational.

## Extending

Add a feature the same way each existing vertical was built:
`migration → model → projection → controller → view → route`, then keep the
design linter and the no-chat guard green.
</content>
