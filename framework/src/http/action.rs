//! Runtime types for the `#[action]` proc-macro.
//!
//! `#[action]` decorates POST-style handlers that mutate state and redirect.
//! Per D-03 the user body returns `ActionResult = Result<(), ActionError>`.
//! Success-side overrides (per D-02) are recorded via `Request::flash(...)` and
//! `Request::redirect_to(...)` — see [`crate::http::Request`].
//!
//! # Killer-feature contract
//!
//! Inside an `#[action]`-decorated function body:
//!
//! - `Ok(())` is the success expression — no helper type to construct.
//! - `?` works on `String`, `&'static str`, `FrameworkError`, and (with
//!   `sea_orm` available) `sea_orm::DbErr` via concrete [`From`] impls below.
//! - For any other error type implementing [`std::fmt::Display`], use the
//!   [`ActionResultExt::action_err`] extension method on the `Result` to
//!   convert into [`ActionError`] without a `.map_err` closure.
//!
//! # Security
//!
//! - **T-180-01** (flash message injection): [`ActionError::message`] is treated
//!   as untrusted display text. Consumer templates MUST HTML-escape it.
//! - **T-180-02** (open redirect): [`ActionError::redirect_override`] is
//!   validated as same-origin (path starting with `/`) at use time;
//!   external URLs are rejected and a `tracing::warn!` is emitted.
//! - **T-180-03** (log injection): control characters are stripped from
//!   `message` before any `tracing::error!` call.

use form_urlencoded::byte_serialize;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Semantic kind of an action error. Surfaces in the back-compat query string
/// (`?error=<kind_snake_case>`) and in tracing fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    /// General-purpose error with no specific HTTP semantic.
    #[default]
    Generic,
    /// The requested resource was not found (404-shape).
    NotFound,
    /// The caller is authenticated but lacks permission (403-shape).
    Forbidden,
    /// The caller is not authenticated (401-shape).
    Unauthorized,
}

impl ActionKind {
    pub(crate) fn as_query_str(&self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::NotFound => "not_found",
            Self::Forbidden => "forbidden",
            Self::Unauthorized => "unauthorized",
        }
    }
}

/// Flash banner variant. Templates use this to choose the CSS class.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlashVariant {
    /// Error state — typically rendered with a red/destructive style.
    #[default]
    Error,
    /// Warning state — typically rendered with a yellow/caution style.
    Warning,
    /// Informational state — typically rendered with a blue/neutral style.
    Info,
}

/// Action-handler error type. Drives the 303 redirect, the session flash
/// payload, the back-compat query string, and the `tracing::error!` log line.
///
/// # Security
///
/// `message` is rendered to users via the flash payload. Consumer templates
/// MUST HTML-escape it (T-180-01). `redirect_override` is validated as
/// same-origin at use time (T-180-02).
#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct ActionError {
    /// User-facing error message. Treat as untrusted; templates must HTML-escape (T-180-01).
    pub message: String,
    /// Semantic kind used for routing and back-compat query strings.
    pub kind: ActionKind,
    /// Flash banner variant picked up by consumer templates.
    pub flash_variant: FlashVariant,
    /// Optional redirect override. Validated as same-origin when applied (T-180-02).
    pub redirect_override: Option<String>,
}

impl ActionError {
    /// Generic error — most common constructor.
    pub fn msg(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ActionKind::Generic,
            flash_variant: FlashVariant::Error,
            redirect_override: None,
        }
    }

    /// 404-shape error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::NotFound,
            ..Self::msg(message)
        }
    }

    /// 403-shape error.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::Forbidden,
            ..Self::msg(message)
        }
    }

    /// 401-shape error.
    ///
    /// `redirect_override` defaults to `None` — ferro is project-agnostic and
    /// does not hardcode any consumer auth path (D-08). Callers configure
    /// the redirect target explicitly:
    /// `ActionError::unauthorized("...").redirect_to("/your-login-path")`.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            kind: ActionKind::Unauthorized,
            ..Self::msg(message)
        }
    }

    /// Builder — set the flash variant.
    #[must_use]
    pub fn with_flash(mut self, variant: FlashVariant) -> Self {
        self.flash_variant = variant;
        self
    }

    /// Builder — set the redirect override. The override is validated as
    /// same-origin (T-180-02) when applied by the action runtime; external
    /// URLs are silently rejected and a `tracing::warn!` is emitted.
    #[must_use]
    pub fn redirect_to(mut self, url: impl Into<String>) -> Self {
        self.redirect_override = Some(url.into());
        self
    }
}

impl From<String> for ActionError {
    fn from(s: String) -> Self {
        Self::msg(s)
    }
}

impl From<&'static str> for ActionError {
    fn from(s: &'static str) -> Self {
        Self::msg(s)
    }
}

impl From<crate::error::FrameworkError> for ActionError {
    fn from(err: crate::error::FrameworkError) -> Self {
        Self::msg(err.to_string())
    }
}

impl From<sea_orm::DbErr> for ActionError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::msg(err.to_string())
    }
}

/// Conversion trait for the long-tail Display types not covered by the concrete
/// `From` impls. Use [`ActionResultExt::action_err`] for ergonomic `?`-style
/// conversion at the call site.
pub trait IntoActionError {
    /// Convert this error into an [`ActionError`].
    fn into_action_error(self) -> ActionError;
}

impl<E: std::fmt::Display> IntoActionError for E {
    fn into_action_error(self) -> ActionError {
        ActionError::msg(self.to_string())
    }
}

/// Extension trait on `Result` for converting any `Display` error into an
/// `ActionError` without a `.map_err` closure.
pub trait ActionResultExt<T> {
    /// Convert the error side of this `Result` into an [`ActionError`].
    fn action_err(self) -> Result<T, ActionError>;
}

impl<T, E: IntoActionError> ActionResultExt<T> for Result<T, E> {
    fn action_err(self) -> Result<T, ActionError> {
        self.map_err(|e| e.into_action_error())
    }
}

/// REVISED 2026-05-30 per CONTEXT D-03.
///
/// `()` on the Ok side — `Ok(())` is the success expression. Success-side
/// overrides are recorded via [`crate::http::Request::flash`] and
/// [`crate::http::Request::redirect_to`] (D-02).
pub type ActionResult = Result<(), ActionError>;

/// Internal carrier for success-side overrides recorded by
/// [`crate::http::Request::flash`] / [`crate::http::Request::redirect_to`].
/// Read by [`handle_action_result`] after the user body returns.
#[derive(Debug, Default, Clone)]
pub(crate) struct ActionOverrides {
    pub flash: Option<String>,
    pub redirect_override: Option<String>,
}

/// Same-origin check — replicates the pattern in
/// `framework/src/validation/error.rs:172-179` and `response.rs::same_origin_path_from_referer`.
/// Accepts only relative paths that start with `/` but NOT scheme-relative URLs (`//`),
/// which could redirect to an attacker-controlled host. T-180-02 mitigation.
pub(crate) fn is_same_origin(url: &str) -> bool {
    url.starts_with('/') && !url.starts_with("//")
}

/// Strip control characters from the user-facing message before logging.
/// T-180-03 mitigation.
pub(crate) fn sanitize_for_log(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// JSON payload written to the `_action` session flash slot. Read by consumer
/// templates / shared Inertia props middleware.
#[derive(Debug, Serialize, Deserialize)]
struct ActionFlashPayload<'a> {
    variant: &'a str,
    message: &'a str,
}

/// Runtime helper called from macro-generated code. NOT a public API.
///
/// On `Ok(())`:
///   - Reads `req.action_overrides()` — if `flash` is set, writes
///     `{variant: "success", message: "<flash_key>"}` to the `_action` flash slot.
///   - If `redirect_override` is set AND same-origin, redirects there;
///     otherwise falls back to `redirect_to` (T-180-02).
///   - Appends back-compat `?success=<flash_key_or_1>` to the redirect URL (D-06).
///
/// On `Err(err)`:
///   - Writes `{variant: "<err.flash_variant>", message: "<err.message>"}` to the
///     `_action` flash slot.
///   - If `err.redirect_override` is set AND same-origin, redirects there;
///     otherwise falls back to `redirect_to`.
///   - Appends back-compat `?error=<err.kind>&msg=<pct(err.message)>`.
///   - Emits `tracing::error!(handler=%name, msg=%sanitize, kind=?err.kind, ...)`.
// Called from macro-generated code in Plan 03; no direct call site exists yet.
#[allow(dead_code)]
pub(crate) fn handle_action_result(
    result: ActionResult,
    redirect_to: &'static str,
    handler_name: &'static str,
    req: &mut crate::http::Request,
) -> crate::http::Response {
    match result {
        Ok(()) => {
            let overrides = req.action_overrides().clone();

            let target = match overrides.redirect_override.as_deref() {
                Some(url) if is_same_origin(url) => url.to_string(),
                Some(rejected) => {
                    tracing::warn!(
                        handler = %handler_name,
                        rejected_url = %sanitize_for_log(rejected),
                        "redirect_override rejected: not same-origin (success path)"
                    );
                    redirect_to.to_string()
                }
                None => redirect_to.to_string(),
            };

            // Flash write (success).
            if let Some(key) = overrides.flash.as_deref() {
                let payload = ActionFlashPayload {
                    variant: "success",
                    message: key,
                };
                crate::session::session_mut(|s| s.flash("_action", &payload));
            }

            // Back-compat query string (D-06 fallback).
            let suffix = match overrides.flash.as_deref() {
                Some(k) if !k.is_empty() => format!("?success={k}"),
                _ => "?success=1".to_string(),
            };
            let location = format!("{target}{suffix}");

            Ok(crate::http::HttpResponse::new()
                .status(303)
                .header("Location", &location))
        }
        Err(err) => {
            let safe_msg = sanitize_for_log(&err.message);
            tracing::error!(
                handler = %handler_name,
                msg = %safe_msg,
                kind = ?err.kind,
                "action handler error — redirecting"
            );

            let target = match err.redirect_override.as_deref() {
                Some(url) if is_same_origin(url) => url.to_string(),
                Some(rejected) => {
                    tracing::warn!(
                        handler = %handler_name,
                        rejected_url = %sanitize_for_log(rejected),
                        "redirect_override rejected: not same-origin (error path)"
                    );
                    redirect_to.to_string()
                }
                None => redirect_to.to_string(),
            };

            // Flash write (error / warning / info).
            let variant_str = match err.flash_variant {
                FlashVariant::Error => "error",
                FlashVariant::Warning => "warning",
                FlashVariant::Info => "info",
            };
            let payload = ActionFlashPayload {
                variant: variant_str,
                message: &err.message,
            };
            crate::session::session_mut(|s| s.flash("_action", &payload));

            // Back-compat query string.
            let encoded_msg: String = byte_serialize(err.message.as_bytes()).collect();
            let location = format!(
                "{target}?error={kind}&msg={msg}",
                target = target,
                kind = err.kind.as_query_str(),
                msg = encoded_msg
            );

            Ok(crate::http::HttpResponse::new()
                .status(303)
                .header("Location", &location))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msg_constructor_defaults() {
        let e = ActionError::msg("boom");
        assert_eq!(e.message, "boom");
        assert!(matches!(e.kind, ActionKind::Generic));
        assert!(matches!(e.flash_variant, FlashVariant::Error));
        assert!(e.redirect_override.is_none());
    }

    #[test]
    fn not_found_constructor_sets_kind() {
        let e = ActionError::not_found("missing");
        assert!(matches!(e.kind, ActionKind::NotFound));
    }

    #[test]
    fn forbidden_constructor_sets_kind() {
        let e = ActionError::forbidden("nope");
        assert!(matches!(e.kind, ActionKind::Forbidden));
    }

    #[test]
    fn unauthorized_constructor_no_default_redirect() {
        // D-08: ferro MUST NOT hardcode any auth path.
        let e = ActionError::unauthorized("login first");
        assert!(matches!(e.kind, ActionKind::Unauthorized));
        assert!(
            e.redirect_override.is_none(),
            "ferro must not hardcode a default auth-redirect path (D-08)"
        );
    }

    #[test]
    fn builders_consume_self() {
        let e = ActionError::msg("x")
            .with_flash(FlashVariant::Warning)
            .redirect_to("/login");
        assert!(matches!(e.flash_variant, FlashVariant::Warning));
        assert_eq!(e.redirect_override.as_deref(), Some("/login"));
    }

    #[test]
    fn from_string_impl() {
        let e: ActionError = "oops".to_string().into();
        assert_eq!(e.message, "oops");
    }

    #[test]
    fn from_static_str_impl() {
        let e: ActionError = "static".into();
        assert_eq!(e.message, "static");
    }

    #[test]
    fn from_framework_error_impl() {
        let fe = crate::error::FrameworkError::internal("framework boom");
        let e: ActionError = fe.into();
        assert!(e.message.contains("framework boom"));
    }

    #[test]
    fn into_action_error_blanket_for_display_types() {
        // Any Display type works through the trait.
        let n: i32 = 42;
        let e = n.into_action_error();
        assert_eq!(e.message, "42");
    }

    #[test]
    fn action_err_extension_on_result() {
        let r: Result<(), i32> = Err(7);
        let converted: Result<(), ActionError> = r.action_err();
        assert!(converted.is_err());
        assert_eq!(converted.unwrap_err().message, "7");
    }

    #[test]
    fn sanitize_strips_control_chars() {
        assert_eq!(sanitize_for_log("a\nb\tc\x00d"), "a b c d");
    }

    #[test]
    fn is_same_origin_accepts_relative() {
        assert!(is_same_origin("/dashboard"));
        assert!(is_same_origin("/"));
    }

    #[test]
    fn is_same_origin_rejects_absolute() {
        assert!(!is_same_origin("https://evil.example/"));
        assert!(!is_same_origin("//evil.example/"));
        assert!(!is_same_origin("http://localhost/"));
    }

    #[test]
    fn action_kind_query_strings() {
        assert_eq!(ActionKind::Generic.as_query_str(), "generic");
        assert_eq!(ActionKind::NotFound.as_query_str(), "not_found");
        assert_eq!(ActionKind::Forbidden.as_query_str(), "forbidden");
        assert_eq!(ActionKind::Unauthorized.as_query_str(), "unauthorized");
    }
}
