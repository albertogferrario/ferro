# AI & Confirmation

Ferro provides two AI primitives in the `ferro-ai` crate, accessible via the `ai` feature flag:

- **Classification** — structured LLM output with provider abstraction and retry logic
- **Confirmation** — gate destructive actions behind explicit user confirmation with TTL expiry

## Setup

Add the `ai` feature to your `ferro-rs` dependency:

```toml
[dependencies]
ferro-rs = { version = "0.1", features = ["ai"] }
```

Set `ANTHROPIC_API_KEY` in your `.env` or environment before using the `AnthropicProvider`.

---

## AI Classification

Classification turns unstructured user input into typed, schema-validated Rust structs by calling an LLM with a JSON Schema constraint.

### Basic Usage

Define an output struct that implements `serde::Deserialize`. Include a `confidence: f64` field to enable threshold enforcement:

```rust
use ferro::{Classifier, ClassifierConfig, AnthropicProvider};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct CommandIntent {
    action: String,
    target: Option<String>,
    confidence: f64,
}

async fn classify_command(text: &str) -> ferro::AiError {
    let provider = AnthropicProvider::from_env()?;
    let classifier = Classifier::<CommandIntent>::new(
        Arc::new(provider),
        ClassifierConfig::default(),
    );

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "action": { "type": "string", "enum": ["delete", "update", "list"] },
            "target": { "type": "string" },
            "confidence": { "type": "number" }
        },
        "required": ["action", "confidence"]
    });

    let result = classifier
        .classify("You classify user commands.", text, &schema)
        .await?;

    println!("action: {}", result.value.action);
    println!("confidence: {:?}", result.confidence);
    Ok(())
}
```

### Schema Generation

For complex output types, use `schemars` to derive the schema automatically. Add it to your `Cargo.toml`:

```toml
schemars = "1"
```

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct IntentClassification {
    intent: String,
    confidence: f64,
    parameters: std::collections::HashMap<String, String>,
}

let schema = schemars::schema_for!(IntentClassification);
let schema_value = serde_json::to_value(&schema).unwrap();
```

### Configuration

`ClassifierConfig` controls model selection, token limits, retry behavior, and confidence thresholds:

```rust
use ferro::ClassifierConfig;
use std::time::Duration;

let config = ClassifierConfig {
    model: "claude-opus-4-6".to_string(),
    max_tokens: 2048,
    max_retries: 2,
    retry_delay: Duration::from_secs(2),
    confidence_threshold: 0.8,
};
```

| Field | Default | Description |
|-------|---------|-------------|
| `model` | `claude-sonnet-4-6` | Model ID passed to the provider |
| `max_tokens` | `1024` | Maximum response tokens |
| `max_retries` | `1` | Additional retry attempts on transient errors |
| `retry_delay` | `1s` | Delay between retries |
| `confidence_threshold` | `0.7` | Minimum confidence required (set to `0.0` to disable) |

### Custom Providers

Implement the `ClassificationProvider` trait to use any LLM backend:

```rust
use ferro::{ClassificationProvider, ClassifierConfig};
use async_trait::async_trait;

struct MyProvider;

#[async_trait]
impl ClassificationProvider for MyProvider {
    async fn classify_raw(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> Result<serde_json::Value, ferro::AiError> {
        // Call your LLM API here, return JSON matching schema
        Ok(serde_json::json!({"action": "list"}))
    }
}
```

### Error Handling

```rust
use ferro::AiError;

match classifier.classify(system, user, &schema).await {
    Ok(result) => { /* use result.value */ }
    Err(AiError::LowConfidence { best_guess, confidence }) => {
        // Response was below confidence_threshold
        eprintln!("Low confidence: {confidence}");
    }
    Err(AiError::Provider(msg)) => {
        // HTTP error from the provider (permanent: 4xx, transient: 5xx)
        eprintln!("Provider error: {msg}");
    }
    Err(AiError::Timeout) => {
        // Request timed out after all retry attempts
    }
    Err(e) => eprintln!("Other error: {e}"),
}
```

**Retry behavior:**
- Transient errors (429, 500, 503, 529) are retried up to `max_retries` additional times
- Permanent errors (400, 401, 403, 404, 422) are not retried
- `LowConfidence` is returned immediately without retrying

### WhatsApp Command Example

```rust
#[derive(Deserialize, JsonSchema)]
struct WhatsAppIntent {
    intent: String, // "add_expense", "list_expenses", "help"
    amount: Option<f64>,
    category: Option<String>,
    confidence: f64,
}

let classifier = Classifier::<WhatsAppIntent>::new(
    Arc::new(AnthropicProvider::from_env()?),
    ClassifierConfig {
        confidence_threshold: 0.75,
        ..Default::default()
    },
);

let schema = serde_json::to_value(schemars::schema_for!(WhatsAppIntent)).unwrap();
let result = classifier
    .classify(
        "You classify expense management commands from WhatsApp messages. \
         Extract intent, amount (if present), and category.",
        "spent 15 on coffee",
        &schema,
    )
    .await?;

// result.value.intent == "add_expense"
// result.value.amount == Some(15.0)
// result.value.category == Some("coffee")
```

---

## Confirmation

The confirmation primitive gates destructive actions behind explicit user acknowledgement. It uses an in-memory store with per-action TTL expiry and integrates with the Ferro event system.

### Basic Usage

```rust
use ferro::{InMemoryConfirmationStore, ConfirmationStore};
use std::time::Duration;

// Create store (typically as a shared Arc in your app state)
let store = std::sync::Arc::new(InMemoryConfirmationStore::new());

// Request confirmation — stores the action payload and starts TTL timer
let key = format!("delete:expense:{}:{}", tenant_id, expense_id);
let payload = serde_json::json!({ "expense_id": expense_id, "amount": 42.50 });
store.request_confirmation(&key, payload, Duration::from_secs(300)).await?;

// User confirms (e.g., replies "yes" or taps a button)
let confirmed_payload = store.confirm(&key).await?;

// Or user cancels
store.reject(&key).await?;
```

### Key Design

Use composite string keys to scope confirmations by context:

```
"scope:user:action:id"
```

Examples:
- `"expense:user-42:delete:expense-7"`
- `"subscription:user-42:cancel"`
- `"batch:user-42:import:file-123"`

This prevents cross-user confirmation attacks and keeps keys self-documenting.

### TTL and Auto-Expiry

When the TTL expires before confirmation or rejection, the store automatically:

1. Removes the pending action
2. Dispatches a `ConfirmationExpired` event via `ferro_events::dispatch`

Listen for this event to notify users:

```rust
use ferro::{Listener, ConfirmationExpired, EventError};
use async_trait::async_trait;

pub struct NotifyExpired;

#[async_trait]
impl Listener<ConfirmationExpired> for NotifyExpired {
    async fn handle(&self, event: &ConfirmationExpired) -> Result<(), EventError> {
        // event.key — the confirmation key that expired
        // event.payload — the original payload
        println!("Confirmation expired for: {}", event.key);
        // Send notification to user
        Ok(())
    }
}
```

Register the listener in your `bootstrap.rs`:

```rust
ferro::dispatch_event(ConfirmationExpired { key, payload });
```

### Delete Expense Flow

```rust
#[handler]
pub async fn request_delete(req: Request, store: Arc<InMemoryConfirmationStore>) -> Response {
    let expense_id: i64 = req.param("id")?;
    let tenant_id = current_tenant().map(|t| t.id).unwrap_or(0);

    let key = format!("expense:{}:delete:{}", tenant_id, expense_id);
    let payload = serde_json::json!({ "expense_id": expense_id });

    store
        .request_confirmation(&key, payload, Duration::from_secs(300))
        .await?;

    Ok(json!({ "status": "pending", "message": "Reply 'confirm' to delete this expense." }))
}

#[handler]
pub async fn confirm_delete(req: Request, store: Arc<InMemoryConfirmationStore>) -> Response {
    let expense_id: i64 = req.param("id")?;
    let tenant_id = current_tenant().map(|t| t.id).unwrap_or(0);

    let key = format!("expense:{}:delete:{}", tenant_id, expense_id);
    let payload = store.confirm(&key).await?;

    // payload contains the original data — perform the deletion
    let expense_id: i64 = payload["expense_id"].as_i64().unwrap_or(0);
    Expense::delete_by_id(expense_id).exec(&DB.get().unwrap()).await?;

    Ok(json!({ "status": "deleted" }))
}
```

### Listing Pending Actions

```rust
let pending = store.list_pending().await;
// Returns Vec<PendingActionInfo> with key, payload, and expires_at
for action in pending {
    println!("{}: expires at {:?}", action.key, action.expires_at);
}
```

---

## MCP Tools

Two MCP tools are available for debugging AI features during development.

### `test_classifier`

Test a classification prompt against the Anthropic API without writing handler code:

- **When to use:** Iterating on prompts, verifying schema compliance, debugging output shape
- **Requires:** `ANTHROPIC_API_KEY` in environment or `.env`
- **Note:** Makes a real API call. Costs tokens.
- **Returns:** `{ success, result, model, error }`

### `list_pending_confirmations`

Scan source code for `request_confirmation` call sites:

- **When to use:** Auditing which handlers use confirmation, understanding confirmation flows
- **Note:** Confirmation state is in-memory — this tool scans source files, not runtime state
- **Returns:** File paths and line numbers where `request_confirmation` is called
