//! Google Wallet save-link issuance — RS256 JWT pointing at an eventTicketObject.

pub mod jwt;
pub mod object;

/// Issues Google Wallet save links for any [`WalletSubject`](crate::subject::WalletSubject).
///
/// The full `impl` (constructor + `save_jwt` + `save_url`) lands together with this
/// struct's field set; the type is declared up-front so the `jwt::sign_save_jwt` and
/// `object::build_event_ticket_object` helpers can take `&GoogleWalletBuilder` arguments
/// without a forward-declaration trick.
// `dead_code` is suppressed until the `impl GoogleWalletBuilder` block in this module
// lands (Task 3). Field consumers (`jwt::sign_save_jwt`, `object::build_event_ticket_object`)
// already exist, but no constructor calls them until the impl is added.
#[allow(dead_code)]
pub struct GoogleWalletBuilder {
    pub(crate) issuer_id: String,
    pub(crate) service_account_email: String,
    pub(crate) private_key_pem: String,
    /// Stored for symmetry with ApplePassBuilder; not in v1 JWT payload (D-08).
    pub(crate) app_name: String,
    pub(crate) app_url: String,
}
