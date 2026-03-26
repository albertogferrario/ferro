# Queues & Background Jobs

Ferro provides a Redis-backed queue system for processing jobs asynchronously. This is essential for handling time-consuming tasks like sending emails, processing uploads, or generating reports without blocking HTTP requests.

## Configuration

### Environment Variables

Configure queues in your `.env` file:

```env
# Queue driver: "sync" for development, "redis" for production
QUEUE_CONNECTION=sync

# Default queue name
QUEUE_DEFAULT=default

# Redis connection
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DATABASE=0
```

### Bootstrap Setup

In `src/bootstrap.rs`, initialize the queue system:

```rust
use ferro::{Queue, QueueConfig};

pub async fn register() {
    // ... other setup ...

    // Initialize queue (for production with Redis)
    if !QueueConfig::is_sync_mode() {
        let config = QueueConfig::from_env();
        Queue::init(config).await.expect("Failed to initialize queue");
    }
}
```

## Creating Jobs

### Using the CLI

Generate a new job:

```bash
ferro make:job ProcessPayment
```

This creates `src/jobs/process_payment.rs`:

```rust
use ferro::{Job, Error, async_trait};
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

    fn retry_delay(&self, attempt: u32) -> std::time::Duration {
        // Exponential backoff: 2s, 4s, 8s...
        std::time::Duration::from_secs(2u64.pow(attempt))
    }
}
```

### Job Trait Methods

| Method | Description | Default |
|--------|-------------|---------|
| `handle()` | Job execution logic | Required |
| `name()` | Job identifier for logging | Type name |
| `max_retries()` | Retry attempts on failure | 3 |
| `retry_delay(attempt)` | Delay before retry | 5 seconds |
| `timeout()` | Maximum execution time | 60 seconds |
| `failed(error)` | Called when all retries exhausted | Logs error |

## Dispatching Jobs

### Basic Dispatch

```rust
use crate::jobs::ProcessPayment;

// In a controller or service
ProcessPayment {
    order_id: 123,
    amount: 99.99,
}
.dispatch()
.await?;
```

### With Delay

Process the job after a delay:

```rust
use std::time::Duration;

ProcessPayment { order_id: 123, amount: 99.99 }
    .delay(Duration::from_secs(60))  // Wait 1 minute
    .dispatch()
    .await?;
```

### To Specific Queue

Route jobs to different queues for priority handling:

```rust
ProcessPayment { order_id: 123, amount: 99.99 }
    .on_queue("high-priority")
    .dispatch()
    .await?;
```

### Combining Options

```rust
ProcessPayment { order_id: 123, amount: 99.99 }
    .delay(Duration::from_secs(300))  // 5 minute delay
    .on_queue("payments")
    .dispatch()
    .await?;
```

## Running Workers

### Development

For development, use sync mode (`QUEUE_CONNECTION=sync`) which processes jobs immediately during the HTTP request.

### Production

Run a worker process to consume jobs from Redis:

```rust
// src/bin/worker.rs
use ferro::{Worker, WorkerConfig};
use myapp::jobs::{ProcessPayment, SendEmail, GenerateReport};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize app (loads .env, connects to Redis)
    myapp::bootstrap::register().await;

    let worker = Worker::new(WorkerConfig {
        queue: "default".into(),
        ..Default::default()
    });

    // Register all job types this worker handles
    worker.register::<ProcessPayment>();
    worker.register::<SendEmail>();
    worker.register::<GenerateReport>();

    // Run forever (handles graceful shutdown)
    worker.run().await?;

    Ok(())
}
```

Run with:

```bash
cargo run --bin worker
```

### Multiple Queues

Run separate workers for different queues:

```bash
# High priority worker
QUEUE_NAME=high-priority cargo run --bin worker

# Default queue worker
cargo run --bin worker

# Email-specific worker
QUEUE_NAME=emails cargo run --bin worker
```

## Error Handling

### Automatic Retries

Failed jobs are automatically retried based on `max_retries()` and `retry_delay()`:

```rust
impl Job for ProcessPayment {
    fn max_retries(&self) -> u32 {
        5  // Try 5 times total
    }

    fn retry_delay(&self, attempt: u32) -> Duration {
        // Exponential backoff with jitter
        let base = Duration::from_secs(2u64.pow(attempt));
        let jitter = Duration::from_millis(rand::random::<u64>() % 1000);
        base + jitter
    }
}
```

### Failed Job Handler

Handle permanent failures:

```rust
impl Job for ProcessPayment {
    async fn failed(&self, error: &Error) {
        tracing::error!(
            order_id = self.order_id,
            error = ?error,
            "Payment processing permanently failed"
        );

        // Notify admins, update order status, etc.
    }
}
```

## Best Practices

1. **Keep jobs small** - Jobs should do one thing well
2. **Make jobs idempotent** - Safe to run multiple times
3. **Use appropriate timeouts** - Set `timeout()` based on expected duration
4. **Handle failures gracefully** - Implement `failed()` for cleanup
5. **Use dedicated queues** - Separate critical jobs from bulk processing
6. **Monitor queue depth** - Alert on growing backlogs

## Environment Variables Reference

| Variable | Description | Default |
|----------|-------------|---------|
| `QUEUE_CONNECTION` | "sync" or "redis" | sync |
| `QUEUE_DEFAULT` | Default queue name | default |
| `QUEUE_PREFIX` | Redis key prefix | ferro_queue |
| `QUEUE_BLOCK_TIMEOUT` | Worker polling timeout (seconds) | 5 |
| `QUEUE_MAX_CONCURRENT` | Max parallel jobs per worker | 10 |
| `REDIS_URL` | Full Redis URL (overrides individual settings) | - |
| `REDIS_HOST` | Redis server host | 127.0.0.1 |
| `REDIS_PORT` | Redis server port | 6379 |
| `REDIS_PASSWORD` | Redis password | - |
| `REDIS_DATABASE` | Redis database number | 0 |

## MCP Tools

Use these tools to monitor and debug queue state during development and in running applications.

### `list_jobs`

Returns all `Job` implementations found in `src/jobs/`, including the job struct name, max retries, and timeout configuration. Use this to audit what jobs exist before dispatching or debugging failures.

### `job_history`

Returns recent job execution history from Redis: job name, status (completed, failed, retrying), attempt count, and timestamp. Use this to diagnose jobs that are silently failing or retrying excessively.

### `queue_status`

Returns current queue depth, active workers, and pending job counts per queue name. Use this to check whether a queue is backed up or whether workers are running.
