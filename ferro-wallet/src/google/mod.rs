//! Google Wallet save-link issuance — RS256 JWT pointing at an eventTicketObject.

pub mod jwt;
pub mod object;

use crate::config::GoogleConfig;
use crate::subject::WalletSubject;
use crate::WalletError;

/// Issues Google Wallet save links for any [`WalletSubject`].
///
/// Constructed once per process (or per signing-material rotation) via
/// [`GoogleWalletBuilder::new`], which retains the issuer ID, service-account email,
/// private-key PEM, and app metadata. [`GoogleWalletBuilder::save_jwt`] composes a
/// fresh JWT per subject; [`GoogleWalletBuilder::save_url`] wraps the JWT in the
/// `pay.google.com/gp/v/save/{jwt}` URL Google's save endpoint accepts.
pub struct GoogleWalletBuilder {
    pub(crate) issuer_id: String,
    pub(crate) service_account_email: String,
    pub(crate) private_key_pem: String,
    /// Stored for symmetry with [`crate::apple::ApplePassBuilder`]; the v1 JWT payload
    /// does not include it (D-08).
    #[allow(dead_code)]
    pub(crate) app_name: String,
    pub(crate) app_url: String,
}

impl GoogleWalletBuilder {
    /// Build a new builder from the supplied [`GoogleConfig`] and app metadata.
    ///
    /// The `Result` return is retained for forward compatibility — additional
    /// PEM / config validation may land in future versions. The current
    /// implementation never returns `Err`.
    pub fn new(cfg: GoogleConfig, app_name: String, app_url: String) -> Result<Self, WalletError> {
        Ok(Self {
            issuer_id: cfg.issuer_id,
            service_account_email: cfg.service_account_email,
            private_key_pem: cfg.service_account_private_key_pem,
            app_name,
            app_url,
        })
    }

    /// Sign the save JWT for a single subject.
    ///
    /// Composes the per-subject [`object::build_event_ticket_object`] JSON and signs
    /// the surrounding [`jwt::sign_save_jwt`] envelope with RS256.
    ///
    /// # Errors
    ///
    /// - `WalletError::GoogleJwt(_)` if the private-key PEM is malformed or the
    ///   `jsonwebtoken::encode` call fails.
    pub fn save_jwt<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError> {
        let obj = object::build_event_ticket_object(self, s)?;
        jwt::sign_save_jwt(self, obj)
    }

    /// Sign + format the full Google Wallet save URL for a single subject.
    ///
    /// Convenience wrapper around [`Self::save_jwt`] + [`jwt::save_url`].
    pub fn save_url<S: WalletSubject>(&self, s: &S) -> Result<String, WalletError> {
        Ok(jwt::save_url(&self.save_jwt(s)?))
    }
}
