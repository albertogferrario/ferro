# Queues & Background Jobs

Ferro provides a database-backed queue for processing jobs asynchronously. The queue uses the application's existing `DatabaseConnection` — no separate external queue server is needed. The `WorkerLoop` runs in-process inside `Application::run` and is started automatically when at least one job type is registered.

Atomic claim is dual-backend:

- **Postgres** — `SELECT … FOR UPDATE SKIP LOCKED` inside a transaction
- **SQLite** — raw `BEGIN IMMEDIATE` + `UPDATE … RETURNING`

Both paths claim exactly one job per cycle; two workers on the same table cannot double-claim a row.

## Setup

### Migration

Register `CreateJobsTable` in your application's `Migrator` alongside your own migrations:

```rust
use ferro_queue::CreateJobsTable;
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(CreateJobsTable),
            // ... your own migrations
        ]
    }
}
```

### Registration

Register job types in your bootstrap before the server starts. The framework's server boot path (inside `Application::run`) detects registered job types and spawns a `WorkerLoop` automatically — no separate process or CLI command required.

```rust
// src/bootstrap.rs
use ferro::queue::Queue;
use crate::jobs::{ProcessPayment, SendEmail, GenerateReport};

pub async fn register() {
    // Register job types — the framework auto-starts the WorkerLoop.
    Queue::register::<ProcessPayment>();
    Queue::register::<SendEmail>();
    Queue::register::<GenerateReport>();
}
```

### Environment Variables

```env
# Queue driver: "sync" for development (jobs run inline), any other value for background.
# IMPORTANT: when QUEUE_CONNECTION is UNSET it defaults to "sync" — background
# processing is OFF unless you set this to a non-sync value (e.g. "db").
QUEUE_CONNECTION=db

# Default queue name
QUEUE_DEFAULT=default

# Maximum concurrent jobs per worker instance
QUEUE_MAX_CONCURRENT=10
```

`QUEUE_CONNECTION` defaults to `sync` when unset. In sync mode jobs run inline during the HTTP request — no background worker, no database polling — and `.delay()` / `.on_queue()` are ignored. Set any other value (e.g. `db`) to enable background processing. If jobs are registered while the queue is in sync mode, the server logs a startup warning, since this combination is usually unintended in production.

## Creating Jobs

### Using the CLI

```bash
ferro make:job ProcessPayment
```

This creates `src/jobs/process_payment.rs`:

```rust
use ferro::queue::{Job, Error, async_trait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPayment {
    pub order_id: i64,
    pub amount: f64,
}

#[async_trait]
impl Job for ProcessPayment {
    async fn handle(&self) -> Result<(), Error> {
        tracing::info!("Processing payment for order {}", self.order_id);
        // Payment processing logic...
        Ok(())
    }

    fn max_retries(&self) -> u32 {
        3
    }
}
```

### Job Trait Methods

| Method | Description | Default |
|--------|-------------|---------|
| `handle()` | Job execution logic | Required |
| `name()` | Job identifier for logging | Type name |
| `max_retries()` | Retry attempts on failure | 3 |
| `retry_delay(attempt)` | Delay before retry | Full-jitter exponential (see below) |
| `timeout()` | Maximum execution time | 60 seconds |
| `failed(error)` | Called when all retries exhausted | Logs error |
| `idempotency_key()` | Deduplication key on enqueue | `None` |

### Retry Delay Default

The default `retry_delay` uses full-jitter exponential backoff: `rand(0..=min(cap, base × 2^attempt))` where base = 5 s and cap = 15 min. Override it on individual job types:

```rust
fn retry_delay(&self, attempt: u32) -> std::time::Duration {
    // Fixed 30-second delay regardless of attempt count.
    std::time::Duration::from_secs(30)
}
```

### Idempotency Keys

Provide `idempotency_key()` to prevent duplicate jobs when the same event fires more than once. Enqueue skips insertion when a `pending` or `claimed` row with the same `(job_type, idempotency_key)` already exists:

```rust
impl Job for SendInvoice {
    fn idempotency_key(&self) -> Option<String> {
        Some(format!("send-invoice-{}", self.invoice_id))
    }

    async fn handle(&self) -> Result<(), Error> {
        // Will only run once per invoice_id even if dispatched multiple times.
        Ok(())
    }
}
```

## Dispatching Jobs

### Basic Dispatch

```rust
use crate::jobs::ProcessPayment;

ProcessPayment {
    order_id: 123,
    amount: 99.99,
}
.dispatch()
.await?;
```

### With Delay

```rust
use std::time::Duration;

ProcessPayment { order_id: 123, amount: 99.99 }
    .delay(Duration::from_secs(300))  // Run after 5 minutes
    .dispatch()
    .await?;
```

### To Specific Queue

```rust
ProcessPayment { order_id: 123, amount: 99.99 }
    .on_queue("high-priority")
    .dispatch()
    .await?;
```

### Combining Options

```rust
ProcessPayment { order_id: 123, amount: 99.99 }
    .delay(Duration::from_secs(60))
    .on_queue("payments")
    .dispatch()
    .await?;
```

## WorkerLoop Configuration

The framework creates a `WorkerLoop` with `WorkerConfig::default()` when job types are registered. Override the configuration by calling `WorkerLoop::new(config)` directly if you need custom settings.

```rust
use ferro::queue::{WorkerConfig, WorkerLoop};
use std::time::Duration;

let config = WorkerConfig {
    queues: vec!["high-priority".into(), "default".into()],
    max_jobs: 20,
    sleep_duration: Duration::from_millis(500),
    visibility_timeout: Duration::from_secs(300), // 5 min default
    ..Default::default()
};
```

| Field | Description | Default |
|-------|-------------|---------|
| `queues` | Queue names to process, in priority order | `["default"]` |
| `max_jobs` | Maximum concurrent in-flight jobs | `10` |
| `sleep_duration` | Idle poll interval when queue is empty | `1s` |
| `visibility_timeout` | Time before a claimed job is reclaimed by the reaper | `300s` |

## CPU-Heavy Jobs

The `WorkerLoop` runs on the async executor. Jobs that do CPU-bound work (PDF rendering, image processing, compression) must wrap that work in `tokio::task::spawn_blocking` to avoid starving the executor of threads and blocking other jobs from running:

```rust
use ferro::queue::{Job, Error, async_trait};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderDocumentPdfJob {
    pub document_id: i64,
}

#[async_trait]
impl Job for RenderDocumentPdfJob {
    async fn handle(&self) -> Result<(), Error> {
        let document_id = self.document_id;

        // spawn_blocking moves CPU work off the async executor thread pool.
        tokio::task::spawn_blocking(move || {
            // CPU-intensive PDF rendering here...
            render_pdf(document_id)
        })
        .await
        .map_err(|e| Error::custom(format!("spawn_blocking join: {e}")))?
        .map_err(|e| Error::custom(format!("render_pdf: {e}")))
    }
}

fn render_pdf(_document_id: i64) -> Result<(), String> {
    // synchronous rendering work
    Ok(())
}
```

This applies to any job doing significant CPU work: document rendering, image resizing, compression, or large in-memory data transformations.

## Error Handling

### Automatic Retries

Failed jobs are automatically retried based on `max_retries()` and `retry_delay()`. After all retries are exhausted, the job is parked as `failed` with the error message recorded:

```rust
impl Job for ProcessPayment {
    fn max_retries(&self) -> u32 {
        5
    }

    async fn failed(&self, error: &Error) {
        tracing::error!(
            order_id = self.order_id,
            error = %error,
            "Payment processing permanently failed"
        );
        // Notify, update order status, etc.
    }
}
```

### Stuck Job Reaper

The worker runs a reaper before each claim cycle. Jobs that have been `claimed` for longer than the `visibility_timeout` (default 5 min) are:

- Reset to `pending` with `attempts + 1` if they have retries remaining
- Parked as `failed` if they have exhausted `max_retries`

This recovers from worker crashes without any manual intervention.

### Graceful Shutdown

On SIGTERM or Ctrl-C the worker stops claiming new jobs, waits for in-flight jobs to finish, and resets any `claimed` rows it held back to `pending` — those jobs will be claimed by the next worker instance.

## Failed Job Inspection

Failed jobs are stored in the `jobs` table with `status = 'failed'`. Inspect them via the debug endpoint or ferro-mcp:

```bash
# Debug endpoint (requires APP_ENV=local or DEBUG_MODE=true)
curl http://localhost:3000/_ferro/queue/stats
curl http://localhost:3000/_ferro/queue/jobs
```

## Migration Guide

The following table maps the previous external-broker API to the current DB-backed API.

| Old API | New (DB) | Notes |
|---------|----------|-------|
| `Queue::init(QueueConfig::new(broker_url))` | `Queue::register::<J>()` in bootstrap; framework auto-inits | Connection injected at bootstrap from the app DB |
| Separate worker process / `cargo run --bin worker` | `WorkerLoop` auto-started inside `Application::run` | Single binary, work-stealing |
| External broker env vars (`HOST`, `PORT`, `PASSWORD`) | None required | Queue uses the app's `DATABASE_URL` |
| `failed_jobs` table | `jobs WHERE status='failed'` | Single table, error recorded inline |
| `2^attempt` fixed backoff | Full-jitter exponential default | Override via `Job::retry_delay` |
| No deduplication hook | `Job::idempotency_key()` | Dedup on enqueue when `Some` |
| `QueueConnection` type | Removed | `Queue::connection()` returns `&DatabaseConnection` |

### Gestiscilo Consumer Migration (Phase 188)

The following job types migrate in gestiscilo Phase 188. Each keeps its `Job` implementation unchanged; only the registration and migration registration change:

| Job | Old registration | New registration |
|-----|-----------------|-----------------|
| `RenderDocumentPdfJob` | `worker.register::<RenderDocumentPdfJob>()` in worker binary | `Queue::register::<RenderDocumentPdfJob>()` in bootstrap |
| `SendBookingReminderJob` | `worker.register::<SendBookingReminderJob>()` in worker binary | `Queue::register::<SendBookingReminderJob>()` in bootstrap |
| `DeliverNotificationJob` | `worker.register::<DeliverNotificationJob>()` in worker binary | `Queue::register::<DeliverNotificationJob>()` in bootstrap |
| `screenshot_worker` | separate process binary | `Queue::register::<ScreenshotJob>()` in bootstrap |

Add `Box::new(ferro_queue::CreateJobsTable)` to your `Migrator::migrations()` list (one-time migration). The `failed_jobs` table (if present) can be dropped after migration — failed job history is now in `jobs WHERE status='failed'`.

## MCP Tools

Use these tools to monitor and debug queue state during development and in running applications.

### `list_jobs`

Returns all `Job` implementations found in `src/jobs/`, including the job struct name, max retries, and timeout configuration. Use this to audit what jobs exist before dispatching or debugging failures.

### `job_history`

Returns recent failed job history from `jobs WHERE status='failed'`: job name, error message, attempt count, and timestamp. Use this to diagnose jobs that are permanently failing.

### `queue_status`

Returns current queue depth and pending job counts per queue name. Use this to check whether a queue is backed up.
