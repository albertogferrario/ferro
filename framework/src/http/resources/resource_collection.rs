use super::pagination::PaginationMeta;
use super::resource::Resource;
use crate::http::{HttpResponse, Request};
use serde_json::{json, Value};

/// A collection of resources with optional pagination metadata.
///
/// Wraps `Vec<T: Resource>` and produces the standard JSON envelope:
/// - Without pagination: `{"data": [...]}`
/// - With pagination: `{"data": [...], "meta": {...}, "links": {...}}`
///
/// # Example
///
/// ```rust,ignore
/// use ferro::{Resource, ResourceCollection, PaginationMeta};
///
/// let resources: Vec<UserResource> = users.into_iter()
///     .map(UserResource::from)
///     .collect();
///
/// // Without pagination
/// let collection = ResourceCollection::new(resources);
/// Ok(collection.to_response(&req))
///
/// // With pagination
/// let meta = PaginationMeta::new(page, per_page, total);
/// let collection = ResourceCollection::paginated(resources, meta);
/// Ok(collection.to_response(&req))
/// ```
pub struct ResourceCollection<T: Resource> {
    items: Vec<T>,
    pagination: Option<PaginationMeta>,
    additional: Option<Value>,
}

impl<T: Resource> ResourceCollection<T> {
    /// Create a collection without pagination.
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            pagination: None,
            additional: None,
        }
    }

    /// Create a collection with pagination metadata.
    pub fn paginated(items: Vec<T>, meta: PaginationMeta) -> Self {
        Self {
            items,
            pagination: Some(meta),
            additional: None,
        }
    }

    /// Add extra top-level fields merged alongside data/meta/links.
    pub fn additional(mut self, value: Value) -> Self {
        self.additional = Some(value);
        self
    }

    /// Produce the JSON response.
    ///
    /// - Without pagination: `{"data": [...]}`
    /// - With pagination: `{"data": [...], "meta": {...}, "links": {...}}`
    /// - With additional: merges additional fields at top level
    pub fn to_response(&self, req: &Request) -> HttpResponse {
        let data: Vec<Value> = self
            .items
            .iter()
            .map(|item| item.to_resource(req))
            .collect();

        let mut response = json!({ "data": data });

        if let Some(meta) = &self.pagination {
            let path = req.path();
            let query = req.inner().uri().query();
            let links = meta.links(path, query);

            if let Some(obj) = response.as_object_mut() {
                obj.insert("meta".to_string(), serde_json::to_value(meta).unwrap());
                obj.insert("links".to_string(), serde_json::to_value(&links).unwrap());
            }
        }

        if let Some(additional) = &self.additional {
            if let (Some(obj), Some(add)) = (response.as_object_mut(), additional.as_object()) {
                for (k, v) in add {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }

        HttpResponse::json(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::resources::ResourceMap;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::oneshot;

    struct TestItem {
        id: i32,
        name: String,
    }

    impl Resource for TestItem {
        fn to_resource(&self, _req: &Request) -> Value {
            ResourceMap::new()
                .field("id", json!(self.id))
                .field("name", json!(self.name))
                .build()
        }
    }

    /// Helper to execute a closure with a real Request object.
    /// Uses a TCP loopback to create a genuine hyper::body::Incoming.
    async fn with_test_request_uri<F, R>(uri: &str, f: F) -> R
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

        let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await.unwrap();
        tokio::spawn(conn);

        let req = hyper::Request::builder()
            .uri(uri)
            .body(http_body_util::Empty::<bytes::Bytes>::new())
            .unwrap();
        let _ = sender.send_request(req).await;

        let result = result_rx.await.expect("server should have sent result");
        server.abort();
        result
    }

    fn make_items(count: usize) -> Vec<TestItem> {
        (1..=count)
            .map(|i| TestItem {
                id: i as i32,
                name: format!("Item {}", i),
            })
            .collect()
    }

    #[tokio::test]
    async fn test_collection_no_pagination() {
        let items = make_items(2);
        let result = with_test_request_uri("/items", move |req| {
            let collection = ResourceCollection::new(items);
            let response = collection.to_response(&req);
            let body: Value = serde_json::from_str(response.body()).unwrap();
            body
        })
        .await;

        assert_eq!(
            result,
            json!({
                "data": [
                    {"id": 1, "name": "Item 1"},
                    {"id": 2, "name": "Item 2"}
                ]
            })
        );
    }

    #[tokio::test]
    async fn test_collection_with_pagination() {
        let items = make_items(2);
        let result = with_test_request_uri("/items?page=1", move |req| {
            let meta = PaginationMeta::new(1, 2, 6);
            let collection = ResourceCollection::paginated(items, meta);
            let response = collection.to_response(&req);
            let body: Value = serde_json::from_str(response.body()).unwrap();
            body
        })
        .await;

        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("data"));
        assert!(obj.contains_key("meta"));
        assert!(obj.contains_key("links"));

        let meta = &obj["meta"];
        assert_eq!(meta["current_page"], 1);
        assert_eq!(meta["per_page"], 2);
        assert_eq!(meta["total"], 6);
        assert_eq!(meta["last_page"], 3);
        assert_eq!(meta["from"], 1);
        assert_eq!(meta["to"], 2);

        let links = &obj["links"];
        assert_eq!(links["first"], "/items?page=1");
        assert_eq!(links["last"], "/items?page=3");
        assert!(links["prev"].is_null());
        assert_eq!(links["next"], "/items?page=2");
    }

    #[tokio::test]
    async fn test_collection_with_additional() {
        let items = make_items(1);
        let result = with_test_request_uri("/items", move |req| {
            let collection = ResourceCollection::new(items).additional(json!({"version": "v1"}));
            let response = collection.to_response(&req);
            let body: Value = serde_json::from_str(response.body()).unwrap();
            body
        })
        .await;

        let obj = result.as_object().unwrap();
        assert!(obj.contains_key("data"));
        assert_eq!(obj["version"], "v1");
    }

    #[tokio::test]
    async fn test_collection_empty() {
        let items: Vec<TestItem> = vec![];
        let result = with_test_request_uri("/items", move |req| {
            let collection = ResourceCollection::new(items);
            let response = collection.to_response(&req);
            let body: Value = serde_json::from_str(response.body()).unwrap();
            body
        })
        .await;

        assert_eq!(result, json!({"data": []}));
    }
}
