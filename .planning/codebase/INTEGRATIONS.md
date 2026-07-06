# External Integrations

**Analysis Date:** 2026-01-15

## APIs & External Services

**Payment Processing:**
- Not detected - No payment integration currently implemented

**Email/SMS:**
- SMTP - Generic mail provider support via Lettre
  - SDK/Client: lettre 0.11 (`ferro-notifications/Cargo.toml`)
  - Auth: `MAIL_HOST`, `MAIL_PORT`, `MAIL_USERNAME`, `MAIL_PASSWORD` env vars
  - Configuration: `app/src/config/mail.rs`
  - Drivers supported: smtp, resend, sendgrid (`MAIL_DRIVER` env var)

**Webhook Notifications:**
- Slack - Webhook-based notifications
  - Integration: HTTP POST via reqwest (`ferro-notifications/src/dispatcher.rs`)
  - Auth: `SLACK_WEBHOOK_URL` env var

## Data Storage

**Databases:**
- PostgreSQL - Primary production database
  - Connection: `DATABASE_URL` env var
  - Client: SeaORM 1.0 (`framework/Cargo.toml`)
  - Migrations: SeaORM migrations in `app/src/migrations/`
  - Config: `DB_MAX_CONNECTIONS`, `DB_MIN_CONNECTIONS`, `DB_CONNECT_TIMEOUT`, `DB_LOGGING`

- SQLite - Development/testing database
  - Connection: `DATABASE_URL` env var (file path)
  - Client: SeaORM with sqlx-sqlite feature

**File Storage:**
- Local filesystem - Default storage driver
  - Client: Built-in (`ferro-storage/src/drivers/local.rs`)

- AWS S3 - Optional cloud storage (feature-gated)
  - SDK/Client: aws-sdk-s3 1.x, aws-config 1.x (`ferro-storage/Cargo.toml`)
  - Auth: Standard AWS credentials (IAM, env vars, profile)
  - Feature flag: `s3` feature in `ferro-storage`

**Caching:**
- Redis - Distributed cache backend
  - Connection: `REDIS_URL` or `REDIS_HOST`/`REDIS_PORT`/`REDIS_PASSWORD`/`REDIS_DATABASE`
  - Client: redis 0.25 with tokio-comp feature (`framework/Cargo.toml`)
  - Config: `framework/src/cache/config.rs`

- In-memory (Moka) - Local cache option
  - Client: moka 0.12 (`ferro-cache/Cargo.toml`)
  - Use case: Single-instance deployments

## Authentication & Identity

**Auth Provider:**
- Custom session-based authentication
  - Implementation: `framework/src/auth/` (guard, provider, authenticatable)
  - Token storage: Server-side sessions
  - Session drivers: Database or memory (`framework/src/session/driver/`)

**OAuth Integrations:**
- Not detected - No OAuth providers currently integrated

## Monitoring & Observability

**Error Tracking:**
- Not detected - No external error tracking service

**Analytics:**
- Not detected - No analytics service integrated

**Logs:**
- Tracing crate - Structured logging
  - Integration: tracing 0.1 across all crates
  - Output: stdout/stderr
  - No external log aggregation service

**Metrics:**
- Built-in metrics collection
  - Location: `framework/src/metrics/mod.rs`
  - Endpoints: `/_ferro/metrics` (debug mode only)

## CI/CD & Deployment

**Hosting:**
- Not specified - Framework is self-hostable binary

**CI Pipeline:**
- Not detected - No CI configuration files found
- Development: Bacon watch tool (`bacon.toml`)

## Environment Configuration

**Development:**
- Required env vars: `DATABASE_URL`
- Optional: `REDIS_URL`, `MAIL_*` variables
- Secrets location: `.env` file (gitignored)
- Mock/stub services: SQLite for database, in-memory cache

**Staging:**
- Not applicable - No staging environment configured

**Production:**
- Required: `DATABASE_URL`, `APP_URL`
- Optional: `REDIS_URL` for distributed cache/queue
- Mail: `MAIL_DRIVER`, `MAIL_HOST`, `MAIL_PORT`, etc.
- Debug: `FERRO_DEBUG_ENDPOINTS=true` to enable debug routes

## Webhooks & Callbacks

**Incoming:**
- Not detected - No webhook endpoints defined

**Outgoing:**
- Slack notifications - Webhook POST to `SLACK_WEBHOOK_URL`
  - Trigger: Via notification dispatcher (`ferro-notifications/src/dispatcher.rs`)
  - Events: Custom notifications routed to Slack channel

## Queue System

**Background Jobs:**
- Redis-backed queue
  - Connection: Same as cache (`REDIS_URL`)
  - Configuration: `QUEUE_CONNECTION`, `QUEUE_DEFAULT`, `QUEUE_PREFIX`, `QUEUE_BLOCK_TIMEOUT`, `QUEUE_MAX_CONCURRENT`
  - Location: `ferro-queue/src/`
  - Worker: `ferro-queue/src/worker.rs`

## WebSocket/Real-time

**Broadcasting:**
- Tokio-tungstenite WebSocket
  - Location: `ferro-broadcast/src/`
  - Config: `ferro-broadcast/src/config.rs`
  - Use case: Real-time updates to frontend

---

*Integration audit: 2026-01-15*
*Update when adding/removing external services*
