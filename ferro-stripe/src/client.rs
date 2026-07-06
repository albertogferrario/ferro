use crate::StripeConfig;
use std::sync::OnceLock;

static STRIPE_CLIENT: OnceLock<stripe::Client> = OnceLock::new();
static STRIPE_CONFIG: OnceLock<StripeConfig> = OnceLock::new();

/// Static facade for the Stripe client, initialized once at application startup.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_stripe::{Stripe, StripeConfig};
///
/// let config = StripeConfig::from_env().expect("stripe config");
/// Stripe::init(config);
///
/// // Later, access the client from any handler
/// let client = Stripe::client();
/// ```
pub struct Stripe;

impl Stripe {
    /// Initializes the static Stripe client with the given configuration.
    ///
    /// Safe to call multiple times; subsequent calls are no-ops.
    pub fn init(config: StripeConfig) {
        let client = stripe::Client::new(&config.api_key);
        STRIPE_CLIENT.set(client).ok();
        STRIPE_CONFIG.set(config).ok();
    }

    /// Returns a reference to the initialized Stripe client.
    ///
    /// # Panics
    ///
    /// Panics if [`Stripe::init`] has not been called.
    pub fn client() -> &'static stripe::Client {
        STRIPE_CLIENT
            .get()
            .expect("Stripe::init() not called before Stripe::client()")
    }

    /// Returns a reference to the Stripe configuration.
    ///
    /// # Panics
    ///
    /// Panics if [`Stripe::init`] has not been called.
    pub fn config() -> &'static StripeConfig {
        STRIPE_CONFIG
            .get()
            .expect("Stripe::init() not called before Stripe::config()")
    }

    /// Returns a scoped Stripe client for the given API key.
    ///
    /// Use for per-tenant direct-charges scenarios where a different
    /// Stripe account key is needed per request.
    /// Does not affect the global static client initialized by [`Stripe::init`].
    pub fn with(api_key: &str) -> stripe::Client {
        stripe::Client::new(api_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_does_not_populate_global_static() {
        // Calling Stripe::with should not initialize STRIPE_CLIENT.
        // This test relies on the fact that Stripe::init has not been called
        // in this test process. (Each `cargo test` binary is a fresh process.)
        let _scoped = Stripe::with("sk_test_scoped_key");
        assert!(
            STRIPE_CLIENT.get().is_none(),
            "Stripe::with must not populate the global static client"
        );
    }

    #[test]
    fn with_returns_independent_client_values() {
        // Compile-time proof that `with` returns a `stripe::Client` by value,
        // and two calls produce two distinct values (not a shared reference).
        let a: stripe::Client = Stripe::with("sk_test_a");
        let b: stripe::Client = Stripe::with("sk_test_b");
        // Clients have no public equality; assert the values are usable by dropping them.
        drop(a);
        drop(b);
    }
}
