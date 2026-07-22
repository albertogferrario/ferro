# Nearly — Architecture

Nearly is a standard Ferro application with an Inertia.js + React frontend.
This document maps the code to the framework's patterns.

## Module map

```
nearly
├── src                     # Rust backend
│   ├── main.rs             # entry point: serve + db:* subcommands
│   ├── bootstrap.rs        # DB init, session + ShareInertiaData middleware, demo seed
│   ├── middleware/         # logging, auth/guest helpers, ShareInertiaData
│   ├── models/             # SeaORM entities + query helpers
│   ├── migrations/         # one migration per table (+ sessions)
│   ├── projections/        # ServiceDef per domain service — the core abstraction
│   ├── controllers/        # one module per screen; assemble props, Inertia::render
│   ├── routes.rs           # route table with guest/auth middleware groups
│   └── tests/              # projection intents, presence freshness, no-chat guard
├── frontend                # Inertia + React (Vite, TypeScript)
│   └── src
│       ├── main.tsx        # createInertiaApp — resolves ./pages/*
│       ├── styles.css      # the design system (brand tokens, shell, pins)
│       ├── Layout.tsx      # app shell: header + bottom tab bar
│       └── pages/          # one component per screen
└── public                  # Vite build output (git-ignored): manifest + assets
```

## Request lifecycle

1. A route maps method + path to a controller. `SessionMiddleware` and
   `ShareInertiaData` (globals) wrap every request; guest/auth groups gate access.
2. The controller queries models and returns `Inertia::render(&req, "Page", props)`.
3. The framework performs content negotiation:
   - **First load / full navigation** → full HTML document. In development it
     emits the Vite dev-server script tags (`@vite/client` + entry); in
     production it reads `public/.vite/manifest.json` and emits the hashed
     `/assets/*` tags. The `<div id="app" data-page="…">` carries the page JSON.
   - **Inertia XHR** (`X-Inertia` header) → a JSON `{component, props, url,
     version}` body; React swaps the page without a full reload.
4. `ShareInertiaData` merges the auth user + CSRF token into every page's props,
   so the shell (`Layout.tsx`) can render nav/logout and forms can post safely.

## Asset contract (production)

`vite.config.ts` builds to `../public` with `manifest: true`:

- manifest → `public/.vite/manifest.json` (the framework default `manifest_path`)
- assets   → `public/assets/*` → served at `/assets/*` (immutable cache)

`InertiaConfig::from_env()` selects dev vs prod from `APP_ENV`
(`production`/`staging` ⇒ production asset resolution).

## Projection / intent

`src/projections/*.rs` return a `ServiceDef` describing each service's fields and
their `FieldMeaning`. `ferro_projections::derive_intents(&svc)` scores a service
into the seven intents (`browse`, `focus`, `collect`, `process`, `summarize`,
`analyze`, `track`). These stay as backend truth for introspection even though
rendering is now React.

## The map data flow

`GET /map` (`controllers/home.rs`):

1. Load visible profiles, all presences, all places.
2. Join profiles↔presences by `user_id`, keeping only **fresh** presences
   (`Presence::is_fresh`) so stale positions expire off the map.
3. Emit `people` and `places` arrays as props.
4. `pages/Map.tsx` renders a react-leaflet map with custom `DivIcon` pins and
   pop-ups; person pop-ups link to `/utenti/:id`; a check-in button posts to
   `/presence/checkin`.

## The trillo (no-chat) design

A `trillo` row is `{from_user_id, to_user_id, status, created_at}` — no message
column. Sending posts only a hidden `to_user_id`; responding is
`accept`/`decline`. The `no_chat_surface` test fails the build if the trillo
projection grows a message field or any React page adds a chat component or a
message input.

## Real-time (WebSocket)

The framework hosts the socket (`/_ferro/ws`) and resolves the `Broadcaster`
registered in `bootstrap` (`App::singleton`). Two channels:

- **`nearby`** (public) — `controllers/presence.rs` calls `realtime::emit` on
  every `POST /presence` and check-in, sending `PresenceUpdated`. `Map.tsx`
  seeds `people` from the server render, then upserts on each event via the
  `useChannel` hook — pins appear/move with no reload.
- **`private-user.{id}`** (private, signed) — `controllers/trilli.rs` emits
  `TrilloReceived` to the recipient. `Layout.tsx` subscribes to the current
  user's channel and flashes a toast.

`NearlyChannelAuth` (in `bootstrap`) authorizes only a user's own private
channel; `/broadcasting/auth` then HMAC-signs the subscription (`BROADCAST_SECRET`)
so the socket can't be forged. The browser flow lives in `frontend/src/useChannel.ts`.

## Extending

Add a feature the same way each vertical was built:
`migration → model → projection → controller → React page → route`, then keep
the projection-intent and no-chat guards green.
