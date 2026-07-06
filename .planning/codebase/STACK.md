# Technology Stack

**Analysis Date:** 2026-01-15

## Languages

**Primary:**
- Rust (Edition 2021) - All application and framework code

**Secondary:**
- TypeScript 5.3 - Frontend application (`app/frontend/`)
- JavaScript - Build configuration

## Runtime

**Environment:**
- Tokio 1.x (full features) - Async runtime
- Hyper 1.x - HTTP server
- Node.js - Frontend build tooling

**Package Manager:**
- Cargo with workspace configuration - `Cargo.toml`
- npm 10.x - `app/frontend/package-lock.json`
- Lockfile: `Cargo.lock` and `package-lock.json` present

## Frameworks

**Core:**
- Hyper 1.x - HTTP server foundation (`framework/Cargo.toml`)
- SeaORM 1.0 - ORM with migrations (`framework/Cargo.toml`)
- Inertia.js - Server-side rendering bridge (`ferro-inertia/`)

**Frontend:**
- React 18.2 - UI framework (`app/frontend/package.json`)
- Vite 5.0 - Build tool and dev server (`app/frontend/vite.config.ts`)
- Inertia React 2.0 - SPA adapter (`app/frontend/package.json`)

**Testing:**
- Rust built-in `#[test]` and `#[tokio::test]`
- Pretty_assertions 1.4 - Enhanced test assertions
- Serial_test 3.x - Serial test execution

**Build/Dev:**
- Cargo - Rust build system
- Bacon - Code watcher (`bacon.toml`)
- Vite - Frontend bundling

## Key Dependencies

**Critical:**
- sea-orm 1.0 - Database ORM (`framework/Cargo.toml`)
- redis 0.25 - Cache and queue backend (`framework/Cargo.toml`, `ferro-queue/Cargo.toml`)
- tokio 1.x - Async runtime (`framework/Cargo.toml`)
- serde 1.x - Serialization (`framework/Cargo.toml`)

**Authentication & Security:**
- bcrypt 0.15 - Password hashing (`framework/Cargo.toml`)
- validator 0.18 - Input validation (`framework/Cargo.toml`)

**Infrastructure:**
- hyper 1.x - HTTP routing (`framework/Cargo.toml`)
- matchit 0.8 - Fast route matching (`framework/Cargo.toml`)
- lettre 0.11 - SMTP mail transport (`ferro-notifications/Cargo.toml`)
- tokio-tungstenite 0.26 - WebSocket support (`ferro-broadcast/Cargo.toml`)

**Macros & Code Generation:**
- proc-macro2, quote, syn 2.x - Procedural macros (`ferro-macros/Cargo.toml`)
- clap 4.x - CLI parsing (`ferro-cli/Cargo.toml`)

**Optional:**
- aws-sdk-s3 1.x - S3 file storage (feature-gated) (`ferro-storage/Cargo.toml`)
- moka 0.12 - In-memory cache (`ferro-cache/Cargo.toml`)

## Configuration

**Environment:**
- `.env` files loaded via dotenvy
- Key variables: `DATABASE_URL`, `REDIS_URL`, `MAIL_*`, `APP_URL`
- Configuration modules in `app/src/config/`

**Build:**
- `Cargo.toml` - Workspace and crate configuration
- `bacon.toml` - Development watch configuration
- `app/frontend/vite.config.ts` - Frontend build
- `app/frontend/tsconfig.json` - TypeScript configuration

## Platform Requirements

**Development:**
- macOS/Linux/Windows (any platform with Rust toolchain)
- PostgreSQL or SQLite for database
- Redis (optional, for cache/queue)
- Node.js for frontend build

**Production:**
- Rust compiled binary
- PostgreSQL (recommended) or SQLite
- Redis for distributed cache/queue
- Vite-built frontend assets in `app/public/assets/`

---

*Stack analysis: 2026-01-15*
*Update after major dependency changes*
