# Testing

Ferro provides a comprehensive testing suite with HTTP test client, Jest-like assertions, database factories, and isolated test databases.

## HTTP Testing

### TestClient

The `TestClient` provides a fluent API for making HTTP requests to your application.

```rust
use ferro::TestClient;

#[tokio::test]
async fn test_homepage() {
    let client = TestClient::new(app());

    let response = client.get("/").send().await;

    response.assert_ok();
    response.assert_see("Welcome");
}
```

### Making Requests

```rust
use ferro::TestClient;

let client = TestClient::new(app());

// GET request
let response = client.get("/users").send().await;

// POST request with JSON body
let response = client
    .post("/users")
    .json(&serde_json::json!({
        "name": "John Doe",
        "email": "john@example.com"
    }))
    .send()
    .await;

// PUT request
let response = client
    .put("/users/1")
    .json(&serde_json::json!({ "name": "Jane Doe" }))
    .send()
    .await;

// PATCH request
let response = client
    .patch("/users/1")
    .json(&serde_json::json!({ "active": true }))
    .send()
    .await;

// DELETE request
let response = client.delete("/users/1").send().await;
```

### Request Builder

Customize requests with headers, authentication, and body data.

```rust
// With headers
let response = client
    .get("/api/data")
    .header("X-Custom-Header", "value")
    .header("Accept", "application/json")
    .send()
    .await;

// With bearer token authentication
let response = client
    .get("/api/protected")
    .bearer_token("your-jwt-token")
    .send()
    .await;

// With query parameters
let response = client
    .get("/search")
    .query(&[("q", "rust"), ("page", "1")])
    .send()
    .await;

// With form data
let response = client
    .post("/login")
    .form(&[("email", "user@example.com"), ("password", "secret")])
    .send()
    .await;

// With JSON body
let response = client
    .post("/api/posts")
    .json(&serde_json::json!({
        "title": "My Post",
        "content": "Hello, World!"
    }))
    .send()
    .await;
```

### Acting As User

Test authenticated routes by acting as a specific user.

```rust
use ferro::models::user;

let user = user::Entity::find_by_pk(1).await?.expect("user with id=1 exists");

let response = client
    .get("/dashboard")
    .acting_as(&user)
    .send()
    .await;

response.assert_ok();
```

## Response Assertions

### Status Assertions

```rust
// Specific status code
response.assert_status(200);
response.assert_status(201);
response.assert_status(422);

// Common status helpers
response.assert_ok();           // 200
response.assert_created();      // 201
response.assert_no_content();   // 204
response.assert_redirect();     // 3xx
response.assert_not_found();    // 404
response.assert_unauthorized(); // 401
response.assert_forbidden();    // 403
response.assert_unprocessable(); // 422
response.assert_server_error(); // 5xx
```

### JSON Assertions

```rust
// Assert JSON path exists
response.assert_json_has("data.user.name");
response.assert_json_has("data.items[0].id");

// Assert JSON path has specific value
response.assert_json_is("data.user.name", "John Doe");
response.assert_json_is("data.count", 42);
response.assert_json_is("data.active", true);

// Assert entire JSON structure
response.assert_json_equals(&serde_json::json!({
    "status": "success",
    "data": {
        "id": 1,
        "name": "John"
    }
}));

// Assert array count at path
response.assert_json_count("data.items", 5);

// Assert JSON matches predicate
response.assert_json_matches("data.items", |items| {
    items.as_array().map(|arr| arr.len() > 0).unwrap_or(false)
});

// Assert JSON path is missing
response.assert_json_missing("data.password");
```

### Content Assertions

```rust
// Assert body contains text
response.assert_see("Welcome");
response.assert_see("Hello, World!");

// Assert body does NOT contain text
response.assert_dont_see("Error");
response.assert_dont_see("Unauthorized");
```

### Validation Error Assertions

```rust
// Assert validation errors for specific fields
response.assert_validation_errors(&["email", "password"]);

// Typical validation error test
#[tokio::test]
async fn test_registration_validation() {
    let client = TestClient::new(app());

    let response = client
        .post("/register")
        .json(&serde_json::json!({
            "email": "invalid-email",
            "password": "123"  // too short
        }))
        .send()
        .await;

    response.assert_unprocessable();
    response.assert_validation_errors(&["email", "password"]);
}
```

### Header Assertions

```rust
// Assert header exists and has value
response.assert_header("Content-Type", "application/json");
response.assert_header("X-Request-Id", "abc123");
```

### Accessing Response Data

```rust
// Get status code
let status = response.status();

// Get response body as string
let body = response.text();

// Get response body as JSON
let json: serde_json::Value = response.json();

// Get specific header
let content_type = response.header("Content-Type");
```

## Expect Assertions

Ferro provides Jest-like `expect` assertions for expressive tests.

```rust
use ferro::testing::Expect; // not re-exported at crate root

#[tokio::test]
async fn test_user_creation() {
    let user = create_user().await;

    Expect::that(&user.name).to_equal("John Doe");
    Expect::that(&user.email).to_contain("@");
    Expect::that(&user.age).to_be_greater_than(&18);
}
```

### Equality

```rust
Expect::that(&value).to_equal("expected");
Expect::that(&value).to_not_equal("unexpected");
```

### Boolean

```rust
Expect::that(&result).to_be_true();
Expect::that(&result).to_be_false();
```

### Option

```rust
let some_value: Option<i32> = Some(42);
let none_value: Option<i32> = None;

Expect::that(&some_value).to_be_some();
Expect::that(&none_value).to_be_none();
Expect::that(&some_value).to_contain_value(&42);
```

### Result

```rust
let ok_result: Result<i32, &str> = Ok(42);
let err_result: Result<i32, &str> = Err("error");

Expect::that(&ok_result).to_be_ok();
Expect::that(&err_result).to_be_err();
Expect::that(&ok_result).to_contain_ok(&42);
Expect::that(&err_result).to_contain_err(&"error");
```

### Strings

```rust
Expect::that(&text).to_contain("hello");
Expect::that(&text).to_start_with("Hello");
Expect::that(&text).to_end_with("!");
Expect::that(&text).to_have_length(11);
Expect::that(&text).to_be_empty();
Expect::that(&text).to_not_be_empty();
```

### Vectors

```rust
let items = vec![1, 2, 3, 4, 5];

Expect::that(&items).to_have_length(5);
Expect::that(&items).to_contain(&3);
Expect::that(&items).to_be_empty();
Expect::that(&items).to_not_be_empty();
```

### Numeric Comparisons

```rust
Expect::that(&value).to_be_greater_than(&10);
Expect::that(&value).to_be_less_than(&100);
Expect::that(&value).to_be_greater_than_or_equal(&10);
Expect::that(&value).to_be_less_than_or_equal(&100);
```

## Database Factories

Factories generate fake data for testing, inspired by Laravel's model factories.

### Defining a Factory

```rust
use ferro::{Factory, FactoryBuilder, Fake};
use sea_orm::Set;
use crate::models::user;

pub struct UserFactory;

impl Factory for UserFactory {
    type Model = user::ActiveModel;

    fn definition() -> Self::Model {
        user::ActiveModel {
            name: Set(Fake::name()),
            email: Set(Fake::email()),
            password: Set(Fake::sentence(3)),
            active: Set(true),
            ..Default::default()
        }
    }
}
```

### Using Factories

```rust
use crate::factories::UserFactory;

// Make a single model (not persisted)
let user = UserFactory::factory().make();

// Make multiple models
let users = UserFactory::factory().count(5).make_many();

// Override attributes
let admin = UserFactory::factory()
    .set("role", "admin")
    .set("active", true)
    .make();
```

### Factory States

Define reusable states for common variations.

```rust
use ferro::Factory;
use ferro::testing::FactoryTraits; // not re-exported at crate root

impl Factory for UserFactory {
    type Model = user::ActiveModel;

    fn definition() -> Self::Model {
        user::ActiveModel {
            name: Set(Fake::name()),
            email: Set(Fake::email()),
            active: Set(true),
            role: Set("user".to_string()),
            ..Default::default()
        }
    }

    fn traits() -> FactoryTraits<Self::Model> {
        FactoryTraits::new()
            .register("admin", |model| {
                let mut model = model;
                model.role = Set("admin".to_string());
                model
            })
            .register("inactive", |model| {
                let mut model = model;
                model.active = Set(false);
                model
            })
            .register("unverified", |model| {
                let mut model = model;
                model.email_verified_at = Set(None);
                model
            })
    }
}

// Using traits
let admin = UserFactory::factory().trait_("admin").make();
let inactive_admin = UserFactory::factory()
    .trait_("admin")
    .trait_("inactive")
    .make();
```

### Database Factory

For factories that persist to the database.

```rust
use ferro::{Factory, Fake};
use ferro::testing::DatabaseFactory; // not re-exported at crate root
use ferro::DB;

pub struct UserFactory;

impl Factory for UserFactory {
    type Model = user::ActiveModel;

    fn definition() -> Self::Model {
        user::ActiveModel {
            name: Set(Fake::name()),
            email: Set(Fake::email()),
            ..Default::default()
        }
    }
}

impl DatabaseFactory for UserFactory {
    type Entity = user::Entity;
}

// Create and persist to database
let user = UserFactory::factory().create().await?;

// Create multiple
let users = UserFactory::factory().count(10).create_many().await?;
```

### Factory Callbacks

Execute code after making or creating models.

```rust
let user = UserFactory::factory()
    .after_make(|user| {
        println!("Made user: {:?}", user);
    })
    .after_create(|user| {
        // Send welcome email, create related records, etc.
        println!("Created user in database: {:?}", user);
    })
    .create()
    .await?;
```

### Sequences

Generate unique sequential values.

```rust
use ferro::Sequence;

let seq = Sequence::new();

// Get next value
let id1 = seq.next(); // 1
let id2 = seq.next(); // 2
let id3 = seq.next(); // 3

// Use in factories
fn definition() -> Self::Model {
    static SEQ: Sequence = Sequence::new();

    user::ActiveModel {
        email: Set(format!("user{}@example.com", SEQ.next())),
        ..Default::default()
    }
}
```

## Fake Data Generation

The `Fake` helper generates realistic test data.

### Personal Information

```rust
use ferro::Fake;

let name = Fake::name();           // "John Smith"
let first = Fake::first_name();    // "John"
let last = Fake::last_name();      // "Smith"
let email = Fake::email();         // "john.smith@example.com"
let phone = Fake::phone();         // "+1-555-123-4567"
```

### Text Content

```rust
let word = Fake::word();               // "lorem"
let sentence = Fake::sentence(5);      // 5-word sentence
let paragraph = Fake::paragraph(3);    // 3-sentence paragraph
let text = Fake::text(100);            // ~100 characters
```

### Numbers

```rust
let num = Fake::number(1, 100);        // Random 1-100
let float = Fake::float(0.0, 1.0);     // Random 0.0-1.0
let bool = Fake::boolean();            // true or false
```

### Identifiers

```rust
let uuid = Fake::uuid();               // "550e8400-e29b-41d4-a716-446655440000"
let slug = Fake::slug(3);              // "lorem-ipsum-dolor"
```

### Addresses

```rust
let address = Fake::address();         // "123 Main St"
let city = Fake::city();               // "New York"
let country = Fake::country();         // "United States"
let zip = Fake::zip_code();            // "10001"
```

### Internet

```rust
let url = Fake::url();                 // "https://example.com/page"
let domain = Fake::domain();           // "example.com"
let ip = Fake::ip_v4();                // "192.168.1.1"
let user_agent = Fake::user_agent();   // "Mozilla/5.0..."
```

### Dates and Times

```rust
use chrono::{NaiveDate, NaiveDateTime};

let date = Fake::date();               // Random date
let datetime = Fake::datetime();       // Random datetime
let past = Fake::past_date(30);        // Within last 30 days
let future = Fake::future_date(30);    // Within next 30 days
```

### Collections

```rust
// Pick one from list
let status = Fake::one_of(&["pending", "active", "completed"]);

// Pick multiple from list
let tags = Fake::many_of(&["rust", "web", "api", "testing"], 2);
```

### Custom Generators

```rust
// With closure
let custom = Fake::custom(|| {
    format!("USER-{}", Fake::number(1000, 9999))
});
```

## Test Database

Ferro provides isolated database testing with automatic migrations.

### Using test_database! Macro

```rust
use ferro::test_database;
use ferro::models::user;

#[tokio::test]
async fn test_user_creation() {
    // Creates fresh in-memory SQLite with migrations
    let db = test_database!();

    // Create a user
    let new_user = user::ActiveModel {
        name: Set("Test User".to_string()),
        email: Set("test@example.com".to_string()),
        ..Default::default()
    };

    let user = user::Entity::insert_one(new_user).await.unwrap();
    assert!(user.id > 0);

    // Query using test database connection
    let found = user::Entity::find_by_id(user.id)
        .one(db.conn())
        .await
        .unwrap();

    assert!(found.is_some());
}
```

### Custom Migrator

```rust
use ferro::TestDatabase;

#[tokio::test]
async fn test_with_custom_migrator() {
    let db = TestDatabase::fresh::<my_crate::CustomMigrator>()
        .await
        .unwrap();

    // Test code here
}
```

### Database Isolation

Each `TestDatabase`:
- Creates a fresh in-memory SQLite database
- Runs all migrations automatically
- Is completely isolated from other tests
- Is cleaned up when dropped

This ensures tests don't interfere with each other.

## Test Container

Mock dependencies using the test container.

### Faking Services

```rust
use ferro::{TestContainer, TestContainerGuard};
use std::sync::Arc;

#[tokio::test]
async fn test_with_fake_service() {
    // Create isolated container for this test
    let _guard = TestContainer::fake();

    // Register a fake singleton
    TestContainer::singleton(FakePaymentGateway::new());

    // Register a factory
    TestContainer::factory(|| {
        Box::new(MockEmailService::default())
    });

    // Your test code - Container::get() returns fakes
    let gateway = Container::get::<FakePaymentGateway>();
    // ...
}
```

### Binding Interfaces

```rust
use std::sync::Arc;

#[tokio::test]
async fn test_with_mock_repository() {
    let _guard = TestContainer::fake();

    // Bind a mock implementation of a trait
    let mock_repo: Arc<dyn UserRepository> = Arc::new(MockUserRepository::new());
    TestContainer::bind(mock_repo);

    // Or with a factory
    TestContainer::bind_factory::<dyn UserRepository, _>(|| {
        Arc::new(MockUserRepository::with_users(vec![
            User { id: 1, name: "Test".into() }
        ]))
    });

    // Test code uses the mock
}
```

## Complete Test Example

```rust
use ferro::{TestClient, TestDatabase, Fake};
use ferro::testing::Expect; // not re-exported at crate root
use ferro::test_database;
use crate::factories::UserFactory;

#[tokio::test]
async fn test_user_registration_flow() {
    // Set up isolated test database
    let _db = test_database!();

    // Create test client
    let client = TestClient::new(app());

    // Test validation errors
    let response = client
        .post("/api/register")
        .json(&serde_json::json!({
            "email": "invalid"
        }))
        .send()
        .await;

    response.assert_unprocessable();
    response.assert_validation_errors(&["email", "password", "name"]);

    // Test successful registration
    let email = Fake::email();
    let response = client
        .post("/api/register")
        .json(&serde_json::json!({
            "name": Fake::name(),
            "email": &email,
            "password": "password123",
            "password_confirmation": "password123"
        }))
        .send()
        .await;

    response.assert_created();
    response.assert_json_has("data.user.id");
    response.assert_json_is("data.user.email", &email);

    // Verify user was created in database
    let user = user::Entity::query()
        .filter(user::Column::Email.eq(&email))
        .first()
        .await
        .unwrap();

    Expect::that(&user).to_be_some();
}

#[tokio::test]
async fn test_authenticated_endpoint() {
    let _db = test_database!();

    // Create a user with factory
    let user = UserFactory::factory()
        .trait_("admin")
        .create()
        .await
        .unwrap();

    let client = TestClient::new(app());

    // Test unauthorized access
    let response = client.get("/api/admin/users").send().await;
    response.assert_unauthorized();

    // Test authorized access
    let response = client
        .get("/api/admin/users")
        .acting_as(&user)
        .send()
        .await;

    response.assert_ok();
    response.assert_json_has("data.users");
}
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_user_registration

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test

# Run tests sequentially
cargo test -- --test-threads=1
```

## Best Practices

1. **Use test_database! for isolation** - Each test gets a fresh database
2. **Use factories for test data** - Consistent, readable test setup
3. **Test both success and failure cases** - Validate error handling
4. **Use meaningful test names** - `test_user_cannot_access_admin_panel`
5. **Keep tests focused** - One assertion concept per test
6. **Use Expect for readable assertions** - Fluent API improves clarity
7. **Mock external services** - Use TestContainer to isolate from APIs
8. **Test validation thoroughly** - Cover edge cases in input validation
