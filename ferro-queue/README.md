# ferro-queue

Background job queue system for the Ferro framework.

## Features

- Redis-backed job queues
- Job delays and retries with exponential backoff
- Multiple named queues
- Concurrent job processing
- Graceful shutdown
- Environment-based configuration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ferro-queue = "0.2"
```

Or use it through the main Ferro framework which re-exports all queue types.

## Configuration

### From Environment Variables

```rust
use ferro_queue::{Queue, QueueConfig};

// Load configuration from environment
let config = QueueConfig::from_env();
Queue::init(config).await?;
```

Environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `QUEUE_CONNECTION` | "sync" or "redis" | sync |
| `QUEUE_DEFAULT` | Default queue name | default |
| `QUEUE_PREFIX` | Redis key prefix | ferro_queue |
| `QUEUE_BLOCK_TIMEOUT` | Seconds to block waiting for jobs | 5 |
| `QUEUE_MAX_CONCURRENT` | Max concurrent jobs per worker | 10 |
| `REDIS_URL` | Full Redis URL (takes precedence) | - |
| `REDIS_HOST` | Redis host | 127.0.0.1 |
| `REDIS_PORT` | Redis port | 6379 |
| `REDIS_PASSWORD` | Redis password | - |
| `REDIS_DATABASE` | Redis database number | 0 |

### Programmatic Configuration

```rust
use ferro_queue::QueueConfig;
use std::time::Duration;

let config = QueueConfig::new("redis://localhost:6379")
    .default_queue("high-priority")
    .prefix("myapp")
    .max_concurrent_jobs(5)
    .block_timeout(Duration::from_secs(10));
```

## Defining Jobs

```rust
use ferro_queue::{Job, Queueable, Error};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SendEmail {
    to: String,
    subject: String,
    body: String,
}

#[async_trait]
impl Job for SendEmail {
    async fn handle(&self) -> Result<(), Error> {
        // Job logic here
        println!("Sending email to {}: {}", self.to, self.subject);
        Ok(())
    }

    fn max_retries(&self) -> u32 {
        3
    }

    fn retry_delay(&self, attempt: u32) -> std::time::Duration {
        // Exponential backoff
        std::time::Duration::from_secs(2u64.pow(attempt))
    }

    async fn failed(&self, error: &Error) {
        tracing::error!("Email job failed: {:?}", error);
    }
}
```

## Dispatching Jobs

```rust
// Dispatch immediately
SendEmail {
    to: "user@example.com".into(),
    subject: "Hello".into(),
    body: "Welcome!".into(),
}
.dispatch()
.await?;

// Dispatch with delay
SendEmail { /* ... */ }
    .delay(std::time::Duration::from_secs(60))
    .dispatch()
    .await?;

// Dispatch to specific queue
SendEmail { /* ... */ }
    .on_queue("emails")
    .dispatch()
    .await?;

// Combine options
SendEmail { /* ... */ }
    .delay(std::time::Duration::from_secs(300))
    .on_queue("high-priority")
    .dispatch()
    .await?;
```

## Running Workers

```rust
use ferro_queue::{Worker, WorkerConfig};

// Create worker for default queue
let worker = Worker::new(WorkerConfig::default());

// Register job handlers
worker.register::<SendEmail>();

// Run the worker (blocks until shutdown)
worker.run().await?;
```

## Sync Mode (Development)

For development, you can use sync mode which processes jobs immediately without Redis:

```env
QUEUE_CONNECTION=sync
```

Check if sync mode is enabled:

```rust
use ferro_queue::QueueConfig;

if QueueConfig::is_sync_mode() {
    // Jobs will be processed synchronously
}
```

## CLI Generator

Generate a new job with the CLI:

```bash
ferro make:job SendWelcomeEmail
```

This creates `src/jobs/send_welcome_email.rs` with boilerplate code.

## License

MIT
