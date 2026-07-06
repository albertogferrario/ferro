//! Conduit DTOs (request envelopes + response envelopes) and the shared
//! error-envelope helper used by every controller.

pub mod requests;
pub mod responses;

use ferro::serde_json::{json, Map, Value};
use ferro::HttpResponse;

/// Build a Conduit error envelope `{"errors":{field:[msgs...]}}` with the given
/// HTTP status. Conduit error responses are always a `422` (validation) or
/// `401`/`403`/`404`/`409`, never a bare string.
pub fn error_envelope(status: u16, field: &str, msgs: &[&str]) -> HttpResponse {
    let mut errors = Map::new();
    errors.insert(
        field.to_string(),
        Value::Array(msgs.iter().map(|m| json!(m)).collect()),
    );
    HttpResponse::json(json!({ "errors": errors })).status(status)
}

/// 422 validation envelope for a single field (`"can't be blank"` style).
pub fn validation_error_envelope(field: &str, msgs: &[&str]) -> HttpResponse {
    error_envelope(422, field, msgs)
}
