//! Service auto-registration for Ferro framework
//!
//! This module provides automatic service registration via macros:
//! - `#[service(ConcreteType)]` - auto-register trait bindings
//! - `#[derive(Injectable)]` - auto-register concrete types as singletons
//!
//! # Example - Trait binding
//!
//! ```rust,ignore
//! use ferro_rs::service;
//!
//! // Auto-register: dyn CacheStore → RedisCache
//! #[service(RedisCache)]
//! pub trait CacheStore: Send + Sync + 'static {
//!     fn get(&self, key: &str) -> Option<String>;
//!     fn set(&self, key: &str, value: &str);
//! }
//!
//! pub struct RedisCache;
//! impl Default for RedisCache {
//!     fn default() -> Self { Self }
//! }
//! impl CacheStore for RedisCache { ... }
//! ```
//!
//! # Example - Concrete singleton
//!
//! ```rust,ignore
//! use ferro_rs::injectable;
//!
//! #[injectable]
//! pub struct AppState {
//!     pub counter: u32,
//! }
//!
//! // Resolve via:
//! let state: AppState = App::get().unwrap();
//! ```

/// Entry for inventory-collected service bindings (trait → impl)
///
/// Used internally by the `#[service(ConcreteType)]` macro to register
/// service bindings at compile time.
pub struct ServiceBindingEntry {
    /// Function to register the service binding
    pub register: fn(),
    /// Service name for debugging/logging
    pub name: &'static str,
}

/// Entry for inventory-collected singleton registrations (concrete types)
///
/// Used internally by the `#[derive(Injectable)]` macro to register
/// concrete singletons at compile time.
pub struct SingletonEntry {
    /// Function to register the singleton
    pub register: fn(),
    /// Type name for debugging/logging
    pub name: &'static str,
}

// Inventory collection for auto-registered service bindings
inventory::collect!(ServiceBindingEntry);

// Inventory collection for auto-registered singletons
inventory::collect!(SingletonEntry);

/// Register all service bindings from inventory
///
/// This is called automatically by `Server::from_config()`.
/// It registers all services marked with `#[service(ConcreteType)]`.
pub fn register_service_bindings() {
    for entry in inventory::iter::<ServiceBindingEntry> {
        (entry.register)();
    }
}

/// Register all singleton entries from inventory
///
/// This is called automatically by `Server::from_config()`.
/// It registers all types marked with `#[derive(Injectable)]`.
pub fn register_singletons() {
    for entry in inventory::iter::<SingletonEntry> {
        (entry.register)();
    }
}

/// Full bootstrap sequence for services
///
/// Called automatically by `Server::from_config()`.
pub fn bootstrap() {
    register_service_bindings();
    register_singletons();
}

/// Service info for introspection
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceInfo {
    /// Service name (trait or concrete type)
    pub name: String,
    /// Type of binding
    pub binding_type: ServiceBindingType,
}

/// Type of service binding
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceBindingType {
    /// Trait binding (dyn Trait → Impl)
    TraitBinding,
    /// Concrete singleton
    Singleton,
}

/// Get all registered services for introspection
///
/// Returns a list of all services registered via `#[service(ConcreteType)]`
/// and `#[injectable]` macros.
pub fn get_registered_services() -> Vec<ServiceInfo> {
    let mut services = Vec::new();

    // Collect trait bindings
    for entry in inventory::iter::<ServiceBindingEntry> {
        services.push(ServiceInfo {
            name: entry.name.to_string(),
            binding_type: ServiceBindingType::TraitBinding,
        });
    }

    // Collect singletons
    for entry in inventory::iter::<SingletonEntry> {
        services.push(ServiceInfo {
            name: entry.name.to_string(),
            binding_type: ServiceBindingType::Singleton,
        });
    }

    services
}
