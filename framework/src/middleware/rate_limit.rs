//! Rate limiting middleware for Ferro framework
//!
//! Provides cache-backed rate limiting with Laravel-style declarative named limiters.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro::middleware::{RateLimiter, Limit, Throttle};
//!
//! // Register a named limiter in bootstrap
//! RateLimiter::define("api", |req| {
//!     Limit::per_minute(60)
//! });
//!
//! // Apply to routes
//! get!("/api/users", controllers::users::index).middleware(Throttle::named("api"))
//!
//! // Inline limit without registry
//! get!("/health", controllers::health::check).middleware(Throttle::per_minute(120))
//! ```

use crate::cache::Cache;
use crate::http::{HttpResponse, Request, Response};
use crate::middleware::{Middleware, Next};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Type alias for the limiter closure stored in the registry
type LimiterFn = Arc<dyn Fn(&Request) -> Vec<Limit> + Send + Sync>;

/// Global limiter registry
fn limiter_registry() -> &'static DashMap<String, LimiterFn> {
    static REGISTRY: OnceLock<DashMap<String, LimiterFn>> = OnceLock::new();
    REGISTRY.get_or_init(DashMap::new)
}

/// Declarative rate limit configuration
///
/// Defines how many requests are allowed in a time window, with optional
/// per-key segmentation and custom 429 responses.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::middleware::Limit;
///
/// // 60 requests per minute, keyed by client IP (default)
/// let limit = Limit::per_minute(60);
///
/// // 120 requests per minute, keyed by user ID
/// let limit = Limit::per_minute(120).by(format!("user:{}", user_id));
///
/// // Custom 429 response
/// let limit = Limit::per_hour(1000).response(|| {
///     HttpResponse::json(serde_json::json!({"error": "Quota exceeded"})).status(429)
/// });
/// ```
pub struct Limit {
    /// Maximum requests allowed in the window
    pub max_requests: u32,
    /// Window duration in seconds
    pub window_seconds: u64,
    /// Custom key for segmentation (defaults to client IP if None)
    key: Option<String>,
    /// Custom 429 response factory
    response_fn: Option<Arc<dyn Fn() -> HttpResponse + Send + Sync>>,
}

impl Limit {
    /// Create a limit allowing N requests per second
    pub fn per_second(max: u32) -> Self {
        Self {
            max_requests: max,
            window_seconds: 1,
            key: None,
            response_fn: None,
        }
    }

    /// Create a limit allowing N requests per minute
    pub fn per_minute(max: u32) -> Self {
        Self {
            max_requests: max,
            window_seconds: 60,
            key: None,
            response_fn: None,
        }
    }

    /// Create a limit allowing N requests per hour
    pub fn per_hour(max: u32) -> Self {
        Self {
            max_requests: max,
            window_seconds: 3600,
            key: None,
            response_fn: None,
        }
    }

    /// Create a limit allowing N requests per day
    pub fn per_day(max: u32) -> Self {
        Self {
            max_requests: max,
            window_seconds: 86400,
            key: None,
            response_fn: None,
        }
    }

    /// Set a custom key for rate limit segmentation
    ///
    /// When set, this key is used instead of the client IP address.
    /// Useful for per-user or per-API-key rate limiting.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Limit::per_minute(120).by(format!("user:{}", user_id))
    /// ```
    pub fn by(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set a custom response for 429 Too Many Requests
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Limit::per_minute(60).response(|| {
    ///     HttpResponse::json(serde_json::json!({"error": "Slow down!"})).status(429)
    /// })
    /// ```
    pub fn response<F>(mut self, f: F) -> Self
    where
        F: Fn() -> HttpResponse + Send + Sync + 'static,
    {
        self.response_fn = Some(Arc::new(f));
        self
    }
}

/// Return type for limiter closures registered with `RateLimiter::define()`
///
/// Allows closures to return either a single `Limit` or a `Vec<Limit>`.
pub enum LimiterResponse {
    /// A single rate limit
    Single(Limit),
    /// Multiple rate limits (all checked, first exceeded triggers 429)
    Multiple(Vec<Limit>),
}

impl From<Limit> for LimiterResponse {
    fn from(limit: Limit) -> Self {
        LimiterResponse::Single(limit)
    }
}

impl From<Vec<Limit>> for LimiterResponse {
    fn from(limits: Vec<Limit>) -> Self {
        LimiterResponse::Multiple(limits)
    }
}

impl LimiterResponse {
    fn into_vec(self) -> Vec<Limit> {
        match self {
            LimiterResponse::Single(limit) => vec![limit],
            LimiterResponse::Multiple(limits) => limits,
        }
    }
}

/// Static registry for named rate limiters
///
/// Register named limiters with closures that receive the request and return
/// dynamic rate limits. Closures are evaluated per-request, enabling limits
/// based on authentication state, user tier, or request properties.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::middleware::{RateLimiter, Limit};
///
/// // Register in bootstrap
/// RateLimiter::define("api", |req| {
///     Limit::per_minute(60)
/// });
///
/// // Dynamic limits based on auth
/// RateLimiter::define("api", |req| {
///     match req.header("X-API-Key") {
///         Some(_) => Limit::per_minute(120),
///         None => Limit::per_minute(30),
///     }
/// });
///
/// // Multiple limits
/// RateLimiter::define("login", |req| {
///     vec![
///         Limit::per_minute(500),
///         Limit::per_minute(5).by("per-ip".to_string()),
///     ]
/// });
/// ```
pub struct RateLimiter;

impl RateLimiter {
    /// Register a named rate limiter
    ///
    /// The closure receives `&Request` and returns a `Limit` or `Vec<Limit>`.
    pub fn define<F, T>(name: &str, f: F)
    where
        F: Fn(&Request) -> T + Send + Sync + 'static,
        T: Into<LimiterResponse>,
    {
        let wrapped: LimiterFn = Arc::new(move |req| {
            let response: LimiterResponse = f(req).into();
            response.into_vec()
        });
        limiter_registry().insert(name.to_string(), wrapped);
    }

    /// Resolve a named limiter for a given request
    ///
    /// Returns `None` if the named limiter is not registered.
    pub fn resolve(name: &str, req: &Request) -> Option<Vec<Limit>> {
        limiter_registry().get(name).map(|f| f(req))
    }

    /// Create an inline limit of N requests per second
    pub fn per_second(max: u32) -> Limit {
        Limit::per_second(max)
    }

    /// Create an inline limit of N requests per minute
    pub fn per_minute(max: u32) -> Limit {
        Limit::per_minute(max)
    }

    /// Create an inline limit of N requests per hour
    pub fn per_hour(max: u32) -> Limit {
        Limit::per_hour(max)
    }

    /// Create an inline limit of N requests per day
    pub fn per_day(max: u32) -> Limit {
        Limit::per_day(max)
    }
}

/// Result of a rate limit check
struct RateLimitResult {
    allowed: bool,
    limit: u32,
    remaining: u32,
    retry_after: u64,
}

/// Extract client IP from request headers
///
/// Checks X-Forwarded-For (first entry), X-Real-IP, falls back to "unknown".
fn get_client_ip(request: &Request) -> String {
    request
        .header("X-Forwarded-For")
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| request.header("X-Real-IP").map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

/// Check a single rate limit against the cache backend
///
/// Uses fixed-window counter: INCR + EXPIRE pattern.
/// Fail-open: if cache is unavailable, allows the request with a warning.
async fn check_rate_limit(limit: &Limit, name: &str, identifier: &str) -> RateLimitResult {
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let window_number = now_secs / limit.window_seconds;
    let key = format!("rate_limit:{}:{}:{}", name, identifier, window_number);

    // Atomic increment; fail-open if cache unavailable
    let count = match Cache::increment(&key, 1).await {
        Ok(c) => c as u32,
        Err(e) => {
            eprintln!("[ferro] Rate limiter cache error (fail-open): {}", e);
            return RateLimitResult {
                allowed: true,
                limit: limit.max_requests,
                remaining: limit.max_requests,
                retry_after: limit.window_seconds,
            };
        }
    };

    // Set TTL on first request in window
    if count == 1 {
        let ttl = Duration::from_secs(limit.window_seconds + 1);
        if let Err(e) = Cache::expire(&key, ttl).await {
            eprintln!("[ferro] Rate limiter expire error: {}", e);
        }
    }

    let remaining = limit.max_requests.saturating_sub(count);
    let retry_after = limit.window_seconds - (now_secs % limit.window_seconds);

    RateLimitResult {
        allowed: count <= limit.max_requests,
        limit: limit.max_requests,
        remaining,
        retry_after,
    }
}

/// Add rate limit headers to an HttpResponse
fn add_rate_limit_headers(
    response: HttpResponse,
    limit: u32,
    remaining: u32,
    retry_after: u64,
) -> HttpResponse {
    response
        .header("X-RateLimit-Limit", limit.to_string())
        .header("X-RateLimit-Remaining", remaining.to_string())
        .header("X-RateLimit-Reset", retry_after.to_string())
}

/// Rate limiting middleware
///
/// Apply rate limits to routes using named limiters or inline limits.
/// Implements `Middleware` directly for use with `.middleware()`.
///
/// # Named limiter (from registry)
///
/// ```rust,ignore
/// // Register in bootstrap
/// RateLimiter::define("api", |req| Limit::per_minute(60));
///
/// // Apply to routes
/// get!("/api/users", handler).middleware(Throttle::named("api"))
/// ```
///
/// # Inline limits
///
/// ```rust,ignore
/// get!("/health", handler).middleware(Throttle::per_minute(120))
/// ```
pub struct Throttle {
    /// Named limiter to resolve from registry
    name: Option<String>,
    /// Inline limits (used when not resolving from registry)
    inline_limits: Vec<Limit>,
}

impl Throttle {
    /// Create a throttle that resolves from the named limiter registry
    ///
    /// The named limiter is evaluated per-request, allowing dynamic limits.
    /// If the named limiter doesn't exist, the request is allowed (fail-open).
    pub fn named(name: &str) -> Self {
        Self {
            name: Some(name.to_string()),
            inline_limits: Vec::new(),
        }
    }

    /// Create a throttle with an inline limit of N requests per second
    pub fn per_second(max: u32) -> Self {
        Self {
            name: None,
            inline_limits: vec![Limit::per_second(max)],
        }
    }

    /// Create a throttle with an inline limit of N requests per minute
    pub fn per_minute(max: u32) -> Self {
        Self {
            name: None,
            inline_limits: vec![Limit::per_minute(max)],
        }
    }

    /// Create a throttle with an inline limit of N requests per hour
    pub fn per_hour(max: u32) -> Self {
        Self {
            name: None,
            inline_limits: vec![Limit::per_hour(max)],
        }
    }

    /// Create a throttle with an inline limit of N requests per day
    pub fn per_day(max: u32) -> Self {
        Self {
            name: None,
            inline_limits: vec![Limit::per_day(max)],
        }
    }
}

#[async_trait]
impl Middleware for Throttle {
    async fn handle(&self, request: Request, next: Next) -> Response {
        // Resolve limits: either from named registry or inline
        let (limiter_name, limits) = if let Some(ref name) = self.name {
            match RateLimiter::resolve(name, &request) {
                Some(limits) => (name.clone(), limits),
                None => {
                    eprintln!(
                        "[ferro] Rate limiter '{}' not registered (fail-open, allowing request)",
                        name
                    );
                    return next(request).await;
                }
            }
        } else {
            // Inline limits can't be moved out of &self, so we recreate them
            // This is cheap since Limit is just a few fields
            let limits: Vec<Limit> = self
                .inline_limits
                .iter()
                .map(|l| Limit {
                    max_requests: l.max_requests,
                    window_seconds: l.window_seconds,
                    key: l.key.clone(),
                    response_fn: l.response_fn.clone(),
                })
                .collect();
            ("inline".to_string(), limits)
        };

        // Get client IP for default key
        let client_ip = get_client_ip(&request);

        // Track the most restrictive result (lowest remaining) for headers
        let mut most_restrictive: Option<(
            RateLimitResult,
            Option<Arc<dyn Fn() -> HttpResponse + Send + Sync>>,
        )> = None;

        // Check all limits; first exceeded triggers 429
        for limit in &limits {
            let identifier = limit.key.as_deref().unwrap_or(&client_ip);
            let result = check_rate_limit(limit, &limiter_name, identifier).await;

            if !result.allowed {
                // Rate limit exceeded - return 429
                let error_response = if let Some(ref response_fn) = limit.response_fn {
                    response_fn()
                } else {
                    HttpResponse::json(serde_json::json!({
                        "error": "Too Many Requests",
                        "message": "Rate limit exceeded. Please try again later.",
                        "retry_after": result.retry_after
                    }))
                    .status(429)
                };

                let error_response =
                    add_rate_limit_headers(error_response, result.limit, 0, result.retry_after)
                        .header("Retry-After", result.retry_after.to_string());

                return Err(error_response);
            }

            // Track most restrictive for headers on successful response
            let is_more_restrictive = most_restrictive
                .as_ref()
                .map(|(prev, _)| result.remaining < prev.remaining)
                .unwrap_or(true);

            if is_more_restrictive {
                most_restrictive = Some((result, limit.response_fn.clone()));
            }
        }

        // All limits passed - proceed with request
        let response = next(request).await;

        // Add rate limit headers from the most restrictive limit
        if let Some((result, _)) = most_restrictive {
            match response {
                Ok(http_response) => Ok(add_rate_limit_headers(
                    http_response,
                    result.limit,
                    result.remaining,
                    result.retry_after,
                )),
                Err(http_response) => Err(add_rate_limit_headers(
                    http_response,
                    result.limit,
                    result.remaining,
                    result.retry_after,
                )),
            }
        } else {
            response
        }
    }
}
