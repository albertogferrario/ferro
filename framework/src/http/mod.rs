mod body;
pub mod cookie;
mod extract;
mod form_request;
mod request;
pub mod resources;
mod response;

pub use body::{collect_body, parse_form, parse_json};
pub use cookie::{parse_cookies, Cookie, CookieOptions, SameSite};
pub use extract::{FromParam, FromRequest};
pub use form_request::FormRequest;
pub use request::{Request, RequestParts};
pub use resources::{PaginationLinks, PaginationMeta, Resource, ResourceCollection, ResourceMap};
pub use response::{
    HttpResponse, InertiaRedirect, Redirect, RedirectRouteBuilder, Response, ResponseExt,
};

/// Error type for missing route parameters
///
/// This type is kept for backward compatibility. New code should use
/// `FrameworkError::param()` instead.
#[derive(Debug)]
pub struct ParamError {
    pub param_name: String,
}

impl From<ParamError> for HttpResponse {
    fn from(err: ParamError) -> HttpResponse {
        HttpResponse::json(serde_json::json!({
            "error": format!("Missing required parameter: {}", err.param_name)
        }))
        .status(400)
    }
}

impl From<ParamError> for crate::error::FrameworkError {
    fn from(err: ParamError) -> crate::error::FrameworkError {
        crate::error::FrameworkError::ParamError {
            param_name: err.param_name,
        }
    }
}

impl From<ParamError> for Response {
    fn from(err: ParamError) -> Response {
        Err(HttpResponse::from(crate::error::FrameworkError::from(err)))
    }
}

/// Create a text response
pub fn text(body: impl Into<String>) -> Response {
    Ok(HttpResponse::text(body))
}

/// Create a JSON response from a serde_json::Value
pub fn json(body: serde_json::Value) -> Response {
    Ok(HttpResponse::json(body))
}

/// Create a binary response from raw bytes.
///
/// No default Content-Type is set; add one via `.header()` on the inner `HttpResponse`.
pub fn bytes(body: impl Into<bytes::Bytes>) -> Response {
    Ok(HttpResponse::bytes(body))
}
