use crate::http::{HttpResponse, Request};

/// Trait for transforming models into JSON API responses.
///
/// Implement this trait on resource structs to define how models are
/// serialized for API consumers. The `Request` parameter enables
/// context-dependent field selection (e.g., based on auth or roles).
///
/// # Example
///
/// ```rust,ignore
/// use ferro_rs::{Resource, ResourceMap, Request};
/// use serde_json::json;
///
/// struct UserResource {
///     id: i32,
///     name: String,
///     email: String,
/// }
///
/// impl Resource for UserResource {
///     fn to_resource(&self, _req: &Request) -> serde_json::Value {
///         ResourceMap::new()
///             .field("id", json!(self.id))
///             .field("name", json!(self.name))
///             .field("email", json!(self.email))
///             .build()
///     }
/// }
/// ```
pub trait Resource {
    /// Transform this into a JSON value for API responses.
    /// Request is available for context-dependent field selection (auth, roles, etc).
    fn to_resource(&self, req: &Request) -> serde_json::Value;

    /// Return a JSON HTTP response with the resource data.
    fn to_response(&self, req: &Request) -> HttpResponse {
        HttpResponse::json(self.to_resource(req))
    }

    /// Return a JSON HTTP response wrapped in `{"data": ...}` envelope.
    fn to_wrapped_response(&self, req: &Request) -> HttpResponse {
        HttpResponse::json(serde_json::json!({"data": self.to_resource(req)}))
    }

    /// Return a wrapped response with additional top-level fields merged.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let response = resource.to_response_with(&req, json!({"meta": {"version": "v1"}}));
    /// // Output: {"data": {...}, "meta": {"version": "v1"}}
    /// ```
    fn to_response_with(&self, req: &Request, additional: serde_json::Value) -> HttpResponse {
        let mut response = serde_json::json!({"data": self.to_resource(req)});
        if let (Some(obj), Some(add)) = (response.as_object_mut(), additional.as_object()) {
            for (k, v) in add {
                obj.insert(k.clone(), v.clone());
            }
        }
        HttpResponse::json(response)
    }
}
