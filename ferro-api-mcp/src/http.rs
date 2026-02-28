use std::time::Duration;

use serde_json::Value;
use url::Url;

use crate::error::Error;
use crate::types::{ApiOperation, ApiParam, ParamLocation};

/// HTTP client for executing API calls against the target service.
pub struct HttpClient {
    client: reqwest::Client,
    base_url: Url,
    api_key: Option<String>,
}

impl HttpClient {
    /// Creates a new `HttpClient` with a 30-second request timeout.
    pub fn new(base_url: Url, api_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            base_url,
            api_key,
        }
    }

    /// Executes an API operation with the given arguments.
    ///
    /// Handles path interpolation, query parameters, request body, and
    /// authorization headers. Returns the parsed JSON response or an error.
    pub async fn execute(
        &self,
        op: &ApiOperation,
        args: &serde_json::Map<String, Value>,
    ) -> Result<Value, Error> {
        let path = interpolate_path(&op.path, &op.parameters, args);

        let url = self
            .base_url
            .join(&path)
            .map_err(|e| Error::HttpClient(format!("invalid URL: {e}")))?;

        let method: reqwest::Method = op
            .method
            .to_uppercase()
            .parse()
            .map_err(|e| Error::HttpClient(format!("invalid HTTP method: {e}")))?;

        let mut request = self.client.request(method.clone(), url);

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {key}"));
        }

        let query_params = build_query_params(&op.parameters, args);
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        let is_body_method = matches!(
            method,
            reqwest::Method::POST | reqwest::Method::PUT | reqwest::Method::PATCH
        );
        if is_body_method {
            if let Some(body) = args.get("body") {
                request = request.json(body);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        let status = response.status();
        let body_text = response
            .text()
            .await
            .map_err(|e| Error::HttpClient(e.to_string()))?;

        if status.is_success() {
            match serde_json::from_str(&body_text) {
                Ok(json) => Ok(json),
                Err(_) => Ok(Value::String(body_text)),
            }
        } else {
            Err(Error::ApiError {
                status: status.as_u16(),
                body: body_text,
            })
        }
    }
}

/// Replaces `{param_name}` placeholders in the path with values from `args`.
///
/// Only parameters with `location == Path` are interpolated. String values are
/// used directly (without surrounding quotes); other types are converted via
/// `to_string()`. Missing parameters leave the placeholder unchanged.
pub fn interpolate_path(
    path: &str,
    params: &[ApiParam],
    args: &serde_json::Map<String, Value>,
) -> String {
    let mut result = path.to_string();

    for param in params {
        if param.location != ParamLocation::Path {
            continue;
        }
        if let Some(value) = args.get(&param.name) {
            let replacement = match value {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            result = result.replace(&format!("{{{}}}", param.name), &replacement);
        }
    }

    result
}

/// Extracts query parameters from `args` for params with `location == Query`.
///
/// Returns key-value pairs suitable for `reqwest::RequestBuilder::query()`.
/// String values are used directly; other types are converted via `to_string()`.
pub fn build_query_params(
    params: &[ApiParam],
    args: &serde_json::Map<String, Value>,
) -> Vec<(String, String)> {
    let mut query = Vec::new();

    for param in params {
        if param.location != ParamLocation::Query {
            continue;
        }
        if let Some(value) = args.get(&param.name) {
            let str_value = match value {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            query.push((param.name.clone(), str_value));
        }
    }

    query
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn path_param(name: &str) -> ApiParam {
        ApiParam {
            name: name.to_string(),
            location: ParamLocation::Path,
            required: true,
            schema: json!({"type": "string"}),
            description: None,
        }
    }

    fn query_param(name: &str) -> ApiParam {
        ApiParam {
            name: name.to_string(),
            location: ParamLocation::Query,
            required: false,
            schema: json!({"type": "string"}),
            description: None,
        }
    }

    #[test]
    fn interpolate_path_single_param() {
        let params = vec![path_param("id")];
        let mut args = serde_json::Map::new();
        args.insert("id".to_string(), json!("123"));

        let result = interpolate_path("/users/{id}", &params, &args);
        assert_eq!(result, "/users/123");
    }

    #[test]
    fn interpolate_path_multiple_params() {
        let params = vec![path_param("user_id"), path_param("post_id")];
        let mut args = serde_json::Map::new();
        args.insert("user_id".to_string(), json!("abc"));
        args.insert("post_id".to_string(), json!("456"));

        let result = interpolate_path("/users/{user_id}/posts/{post_id}", &params, &args);
        assert_eq!(result, "/users/abc/posts/456");
    }

    #[test]
    fn interpolate_path_numeric_value() {
        let params = vec![path_param("id")];
        let mut args = serde_json::Map::new();
        args.insert("id".to_string(), json!(42));

        let result = interpolate_path("/users/{id}", &params, &args);
        assert_eq!(result, "/users/42");
    }

    #[test]
    fn interpolate_path_missing_param() {
        let params = vec![path_param("id")];
        let args = serde_json::Map::new();

        let result = interpolate_path("/users/{id}", &params, &args);
        assert_eq!(result, "/users/{id}");
    }

    #[test]
    fn build_query_params_extracts_query_only() {
        let params = vec![path_param("id"), query_param("page"), query_param("limit")];
        let mut args = serde_json::Map::new();
        args.insert("id".to_string(), json!("123"));
        args.insert("page".to_string(), json!("2"));
        args.insert("limit".to_string(), json!(50));

        let result = build_query_params(&params, &args);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&("page".to_string(), "2".to_string())));
        assert!(result.contains(&("limit".to_string(), "50".to_string())));
    }

    #[test]
    fn build_query_params_empty_when_no_query_params() {
        let params = vec![path_param("id")];
        let mut args = serde_json::Map::new();
        args.insert("id".to_string(), json!("123"));

        let result = build_query_params(&params, &args);
        assert!(result.is_empty());
    }

    #[test]
    fn build_query_params_skips_absent_args() {
        let params = vec![query_param("page"), query_param("limit")];
        let mut args = serde_json::Map::new();
        args.insert("page".to_string(), json!("1"));
        // limit not in args

        let result = build_query_params(&params, &args);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("page".to_string(), "1".to_string()));
    }
}
