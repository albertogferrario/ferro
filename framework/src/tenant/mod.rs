//! Multi-tenant middleware support for Ferro framework.
//!
//! Provides task-local tenant context, resolver and lookup trait contracts,
//! and a default cached database lookup implementation.
//!
//! # Overview
//!
//! - [`TenantContext`] — holds id, slug, name, and optional plan fields
//! - [`current_tenant()`] — reads the current tenant from task-local storage
//! - [`TenantResolver`] — trait for pluggable tenant resolution strategies
//! - [`TenantLookup`] / [`DbTenantLookup`] — trait + cached implementation for DB queries
//! - [`TenantFailureMode`] — controls behavior when no tenant is resolved

pub mod context;
pub mod lookup;
pub mod resolver;

pub use context::current_tenant;
pub use lookup::{DbTenantLookup, TenantLookup};
pub use resolver::TenantResolver;

/// Core data for the resolved tenant.
///
/// Populated by [`TenantResolver`] and stored in task-local scope during a request.
/// The `plan` field is nullable — tenants may not have a billing plan assigned
/// until Stripe integration is complete (Phase 96).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TenantContext {
    /// Unique numeric tenant ID (primary key).
    pub id: i64,
    /// URL-safe slug used for subdomain or path-based routing.
    pub slug: String,
    /// Human-readable tenant name.
    pub name: String,
    /// Optional billing plan identifier.
    pub plan: Option<String>,
}

/// Controls framework behavior when no tenant is resolved for a request.
#[derive(Debug, Clone)]
pub enum TenantFailureMode {
    /// Return 404 Not Found when the tenant cannot be resolved.
    NotFound,
    /// Return 403 Forbidden when the tenant cannot be resolved.
    Forbidden,
    /// Pass through — allow the request even without a resolved tenant.
    Allow,
}
