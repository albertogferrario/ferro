use crate::client::anthropic::AnthropicClient;
use crate::client::{CompletionRequest, LlmClient, Message, Role};
use crate::error::Error;
use async_trait::async_trait;
use std::sync::Arc;

use super::{ClassificationProvider, ClassifierConfig};

/// Anthropic API-based classification provider.
///
/// Thin adapter over [`AnthropicClient`]. All HTTP logic lives in the client;
/// this type builds the [`CompletionRequest`] and delegates (D-10).
///
/// # Authentication
///
/// Requires an `ANTHROPIC_API_KEY` environment variable or an explicit API key
/// passed to [`AnthropicProvider::new`].
pub struct AnthropicProvider {
    client: Arc<AnthropicClient>,
}

impl AnthropicProvider {
    /// Create a new provider with an explicit API key.
    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(AnthropicClient::new(api_key, None)),
        }
    }

    /// Create a provider reading the API key from `ANTHROPIC_API_KEY`.
    pub fn from_env() -> Result<Self, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| Error::Config("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(api_key))
    }
}

#[async_trait]
impl ClassificationProvider for AnthropicProvider {
    async fn classify_raw(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> Result<serde_json::Value, Error> {
        let request = CompletionRequest {
            system: Some(system_prompt.to_string()),
            messages: vec![Message {
                role: Role::User,
                content: user_prompt.to_string(),
                tool_call_id: None,
            }],
            max_tokens: config.max_tokens,
            model_override: if config.model.is_empty() {
                None
            } else {
                Some(config.model.clone())
            },
            schema: Some(schema.clone()),
            tools: None,
            tool_choice: None,
        };
        let text = self.client.complete(request).await?;
        serde_json::from_str(&text).map_err(|e| Error::Deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::anthropic::AnthropicClient;
    use crate::client::{CompletionRequest, Message, Role};

    /// Verify the CompletionRequest built for classification carries the schema
    /// and model_override fields correctly. Exercises AnthropicClient::build_body
    /// (the body-shape assertions moved to client/anthropic.rs in Plan 02).
    #[test]
    fn test_classify_request_shape_with_explicit_model() {
        let client = AnthropicClient::new("k".into(), None);
        let schema =
            serde_json::json!({"type": "object", "properties": {"category": {"type": "string"}}});
        let request = CompletionRequest {
            system: Some("You classify intents.".into()),
            messages: vec![Message {
                role: Role::User,
                content: "Hello world".into(),
                tool_call_id: None,
            }],
            max_tokens: 1024,
            model_override: Some("claude-opus-4-6".into()),
            schema: Some(schema.clone()),
            tools: None,
            tool_choice: None,
        };
        let body = client.build_body(&request, false);

        assert_eq!(body["model"], "claude-opus-4-6");
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["format"]["schema"], schema);
        let system = &body["system"][0];
        assert_eq!(system["type"], "text");
        assert_eq!(system["text"], "You classify intents.");
        assert_eq!(system["cache_control"]["type"], "ephemeral");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "Hello world");
    }

    #[test]
    fn test_classify_request_empty_model_uses_client_default() {
        let client = AnthropicClient::new("k".into(), None);
        let request = CompletionRequest {
            system: None,
            messages: vec![Message {
                role: Role::User,
                content: "hi".into(),
                tool_call_id: None,
            }],
            max_tokens: 512,
            model_override: None, // empty config.model → no override
            schema: None,
            tools: None,
            tool_choice: None,
        };
        let body = client.build_body(&request, false);
        // AnthropicClient::default_model() returns "claude-sonnet-4-6"
        assert_eq!(body["model"], "claude-sonnet-4-6");
    }
}
