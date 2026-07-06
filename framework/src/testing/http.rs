//! HTTP test client for making requests to the application
//!
//! Provides a fluent API for testing HTTP endpoints with assertions.
//!
//! # Example
//!
//! ```rust,ignore
//! use ferro_rs::testing::{TestClient, TestResponse};
//!
//! #[tokio::test]
//! async fn test_api_endpoint() {
//!     let client = TestClient::new();
//!
//!     let response = client
//!         .get("/api/users")
//!         .header("Accept", "application/json")
//!         .send()
//!         .await;
//!
//!     response
//!         .assert_status(200)
//!         .assert_json_path("$.users", |users| users.is_array());
//! }
//! ```

use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::routing::Router;

/// HTTP test client for making requests to the application
pub struct TestClient {
    #[allow(dead_code)]
    router: Option<Arc<Router>>,
    default_headers: HeaderMap,
}

impl TestClient {
    /// Create a new test client without a router (for unit tests)
    pub fn new() -> Self {
        Self {
            router: None,
            default_headers: HeaderMap::new(),
        }
    }

    /// Create a test client with a router for integration tests
    pub fn with_router(router: Router) -> Self {
        Self {
            router: Some(Arc::new(router)),
            default_headers: HeaderMap::new(),
        }
    }

    /// Add a default header to all requests
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(name), HeaderValue::try_from(value)) {
            self.default_headers.insert(name, value);
        }
        self
    }

    /// Add JSON accept header by default
    pub fn json(self) -> Self {
        self.with_header("Accept", "application/json")
            .with_header("Content-Type", "application/json")
    }

    /// Start building a GET request
    pub fn get(&self, path: &str) -> TestRequestBuilder<'_> {
        TestRequestBuilder::new(self, Method::GET, path)
    }

    /// Start building a POST request
    pub fn post(&self, path: &str) -> TestRequestBuilder<'_> {
        TestRequestBuilder::new(self, Method::POST, path)
    }

    /// Start building a PUT request
    pub fn put(&self, path: &str) -> TestRequestBuilder<'_> {
        TestRequestBuilder::new(self, Method::PUT, path)
    }

    /// Start building a PATCH request
    pub fn patch(&self, path: &str) -> TestRequestBuilder<'_> {
        TestRequestBuilder::new(self, Method::PATCH, path)
    }

    /// Start building a DELETE request
    pub fn delete(&self, path: &str) -> TestRequestBuilder<'_> {
        TestRequestBuilder::new(self, Method::DELETE, path)
    }
}

impl Default for TestClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing test requests
pub struct TestRequestBuilder<'a> {
    #[allow(dead_code)]
    client: &'a TestClient,
    #[allow(dead_code)]
    method: Method,
    path: String,
    headers: HeaderMap,
    body: Option<Bytes>,
    query_params: HashMap<String, String>,
}

impl<'a> TestRequestBuilder<'a> {
    fn new(client: &'a TestClient, method: Method, path: &str) -> Self {
        let headers = client.default_headers.clone();

        Self {
            client,
            method,
            path: path.to_string(),
            headers,
            body: None,
            query_params: HashMap::new(),
        }
    }

    /// Add a header to the request
    pub fn header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(name), Ok(value)) = (HeaderName::try_from(name), HeaderValue::try_from(value)) {
            self.headers.insert(name, value);
        }
        self
    }

    /// Add a bearer token authorization header
    pub fn bearer_token(self, token: &str) -> Self {
        self.header("Authorization", &format!("Bearer {token}"))
    }

    /// Add basic auth header
    pub fn basic_auth(self, username: &str, password: &str) -> Self {
        use base64::Engine;
        let credentials =
            base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
        self.header("Authorization", &format!("Basic {credentials}"))
    }

    /// Add a query parameter
    pub fn query(mut self, key: &str, value: &str) -> Self {
        self.query_params.insert(key.to_string(), value.to_string());
        self
    }

    /// Set the request body as raw bytes
    pub fn body(mut self, body: impl Into<Bytes>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set the request body as JSON
    pub fn json<T: serde::Serialize>(mut self, data: &T) -> Self {
        if let Ok(bytes) = serde_json::to_vec(data) {
            self.body = Some(Bytes::from(bytes));
            self.headers.insert(
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("application/json"),
            );
        }
        self
    }

    /// Set form-urlencoded body
    pub fn form(mut self, data: &[(String, String)]) -> Self {
        let encoded = serde_urlencoded::to_string(data).unwrap_or_default();
        self.body = Some(Bytes::from(encoded));
        self.headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        self
    }

    /// Build the full path with query parameters
    fn build_path(&self) -> String {
        if self.query_params.is_empty() {
            self.path.clone()
        } else {
            let query = self
                .query_params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", self.path, query)
        }
    }

    /// Send the request and get a test response
    ///
    /// For now, this creates a mock response. In a full implementation,
    /// this would route through the actual application.
    pub async fn send(self) -> TestResponse {
        // Build the request for potential router handling
        let _full_path = self.build_path();

        // TODO: When router is available, route through actual handlers
        // For now, return a mock response for testing the API

        TestResponse {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Bytes::new(),
            location: None,
        }
    }
}

/// Test response with assertion methods
#[derive(Debug, Clone)]
pub struct TestResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
    location: Option<String>,
}

impl TestResponse {
    /// Create a new test response
    pub fn new(status: StatusCode, headers: HeaderMap, body: Bytes) -> Self {
        let location = headers
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            status,
            headers,
            body,
            location,
        }
    }

    /// Create a test response from status, headers, and body
    pub fn from_parts(status: u16, headers: Vec<(&str, &str)>, body: impl Into<Bytes>) -> Self {
        let mut header_map = HeaderMap::new();
        for (name, value) in headers {
            if let (Ok(n), Ok(v)) = (HeaderName::try_from(name), HeaderValue::try_from(value)) {
                header_map.insert(n, v);
            }
        }

        let location = header_map
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            status: StatusCode::from_u16(status).unwrap_or(StatusCode::OK),
            headers: header_map,
            body: body.into(),
            location,
        }
    }

    /// Get the response status code
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the response headers
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a specific header value
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// Get the response body as bytes
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// Get the response body as a string
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// Parse the response body as JSON
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Get the response body as a JSON Value
    pub fn json_value(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    /// Get the redirect location if any
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    // === Assertion Methods ===

    /// Assert the response has a specific status code
    pub fn assert_status(self, expected: u16) -> Self {
        let actual = self.status.as_u16();
        if actual != expected {
            panic!(
                "\nHTTP Status Assertion Failed\n\n  Expected: {}\n  Received: {}\n  Body: {}\n",
                expected,
                actual,
                self.text()
            );
        }
        self
    }

    /// Assert the response is successful (2xx)
    pub fn assert_ok(self) -> Self {
        if !self.status.is_success() {
            panic!(
                "\nHTTP Status Assertion Failed\n\n  Expected: 2xx (success)\n  Received: {}\n  Body: {}\n",
                self.status.as_u16(),
                self.text()
            );
        }
        self
    }

    /// Assert the response is a redirect (3xx)
    pub fn assert_redirect(self) -> Self {
        if !self.status.is_redirection() {
            panic!(
                "\nHTTP Status Assertion Failed\n\n  Expected: 3xx (redirect)\n  Received: {}\n",
                self.status.as_u16()
            );
        }
        self
    }

    /// Assert the response redirects to a specific path
    pub fn assert_redirect_to(self, expected_path: &str) -> Self {
        if !self.status.is_redirection() {
            panic!(
                "\nHTTP Status Assertion Failed\n\n  Expected: 3xx (redirect)\n  Received: {}\n",
                self.status.as_u16()
            );
        }

        match &self.location {
            Some(location) if location.contains(expected_path) => self,
            Some(location) => {
                panic!(
                    "\nRedirect Location Assertion Failed\n\n  Expected to contain: {expected_path}\n  Received: {location}\n"
                );
            }
            None => {
                panic!(
                    "\nRedirect Location Assertion Failed\n\n  Expected Location header but none found\n"
                );
            }
        }
    }

    /// Assert the response is a client error (4xx)
    pub fn assert_client_error(self) -> Self {
        if !self.status.is_client_error() {
            panic!(
                "\nHTTP Status Assertion Failed\n\n  Expected: 4xx (client error)\n  Received: {}\n",
                self.status.as_u16()
            );
        }
        self
    }

    /// Assert the response is a server error (5xx)
    pub fn assert_server_error(self) -> Self {
        if !self.status.is_server_error() {
            panic!(
                "\nHTTP Status Assertion Failed\n\n  Expected: 5xx (server error)\n  Received: {}\n",
                self.status.as_u16()
            );
        }
        self
    }

    /// Assert the response is not found (404)
    pub fn assert_not_found(self) -> Self {
        self.assert_status(404)
    }

    /// Assert the response is unauthorized (401)
    pub fn assert_unauthorized(self) -> Self {
        self.assert_status(401)
    }

    /// Assert the response is forbidden (403)
    pub fn assert_forbidden(self) -> Self {
        self.assert_status(403)
    }

    /// Assert the response is unprocessable entity (422)
    pub fn assert_unprocessable(self) -> Self {
        self.assert_status(422)
    }

    /// Assert the response has a specific header
    pub fn assert_header(self, name: &str, expected: &str) -> Self {
        match self.header(name) {
            Some(actual) if actual == expected => self,
            Some(actual) => {
                panic!(
                    "\nHeader Assertion Failed\n\n  Header: {name}\n  Expected: {expected}\n  Received: {actual}\n"
                );
            }
            None => {
                panic!(
                    "\nHeader Assertion Failed\n\n  Header '{}' not found in response\n  Available headers: {:?}\n",
                    name,
                    self.headers.keys().map(|k| k.as_str()).collect::<Vec<_>>()
                );
            }
        }
    }

    /// Assert the response has a header (regardless of value)
    pub fn assert_header_exists(self, name: &str) -> Self {
        if self.header(name).is_none() {
            panic!(
                "\nHeader Assertion Failed\n\n  Expected header '{}' to exist\n  Available headers: {:?}\n",
                name,
                self.headers.keys().map(|k| k.as_str()).collect::<Vec<_>>()
            );
        }
        self
    }

    /// Assert the response is JSON
    pub fn assert_json(self) -> Self {
        let content_type = self.header("content-type").unwrap_or("");
        if !content_type.contains("application/json") {
            panic!(
                "\nContent-Type Assertion Failed\n\n  Expected: application/json\n  Received: {content_type}\n"
            );
        }
        self
    }

    /// Assert the response JSON contains a specific key
    pub fn assert_json_has(self, key: &str) -> Self {
        match self.json_value() {
            Ok(json) => {
                if json.get(key).is_none() {
                    panic!(
                        "\nJSON Assertion Failed\n\n  Expected key '{}' in JSON\n  Received: {}\n",
                        key,
                        serde_json::to_string_pretty(&json).unwrap_or_default()
                    );
                }
                self
            }
            Err(e) => {
                panic!(
                    "\nJSON Parse Error\n\n  Error: {}\n  Body: {}\n",
                    e,
                    self.text()
                );
            }
        }
    }

    /// Assert the response JSON has a specific value at a key
    pub fn assert_json_is<T: serde::Serialize + Debug>(self, key: &str, expected: T) -> Self {
        match self.json_value() {
            Ok(json) => {
                let expected_value = serde_json::to_value(&expected).unwrap();
                match json.get(key) {
                    Some(actual) if actual == &expected_value => self,
                    Some(actual) => {
                        panic!(
                            "\nJSON Value Assertion Failed\n\n  Key: {key}\n  Expected: {expected_value:?}\n  Received: {actual:?}\n"
                        );
                    }
                    None => {
                        panic!(
                            "\nJSON Assertion Failed\n\n  Key '{}' not found in JSON\n  Available keys: {:?}\n",
                            key,
                            json.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()
                        );
                    }
                }
            }
            Err(e) => {
                panic!(
                    "\nJSON Parse Error\n\n  Error: {}\n  Body: {}\n",
                    e,
                    self.text()
                );
            }
        }
    }

    /// Assert the response JSON matches a predicate at a key
    pub fn assert_json_matches<F>(self, key: &str, predicate: F) -> Self
    where
        F: FnOnce(&Value) -> bool,
    {
        match self.json_value() {
            Ok(json) => match json.get(key) {
                Some(value) if predicate(value) => self,
                Some(value) => {
                    panic!(
                            "\nJSON Predicate Assertion Failed\n\n  Key: {key}\n  Value: {value:?}\n  The predicate returned false\n"
                        );
                }
                None => {
                    panic!("\nJSON Assertion Failed\n\n  Key '{key}' not found in JSON\n");
                }
            },
            Err(e) => {
                panic!(
                    "\nJSON Parse Error\n\n  Error: {}\n  Body: {}\n",
                    e,
                    self.text()
                );
            }
        }
    }

    /// Assert the response JSON equals the expected structure
    pub fn assert_json_equals<T: serde::Serialize + Debug>(self, expected: T) -> Self {
        match self.json_value() {
            Ok(actual) => {
                let expected_value = serde_json::to_value(&expected).unwrap();
                if actual != expected_value {
                    panic!(
                        "\nJSON Equality Assertion Failed\n\n  Expected:\n{}\n\n  Received:\n{}\n",
                        serde_json::to_string_pretty(&expected_value).unwrap_or_default(),
                        serde_json::to_string_pretty(&actual).unwrap_or_default()
                    );
                }
                self
            }
            Err(e) => {
                panic!(
                    "\nJSON Parse Error\n\n  Error: {}\n  Body: {}\n",
                    e,
                    self.text()
                );
            }
        }
    }

    /// Assert the response body contains a string
    pub fn assert_see(self, needle: &str) -> Self {
        let body = self.text();
        if !body.contains(needle) {
            panic!("\nBody Assertion Failed\n\n  Expected to see: {needle}\n  Body:\n{body}\n");
        }
        self
    }

    /// Assert the response body does not contain a string
    pub fn assert_dont_see(self, needle: &str) -> Self {
        let body = self.text();
        if body.contains(needle) {
            panic!("\nBody Assertion Failed\n\n  Expected NOT to see: {needle}\n  Body:\n{body}\n");
        }
        self
    }

    /// Assert the JSON response has validation errors for specific fields
    pub fn assert_validation_errors(self, fields: &[&str]) -> Self {
        match self.json_value() {
            Ok(json) => {
                // Check for common validation error structures
                let errors = json
                    .get("errors")
                    .or_else(|| json.get("validation_errors"))
                    .or_else(|| {
                        json.get("message")
                            .and_then(|m| if m.is_object() { Some(m) } else { None })
                    });

                match errors {
                    Some(errors_obj) => {
                        for field in fields {
                            if errors_obj.get(*field).is_none() {
                                panic!(
                                    "\nValidation Error Assertion Failed\n\n  Expected error for field: {}\n  Errors: {}\n",
                                    field,
                                    serde_json::to_string_pretty(errors_obj).unwrap_or_default()
                                );
                            }
                        }
                        self
                    }
                    None => {
                        panic!(
                            "\nValidation Error Assertion Failed\n\n  Expected 'errors' key in response\n  Response: {}\n",
                            serde_json::to_string_pretty(&json).unwrap_or_default()
                        );
                    }
                }
            }
            Err(e) => {
                panic!(
                    "\nJSON Parse Error\n\n  Error: {}\n  Body: {}\n",
                    e,
                    self.text()
                );
            }
        }
    }

    /// Assert the JSON array has the expected count
    pub fn assert_json_count(self, key: &str, expected: usize) -> Self {
        match self.json_value() {
            Ok(json) => match json.get(key) {
                Some(Value::Array(arr)) if arr.len() == expected => self,
                Some(Value::Array(arr)) => {
                    panic!(
                            "\nJSON Count Assertion Failed\n\n  Key: {}\n  Expected: {} items\n  Received: {} items\n",
                            key, expected, arr.len()
                        );
                }
                Some(other) => {
                    panic!(
                        "\nJSON Count Assertion Failed\n\n  Key '{}' is not an array\n  Type: {}\n",
                        key,
                        match other {
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Object(_) => "object",
                            Value::Array(_) => "array",
                        }
                    );
                }
                None => {
                    panic!("\nJSON Count Assertion Failed\n\n  Key '{key}' not found\n");
                }
            },
            Err(e) => {
                panic!(
                    "\nJSON Parse Error\n\n  Error: {}\n  Body: {}\n",
                    e,
                    self.text()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_assert_status() {
        let response = TestResponse::from_parts(200, vec![], "");
        response.assert_status(200);
    }

    #[test]
    fn test_response_assert_ok() {
        let response = TestResponse::from_parts(201, vec![], "");
        response.assert_ok();
    }

    #[test]
    fn test_response_assert_json_has() {
        let body = r#"{"name": "test", "email": "test@example.com"}"#;
        let response =
            TestResponse::from_parts(200, vec![("content-type", "application/json")], body);
        response.assert_json_has("name").assert_json_has("email");
    }

    #[test]
    fn test_response_assert_json_is() {
        let body = r#"{"count": 5, "name": "test"}"#;
        let response =
            TestResponse::from_parts(200, vec![("content-type", "application/json")], body);
        response
            .assert_json_is("count", 5)
            .assert_json_is("name", "test");
    }

    #[test]
    fn test_response_assert_see() {
        let body = "Hello, World!";
        let response = TestResponse::from_parts(200, vec![], body);
        response.assert_see("Hello").assert_dont_see("Goodbye");
    }

    #[test]
    fn test_response_assert_redirect() {
        let response = TestResponse::from_parts(302, vec![("location", "/dashboard")], "");
        response.assert_redirect().assert_redirect_to("/dashboard");
    }

    #[test]
    fn test_response_assert_json_count() {
        let body = r#"{"items": [1, 2, 3]}"#;
        let response =
            TestResponse::from_parts(200, vec![("content-type", "application/json")], body);
        response.assert_json_count("items", 3);
    }
}
