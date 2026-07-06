//! Security headers middleware
//!
//! Adds OWASP-recommended security headers to all responses.
//! Provides sensible defaults that work for both Inertia.js and JSON-UI apps
//! without breaking development workflows.
//!
//! Reference: <https://owasp.org/www-project-secure-headers/>

use crate::http::{HttpResponse, Request, Response};
use crate::middleware::{Middleware, Next};
use async_trait::async_trait;

/// Middleware that adds security headers to every response.
///
/// Ships OWASP-recommended defaults out of the box. Each header can be
/// overridden or disabled via the builder API.
///
/// HSTS is **off by default** because it breaks `localhost` over HTTP.
/// Call [`with_hsts`](Self::with_hsts) to enable it in production.
///
/// # Example
///
/// ```rust,ignore
/// use ferro::SecurityHeaders;
///
/// // Use defaults (safe for development)
/// global_middleware!(SecurityHeaders::new());
///
/// // Production: enable HSTS
/// global_middleware!(SecurityHeaders::new().with_hsts());
///
/// // Custom overrides
/// global_middleware!(
///     SecurityHeaders::new()
///         .x_frame_options("SAMEORIGIN")
///         .without("Permissions-Policy")
/// );
/// ```
pub struct SecurityHeaders {
    x_content_type_options: Option<String>,
    x_frame_options: Option<String>,
    content_security_policy: Option<String>,
    referrer_policy: Option<String>,
    permissions_policy: Option<String>,
    cross_origin_opener_policy: Option<String>,
    x_xss_protection: Option<String>,
    strict_transport_security: Option<String>,
}

impl SecurityHeaders {
    /// Create with OWASP-recommended defaults.
    ///
    /// All headers except HSTS are enabled. HSTS is off by default
    /// to avoid breaking development over HTTP.
    pub fn new() -> Self {
        Self {
            x_content_type_options: Some("nosniff".to_string()),
            x_frame_options: Some("DENY".to_string()),
            content_security_policy: Some(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: blob:; \
                 font-src 'self' data:; \
                 connect-src 'self' ws: wss:; \
                 frame-ancestors 'none'"
                    .to_string(),
            ),
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some("geolocation=(), camera=(), microphone=()".to_string()),
            cross_origin_opener_policy: Some("same-origin".to_string()),
            x_xss_protection: Some("0".to_string()),
            strict_transport_security: None,
        }
    }

    /// Enable HSTS with `max-age=31536000; includeSubDomains` (no preload).
    ///
    /// Safe for production use. Does not include `preload` because preload
    /// submission is permanent and affects all subdomains.
    pub fn with_hsts(mut self) -> Self {
        self.strict_transport_security = Some("max-age=31536000; includeSubDomains".to_string());
        self
    }

    /// Enable HSTS with `preload` directive.
    ///
    /// Only use this if you intend to submit your domain to the HSTS preload
    /// list. Preload is permanent — removing a domain takes months.
    pub fn with_hsts_preload(mut self) -> Self {
        self.strict_transport_security =
            Some("max-age=31536000; includeSubDomains; preload".to_string());
        self
    }

    /// Disable HSTS (same as default, for explicitness).
    pub fn without_hsts(mut self) -> Self {
        self.strict_transport_security = None;
        self
    }

    /// Override the X-Frame-Options header value.
    ///
    /// Default is `DENY`. Use `SAMEORIGIN` to allow framing from the same origin.
    pub fn x_frame_options(mut self, value: impl Into<String>) -> Self {
        self.x_frame_options = Some(value.into());
        self
    }

    /// Override the Content-Security-Policy header value.
    pub fn content_security_policy(mut self, value: impl Into<String>) -> Self {
        self.content_security_policy = Some(value.into());
        self
    }

    /// Override the Referrer-Policy header value.
    pub fn referrer_policy(mut self, value: impl Into<String>) -> Self {
        self.referrer_policy = Some(value.into());
        self
    }

    /// Override the Permissions-Policy header value.
    pub fn permissions_policy(mut self, value: impl Into<String>) -> Self {
        self.permissions_policy = Some(value.into());
        self
    }

    /// Override the Cross-Origin-Opener-Policy header value.
    pub fn cross_origin_opener_policy(mut self, value: impl Into<String>) -> Self {
        self.cross_origin_opener_policy = Some(value.into());
        self
    }

    /// Disable a specific header by name.
    ///
    /// The name is matched case-insensitively.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// SecurityHeaders::new()
    ///     .without("X-Frame-Options")
    ///     .without("Permissions-Policy");
    /// ```
    pub fn without(mut self, header_name: &str) -> Self {
        match header_name.to_ascii_lowercase().as_str() {
            "x-content-type-options" => self.x_content_type_options = None,
            "x-frame-options" => self.x_frame_options = None,
            "content-security-policy" => self.content_security_policy = None,
            "referrer-policy" => self.referrer_policy = None,
            "permissions-policy" => self.permissions_policy = None,
            "cross-origin-opener-policy" => self.cross_origin_opener_policy = None,
            "x-xss-protection" => self.x_xss_protection = None,
            "strict-transport-security" => self.strict_transport_security = None,
            _ => {}
        }
        self
    }

    /// Apply all enabled headers to a response.
    pub(crate) fn apply_headers(&self, resp: HttpResponse) -> HttpResponse {
        let mut resp = resp;
        if let Some(ref v) = self.x_content_type_options {
            resp = resp.header("X-Content-Type-Options", v.as_str());
        }
        if let Some(ref v) = self.x_frame_options {
            resp = resp.header("X-Frame-Options", v.as_str());
        }
        if let Some(ref v) = self.content_security_policy {
            resp = resp.header("Content-Security-Policy", v.as_str());
        }
        if let Some(ref v) = self.referrer_policy {
            resp = resp.header("Referrer-Policy", v.as_str());
        }
        if let Some(ref v) = self.permissions_policy {
            resp = resp.header("Permissions-Policy", v.as_str());
        }
        if let Some(ref v) = self.cross_origin_opener_policy {
            resp = resp.header("Cross-Origin-Opener-Policy", v.as_str());
        }
        if let Some(ref v) = self.x_xss_protection {
            resp = resp.header("X-XSS-Protection", v.as_str());
        }
        if let Some(ref v) = self.strict_transport_security {
            resp = resp.header("Strict-Transport-Security", v.as_str());
        }
        resp
    }
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for SecurityHeaders {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let response = next(request).await;
        match response {
            Ok(resp) => Ok(self.apply_headers(resp)),
            Err(resp) => Err(self.apply_headers(resp)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_headers() {
        let sh = SecurityHeaders::new();

        assert_eq!(sh.x_content_type_options.as_deref(), Some("nosniff"));
        assert_eq!(sh.x_frame_options.as_deref(), Some("DENY"));
        assert!(sh
            .content_security_policy
            .as_ref()
            .unwrap()
            .contains("default-src 'self'"));
        assert!(sh
            .content_security_policy
            .as_ref()
            .unwrap()
            .contains("frame-ancestors 'none'"));
        assert_eq!(
            sh.referrer_policy.as_deref(),
            Some("strict-origin-when-cross-origin")
        );
        assert_eq!(
            sh.permissions_policy.as_deref(),
            Some("geolocation=(), camera=(), microphone=()")
        );
        assert_eq!(
            sh.cross_origin_opener_policy.as_deref(),
            Some("same-origin")
        );
        assert_eq!(sh.x_xss_protection.as_deref(), Some("0"));
        assert!(sh.strict_transport_security.is_none());
    }

    #[test]
    fn test_with_hsts() {
        let sh = SecurityHeaders::new().with_hsts();

        let hsts = sh.strict_transport_security.as_ref().unwrap();
        assert!(hsts.contains("max-age=31536000"));
        assert!(hsts.contains("includeSubDomains"));
        assert!(!hsts.contains("preload"));
    }

    #[test]
    fn test_with_hsts_preload() {
        let sh = SecurityHeaders::new().with_hsts_preload();

        let hsts = sh.strict_transport_security.as_ref().unwrap();
        assert!(hsts.contains("max-age=31536000"));
        assert!(hsts.contains("includeSubDomains"));
        assert!(hsts.contains("preload"));
    }

    #[test]
    fn test_builder_overrides() {
        let sh = SecurityHeaders::new().x_frame_options("SAMEORIGIN");
        assert_eq!(sh.x_frame_options.as_deref(), Some("SAMEORIGIN"));

        let sh = SecurityHeaders::new().content_security_policy("default-src 'none'");
        assert_eq!(
            sh.content_security_policy.as_deref(),
            Some("default-src 'none'")
        );

        let sh = SecurityHeaders::new().referrer_policy("no-referrer");
        assert_eq!(sh.referrer_policy.as_deref(), Some("no-referrer"));

        let sh = SecurityHeaders::new().permissions_policy("camera=(self)");
        assert_eq!(sh.permissions_policy.as_deref(), Some("camera=(self)"));

        let sh = SecurityHeaders::new().cross_origin_opener_policy("unsafe-none");
        assert_eq!(
            sh.cross_origin_opener_policy.as_deref(),
            Some("unsafe-none")
        );
    }

    #[test]
    fn test_without_disables_header() {
        let sh = SecurityHeaders::new().without("X-Frame-Options");
        assert!(sh.x_frame_options.is_none());

        // Other headers remain set
        assert!(sh.x_content_type_options.is_some());
        assert!(sh.content_security_policy.is_some());
    }

    #[test]
    fn test_without_case_insensitive() {
        let sh = SecurityHeaders::new().without("x-frame-options");
        assert!(sh.x_frame_options.is_none());

        let sh = SecurityHeaders::new().without("PERMISSIONS-POLICY");
        assert!(sh.permissions_policy.is_none());
    }

    #[test]
    fn test_without_unknown_header_is_noop() {
        let sh = SecurityHeaders::new().without("X-Unknown-Header");
        // All defaults still set
        assert!(sh.x_content_type_options.is_some());
        assert!(sh.x_frame_options.is_some());
        assert!(sh.content_security_policy.is_some());
    }

    #[test]
    fn test_apply_headers() {
        let sh = SecurityHeaders::new();
        let resp = HttpResponse::text("ok");
        let resp = sh.apply_headers(resp);
        let hyper_resp = resp.into_hyper();

        assert_eq!(
            hyper_resp.headers().get("X-Content-Type-Options").unwrap(),
            "nosniff"
        );
        assert_eq!(hyper_resp.headers().get("X-Frame-Options").unwrap(), "DENY");
        assert!(hyper_resp
            .headers()
            .get("Content-Security-Policy")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("default-src 'self'"));
        assert_eq!(
            hyper_resp.headers().get("Referrer-Policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            hyper_resp.headers().get("Permissions-Policy").unwrap(),
            "geolocation=(), camera=(), microphone=()"
        );
        assert_eq!(
            hyper_resp
                .headers()
                .get("Cross-Origin-Opener-Policy")
                .unwrap(),
            "same-origin"
        );
        assert_eq!(hyper_resp.headers().get("X-XSS-Protection").unwrap(), "0");
        // HSTS should not be present by default
        assert!(hyper_resp
            .headers()
            .get("Strict-Transport-Security")
            .is_none());
    }

    #[test]
    fn test_apply_headers_with_hsts() {
        let sh = SecurityHeaders::new().with_hsts();
        let resp = HttpResponse::text("ok");
        let resp = sh.apply_headers(resp);
        let hyper_resp = resp.into_hyper();

        assert!(hyper_resp
            .headers()
            .get("Strict-Transport-Security")
            .is_some());
    }

    #[test]
    fn test_apply_headers_without_disabled() {
        let sh = SecurityHeaders::new()
            .without("X-Frame-Options")
            .without("Permissions-Policy");
        let resp = HttpResponse::text("ok");
        let resp = sh.apply_headers(resp);
        let hyper_resp = resp.into_hyper();

        assert!(hyper_resp.headers().get("X-Frame-Options").is_none());
        assert!(hyper_resp.headers().get("Permissions-Policy").is_none());
        // Others still present
        assert!(hyper_resp.headers().get("X-Content-Type-Options").is_some());
    }

    #[test]
    fn test_default_impl() {
        let from_new = SecurityHeaders::new();
        let from_default = SecurityHeaders::default();

        assert_eq!(
            from_new.x_content_type_options,
            from_default.x_content_type_options
        );
        assert_eq!(from_new.x_frame_options, from_default.x_frame_options);
        assert_eq!(
            from_new.content_security_policy,
            from_default.content_security_policy
        );
        assert_eq!(from_new.referrer_policy, from_default.referrer_policy);
        assert_eq!(from_new.permissions_policy, from_default.permissions_policy);
        assert_eq!(
            from_new.cross_origin_opener_policy,
            from_default.cross_origin_opener_policy
        );
        assert_eq!(from_new.x_xss_protection, from_default.x_xss_protection);
        assert_eq!(
            from_new.strict_transport_security,
            from_default.strict_transport_security
        );
    }
}
