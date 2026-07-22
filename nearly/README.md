# Nearly

A location-based, **deliberately chat-less** social app, built on the
[Ferro](../README.md) framework as a reference application.

Nearly shows the lively places in your city and the people around you on a
live map. The only thing you can send another person is a **trillo** — a
single, wordless "I noticed you, come say hi." There is no chat, no DMs, by
design: connections are meant to happen face to face.

See [`PRODUCT.md`](./PRODUCT.md) for the full product brief (vision, personas,
flows, scope) and [`ARCHITECTURE.md`](./ARCHITECTURE.md) for the engineering map.

## Run it

```bash
cd nearly
cp .env.example .env          # optional; sensible defaults are built in
cargo run -p nearly           # auto-migrates + seeds a demo city (Milan), then serves
# → http://localhost:8080
```

> Run from the `nearly/` directory: JSON-UI views are loaded from
> `src/views/*.json` relative to the working directory (framework convention).

The first boot seeds six people, five venues, and a pending trillo, so the map
is alive immediately.

**Demo login:** `alex@nearly.app` / `password123`

Other CLI commands: `cargo run -p nearly -- db:fresh` (re-seed),
`db:migrate`, `db:status`, `db:rollback [N]`.

## Screens & routes

| Route | Screen | Intent | Auth |
|-------|--------|--------|------|
| `GET /` | Splash / landing | `focus` | public |
| `GET /map` | Full-screen live map (Leaflet) | `track` | public |
| `GET /utenti/:id` | Person pop-up + "invia un trillo" | `focus` | public |
| `GET /places` | Trend + premium venues | `browse` | public |
| `GET/POST /login`, `/register` | Auth forms | `collect` | guest |
| `GET /trilli` · `POST /trilli` | Inbox · send | `browse` | user |
| `POST /trilli/:id/accept` · `/decline` | Respond | — | user |
| `GET/POST /account` | Edit profile | `collect` | user |
| `GET/POST /settings` | Visibility toggle + about | `collect` | user |

## Architecture (the Ferro way)

- **Projections are the core.** Each domain service (`profile`, `presence`,
  `trillo`, `place`) is a `ServiceDef` in `src/projections/` describing its
  fields and their semantic meaning; the framework derives intents from these.
- **Server-driven UI.** Every screen is a declarative JSON-UI view in
  `src/views/*.json` rendered by `JsonUi::render_file`; controllers assemble
  only the data a view needs. No frontend build step.
- **The map** uses Ferro's shipped Leaflet `Map` plugin. The `/map` handler
  joins visible profiles with their presences and emits colored markers
  (blue = people, gold = premium venues, green = trend) whose popups link to
  the person pop-up.
- **The trillo** is modeled with *no message field* — the ping is the whole
  payload. A test (`no_chat_surface`) guards this principle: no view may add a
  chat component or a message field.

## Data model

`users` · `profiles` (identity + `visible`) · `presences` (expiring lat/lng) ·
`trillos` (`pending`→`accepted`/`declined`) · `places` (`premium`).

## Tests

```bash
cargo test -p nearly
```

- `all_views_lint_clean` — every view declares a valid `design.intent` and
  passes the design linter.
- `projections_derive_intents` — each projection derives at least one intent.
- `no_chat_surface` — the product's no-messaging principle is enforced in code.

## What's intentionally out of v1

Any messaging/chat (a permanent product principle), real-time WebSocket
presence streaming (a v2 direction via `ferro-broadcast`), and native GPS
capture (the demo seeds/updates presence server-side).
</content>
