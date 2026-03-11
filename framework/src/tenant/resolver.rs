//! Tenant resolver trait for Ferro framework.
//!
//! Defines the contract for pluggable tenant resolution strategies.
//! Concrete implementations (subdomain, header, path, JWT) are in Plan 02.

use crate::tenant::TenantContext;
use crate::Request;
use async_trait::async_trait;

/// Resolves the current tenant from an incoming request.
///
/// Implement this trait to provide custom resolution strategies such as
/// subdomain parsing, header inspection, or JWT claim extraction.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::{TenantContext, TenantResolver};
/// use ferro_rs::Request;
/// use async_trait::async_trait;
///
/// struct SubdomainResolver;
///
/// #[async_trait]
/// impl TenantResolver for SubdomainResolver {
///     async fn resolve(&self, req: &Request) -> Option<TenantContext> {
///         // Extract subdomain from Host header and resolve tenant
///         None
///     }
/// }
/// ```
#[async_trait]
pub trait TenantResolver: Send + Sync {
    /// Resolve the tenant from the given request.
    ///
    /// Returns `None` if no tenant could be determined.
    async fn resolve(&self, req: &Request) -> Option<TenantContext>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_resolver_is_object_safe() {
        // If TenantResolver were not object-safe, this would not compile.
        let _: Box<dyn TenantResolver>;
    }
}
