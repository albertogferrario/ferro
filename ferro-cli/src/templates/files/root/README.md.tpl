# {project_title}

{description}

Built with the [Ferro](https://github.com/albertogferrario/ferro) web framework.

## Prerequisites

- **Rust** (stable, 1.75+) — install via [rustup](https://rustup.rs)
- **Ferro CLI** — `cargo install ferro-cli` (or build from source)
- **Node.js** 20+ and **npm** — for the frontend (Vite + Inertia)
- **SQLite** is used by default; no extra install needed. For PostgreSQL, set `DATABASE_URL` accordingly.

Run `ferro doctor` to verify your environment.

## Quick start

```bash
git clone <this-repo-url>
cd {project_name}

# 1. Environment
cp .env.example .env

# 2. Backend deps build on first run
# 3. Frontend deps
(cd frontend && npm install)

# 4. Database — migrate and (optionally) seed
ferro db:migrate
ferro db:seed        # optional: populate demo data

# 5. Run
ferro serve
```

The backend is served at <http://localhost:8080>. The Vite dev server runs at <http://localhost:5173>.

## Common commands

| Command              | Purpose                                   |
| -------------------- | ----------------------------------------- |
| `ferro serve`        | Start dev server with hot reload          |
| `ferro routes`       | List all registered HTTP routes           |
| `ferro db:migrate`   | Apply pending database migrations         |
| `ferro db:rollback`  | Revert the last batch of migrations       |
| `ferro db:fresh`     | Drop all tables and re-run migrations     |
| `ferro db:seed`      | Run seeders                               |
| `ferro make:*`       | Scaffold controllers, models, migrations… |
| `cargo test`         | Run the Rust test suite                   |
| `cargo fmt && cargo clippy` | Format and lint before committing  |

Run `ferro --help` for the full command list.

## Project layout

```
src/
  main.rs           # Binary entry point
  routes.rs         # HTTP route registration
  bootstrap.rs      # Application wiring
  controllers/      # HTTP handlers
  actions/          # Business logic units
  models/           # SeaORM models
  migrations/       # Database migrations
  middleware/       # HTTP middleware
  events/ listeners/ jobs/ notifications/ tasks/
  seeders/ factories/
frontend/
  src/pages/        # Inertia pages (React)
  src/layouts/      # Layout components
lang/               # Translation files
storage/            # Runtime artifacts (gitignored)
```

## Configuration

All runtime configuration is in `.env`. See `.env.example` for every supported variable with inline documentation. Never commit `.env`.

## Troubleshooting

- **`ferro: command not found`** — install with `cargo install ferro-cli`.
- **Migrations fail** — delete `database.db` and run `ferro db:fresh`.
- **Frontend assets missing** — run `npm install` inside `frontend/`, then restart `ferro serve`.
- **TypeScript errors about `Cannot find module './types/inertia-props'`** — run `cargo run` once to generate types before running `npm run dev`. Types are regenerated automatically on each server start. See the framework docs page `cli/frontend-types.md` for the full convention.
- **Port 8080 in use** — change `SERVER_PORT` in `.env`.

For framework-level issues, see the [Ferro docs](https://github.com/albertogferrario/ferro).
