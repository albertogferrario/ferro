//! Declarative action handler primitive — typed `Result` return so POST
//! handlers can use `?` end-to-end and redirect-on-error without manual
//! `match` ladders.
//!
//! See `docs/src/the-basics/action-handlers.md` for the user-facing guide
//! and `framework/src/validation/error.rs` for the analog flash-then-redirect
//! pattern this module mirrors.

use form_urlencoded::byte_serialize;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

use crate::error::FrameworkError;
use crate::http::{HttpResponse, Response};

/// Semantic kind of an action error, used for logging and the back-compat
/// query-string fallback (`?error=<kind>`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    /// Generic catch-all (default).
    #[default]
    Generic,
    /// Logical 404 — the resource was not found.
    NotFound,
    /// Logical 403 — the caller may not perform this action.
    Forbidden,
    /// Logical 401 — the caller is not authenticated.
    Unauthorized,
}

/// Visual variant for the flash message rendered on the next page.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlashVariant {
    /// Error styling (default).
    #[default]
    Error,
    /// Warning styling.
    Warning,
    /// Informational styling.
    Info,
}

/// Error returned by `#[action]` handlers.
///
/// Builders (`with_flash`, `redirect_to`) consume `mut self` and return
/// `Self` per the framework convention. The `redirect_override` field, when
/// `Some`, replaces the handler's configured `redirect_to` — but only when
/// it points to a same-origin path (T-180-02). Off-origin overrides are
/// silently dropped and the handler's configured target is used instead.
///
/// # Security
///
/// The `message` field is rendered into the back-compat query string and
/// the session flash payload. Consumer templates that render the flash
/// message into HTML MUST escape it — the framework does not perform
/// HTML escaping at write time (T-180-01). Treat `message` as untrusted
/// text from the consumer's perspective.
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct ActionError {
    /// Human-readable error message. Percent-encoded into the query
    /// string fallback; stored verbatim in the session flash payload.
    pub message: String,
    /// Semantic classification (drives `?error=<kind>` and the tracing log).
    pub kind: ActionKind,
    /// Visual variant for the flash payload.
    pub flash_variant: FlashVariant,
    /// Optional same-origin redirect override. When `Some` and same-origin,
    /// replaces the handler's configured `redirect_to` for this error only.
    pub redirect_override: Option<String>,
}

impl ActionError {
    /// Create a generic error.
    pub fn msg(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ActionKind::Generic,
            flash_variant: FlashVariant::Error,
            redirect_override: None,
        }
    }

    /// Create a `NotFound`-kind error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::NotFound,
            ..Self::msg(message)
        }
    }

    /// Create a `Forbidden`-kind error.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::Forbidden,
            ..Self::msg(message)
        }
    }

    /// Create an `Unauthorized`-kind error.
    ///
    /// Per the framework's project-agnostic-crates rule, this does NOT
    /// carry a default `redirect_override`. Consumers route to their own
    /// login URL via `.redirect_to("/login-path")`.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::Unauthorized,
            ..Self::msg(message)
        }
    }

    /// Builder: set the flash variant.
    pub fn with_flash(mut self, variant: FlashVariant) -> Self {
        self.flash_variant = variant;
        self
    }

    /// Builder: set the redirect override. Only honored if the URL is
    /// same-origin (`starts_with('/')`) — see `is_safe_redirect`.
    pub fn redirect_to(mut self, url: impl Into<String>) -> Self {
        self.redirect_override = Some(url.into());
        self
    }
}

/// Success value returned by `#[action]` handlers.
///
/// The common case is `Ok(())` — converted via `From<()> for ActionOk`.
/// Override the flash key or redirect target for non-standard success paths.
#[derive(Debug, Clone, Default)]
pub struct ActionOk {
    /// Optional success flash key (e.g. `"created"`, `"saved"`).
    pub flash: Option<&'static str>,
    /// Optional same-origin redirect override.
    pub redirect_override: Option<String>,
}

impl ActionOk {
    /// Success with a flash message key.
    pub fn flash(key: &'static str) -> Self {
        Self {
            flash: Some(key),
            redirect_override: None,
        }
    }
    /// Success with a redirect override.
    pub fn redirect_to(url: impl Into<String>) -> Self {
        Self {
            flash: None,
            redirect_override: Some(url.into()),
        }
    }
}

impl From<()> for ActionOk {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

/// Result type for `#[action]` handlers.
pub type ActionResult = Result<ActionOk, ActionError>;

/// Conversion trait for ergonomic `?` mapping into `ActionError`.
///
/// A blanket `impl<E: Display> IntoActionError for E` is provided. Use
/// `Result::map_err(IntoActionError::into_action_error)?` for any error
/// type not covered by the concrete `From` impls below.
pub trait IntoActionError {
    /// Convert into an `ActionError` by formatting `self` as its message.
    fn into_action_error(self) -> ActionError;
}

impl<E: Display> IntoActionError for E {
    fn into_action_error(self) -> ActionError {
        ActionError::msg(self.to_string())
    }
}

// Concrete From impls — per RESEARCH §4.7, do NOT provide a blanket
// `impl<T: IntoActionError> From<T> for ActionError` because it conflicts
// with these concrete impls on coherence (E0119). The four below cover the
// canonical ferro error sources for bare `?`; the trait + `.map_err(...)`
// dance handles everything else.

impl From<FrameworkError> for ActionError {
    fn from(e: FrameworkError) -> Self {
        ActionError::msg(e.to_string())
    }
}
impl From<String> for ActionError {
    fn from(s: String) -> Self {
        ActionError::msg(s)
    }
}
impl From<&'static str> for ActionError {
    fn from(s: &'static str) -> Self {
        ActionError::msg(s)
    }
}
impl From<sea_orm::DbErr> for ActionError {
    fn from(e: sea_orm::DbErr) -> Self {
        ActionError::msg(e.to_string())
    }
}

/// Same-origin check (T-180-02 mitigation). Reuses the rule from
/// `validation/error.rs::is_same_origin` — only `/path` is safe; absolute
/// URLs, scheme-relative `//host`, and javascript: pseudo-URLs are rejected.
fn is_safe_redirect(url: &str) -> bool {
    url.starts_with('/') && !url.starts_with("//")
}

/// Strip control characters from a tracing field value (T-180-03).
///
/// Replaces every `c.is_control()` byte with a single ASCII space so
/// `\n`, `\r`, `\x00`, and friends cannot corrupt structured-log output.
pub(crate) fn sanitize_log_message(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// Build the back-compat query string for an error redirect:
/// `?error=<kind_snake>&msg=<pct-encoded message>`.
fn build_error_query(err: &ActionError) -> String {
    let kind = match err.kind {
        ActionKind::Generic => "generic",
        ActionKind::NotFound => "not_found",
        ActionKind::Forbidden => "forbidden",
        ActionKind::Unauthorized => "unauthorized",
    };
    let encoded: String = byte_serialize(err.message.as_bytes()).collect();
    format!("?error={kind}&msg={encoded}")
}

/// Build the back-compat success query string. Always emits `?success=<key>`
/// where `<key>` is the `ActionOk::flash` value if set, else `1`.
fn build_success_query(ok: &ActionOk) -> String {
    let key = ok.flash.unwrap_or("1");
    format!("?success={key}")
}

/// Runtime dispatcher used by `#[action]`-generated handlers.
///
/// Honors `ActionOk::redirect_override` / `ActionError::redirect_override`
/// when same-origin; falls back to the configured `redirect_to` otherwise.
/// Writes the structured flash payload to the session under the `_action`
/// key (D-06) and appends a `?error=...&msg=...` / `?success=...` back-compat
/// query string. Emits `tracing::error!` (sanitized per T-180-03) on errors
/// and `tracing::warn!` when an unsafe `redirect_override` is rejected.
///
/// `handler_name` is a `&'static str` (the caller passes
/// `concat!(module_path!(), "::", stringify!(fn_name))`) used purely as a
/// tracing field.
pub fn handle_action_result(
    result: ActionResult,
    redirect_to: &str,
    handler_name: &'static str,
) -> Response {
    match result {
        Ok(ok) => {
            let target = ok
                .redirect_override
                .as_deref()
                .and_then(|u| {
                    if is_safe_redirect(u) {
                        Some(u.to_string())
                    } else {
                        tracing::warn!(
                            handler = %handler_name,
                            rejected_url = %sanitize_log_message(u),
                            "action ok redirect_override rejected: not same-origin",
                        );
                        None
                    }
                })
                .unwrap_or_else(|| redirect_to.to_string());

            // Flash write — silently no-ops if no session is active.
            let flash_payload = serde_json::json!({
                "variant": "info",
                "message": ok.flash.unwrap_or(""),
            });
            crate::session::session_mut(|session| {
                session.flash("_action", &flash_payload);
            });

            let url = format!("{target}{}", build_success_query(&ok));
            Ok(HttpResponse::new().status(303).header("Location", url))
        }
        Err(err) => {
            let safe_msg = sanitize_log_message(&err.message);
            tracing::error!(
                handler = %handler_name,
                msg = %safe_msg,
                kind = ?err.kind,
                "action handler error — redirecting",
            );

            let target = err
                .redirect_override
                .as_deref()
                .and_then(|u| {
                    if is_safe_redirect(u) {
                        Some(u.to_string())
                    } else {
                        tracing::warn!(
                            handler = %handler_name,
                            rejected_url = %sanitize_log_message(u),
                            "action err redirect_override rejected: not same-origin",
                        );
                        None
                    }
                })
                .unwrap_or_else(|| redirect_to.to_string());

            let flash_payload = serde_json::json!({
                "variant": match err.flash_variant {
                    FlashVariant::Error => "error",
                    FlashVariant::Warning => "warning",
                    FlashVariant::Info => "info",
                },
                "message": err.message,
            });
            crate::session::session_mut(|session| {
                session.flash("_action", &flash_payload);
            });

            let url = format!("{target}{}", build_error_query(&err));
            Ok(HttpResponse::new().status(303).header("Location", url))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Extract the Location header value from a Response.
    fn location_of(r: Response) -> String {
        match r {
            Ok(resp) | Err(resp) => resp
                .headers()
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("Location"))
                .map(|(_, v)| v.clone())
                .unwrap_or_default(),
        }
    }

    /// Extract the HTTP status code from a Response.
    fn status_of(r: &Response) -> u16 {
        match r {
            Ok(resp) | Err(resp) => resp.status_code(),
        }
    }

    #[test]
    fn action_error_msg_defaults() {
        let e = ActionError::msg("x");
        assert_eq!(e.message, "x");
        assert_eq!(e.kind, ActionKind::Generic);
        assert_eq!(e.flash_variant, FlashVariant::Error);
        assert!(e.redirect_override.is_none(), "D-08: no hardcoded redirect");
    }

    #[test]
    fn action_error_constructors_set_kind_and_keep_no_redirect() {
        assert_eq!(ActionError::not_found("x").kind, ActionKind::NotFound);
        assert!(ActionError::not_found("x").redirect_override.is_none());
        assert_eq!(ActionError::forbidden("x").kind, ActionKind::Forbidden);
        assert!(ActionError::forbidden("x").redirect_override.is_none());
        assert_eq!(
            ActionError::unauthorized("x").kind,
            ActionKind::Unauthorized
        );
        assert!(
            ActionError::unauthorized("x").redirect_override.is_none(),
            "D-08: unauthorized MUST NOT carry a default redirect (no /accedi literal)",
        );
    }

    #[test]
    fn action_error_builders_consume_self() {
        let e = ActionError::msg("x")
            .with_flash(FlashVariant::Warning)
            .redirect_to("/login");
        assert_eq!(e.flash_variant, FlashVariant::Warning);
        assert_eq!(e.redirect_override.as_deref(), Some("/login"));
    }

    #[test]
    fn action_ok_from_unit_is_default() {
        let ok: ActionOk = ().into();
        assert!(ok.flash.is_none());
        assert!(ok.redirect_override.is_none());
    }

    #[test]
    fn action_ok_builders() {
        assert_eq!(ActionOk::flash("created").flash, Some("created"));
        assert_eq!(
            ActionOk::redirect_to("/dashboard/x")
                .redirect_override
                .as_deref(),
            Some("/dashboard/x"),
        );
    }

    #[test]
    fn from_impls_message_round_trip() {
        let fe: ActionError = FrameworkError::Internal {
            message: "fe".into(),
        }
        .into();
        assert_eq!(
            fe.message,
            FrameworkError::Internal {
                message: "fe".into()
            }
            .to_string()
        );

        let se: ActionError = "hello".to_string().into();
        assert_eq!(se.message, "hello");

        let st: ActionError = "static".into();
        assert_eq!(st.message, "static");

        let dbe: ActionError = sea_orm::DbErr::Custom("oops".into()).into();
        assert!(dbe.message.contains("oops"));
    }

    #[test]
    fn into_action_error_blanket_works() {
        let s = "blanket";
        let e = s.into_action_error();
        assert_eq!(e.message, "blanket");
    }

    #[test]
    fn sanitize_log_message_strips_control_chars() {
        let s = sanitize_log_message("a\nb\rc\x00d");
        assert!(!s.contains('\n'));
        assert!(!s.contains('\r'));
        assert!(!s.contains('\x00'));
        assert_eq!(s.len(), 7, "control chars replaced 1:1 with spaces");
    }

    #[test]
    fn handle_action_result_ok_default_303_to_redirect_to_with_success_1() {
        let r = handle_action_result(Ok(ActionOk::default()), "/dashboard/x", "h");
        assert_eq!(status_of(&r), 303);
        assert_eq!(location_of(r), "/dashboard/x?success=1");
    }

    #[test]
    fn handle_action_result_ok_with_flash_key() {
        let r = handle_action_result(Ok(ActionOk::flash("created")), "/dashboard/x", "h");
        assert_eq!(location_of(r), "/dashboard/x?success=created");
    }

    #[test]
    fn handle_action_result_ok_with_safe_override() {
        let r = handle_action_result(Ok(ActionOk::redirect_to("/items/42")), "/dashboard/x", "h");
        assert_eq!(location_of(r), "/items/42?success=1");
    }

    #[test]
    fn handle_action_result_err_default_kind() {
        let r = handle_action_result(Err(ActionError::msg("boom")), "/x", "h");
        assert_eq!(status_of(&r), 303);
        assert_eq!(location_of(r), "/x?error=generic&msg=boom");
    }

    #[test]
    fn handle_action_result_err_kinds_in_query() {
        assert!(location_of(handle_action_result(
            Err(ActionError::not_found("x")),
            "/x",
            "h",
        ))
        .contains("error=not_found"));
        assert!(location_of(handle_action_result(
            Err(ActionError::forbidden("x")),
            "/x",
            "h",
        ))
        .contains("error=forbidden"));
        assert!(location_of(handle_action_result(
            Err(ActionError::unauthorized("x")),
            "/x",
            "h",
        ))
        .contains("error=unauthorized"));
    }

    #[test]
    fn handle_action_result_percent_encodes_message() {
        let r = handle_action_result(Err(ActionError::msg("a b/c?d")), "/x", "h");
        let loc = location_of(r);
        // form_urlencoded::byte_serialize emits +-encoded spaces.
        assert!(loc.contains("msg=a+b%2Fc%3Fd"), "got: {loc}");
    }

    // T-180-02: open redirect via redirect_override
    #[test]
    fn handle_action_result_err_rejects_offsite_redirect_override() {
        let r = handle_action_result(
            Err(ActionError::msg("x").redirect_to("https://evil.example/y")),
            "/safe",
            "h",
        );
        let loc = location_of(r);
        assert!(loc.starts_with("/safe"), "got: {loc}");
        assert!(!loc.contains("evil.example"));
    }

    #[test]
    fn handle_action_result_err_rejects_scheme_relative_redirect_override() {
        let r = handle_action_result(
            Err(ActionError::msg("x").redirect_to("//evil.example")),
            "/safe",
            "h",
        );
        let loc = location_of(r);
        assert!(loc.starts_with("/safe"), "got: {loc}");
    }

    #[test]
    fn handle_action_result_err_rejects_javascript_redirect_override() {
        let r = handle_action_result(
            Err(ActionError::msg("x").redirect_to("javascript:alert(1)")),
            "/safe",
            "h",
        );
        let loc = location_of(r);
        assert!(loc.starts_with("/safe"), "got: {loc}");
    }

    #[test]
    fn handle_action_result_err_accepts_safe_override() {
        let r = handle_action_result(
            Err(ActionError::unauthorized("x").redirect_to("/login")),
            "/dashboard",
            "h",
        );
        let loc = location_of(r);
        assert!(
            loc.starts_with("/login?error=unauthorized&msg=x"),
            "got: {loc}"
        );
    }

    #[test]
    fn handle_action_result_ok_rejects_offsite_redirect_override() {
        let r = handle_action_result(
            Ok(ActionOk::redirect_to("https://evil.example/x")),
            "/safe",
            "h",
        );
        assert!(location_of(r).starts_with("/safe"));
    }
}
