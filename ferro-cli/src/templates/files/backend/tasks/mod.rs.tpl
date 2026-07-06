//! Scheduled Tasks
//!
//! This module contains all scheduled task definitions.
//! Use `ferro make:task <name>` to generate new tasks.
//!
//! # Creating Tasks
//!
//! ```bash
//! ferro make:task CleanupLogs
//! ferro make:task SendReminders
//! ```
//!
//! # Example Task
//!
//! ```rust,ignore
//! use ferro::{ScheduledTask, CronExpression, FrameworkError};
//! use async_trait::async_trait;
//!
//! pub struct MyTask;
//!
//! impl MyTask {
//!     pub fn new() -> Self { Self }
//! }
//!
//! #[async_trait]
//! impl ScheduledTask for MyTask {
//!     fn name(&self) -> &str { "my:task" }
//!
//!     fn schedule(&self) -> CronExpression {
//!         CronExpression::daily_at("09:00")
//!     }
//!
//!     async fn handle(&self) -> Result<(), FrameworkError> {
//!         println!("Task running!");
//!         Ok(())
//!     }
//! }
//! ```

// Tasks will be added here by the make:task command
