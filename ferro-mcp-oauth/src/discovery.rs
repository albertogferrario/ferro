//! OAuth discovery metadata handlers (RFC 8414 / RFC 9728).
//!
//! Implements:
//! - `GET /.well-known/oauth-protected-resource` (RFC 9728 §2)
//! - `GET /.well-known/oauth-authorization-server` (RFC 8414 §2)
//!
//! Both endpoints are public (pre-auth) and read only `APP_URL` via
//! `sanitized_app_url()` — they never fail closed on `MCP_TOKEN_SECRET`.

use serde_json::{json, Value};

/// Build the RFC 9728 protected-resource metadata JSON for the given `app_url`.
///
/// `resource` is the MCP endpoint (`{app_url}/mcp`).
/// `authorization_servers` lists the OAuth server (`[app_url]`).
pub(crate) fn protected_resource_metadata(app_url: &str) -> Value {
    json!({
        "resource": format!("{}/mcp", app_url),
        "authorization_servers": [app_url],
    })
}

/// Build the RFC 8414 authorization-server metadata JSON for the given `app_url`.
///
/// Advertises authorization-code + PKCE S256 as required by the MCP spec, and
/// the device authorization grant (RFC 8628 §4) including `device_authorization_endpoint`.
pub(crate) fn authorization_server_metadata(app_url: &str) -> Value {
    json!({
        "issuer": app_url,
        "authorization_endpoint": format!("{}/authorize", app_url),
        "token_endpoint": format!("{}/token", app_url),
        "registration_endpoint": format!("{}/register", app_url),
        "device_authorization_endpoint": format!("{}/device_authorization", app_url),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "urn:ietf:params:oauth:grant-type:device_code"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
    })
}

/// Handler: `GET /.well-known/oauth-protected-resource` (RFC 9728).
///
/// Public endpoint — no auth required, reads only `APP_URL`.
#[ferro::handler]
pub async fn protected_resource_handler(_req: ferro::Request) -> ferro::Response {
    let url = crate::config::sanitized_app_url();
    Ok(ferro::HttpResponse::json(protected_resource_metadata(&url)))
}

/// Handler: `GET /.well-known/oauth-authorization-server` (RFC 8414).
///
/// Public endpoint — no auth required, reads only `APP_URL`.
#[ferro::handler]
pub async fn authorization_server_handler(_req: ferro::Request) -> ferro::Response {
    let url = crate::config::sanitized_app_url();
    Ok(ferro::HttpResponse::json(authorization_server_metadata(
        &url,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_resource_has_resource_and_authorization_servers() {
        std::env::set_var("APP_URL", "https://app.example.com");
        let url = crate::config::sanitized_app_url();
        std::env::remove_var("APP_URL");

        let val = protected_resource_metadata(&url);
        assert_eq!(
            val["resource"].as_str().unwrap(),
            "https://app.example.com/mcp"
        );
        let servers = val["authorization_servers"].as_array().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].as_str().unwrap(), "https://app.example.com");
    }

    #[test]
    fn authorization_server_has_all_required_fields() {
        let val = authorization_server_metadata("https://app.example.com");

        assert_eq!(val["issuer"].as_str().unwrap(), "https://app.example.com");
        assert_eq!(
            val["authorization_endpoint"].as_str().unwrap(),
            "https://app.example.com/authorize"
        );
        assert_eq!(
            val["token_endpoint"].as_str().unwrap(),
            "https://app.example.com/token"
        );
        assert_eq!(
            val["registration_endpoint"].as_str().unwrap(),
            "https://app.example.com/register"
        );

        let response_types = val["response_types_supported"].as_array().unwrap();
        assert_eq!(response_types[0].as_str().unwrap(), "code");

        let grant_types = val["grant_types_supported"].as_array().unwrap();
        assert!(grant_types
            .iter()
            .any(|v| v.as_str() == Some("authorization_code")));

        let pkce_methods = val["code_challenge_methods_supported"].as_array().unwrap();
        assert_eq!(pkce_methods[0].as_str().unwrap(), "S256");

        let auth_methods = val["token_endpoint_auth_methods_supported"]
            .as_array()
            .unwrap();
        assert_eq!(auth_methods[0].as_str().unwrap(), "none");
    }

    #[test]
    fn discovery_advertises_device_authorization_endpoint() {
        let val = authorization_server_metadata("https://app.example.com");
        assert_eq!(
            val["device_authorization_endpoint"].as_str().unwrap(),
            "https://app.example.com/device_authorization"
        );
    }

    #[test]
    fn discovery_advertises_device_grant_type() {
        let val = authorization_server_metadata("https://app.example.com");
        let grant_types = val["grant_types_supported"].as_array().unwrap();
        assert!(grant_types
            .iter()
            .any(|v| v.as_str() == Some("authorization_code")));
        assert!(grant_types
            .iter()
            .any(|v| v.as_str() == Some("urn:ietf:params:oauth:grant-type:device_code")));
    }

    #[test]
    fn discovery_urls_interpolate_app_url_no_hardcoded_host() {
        let custom_url = "https://custom.host.io";
        let val = authorization_server_metadata(custom_url);
        assert!(
            val["authorization_endpoint"]
                .as_str()
                .unwrap()
                .starts_with(custom_url),
            "authorization_endpoint must start with app_url"
        );
        assert!(
            val["token_endpoint"]
                .as_str()
                .unwrap()
                .starts_with(custom_url),
            "token_endpoint must start with app_url"
        );
    }
}
