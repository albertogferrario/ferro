pub mod action;
mod body;
pub mod cookie;
mod extract;
mod form_request;
mod multipart;
mod request;
pub mod request_context;
/// API resource and pagination types.
pub mod resources;
mod response;
pub mod sse;

pub use action::{
    ActionError, ActionKind, ActionResult, ActionResultExt, FlashVariant, IntoActionError,
};
pub use body::{collect_body, parse_form, parse_json, FerroBody};
pub use cookie::{parse_cookies, Cookie, CookieOptions, SameSite};
pub use extract::{FromParam, FromRequest};
pub use form_request::FormRequest;
pub use multipart::{validate_mime, validate_size, MultipartForm, UploadedFile};
pub use request::{Request, RequestParts};
pub use request_context::request_host;
pub use resources::{PaginationLinks, PaginationMeta, Resource, ResourceCollection, ResourceMap};
pub use response::{
    HttpResponse, InertiaRedirect, Redirect, RedirectRouteBuilder, Response, ResponseExt,
};
pub use sse::{SseEvent, SseStream};

/// Error type for missing route parameters
///
/// This type is kept for backward compatibility. New code should use
/// `FrameworkError::param()` instead.
#[derive(Debug)]
pub struct ParamError {
    /// Name of the missing route parameter.
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
