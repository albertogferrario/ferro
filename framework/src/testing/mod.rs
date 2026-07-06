//! Testing utilities for Ferro framework
//!
//! Provides Jest-like testing helpers including:
//! - `expect!` macro for fluent assertions with clear expected/received output
//! - `describe!` and `test!` macros for test organization
//! - `TestDatabase` for isolated database tests
//! - `TestContainer` for dependency injection in tests
//! - `TestClient` and `TestResponse` for HTTP testing
//! - `Factory` and `Fake` for generating test data
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::{describe, test, expect};
//! use crate::testing::{TestDatabase, TestClient, Fake, Factory};
//!
//! describe!("UserService", {
//!     test!("creates a user", async fn(db: TestDatabase) {
//!         let service = UserService::new();
//!         let user = service.create(Fake::email()).await.unwrap();
//!
//!         expect!(user.email).to_contain("@");
//!     });
//!
//!     test!("lists users via API", async fn(db: TestDatabase) {
//!         let client = TestClient::new();
//!
//!         let response = client.get("/api/users").send().await;
//!
//!         response
//!             .assert_ok()
//!             .assert_json_has("users");
//!     });
//! });
//! ```

mod expect;
mod factory;
mod http;

pub use crate::container::testing::{TestContainer, TestContainerGuard};
pub use crate::database::testing::TestDatabase;
pub use expect::{set_current_test_name, Expect};
pub use factory::{DatabaseFactory, Factory, FactoryBuilder, FactoryTraits, Fake, Sequence};
pub use http::{TestClient, TestRequestBuilder, TestResponse};
