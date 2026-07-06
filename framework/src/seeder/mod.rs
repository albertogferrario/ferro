//! Database seeding for Ferro framework
//!
//! Provides a trait-based system for populating the database with test data,
//! similar to Laravel's database seeders.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::{async_trait, Seeder, FrameworkError};
//!
//! pub struct UsersSeeder;
//!
//! #[async_trait]
//! impl Seeder for UsersSeeder {
//!     async fn run(&self) -> Result<(), FrameworkError> {
//!         User::create()
//!             .set_name("Admin")
//!             .set_email("admin@example.com")
//!             .insert()
//!             .await?;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Registration
//!
//! Register seeders in `src/seeders/mod.rs`:
//!
//! ```rust,ignore
//! use ferro_rs::SeederRegistry;
//!
//! pub fn register() -> SeederRegistry {
//!     SeederRegistry::new()
//!         .add::<UsersSeeder>()
//!         .add::<SheltersSeeder>()
//!         .add::<AnimalsSeeder>()
//! }
//! ```

use crate::FrameworkError;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

/// Trait for database seeders
///
/// Implement this trait to create a seeder that populates the database
/// with test or initial data.
#[async_trait]
pub trait Seeder: Send + Sync + 'static {
    /// Run the seeder
    ///
    /// Receives a direct database connection. Use it for all DB operations,
    /// including `Migrator::fresh(db)` — this connection is non-pooled and
    /// works correctly with DDL operations.
    async fn run(&self, db: &DatabaseConnection) -> Result<(), FrameworkError>;

    /// Optional: Define seeders that must run before this one
    ///
    /// Override this to specify dependencies between seeders.
    /// The framework will ensure dependencies run first.
    fn depends_on(&self) -> Vec<&'static str> {
        vec![]
    }
}

/// Boxed seeder for dynamic dispatch
type BoxedSeeder = Box<dyn Seeder>;

/// Factory function to create a seeder instance
type SeederFactory = Box<dyn Fn() -> BoxedSeeder + Send + Sync>;

/// Entry in the seeder registry
struct SeederEntry {
    name: &'static str,
    factory: SeederFactory,
}

/// Registry for database seeders
///
/// Use this to register and run seeders in your application.
///
/// # Example
///
/// ```rust,ignore
/// pub fn register() -> SeederRegistry {
///     SeederRegistry::new()
///         .add::<UsersSeeder>()
///         .add::<AnimalsSeeder>()
/// }
/// ```
pub struct SeederRegistry {
    seeders: Vec<SeederEntry>,
}

impl SeederRegistry {
    /// Create a new empty seeder registry
    pub fn new() -> Self {
        Self {
            seeders: Vec::new(),
        }
    }

    /// Add a seeder to the registry
    ///
    /// Seeders are run in the order they are added.
    pub fn add<S>(mut self) -> Self
    where
        S: Seeder + Default,
    {
        let name = std::any::type_name::<S>();
        // Extract just the type name without the full path
        let short_name = name.rsplit("::").next().unwrap_or(name);

        self.seeders.push(SeederEntry {
            name: Box::leak(short_name.to_string().into_boxed_str()),
            factory: Box::new(|| Box::new(S::default())),
        });
        self
    }

    /// Add a seeder instance to the registry
    ///
    /// Use this when your seeder needs constructor arguments.
    pub fn add_instance<S>(mut self, seeder: S) -> Self
    where
        S: Seeder + Clone + 'static,
    {
        let name = std::any::type_name::<S>();
        let short_name = name.rsplit("::").next().unwrap_or(name);

        self.seeders.push(SeederEntry {
            name: Box::leak(short_name.to_string().into_boxed_str()),
            factory: Box::new(move || Box::new(seeder.clone())),
        });
        self
    }

    /// Get all registered seeder names
    pub fn names(&self) -> Vec<&'static str> {
        self.seeders.iter().map(|e| e.name).collect()
    }

    /// Run all registered seeders
    pub async fn run_all(&self, db: &DatabaseConnection) -> Result<(), FrameworkError> {
        if self.seeders.is_empty() {
            println!("No seeders registered.");
            return Ok(());
        }

        println!("Running database seeders...\n");

        for entry in &self.seeders {
            print!("  Seeding: {}...", entry.name);
            let seeder = (entry.factory)();

            match seeder.run(db).await {
                Ok(()) => println!(" done"),
                Err(e) => {
                    println!(" FAILED");
                    return Err(FrameworkError::database(format!(
                        "Seeder '{}' failed: {}",
                        entry.name, e
                    )));
                }
            }
        }

        println!("\nSeeding complete!");
        Ok(())
    }

    /// Run a specific seeder by name
    pub async fn run_one(&self, name: &str, db: &DatabaseConnection) -> Result<(), FrameworkError> {
        let entry = self
            .seeders
            .iter()
            .find(|e| e.name == name || e.name.ends_with(&format!("::{name}")))
            .ok_or_else(|| {
                FrameworkError::internal(format!(
                    "Seeder '{}' not found. Available: {:?}",
                    name,
                    self.names()
                ))
            })?;

        println!("Running seeder: {}", entry.name);
        let seeder = (entry.factory)();
        seeder.run(db).await?;
        println!("Seeder completed!");

        Ok(())
    }
}

impl Default for SeederRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for the main DatabaseSeeder that orchestrates all seeders
///
/// Implement this on your main seeder to define the seeding order.
#[async_trait]
pub trait DatabaseSeeder: Send + Sync {
    /// Register all seeders in execution order
    fn register(&self) -> SeederRegistry;

    /// Run all registered seeders
    async fn run(&self, db: &DatabaseConnection) -> Result<(), FrameworkError> {
        self.register().run_all(db).await
    }
}
