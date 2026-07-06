use crate::error::Error;
use async_trait::async_trait;

use super::ClassifierConfig;

/// Trait for AI classification backends.
///
/// Implement this trait to plug in any AI provider (Anthropic, OpenAI, local models).
/// The single method receives raw prompts and a JSON schema, and returns a JSON value
/// matching that schema.
///
/// # Object safety
///
/// This trait is object-safe and can be used as `Arc<dyn ClassificationProvider>`.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_ai::{ClassificationProvider, ClassifierConfig};
/// use async_trait::async_trait;
///
/// struct EchoProvider;
///
/// #[async_trait]
/// impl ClassificationProvider for EchoProvider {
///     async fn classify_raw(
///         &self,
///         _system_prompt: &str,
///         user_prompt: &str,
///         _schema: &serde_json::Value,
///         _config: &ClassifierConfig,
///     ) -> Result<serde_json::Value, ferro_ai::Error> {
///         Ok(serde_json::json!({"echo": user_prompt}))
///     }
/// }
/// ```
#[async_trait]
pub trait ClassificationProvider: Send + Sync {
    /// Call the AI provider with raw prompts and return structured JSON.
    ///
    /// The returned `serde_json::Value` must conform to the provided `schema`.
    /// The schema is passed as a JSON Schema value; callers generate it via
    /// `schemars::schema_for!(T)` or build it manually.
    async fn classify_raw(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
        config: &ClassifierConfig,
    ) -> Result<serde_json::Value, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct EchoProvider {
        response: serde_json::Value,
    }

    #[async_trait]
    impl ClassificationProvider for EchoProvider {
        async fn classify_raw(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
            _schema: &serde_json::Value,
            _config: &ClassifierConfig,
        ) -> Result<serde_json::Value, Error> {
            Ok(self.response.clone())
        }
    }

    #[test]
    fn test_classification_provider_is_object_safe() {
        let provider = EchoProvider {
            response: serde_json::json!({"result": "ok"}),
        };
        // This must compile — verifies object safety
        let _: Arc<dyn ClassificationProvider> = Arc::new(provider);
    }
}
