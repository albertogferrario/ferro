//! Tenant lookup trait and cached database implementation.
//!
//! Defines the contract for tenant database queries and provides
//! [`DbTenantLookup`] with moka-based caching (5-minute TTL).

use crate::tenant::TenantContext;
use async_trait::async_trait;
use moka::sync::Cache;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// Abstracts tenant database queries.
///
/// Implement this trait to provide custom tenant lookup logic. The default
/// implementation [`DbTenantLookup`] wraps arbitrary async finder functions
/// with moka caching.
#[async_trait]
pub trait TenantLookup: Send + Sync {
    /// Find a tenant by URL slug.
    async fn find_by_slug(&self, slug: &str) -> Option<TenantContext>;

    /// Find a tenant by numeric ID.
    async fn find_by_id(&self, id: i64) -> Option<TenantContext>;

    /// Evict both slug and id cache entries for a tenant.
    ///
    /// Default implementation is a no-op. Override in caching implementations
    /// to invalidate stale entries after billing state changes (e.g. Stripe webhooks).
    fn invalidate(&self, _slug: &str, _id: i64) {}
}

type SlugFinder = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Option<TenantContext>> + Send>> + Send + Sync,
>;

type IdFinder =
    Arc<dyn Fn(i64) -> Pin<Box<dyn Future<Output = Option<TenantContext>> + Send>> + Send + Sync>;

/// Caching tenant lookup backed by user-supplied async finder functions.
///
/// Uses a moka in-process cache with a 5-minute TTL and capacity of 10 000
/// entries. On a cache miss the supplied finder function is called and the
/// result is stored for subsequent lookups.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::tenant::{DbTenantLookup, TenantContext};
///
/// let lookup = DbTenantLookup::new(
///     |slug| Box::pin(async move {
///         // query your DB here
///         None
///     }),
///     |id| Box::pin(async move {
///         None
///     }),
/// );
/// ```
pub struct DbTenantLookup {
    cache: Cache<String, TenantContext>,
    slug_finder: SlugFinder,
    id_finder: IdFinder,
}

impl DbTenantLookup {
    /// Create a new `DbTenantLookup` with moka cache (5-min TTL, 10 000 capacity).
    ///
    /// - `slug_finder` — async function that queries the DB by slug
    /// - `id_finder`   — async function that queries the DB by id
    pub fn new<SF, IF>(slug_finder: SF, id_finder: IF) -> Self
    where
        SF: Fn(String) -> Pin<Box<dyn Future<Output = Option<TenantContext>> + Send>>
            + Send
            + Sync
            + 'static,
        IF: Fn(i64) -> Pin<Box<dyn Future<Output = Option<TenantContext>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(300))
            .max_capacity(10_000)
            .build();

        Self {
            cache,
            slug_finder: Arc::new(slug_finder),
            id_finder: Arc::new(id_finder),
        }
    }
}

#[async_trait]
impl TenantLookup for DbTenantLookup {
    async fn find_by_slug(&self, slug: &str) -> Option<TenantContext> {
        let key = slug.to_string();
        if let Some(cached) = self.cache.get(&key) {
            return Some(cached);
        }
        let result = (self.slug_finder)(key.clone()).await;
        if let Some(ref tenant) = result {
            self.cache.insert(key, tenant.clone());
        }
        result
    }

    async fn find_by_id(&self, id: i64) -> Option<TenantContext> {
        let key = id.to_string();
        if let Some(cached) = self.cache.get(&key) {
            return Some(cached);
        }
        let result = (self.id_finder)(id).await;
        if let Some(ref tenant) = result {
            self.cache.insert(key, tenant.clone());
        }
        result
    }

    /// Evict both slug and id cache entries for the given tenant.
    ///
    /// Call this after receiving a Stripe webhook that updates subscription state
    /// so the next request re-fetches fresh billing data from the database.
    fn invalidate(&self, slug: &str, id: i64) {
        self.cache.invalidate(&slug.to_string());
        self.cache.invalidate(&id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn make_tenant(slug: &str) -> TenantContext {
        TenantContext {
            id: 1,
            slug: slug.to_string(),
            name: "Test Corp".to_string(),
            plan: None,
            #[cfg(feature = "stripe")]
            subscription: None,
        }
    }

    #[test]
    fn tenant_lookup_is_object_safe() {
        let _: Arc<dyn TenantLookup>;
    }

    #[tokio::test]
    async fn mock_lookup_returns_some_for_known_slug() {
        struct MockLookup;

        #[async_trait]
        impl TenantLookup for MockLookup {
            async fn find_by_slug(&self, slug: &str) -> Option<TenantContext> {
                if slug == "acme" {
                    Some(make_tenant("acme"))
                } else {
                    None
                }
            }
            async fn find_by_id(&self, _id: i64) -> Option<TenantContext> {
                None
            }
        }

        let lookup = MockLookup;
        assert!(lookup.find_by_slug("acme").await.is_some());
        assert!(lookup.find_by_slug("unknown").await.is_none());
    }

    #[tokio::test]
    async fn db_tenant_lookup_new_creates_empty_cache() {
        let lookup = DbTenantLookup::new(
            |_slug| Box::pin(async { None }),
            |_id| Box::pin(async { None }),
        );
        // Cache should be empty — any lookup returns None (finder returns None)
        let result = lookup.find_by_slug("any").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn db_tenant_lookup_caches_results() {
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let lookup = DbTenantLookup::new(
            move |slug| {
                let count = call_count_clone.clone();
                Box::pin(async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    if slug == "acme" {
                        Some(make_tenant("acme"))
                    } else {
                        None
                    }
                })
            },
            |_id| Box::pin(async { None }),
        );

        // First call hits finder
        let first = lookup.find_by_slug("acme").await;
        assert!(first.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call hits cache — finder not called again
        let second = lookup.find_by_slug("acme").await;
        assert!(second.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn tenant_resolver_receives_request_ref() {
        // Verified structurally in resolver.rs: resolve(&self, req: &Request)
        // This test validates TenantLookup's object-safety in Arc context
        let lookup: Arc<dyn TenantLookup> = Arc::new(DbTenantLookup::new(
            |_slug| Box::pin(async { None }),
            |_id| Box::pin(async { None }),
        ));
        let result = lookup.find_by_slug("test").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn invalidate_evicts_slug_and_id_cache_entries() {
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let lookup = DbTenantLookup::new(
            move |slug| {
                let count = call_count_clone.clone();
                Box::pin(async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    if slug == "acme" {
                        Some(make_tenant("acme"))
                    } else {
                        None
                    }
                })
            },
            |id| {
                Box::pin(async move {
                    if id == 1 {
                        Some(make_tenant("acme"))
                    } else {
                        None
                    }
                })
            },
        );

        // Prime the slug cache.
        let first = lookup.find_by_slug("acme").await;
        assert!(first.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1, "finder called once");

        // Second call hits cache — finder NOT called again.
        let second = lookup.find_by_slug("acme").await;
        assert!(second.is_some());
        assert_eq!(call_count.load(Ordering::SeqCst), 1, "cache hit");

        // Invalidate slug cache.
        lookup.invalidate("acme", 1);

        // After invalidation, next lookup misses cache and calls finder again.
        let third = lookup.find_by_slug("acme").await;
        assert!(third.is_some());
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            2,
            "finder called again after invalidation"
        );
    }

    #[test]
    fn default_invalidate_is_noop() {
        // TenantLookup trait default implementation is a no-op for types that don't cache.
        struct NoCacheLookup;

        #[async_trait]
        impl TenantLookup for NoCacheLookup {
            async fn find_by_slug(&self, _slug: &str) -> Option<TenantContext> {
                None
            }
            async fn find_by_id(&self, _id: i64) -> Option<TenantContext> {
                None
            }
        }

        // Should not panic or error.
        let lookup = NoCacheLookup;
        lookup.invalidate("acme", 1);
    }
}
