# Nearly

A location-based, **deliberately chat-less** social app, built on the
[Ferro](../README.md) framework as a reference application.

Nearly shows the lively places in your city and the people around you on a
live map. The only thing you can send another person is a **trillo** — a
single, wordless "I noticed you, come say hi." There is no chat, no DMs, by
design: connections are meant to happen face to face.

The UI is **Inertia.js + React** (see [`frontend/`](./frontend)); the Rust
backend owns routing, data, and the domain **projections**. See
[`PRODUCT.md`](./PRODUCT.md) for the product brief and
[`ARCHITECTURE.md`](./ARCHITECTURE.md) for the engineering map.

## Run it

Two processes: the Rust server and the Vite build/dev server.

**Quick look (production build):**

```bash
cd nearly/frontend && npm install && npm run build   # → ../public/{.vite/manifest.json, assets/*}
cd .. && APP_ENV=production cargo run -p nearly       # auto-migrates + seeds, serves built assets
# → http://localhost:8080
```

**Live development (HMR):**

```bash
cd nearly/frontend && npm install && npm run dev      # Vite dev server on :5173
cd .. && cargo run -p nearly                          # APP_ENV defaults to dev → uses the Vite server
```

> Run the server from the `nearly/` directory: the Vite manifest and static
> assets are resolved relative to the working directory (`public/…`).

The first boot seeds six people, five venues, and a pending trillo, so the map
is alive immediately. **Demo login:** `alex@nearly.app` / `password123`.

Other CLI commands: `cargo run -p nearly -- db:fresh` (re-seed),
`db:migrate`, `db:status`, `db:rollback [N]`.

## Screens

| Route | React page | Auth |
|-------|-----------|------|
| `GET /` | `Splash` | public |
| `GET /map` | `Map` (react-leaflet, animated pins) | public |
| `GET /utenti/:id` | `User` (pop-up + "invia un trillo") | public |
| `GET /places` | `Places` | public |
| `/login`, `/register` | `auth/Login`, `auth/Register` | guest |
| `/trilli` · `POST /trilli` | `Trilli` inbox · send | user |
| `POST /trilli/:id/accept|decline` | respond | user |
| `POST /presence` · `/presence/checkin` | update location · "I'm still here" | user |
| `/account`, `/settings` | `Account`, `Settings` | user |

## Architecture (the Ferro way)

- **Projections are the core.** Each domain service (`profile`, `presence`,
  `trillo`, `place`) is a `ServiceDef` in `src/projections/` describing its
  fields and their semantic meaning; the framework derives intents from these.
  They remain backend truth even though the UI is React.
- **Inertia bridge.** Controllers return `Inertia::render(&req, "Page", props)`.
  The framework emits the root HTML (Vite dev tags in development, hashed
  manifest assets in production) and hands React the page + props;
  `ShareInertiaData` adds the auth user + CSRF to every response.
- **The map** is a react-leaflet `MapContainer` with custom animated `DivIcon`
  pins (blue people, gold premium venues, green trend), pop-ups linking to the
  person page, and a floating check-in button.
- **The trillo** carries *no message field* — the ping is the whole payload. A
  test (`no_chat_surface`) fails the build if the trillo projection grows a
  message field or any React page adds a chat component / message input.

## Data model

`users` · `profiles` (identity + `visible`) · `presences` (expiring lat/lng) ·
`trillos` (`pending`→`accepted`/`declined`) · `places` (`premium`).

## Tests

```bash
cargo test -p nearly
```

- `projections_derive_intents` — each projection derives at least one intent.
- `presence_freshness` — presence expires after the TTL (stale pins drop off the map).
- `no_chat_surface` — the no-messaging principle is enforced in code.

## What's intentionally out of v1

Any messaging/chat (a permanent product principle), real-time WebSocket
presence streaming (a v2 direction via `ferro-broadcast`), and native GPS
capture (the demo seeds/updates presence server-side).
