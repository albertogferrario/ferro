use crate::http::{HttpResponse, Request};

#[allow(unused_imports)]
use super::ResourceMap;

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::oneshot;

    struct TestUser {
        id: i32,
        name: String,
        email: String,
    }

    impl Resource for TestUser {
        fn to_resource(&self, _req: &Request) -> serde_json::Value {
            ResourceMap::new()
                .field("id", json!(self.id))
                .field("name", json!(self.name))
                .field("email", json!(self.email))
                .build()
        }
    }

    /// Helper to execute a closure with a real Request object.
    /// Uses a TCP loopback to create a genuine hyper::body::Incoming.
    async fn with_test_request<F, R>(f: F) -> R
    where
        F: FnOnce(Request) -> R + Send + 'static,
        R: Send + 'static,
    {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (result_tx, result_rx) = oneshot::channel();

        let callback = Arc::new(std::sync::Mutex::new(Some(f)));
        let tx_holder = Arc::new(std::sync::Mutex::new(Some(result_tx)));

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let io = hyper_util::rt::TokioIo::new(stream);

            let _ = hyper::server::conn::http1::Builder::new()
                .serve_connection(
                    io,
                    hyper::service::service_fn({
                        let callback = callback.clone();
                        let tx_holder = tx_holder.clone();
                        move |req| {
                            let cb = callback.lock().unwrap().take();
                            let tx = tx_holder.lock().unwrap().take();
                            let ferro_req = Request::new(req);
                            if let (Some(cb), Some(tx)) = (cb, tx) {
                                let result = cb(ferro_req);
                                let _ = tx.send(result);
                            }
                            async {
                                Ok::<_, std::convert::Infallible>(hyper::Response::new(
                                    http_body_util::Full::new(bytes::Bytes::from("ok")),
                                ))
                            }
                        }
                    }),
                )
                .await;
        });

        // Send a GET request to trigger the callback
        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
        tokio::spawn(conn);

        let req = hyper::Request::builder()
            .uri("/")
            .body(http_body_util::Empty::<bytes::Bytes>::new())
            .unwrap();
        let _ = sender.send_request(req).await;

        let result = result_rx.await.expect("server should have sent result");
        server.abort();
        result
    }

    #[tokio::test]
    async fn test_resource_to_resource() {
        let user = TestUser {
            id: 42,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let result = with_test_request(move |req| user.to_resource(&req)).await;
        assert_eq!(
            result,
            json!({"id": 42, "name": "Alice", "email": "alice@example.com"})
        );
    }

    #[tokio::test]
    async fn test_resource_to_response() {
        let user = TestUser {
            id: 1,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        };

        let status = with_test_request(move |req| user.to_response(&req).status_code()).await;
        assert_eq!(status, 200);
    }

    #[tokio::test]
    async fn test_resource_to_wrapped_response() {
        let user = TestUser {
            id: 1,
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
        };

        let status =
            with_test_request(move |req| user.to_wrapped_response(&req).status_code()).await;
        assert_eq!(status, 200);
    }
}
