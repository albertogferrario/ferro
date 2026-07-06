//! Environment-driven LLM client factory.
//!
//! [`AiConfig::from_env`] reads four environment variables and constructs the
//! appropriate [`crate::client::LlmClient`] implementation. Unknown providers
//! and missing required keys are rejected at construction time — not at the
//! first LLM call (D-06).
//!
//! ## Environment variables
//!
//! | Variable | Required | Default | Description |
//! |---|---|---|---|
//! | `FERRO_AI_PROVIDER` | No | `anthropic` | Provider name: `anthropic`, `openai`, `groq`, `ollama` |
//! | `FERRO_AI_MODEL` | No | provider default | Model identifier override |
//! | `FERRO_AI_API_KEY` | Yes (except ollama) | — | API key for the selected provider |
//! | `FERRO_AI_BASE_URL` | No | provider default | Base URL override (useful for proxies) |
//! | `FERRO_AI_EMBED_MODEL` | No | provider default | Embedding model override, read per-provider by `embed()` (separate from the chat `FERRO_AI_MODEL`) |
//!
//! For `anthropic`, `ANTHROPIC_API_KEY` is accepted as a fallback when
//! `FERRO_AI_API_KEY` is not set (backward compatibility only; `FERRO_AI_API_KEY`
//! takes precedence).

use crate::client::{AnthropicClient, LlmClient, OllamaClient, OpenAiClient};
use crate::error::Error;

/// Zero-sized marker for the environment-driven client factory.
///
/// All functionality is in the associated function [`AiConfig::from_env`].
pub struct AiConfig;

impl AiConfig {
    /// Construct the configured LLM client from environment variables.
    ///
    /// Reads `FERRO_AI_PROVIDER` (default `"anthropic"`), `FERRO_AI_MODEL`,
    /// `FERRO_AI_API_KEY`, and `FERRO_AI_BASE_URL`. Returns
    /// `Err(Error::Config)` if the provider is unknown or a required key is
    /// missing — fail-fast at startup, not on the first call (D-06).
    ///
    /// # Provider strings
    ///
    /// - `"anthropic"` — [`AnthropicClient`]; requires `FERRO_AI_API_KEY` (or `ANTHROPIC_API_KEY` fallback)
    /// - `"openai"` — [`OpenAiClient`]; requires `FERRO_AI_API_KEY`
    /// - `"groq"` — [`OpenAiClient`] with Groq base URL; requires `FERRO_AI_API_KEY`
    /// - `"ollama"` — [`OllamaClient`]; no key needed
    pub fn from_env() -> Result<Box<dyn LlmClient>, Error> {
        let provider =
            std::env::var("FERRO_AI_PROVIDER").unwrap_or_else(|_| "anthropic".to_string());
        let model = std::env::var("FERRO_AI_MODEL").ok();
        let api_key = std::env::var("FERRO_AI_API_KEY").ok();
        let base_url = std::env::var("FERRO_AI_BASE_URL").ok();

        match provider.to_lowercase().as_str() {
            "anthropic" => {
                let key = api_key
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| Error::Config("FERRO_AI_API_KEY not set".into()))?;
                Ok(Box::new(AnthropicClient::new(key, model)))
            }
            "openai" => {
                let key = api_key
                    .ok_or_else(|| Error::Config("FERRO_AI_API_KEY not set for openai".into()))?;
                Ok(Box::new(OpenAiClient::new(key, model, base_url)))
            }
            "groq" => {
                let key = api_key
                    .ok_or_else(|| Error::Config("FERRO_AI_API_KEY not set for groq".into()))?;
                let url = base_url.unwrap_or_else(|| "https://api.groq.com/openai".into());
                Ok(Box::new(OpenAiClient::new(key, model, Some(url))))
            }
            "ollama" => Ok(Box::new(OllamaClient::new(model, base_url))),
            unknown => Err(Error::Config(format!(
                "unknown FERRO_AI_PROVIDER: '{unknown}'"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_env_fails_on_unknown_provider() {
        let _guard = crate::ENV_LOCK.lock().unwrap();
        std::env::set_var("FERRO_AI_PROVIDER", "bogus");
        std::env::remove_var("FERRO_AI_MODEL");
        std::env::remove_var("FERRO_AI_API_KEY");
        std::env::remove_var("FERRO_AI_BASE_URL");
        let result = AiConfig::from_env();
        std::env::remove_var("FERRO_AI_PROVIDER");
        assert!(
            matches!(result, Err(Error::Config(_))),
            "expected Config error for unknown provider"
        );
    }

    #[test]
    fn from_env_ollama_default_model() {
        let _guard = crate::ENV_LOCK.lock().unwrap();
        std::env::set_var("FERRO_AI_PROVIDER", "ollama");
        std::env::remove_var("FERRO_AI_MODEL");
        std::env::remove_var("FERRO_AI_API_KEY");
        std::env::remove_var("FERRO_AI_BASE_URL");
        let client = AiConfig::from_env().expect("ollama needs no key");
        std::env::remove_var("FERRO_AI_PROVIDER");
        assert_eq!(client.default_model(), "llama3.1");
    }

    #[test]
    fn from_env_anthropic_missing_key_errors() {
        let _guard = crate::ENV_LOCK.lock().unwrap();
        std::env::set_var("FERRO_AI_PROVIDER", "anthropic");
        // Remove both key sources so there is nothing to fall back to.
        std::env::remove_var("FERRO_AI_API_KEY");
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("FERRO_AI_MODEL");
        std::env::remove_var("FERRO_AI_BASE_URL");
        let result = AiConfig::from_env();
        std::env::remove_var("FERRO_AI_PROVIDER");
        assert!(
            matches!(result, Err(Error::Config(_))),
            "expected Config error for missing anthropic key"
        );
    }

    #[test]
    fn from_env_anthropic_with_explicit_key() {
        let _guard = crate::ENV_LOCK.lock().unwrap();
        std::env::set_var("FERRO_AI_PROVIDER", "anthropic");
        std::env::set_var("FERRO_AI_API_KEY", "test-key");
        std::env::remove_var("FERRO_AI_MODEL");
        std::env::remove_var("FERRO_AI_BASE_URL");
        std::env::remove_var("ANTHROPIC_API_KEY");
        let client = AiConfig::from_env().expect("should succeed with explicit key");
        std::env::remove_var("FERRO_AI_PROVIDER");
        std::env::remove_var("FERRO_AI_API_KEY");
        assert_eq!(client.default_model(), "claude-sonnet-4-6");
    }

    #[test]
    fn from_env_groq_base_url_default() {
        let _guard = crate::ENV_LOCK.lock().unwrap();
        std::env::set_var("FERRO_AI_PROVIDER", "groq");
        std::env::set_var("FERRO_AI_API_KEY", "groq-key");
        std::env::remove_var("FERRO_AI_MODEL");
        std::env::remove_var("FERRO_AI_BASE_URL");
        let client = AiConfig::from_env().expect("groq should succeed with key");
        std::env::remove_var("FERRO_AI_PROVIDER");
        std::env::remove_var("FERRO_AI_API_KEY");
        // groq uses OpenAiClient; its default_model() is "gpt-4o"
        assert_eq!(client.default_model(), "gpt-4o");
    }
}
