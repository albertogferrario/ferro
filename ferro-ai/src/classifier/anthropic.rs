use crate::error::Error;
use async_trait::async_trait;

use super::{ClassificationProvider, ClassifierConfig};

/// Anthropic API-based classification provider.
///
/// Uses the Anthropic Messages API with `output_config.format.type = "json_schema"`
/// for guaranteed schema-compliant JSON output.
///
/// # Authentication
///
/// Requires an `ANTHROPIC_API_KEY` environment variable or an explicit API key
/// passed to [`AnthropicProvider::new`].
pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicProvider {
    /// Create a new provider with an explicit API key.
    ///
    /// The internal `reqwest::Client` uses a 60-second timeout.
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");
        Self { client, api_key }
    }

    /// Create a provider reading the API key from `ANTHROPIC_API_KEY`.
    pub fn from_env() -> Result<Self, Error> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| Error::Config("ANTHROPIC_API_KEY not set".to_string()))?;
        Ok(Self::new(api_key))
    }

    /// Build the request body for the Anthropic Messages API.
    ///
    /// Uses `output_config.format.type = "json_schema"` for structured output.
    /// The system prompt is cached with `cache_control.type = "ephemeral"` to
    /// reduce token costs on repeated calls with the same system prompt.
    pub(crate) fn build_request_body(
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> serde_json::Value {
        serde_json::json!({
            "model": config.model,
            "max_tokens": config.max_tokens,
            "system": [{
                "type": "text",
                "text": system_prompt,
                "cache_control": {"type": "ephemeral"}
            }],
            "messages": [{"role": "user", "content": user_prompt}],
            "output_config": {
                "format": {
                    "type": "json_schema",
                    "schema": schema
                }
            }
        })
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
        let body = Self::build_request_body(system_prompt, user_prompt, schema, config);

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::Timeout
                } else {
                    Error::Provider { status: None, message: e.to_string() }
                }
            })?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Provider { status: Some(status), message: text });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        // Extract content[0].text from Anthropic response envelope
        let text = json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|item| item["text"].as_str())
            .ok_or_else(|| {
                Error::Deserialization(format!("unexpected response structure: {json}"))
            })?;

        serde_json::from_str(text).map_err(|e| Error::Deserialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_request_body_contains_output_config() {
        let config = ClassifierConfig {
            model: "claude-sonnet-4-6".to_string(), // explicit model (default is now empty per D-03)
            ..Default::default()
        };
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "category": {"type": "string"}
            }
        });

        let body = AnthropicProvider::build_request_body(
            "You classify intents.",
            "Hello world",
            &schema,
            &config,
        );

        // Verify model and max_tokens from config
        assert_eq!(body["model"], "claude-sonnet-4-6");
        assert_eq!(body["max_tokens"], 1024);

        // Verify output_config.format.type = "json_schema"
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["format"]["schema"], schema);

        // Verify system prompt with cache_control
        let system = &body["system"][0];
        assert_eq!(system["type"], "text");
        assert_eq!(system["text"], "You classify intents.");
        assert_eq!(system["cache_control"]["type"], "ephemeral");

        // Verify user message
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "Hello world");
    }

    #[test]
    fn test_build_request_body_uses_config_model() {
        let config = ClassifierConfig {
            model: "claude-opus-4-6".to_string(),
            max_tokens: 2048,
            ..Default::default()
        };
        let body = AnthropicProvider::build_request_body(
            "system",
            "user",
            &serde_json::json!({}),
            &config,
        );
        assert_eq!(body["model"], "claude-opus-4-6");
        assert_eq!(body["max_tokens"], 2048);
    }
}
