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
}
