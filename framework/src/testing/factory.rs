//! Database factories for generating fake test data
//!
//! Provides Rails/Laravel-like model factories with database persistence.
//!
//! # Basic Usage
//!
//! ```rust,ignore
//! use ferro_rs::testing::{Factory, Fake};
//!
//! // Define a factory for your model
//! impl Factory for User {
//!     fn definition() -> Self {
//!         User {
//!             id: 0, // Will be set by database
//!             name: Fake::name(),
//!             email: Fake::email(),
//!             created_at: Fake::datetime(),
//!         }
//!     }
//! }
//!
//! // Make without persisting
//! let user = User::factory().make();
//!
//! // Create with database persistence
//! let user = User::factory().create().await?;
//!
//! // Create multiple
//! let users = User::factory().count(5).create_many().await?;
//! ```
//!
//! # Factory Traits (Named States)
//!
//! ```rust,ignore
//! impl Factory for User {
//!     fn definition() -> Self { /* ... */ }
//!
//!     fn traits() -> FactoryTraits<Self> {
//!         FactoryTraits::new()
//!             .define("admin", |u| u.role = "admin".into())
//!             .define("verified", |u| u.verified_at = Some(Fake::datetime()))
//!             .define("unverified", |u| u.verified_at = None)
//!     }
//! }
//!
//! // Use traits
//! let admin = User::factory().trait_("admin").create().await?;
//! let verified_admin = User::factory()
//!     .trait_("admin")
//!     .trait_("verified")
//!     .create()
//!     .await?;
//! ```
//!
//! # Callbacks
//!
//! ```rust,ignore
//! let user = User::factory()
//!     .after_make(|u| println!("Made user: {}", u.name))
//!     .after_create(|u| {
//!         // Create related records
//!         Profile::factory()
//!             .state(|p| p.user_id = u.id)
//!             .create()
//!             .await
//!     })
//!     .create()
//!     .await?;
//! ```
//!
//! # Associations
//!
//! Use `set()` for belongs_to relationships:
//!
//! ```rust,ignore
//! // Create a user first, then a post belonging to that user
//! let user = User::factory().create().await?;
//!
//! let post = Post::factory()
//!     .set(user.id, |p, user_id| p.user_id = user_id)
//!     .create()
//!     .await?;
//!
//! // Create multiple posts for the same user (has_many)
//! let posts = Post::factory()
//!     .count(5)
//!     .set(user.id, |p, user_id| p.user_id = user_id)
//!     .create_many()
//!     .await?;
//! ```
//!
//! Use `after_create` for creating child records:
//!
//! ```rust,ignore
//! let user = User::factory()
//!     .after_create(|user| async move {
//!         // Create related profile
//!         Profile::factory()
//!             .set(user.id, |p, id| p.user_id = id)
//!             .create()
//!             .await?;
//!         Ok(())
//!     })
//!     .create()
//!     .await?;
//! ```

use crate::database::DB;
use crate::error::FrameworkError;
use async_trait::async_trait;
use rand::Rng;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EntityTrait, IntoActiveModel};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Trait for models that can be created via factories
pub trait Factory: Sized + Clone + Send + 'static {
    /// Define the default state of the model
    fn definition() -> Self;

    /// Define named traits (states) for this factory
    fn traits() -> FactoryTraits<Self> {
        FactoryTraits::new()
    }

    /// Create a factory builder for this model
    fn factory() -> FactoryBuilder<Self> {
        FactoryBuilder::new()
    }
}

/// Trait for models that can be persisted to database via factories
///
/// Implement this trait for SeaORM entities to enable `create()` and `create_many()`.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::testing::{Factory, DatabaseFactory, Fake};
/// use sea_orm::ActiveValue::Set;
///
/// impl Factory for user::Model {
///     fn definition() -> Self {
///         Self {
///             id: 0,
///             name: Fake::name(),
///             email: Fake::email(),
///             created_at: chrono::Utc::now().naive_utc(),
///         }
///     }
/// }
///
/// impl DatabaseFactory for user::Model {
///     type Entity = user::Entity;
///     type ActiveModel = user::ActiveModel;
///
///     fn to_active_model(model: Self) -> Self::ActiveModel {
///         user::ActiveModel {
///             name: Set(model.name),
///             email: Set(model.email),
///             ..Default::default()
///         }
///     }
/// }
/// ```
/// Trait for models that can be persisted to database via factories
///
/// Implement this trait for SeaORM entities to enable `create()` and `create_many()`.
#[async_trait]
pub trait DatabaseFactory: Factory + IntoActiveModel<Self::ActiveModel> {
    /// The SeaORM entity type
    type Entity: EntityTrait<Model = Self>;
    /// The SeaORM active model type
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity> + ActiveModelBehavior + Send;

    /// Insert a model into the database
    async fn insert(model: Self) -> Result<Self, FrameworkError>
    where
        Self: Sized,
    {
        let db = DB::get()?;
        let active_model: Self::ActiveModel = model.into_active_model();
        let result = active_model.insert(db.inner()).await.map_err(|e| {
            FrameworkError::internal(format!("Failed to insert factory model: {e}"))
        })?;
        Ok(result)
    }
}

/// Type alias for state modifier function
type StateModifier<T> = Arc<dyn Fn(&mut T) + Send + Sync>;

/// Type alias for after-make callback
type AfterMakeCallback<T> = Arc<dyn Fn(&T) + Send + Sync>;

/// Collection of named traits (states) for a factory
pub struct FactoryTraits<T> {
    traits: HashMap<&'static str, StateModifier<T>>,
}

impl<T> FactoryTraits<T> {
    /// Create a new empty traits collection
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
        }
    }

    /// Define a named trait
    pub fn define<F>(mut self, name: &'static str, f: F) -> Self
    where
        F: Fn(&mut T) + Send + Sync + 'static,
    {
        self.traits.insert(name, Arc::new(f));
        self
    }

    /// Get a trait by name
    pub fn get(&self, name: &str) -> Option<StateModifier<T>> {
        self.traits.get(name).cloned()
    }
}

impl<T> Default for FactoryTraits<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for async after_create callbacks
type AfterCreateCallback<T> =
    Box<dyn Fn(T) -> Pin<Box<dyn Future<Output = Result<(), FrameworkError>> + Send>> + Send>;

/// Builder for creating model instances with customizations
pub struct FactoryBuilder<T: Factory> {
    count: usize,
    states: Vec<StateModifier<T>>,
    trait_names: Vec<&'static str>,
    after_make_callbacks: Vec<AfterMakeCallback<T>>,
    after_create_callbacks: Vec<AfterCreateCallback<T>>,
}

impl<T: Factory> FactoryBuilder<T> {
    /// Create a new factory builder
    pub fn new() -> Self {
        Self {
            count: 1,
            states: Vec::new(),
            trait_names: Vec::new(),
            after_make_callbacks: Vec::new(),
            after_create_callbacks: Vec::new(),
        }
    }

    /// Set the number of models to create
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Apply a state transformation to the model
    pub fn state<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut T) + Send + Sync + 'static,
    {
        self.states.push(Arc::new(f));
        self
    }

    /// Set a field value using a setter function
    ///
    /// This is useful for setting foreign keys when creating associated models.
    /// The value is cloned for each model when creating multiple instances.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Create a user first, then create posts belonging to that user
    /// let user = User::factory().create().await?;
    ///
    /// let post = Post::factory()
    ///     .set(user.id, |p, user_id| p.user_id = user_id)
    ///     .create()
    ///     .await?;
    ///
    /// // Create multiple posts for the same user
    /// let posts = Post::factory()
    ///     .count(5)
    ///     .set(user.id, |p, user_id| p.user_id = user_id)
    ///     .create_many()
    ///     .await?;
    /// ```
    pub fn set<V, F>(mut self, value: V, setter: F) -> Self
    where
        V: Clone + Send + Sync + 'static,
        F: Fn(&mut T, V) + Send + Sync + 'static,
    {
        self.states
            .push(Arc::new(move |m| setter(m, value.clone())));
        self
    }

    /// Apply a named trait (state) defined in the Factory
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let admin = User::factory()
    ///     .trait_("admin")
    ///     .trait_("verified")
    ///     .create()
    ///     .await?;
    /// ```
    pub fn trait_(mut self, name: &'static str) -> Self {
        self.trait_names.push(name);
        self
    }

    /// Add a callback to run after making (but before persisting)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user = User::factory()
    ///     .after_make(|u| println!("Made user: {}", u.name))
    ///     .create()
    ///     .await?;
    /// ```
    pub fn after_make<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.after_make_callbacks.push(Arc::new(f));
        self
    }

    /// Add an async callback to run after creating (persisting to database)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user = User::factory()
    ///     .after_create(|u| async move {
    ///         // Create related records
    ///         Profile::factory()
    ///             .state(|p| p.user_id = u.id)
    ///             .create()
    ///             .await?;
    ///         Ok(())
    ///     })
    ///     .create()
    ///     .await?;
    /// ```
    pub fn after_create<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(T) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), FrameworkError>> + Send + 'static,
        T: Clone,
    {
        self.after_create_callbacks
            .push(Box::new(move |model: T| Box::pin(f(model))));
        self
    }

    /// Build a single model instance without persisting
    pub fn make(self) -> T {
        let mut instance = T::definition();

        // Apply named traits
        let traits = T::traits();
        for trait_name in &self.trait_names {
            if let Some(trait_fn) = traits.get(trait_name) {
                trait_fn(&mut instance);
            }
        }

        // Apply inline states
        for state in &self.states {
            state(&mut instance);
        }

        // Run after_make callbacks
        for callback in &self.after_make_callbacks {
            callback(&instance);
        }

        instance
    }

    /// Build multiple model instances without persisting
    pub fn make_many(self) -> Vec<T> {
        let count = self.count;
        let traits = T::traits();

        (0..count)
            .map(|_| {
                let mut instance = T::definition();

                // Apply named traits
                for trait_name in &self.trait_names {
                    if let Some(trait_fn) = traits.get(trait_name) {
                        trait_fn(&mut instance);
                    }
                }

                // Apply inline states
                for state in &self.states {
                    state(&mut instance);
                }

                // Run after_make callbacks
                for callback in &self.after_make_callbacks {
                    callback(&instance);
                }

                instance
            })
            .collect()
    }
}

impl<T: Factory> Default for FactoryBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Database persistence methods (only available when T: DatabaseFactory)
impl<T: DatabaseFactory> FactoryBuilder<T> {
    /// Create a single model instance and persist to database
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user = User::factory()
    ///     .state(|u| u.name = "John".into())
    ///     .create()
    ///     .await?;
    /// ```
    pub async fn create(self) -> Result<T, FrameworkError> {
        let mut instance = T::definition();
        let traits = T::traits();

        // Apply named traits
        for trait_name in &self.trait_names {
            if let Some(trait_fn) = traits.get(trait_name) {
                trait_fn(&mut instance);
            }
        }

        // Apply inline states
        for state in &self.states {
            state(&mut instance);
        }

        // Run after_make callbacks
        for callback in &self.after_make_callbacks {
            callback(&instance);
        }

        // Insert into database
        let created = T::insert(instance).await?;

        // Run after_create callbacks
        for callback in &self.after_create_callbacks {
            callback(created.clone()).await?;
        }

        Ok(created)
    }

    /// Create multiple model instances and persist to database
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let users = User::factory()
    ///     .count(5)
    ///     .create_many()
    ///     .await?;
    /// ```
    pub async fn create_many(self) -> Result<Vec<T>, FrameworkError> {
        let count = self.count;
        let after_create_callbacks = self.after_create_callbacks;
        let traits = T::traits();

        let mut results = Vec::with_capacity(count);

        for _ in 0..count {
            let mut instance = T::definition();

            // Apply named traits
            for trait_name in &self.trait_names {
                if let Some(trait_fn) = traits.get(trait_name) {
                    trait_fn(&mut instance);
                }
            }

            // Apply inline states
            for state in &self.states {
                state(&mut instance);
            }

            // Run after_make callbacks
            for callback in &self.after_make_callbacks {
                callback(&instance);
            }

            // Insert into database
            let created = T::insert(instance).await?;

            // Run after_create callbacks
            for callback in &after_create_callbacks {
                callback(created.clone()).await?;
            }

            results.push(created);
        }

        Ok(results)
    }
}

/// Helper for generating fake data
///
/// Provides convenient methods for generating common types of fake data.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::testing::Fake;
///
/// let name = Fake::name();
/// let email = Fake::email();
/// let sentence = Fake::sentence();
/// ```
pub struct Fake;

impl Fake {
    /// Generate a random first name
    pub fn first_name() -> String {
        let names = [
            "James",
            "Mary",
            "John",
            "Patricia",
            "Robert",
            "Jennifer",
            "Michael",
            "Linda",
            "William",
            "Elizabeth",
            "David",
            "Barbara",
            "Richard",
            "Susan",
            "Joseph",
            "Jessica",
            "Thomas",
            "Sarah",
            "Charles",
            "Karen",
            "Emma",
            "Olivia",
            "Ava",
            "Isabella",
            "Sophia",
            "Mia",
            "Charlotte",
            "Amelia",
            "Harper",
            "Evelyn",
        ];
        let mut rng = rand::thread_rng();
        names[rng.gen_range(0..names.len())].to_string()
    }

    /// Generate a random last name
    pub fn last_name() -> String {
        let names = [
            "Smith",
            "Johnson",
            "Williams",
            "Brown",
            "Jones",
            "Garcia",
            "Miller",
            "Davis",
            "Rodriguez",
            "Martinez",
            "Hernandez",
            "Lopez",
            "Gonzalez",
            "Wilson",
            "Anderson",
            "Thomas",
            "Taylor",
            "Moore",
            "Jackson",
            "Martin",
            "Lee",
            "Perez",
            "Thompson",
            "White",
            "Harris",
            "Sanchez",
            "Clark",
            "Ramirez",
            "Lewis",
            "Robinson",
        ];
        let mut rng = rand::thread_rng();
        names[rng.gen_range(0..names.len())].to_string()
    }

    /// Generate a random full name
    pub fn name() -> String {
        format!("{} {}", Self::first_name(), Self::last_name())
    }

    /// Generate a random email address
    pub fn email() -> String {
        let mut rng = rand::thread_rng();
        let id: u32 = rng.gen_range(1000..9999);
        let domains = ["example.com", "test.com", "mail.test", "fake.org"];
        let domain = domains[rng.gen_range(0..domains.len())];
        format!(
            "{}.{}{}@{}",
            Self::first_name().to_lowercase(),
            Self::last_name().to_lowercase(),
            id,
            domain
        )
    }

    /// Generate a safe email (always @example.com)
    pub fn safe_email() -> String {
        let mut rng = rand::thread_rng();
        let id: u32 = rng.gen_range(1000..9999);
        format!(
            "{}.{}{}@example.com",
            Self::first_name().to_lowercase(),
            Self::last_name().to_lowercase(),
            id
        )
    }

    /// Generate a random username
    pub fn username() -> String {
        let mut rng = rand::thread_rng();
        let id: u32 = rng.gen_range(100..999);
        format!(
            "{}{}{}",
            Self::first_name().to_lowercase(),
            Self::last_name().chars().next().unwrap_or('x'),
            id
        )
    }

    /// Generate a random password (for testing only)
    pub fn password() -> String {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%"
                .chars()
                .collect();
        (0..16)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect()
    }

    /// Generate a random sentence
    pub fn sentence() -> String {
        let words = [
            "the",
            "quick",
            "brown",
            "fox",
            "jumps",
            "over",
            "lazy",
            "dog",
            "lorem",
            "ipsum",
            "dolor",
            "sit",
            "amet",
            "consectetur",
            "adipiscing",
            "elit",
            "sed",
            "do",
            "eiusmod",
            "tempor",
            "incididunt",
            "ut",
            "labore",
            "et",
            "dolore",
            "magna",
            "aliqua",
            "enim",
            "ad",
            "minim",
            "veniam",
        ];
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(5..12);
        let sentence: Vec<&str> = (0..count)
            .map(|_| words[rng.gen_range(0..words.len())])
            .collect();
        let mut s = sentence.join(" ");
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        s.push('.');
        s
    }

    /// Generate a random paragraph
    pub fn paragraph() -> String {
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(3..6);
        (0..count)
            .map(|_| Self::sentence())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Generate random text of approximately the given length
    pub fn text(approx_length: usize) -> String {
        let mut result = String::new();
        while result.len() < approx_length {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&Self::paragraph());
        }
        result.truncate(approx_length);
        result
    }

    /// Generate a random integer in a range
    pub fn number(min: i64, max: i64) -> i64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }

    /// Generate a random float in a range
    pub fn float(min: f64, max: f64) -> f64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }

    /// Generate a random boolean
    pub fn boolean() -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_bool(0.5)
    }

    /// Generate a random boolean with given probability of true
    pub fn boolean_with_probability(probability: f64) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_bool(probability.clamp(0.0, 1.0))
    }

    /// Generate a random UUID v4
    pub fn uuid() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes: [u8; 16] = rng.gen();

        // Set version to 4 (bits 4-7 of byte 6)
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        // Set variant to 10xx (bits 6-7 of byte 8)
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }

    /// Generate a random phone number
    pub fn phone() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "+1-{:03}-{:03}-{:04}",
            rng.gen_range(200..999),
            rng.gen_range(200..999),
            rng.gen_range(1000..9999)
        )
    }

    /// Generate a random street address
    pub fn address() -> String {
        let mut rng = rand::thread_rng();
        let streets = [
            "Main St",
            "Oak Ave",
            "Maple Dr",
            "Cedar Ln",
            "Pine Rd",
            "Elm St",
            "Washington Blvd",
            "Park Ave",
            "Lake Dr",
            "Hill Rd",
        ];
        format!(
            "{} {}",
            rng.gen_range(100..9999),
            streets[rng.gen_range(0..streets.len())]
        )
    }

    /// Generate a random city name
    pub fn city() -> String {
        let cities = [
            "New York",
            "Los Angeles",
            "Chicago",
            "Houston",
            "Phoenix",
            "Philadelphia",
            "San Antonio",
            "San Diego",
            "Dallas",
            "Austin",
            "San Jose",
            "Seattle",
            "Denver",
            "Boston",
            "Portland",
        ];
        let mut rng = rand::thread_rng();
        cities[rng.gen_range(0..cities.len())].to_string()
    }

    /// Generate a random US state
    pub fn state() -> String {
        let states = [
            "Alabama",
            "Alaska",
            "Arizona",
            "Arkansas",
            "California",
            "Colorado",
            "Connecticut",
            "Delaware",
            "Florida",
            "Georgia",
            "Hawaii",
            "Idaho",
            "Illinois",
            "Indiana",
            "Iowa",
        ];
        let mut rng = rand::thread_rng();
        states[rng.gen_range(0..states.len())].to_string()
    }

    /// Generate a random US zip code
    pub fn zip_code() -> String {
        let mut rng = rand::thread_rng();
        format!("{:05}", rng.gen_range(10000..99999))
    }

    /// Generate a random country
    pub fn country() -> String {
        let countries = [
            "United States",
            "United Kingdom",
            "Canada",
            "Australia",
            "Germany",
            "France",
            "Japan",
            "Brazil",
            "India",
            "Mexico",
        ];
        let mut rng = rand::thread_rng();
        countries[rng.gen_range(0..countries.len())].to_string()
    }

    /// Generate a random company name
    pub fn company() -> String {
        let prefixes = ["Tech", "Global", "United", "Advanced", "Premier", "Elite"];
        let suffixes = ["Solutions", "Systems", "Industries", "Corp", "Inc", "LLC"];
        let mut rng = rand::thread_rng();
        format!(
            "{} {} {}",
            prefixes[rng.gen_range(0..prefixes.len())],
            Self::last_name(),
            suffixes[rng.gen_range(0..suffixes.len())]
        )
    }

    /// Generate a random URL
    pub fn url() -> String {
        let mut rng = rand::thread_rng();
        let tlds = ["com", "org", "net", "io", "co"];
        format!(
            "https://www.{}.{}",
            Self::last_name().to_lowercase(),
            tlds[rng.gen_range(0..tlds.len())]
        )
    }

    /// Generate a random image URL (placeholder)
    pub fn image_url(width: u32, height: u32) -> String {
        format!("https://via.placeholder.com/{width}x{height}")
    }

    /// Generate a random hex color
    pub fn hex_color() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "#{:02x}{:02x}{:02x}",
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255)
        )
    }

    /// Generate a random IPv4 address
    pub fn ipv4() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "{}.{}.{}.{}",
            rng.gen_range(1..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(1..255)
        )
    }

    /// Generate a random MAC address
    pub fn mac_address() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255),
            rng.gen_range(0..=255)
        )
    }

    /// Generate a random date in YYYY-MM-DD format
    pub fn date() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "{:04}-{:02}-{:02}",
            rng.gen_range(2000..2025),
            rng.gen_range(1..=12),
            rng.gen_range(1..=28)
        )
    }

    /// Generate a random datetime in ISO 8601 format
    pub fn datetime() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            rng.gen_range(2000..2025),
            rng.gen_range(1..=12),
            rng.gen_range(1..=28),
            rng.gen_range(0..24),
            rng.gen_range(0..60),
            rng.gen_range(0..60)
        )
    }

    /// Generate a random future date
    pub fn future_date() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "{:04}-{:02}-{:02}",
            rng.gen_range(2025..2030),
            rng.gen_range(1..=12),
            rng.gen_range(1..=28)
        )
    }

    /// Generate a random past date
    pub fn past_date() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "{:04}-{:02}-{:02}",
            rng.gen_range(2010..2024),
            rng.gen_range(1..=12),
            rng.gen_range(1..=28)
        )
    }

    /// Generate a random slug from words
    pub fn slug() -> String {
        let words = ["hello", "world", "test", "example", "demo", "sample"];
        let mut rng = rand::thread_rng();
        let count = rng.gen_range(2..4);
        (0..count)
            .map(|_| words[rng.gen_range(0..words.len())])
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Pick a random element from a slice
    pub fn one_of<T: Clone>(items: &[T]) -> T {
        let mut rng = rand::thread_rng();
        items[rng.gen_range(0..items.len())].clone()
    }

    /// Pick n random elements from a slice
    pub fn many_of<T: Clone>(items: &[T], count: usize) -> Vec<T> {
        let mut rng = rand::thread_rng();
        let count = count.min(items.len());
        let mut indices: Vec<usize> = (0..items.len()).collect();

        // Fisher-Yates shuffle for first n elements
        for i in 0..count {
            let j = rng.gen_range(i..items.len());
            indices.swap(i, j);
        }

        indices[..count].iter().map(|&i| items[i].clone()).collect()
    }

    /// Generate a random credit card number (for testing only, not valid)
    pub fn credit_card() -> String {
        let mut rng = rand::thread_rng();
        format!(
            "4{:03}-{:04}-{:04}-{:04}",
            rng.gen_range(0..1000),
            rng.gen_range(0..10000),
            rng.gen_range(0..10000),
            rng.gen_range(0..10000)
        )
    }

    /// Generate random bytes
    pub fn bytes(length: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..length).map(|_| rng.gen()).collect()
    }

    /// Generate a random hex string
    pub fn hex(length: usize) -> String {
        Self::bytes(length / 2 + 1)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            .chars()
            .take(length)
            .collect()
    }
}

/// Convenience function to create a sequence of unique items
pub struct Sequence {
    current: usize,
}

impl Sequence {
    /// Create a new sequence starting at 1
    pub fn new() -> Self {
        Self { current: 0 }
    }

    /// Create a sequence starting at a specific value
    pub fn starting_at(value: usize) -> Self {
        Self {
            current: value.saturating_sub(1),
        }
    }

    /// Get the next value in the sequence
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> usize {
        self.current += 1;
        self.current
    }

    /// Get the next value as a string with prefix
    pub fn next_with_prefix(&mut self, prefix: &str) -> String {
        format!("{}{}", prefix, self.next())
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_name() {
        let name = Fake::name();
        assert!(!name.is_empty());
        assert!(name.contains(' '));
    }

    #[test]
    fn test_fake_email() {
        let email = Fake::email();
        assert!(email.contains('@'));
        assert!(email.contains('.'));
    }

    #[test]
    fn test_fake_uuid() {
        let uuid = Fake::uuid();
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains('-'));
    }

    #[test]
    fn test_fake_sentence() {
        let sentence = Fake::sentence();
        assert!(!sentence.is_empty());
        assert!(sentence.ends_with('.'));
        // First character should be uppercase
        assert!(sentence.chars().next().unwrap().is_uppercase());
    }

    #[test]
    fn test_fake_number() {
        let num = Fake::number(1, 10);
        assert!((1..=10).contains(&num));
    }

    #[test]
    fn test_fake_one_of() {
        let options = vec!["a", "b", "c"];
        let choice = Fake::one_of(&options);
        assert!(options.contains(&choice));
    }

    #[test]
    fn test_sequence() {
        let mut seq = Sequence::new();
        assert_eq!(seq.next(), 1);
        assert_eq!(seq.next(), 2);
        assert_eq!(seq.next(), 3);
    }

    #[test]
    fn test_sequence_with_prefix() {
        let mut seq = Sequence::new();
        assert_eq!(seq.next_with_prefix("user_"), "user_1");
        assert_eq!(seq.next_with_prefix("user_"), "user_2");
    }

    // Test factory pattern with a simple struct
    #[derive(Clone)]
    struct TestModel {
        id: i32,
        name: String,
        email: String,
        role: String,
        verified: bool,
    }

    impl Factory for TestModel {
        fn definition() -> Self {
            Self {
                id: Fake::number(1, 1000) as i32,
                name: Fake::name(),
                email: Fake::email(),
                role: "user".to_string(),
                verified: false,
            }
        }

        fn traits() -> FactoryTraits<Self> {
            FactoryTraits::new()
                .define("admin", |m: &mut Self| m.role = "admin".to_string())
                .define("verified", |m: &mut Self| m.verified = true)
        }
    }

    #[test]
    fn test_factory_make() {
        let model = TestModel::factory().make();
        assert!(!model.name.is_empty());
        assert!(model.email.contains('@'));
    }

    #[test]
    fn test_factory_with_state() {
        let model = TestModel::factory()
            .state(|m| m.name = "Custom Name".to_string())
            .make();
        assert_eq!(model.name, "Custom Name");
    }

    #[test]
    fn test_factory_make_many() {
        let models = TestModel::factory().count(5).make_many();
        assert_eq!(models.len(), 5);
    }

    #[test]
    fn test_factory_with_trait() {
        let admin = TestModel::factory().trait_("admin").make();
        assert_eq!(admin.role, "admin");
    }

    #[test]
    fn test_factory_with_multiple_traits() {
        let verified_admin = TestModel::factory()
            .trait_("admin")
            .trait_("verified")
            .make();
        assert_eq!(verified_admin.role, "admin");
        assert!(verified_admin.verified);
    }

    #[test]
    fn test_factory_trait_with_state_override() {
        let model = TestModel::factory()
            .trait_("admin")
            .state(|m| m.role = "superadmin".to_string())
            .make();
        // State should override trait
        assert_eq!(model.role, "superadmin");
    }

    #[test]
    fn test_factory_after_make_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let callback_ran = Arc::new(AtomicBool::new(false));
        let callback_ran_clone = callback_ran.clone();

        let _model = TestModel::factory()
            .after_make(move |_| {
                callback_ran_clone.store(true, Ordering::SeqCst);
            })
            .make();

        assert!(callback_ran.load(Ordering::SeqCst));
    }

    #[test]
    fn test_factory_set_value() {
        // Test set() for foreign key style associations
        let parent_id = 42;

        let model = TestModel::factory()
            .set(parent_id, |m, id| m.id = id)
            .make();

        assert_eq!(model.id, 42);
    }

    #[test]
    fn test_factory_set_multiple_values() {
        // Test multiple set() calls
        let model = TestModel::factory()
            .set(99, |m, id| m.id = id)
            .set("Manager".to_string(), |m, role| m.role = role)
            .make();

        assert_eq!(model.id, 99);
        assert_eq!(model.role, "Manager");
    }
}
