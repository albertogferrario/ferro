//! Integration tests for the ApiResource derive macro.
//!
//! Tests field selection, rename, skip, and From<Model> generation.

extern crate ferro_rs as ferro;

use ferro_rs::{ApiResource, Request, Resource};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::oneshot;

// ============================================================================
// Test helper: create a real Request via TCP loopback
// ============================================================================

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

// ============================================================================
// Test 1: Simple resource with no attributes
// ============================================================================

#[derive(ApiResource)]
struct SimpleResource {
    id: i32,
    name: String,
}

#[tokio::test]
async fn test_simple_resource_to_resource() {
    let resource = SimpleResource {
        id: 1,
        name: "test".to_string(),
    };

    let result = with_test_request(move |req| resource.to_resource(&req)).await;
    assert_eq!(result, json!({"id": 1, "name": "test"}));
}

// ============================================================================
// Test 2: Resource with rename and skip
// ============================================================================

#[derive(ApiResource)]
#[allow(dead_code)]
struct AnnotatedResource {
    id: i32,
    #[resource(rename = "display_name")]
    name: String,
    #[resource(skip)]
    secret: String,
}

#[tokio::test]
async fn test_annotated_resource_rename_and_skip() {
    let resource = AnnotatedResource {
        id: 42,
        name: "Alice".to_string(),
        secret: "s3cret".to_string(),
    };

    let result = with_test_request(move |req| resource.to_resource(&req)).await;

    // rename: name -> display_name
    assert_eq!(result.get("display_name"), Some(&json!("Alice")));
    // skip: secret absent
    assert!(result.get("secret").is_none());
    // id still present
    assert_eq!(result.get("id"), Some(&json!(42)));
    // original name key absent
    assert!(result.get("name").is_none());
}

// ============================================================================
// Test 3: Resource with model attribute and From<Model> generation
// ============================================================================

#[allow(dead_code)]
struct MockModel {
    id: i32,
    name: String,
    hidden: String,
}

#[derive(ApiResource)]
#[allow(dead_code)]
#[resource(model = "MockModel")]
struct ModelResource {
    id: i32,
    name: String,
    #[resource(skip)]
    hidden: String,
}

#[tokio::test]
async fn test_model_resource_from_conversion() {
    let model = MockModel {
        id: 7,
        name: "Bob".to_string(),
        hidden: "internal".to_string(),
    };

    let resource = ModelResource::from(model);
    assert_eq!(resource.id, 7);
    assert_eq!(resource.name, "Bob");
    assert_eq!(resource.hidden, "internal"); // From copies all fields
}

#[tokio::test]
async fn test_model_resource_to_resource_excludes_hidden() {
    let model = MockModel {
        id: 7,
        name: "Bob".to_string(),
        hidden: "internal".to_string(),
    };

    let resource = ModelResource::from(model);
    let result = with_test_request(move |req| resource.to_resource(&req)).await;

    assert_eq!(result, json!({"id": 7, "name": "Bob"}));
    assert!(result.get("hidden").is_none());
}

// ============================================================================
// Test 4: Resource field order preserved
// ============================================================================

#[derive(ApiResource)]
struct OrderedResource {
    z_last: i32,
    a_first: i32,
    m_middle: i32,
}

#[tokio::test]
async fn test_field_order_preserved() {
    let resource = OrderedResource {
        z_last: 3,
        a_first: 1,
        m_middle: 2,
    };

    let result = with_test_request(move |req| resource.to_resource(&req)).await;
    let keys: Vec<&String> = result.as_object().unwrap().keys().collect();
    assert_eq!(keys, vec!["z_last", "a_first", "m_middle"]);
}
