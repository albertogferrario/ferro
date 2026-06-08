pub mod anthropic;
pub mod provider;

use crate::error::Error;
use provider::ClassificationProvider;
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Configuration for the AI classifier.
///
/// All fields have sensible defaults via [`Default`].
#[derive(Debug, Clone)]
pub struct ClassifierConfig {
    /// Anthropic model ID (or equivalent for other providers).
    pub model: String,
    /// Maximum tokens in the response.
    pub max_tokens: u32,
    /// Number of retry attempts on transient errors (total attempts = max_retries + 1).
    pub max_retries: u32,
    /// Delay between retry attempts.
    pub retry_delay: Duration,
    /// Minimum confidence score required to return a successful result.
    ///
    /// If the response includes a `"confidence"` field below this threshold,
    /// an [`Error::LowConfidence`] is returned. Set to `0.0` to disable.
    pub confidence_threshold: f64,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            model: String::new(), // resolved from client.default_model() at call time (D-03)
            max_tokens: 1024,
            max_retries: 1,
            retry_delay: Duration::from_secs(1),
            confidence_threshold: 0.7,
        }
    }
}

/// The result of a successful classification.
#[derive(Debug)]
pub struct ClassificationResult<T> {
    /// The deserialized output value.
    pub value: T,
    /// Confidence score if the provider included one in the response.
    ///
    /// The schema must include a `"confidence"` field of type `f64` for this
    /// to be populated; the Anthropic API does not return metadata outside the
    /// schema.
    pub confidence: Option<f64>,
    /// Raw JSON returned by the provider, useful for prompt improvement feedback.
    pub raw_json: serde_json::Value,
}

/// Generic AI classification facade.
///
/// `T` is the output type. It must implement [`serde::de::DeserializeOwned`] so
/// the raw JSON from the provider can be deserialized into it.
///
/// # Example
///
/// ```rust,ignore
/// use ferro_ai::{Classifier, ClassifierConfig, AnthropicProvider};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Intent { category: String, confidence: f64 }
///
/// async fn classify_message(text: &str) -> ferro_ai::Error {
///     let provider = AnthropicProvider::from_env().unwrap();
///     let classifier = Classifier::<Intent>::new(
///         std::sync::Arc::new(provider),
///         ClassifierConfig::default(),
///     );
///     let schema = serde_json::json!({ /* ... */ });
///     let result = classifier.classify("You classify intents.", text, &schema).await?;
///     println!("category: {}", result.value.category);
///     Ok(())
/// }
/// ```
pub struct Classifier<T> {
    provider: Arc<dyn ClassificationProvider>,
    config: ClassifierConfig,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> Classifier<T> {
    /// Create a new classifier with the given provider and configuration.
    pub fn new(provider: Arc<dyn ClassificationProvider>, config: ClassifierConfig) -> Self {
        Self {
            provider,
            config,
            _phantom: PhantomData,
        }
    }

    /// Classify using the given prompts and JSON schema.
    ///
    /// Retries on transient errors up to `config.max_retries` additional times.
    /// Fails immediately on permanent errors (auth, bad request, schema mismatch).
    pub async fn classify(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema: &serde_json::Value,
    ) -> Result<ClassificationResult<T>, Error> {
        let max_attempts = self.config.max_retries + 1;
        let mut last_error: Option<Error> = None;

        for attempt in 1..=max_attempts {
            info!(
                model = %self.config.model,
                attempt,
                max_attempts,
                "Classifying"
            );

            match self
                .provider
                .classify_raw(system_prompt, user_prompt, schema, &self.config)
                .await
            {
                Ok(raw_json) => {
                    let confidence = raw_json.get("confidence").and_then(|v| v.as_f64());

                    if let Some(conf) = confidence {
                        if conf < self.config.confidence_threshold {
                            return Err(Error::LowConfidence {
                                best_guess: raw_json,
                                confidence: conf,
                            });
                        }
                    }

                    let value = serde_json::from_value::<T>(raw_json.clone())
                        .map_err(|e| Error::Deserialization(e.to_string()))?;

                    return Ok(ClassificationResult {
                        value,
                        confidence,
                        raw_json,
                    });
                }
                Err(e) if !e.is_retryable() => {
                    // Do not retry permanent errors
                    return Err(e);
                }
                Err(e) => {
                    warn!(attempt, error = %e, "Classification attempt failed, may retry");
                    last_error = Some(e);
                    if attempt < max_attempts {
                        sleep(self.config.retry_delay).await;
                    }
                }
            }
        }

        // All attempts exhausted
        match last_error {
            Some(Error::Timeout) => Err(Error::Timeout),
            Some(e) => Err(e),
            None => Err(Error::Timeout),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde::Deserialize;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_classifier_config_defaults() {
        let config = ClassifierConfig::default();
        // model is empty string after D-03: resolved from client.default_model() at call time
        assert!(config.model.is_empty());
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.max_retries, 1);
        assert_eq!(config.retry_delay, Duration::from_secs(1));
        assert_eq!(config.confidence_threshold, 0.7);
    }

    #[derive(Debug, Deserialize)]
    struct SampleOutput {
        category: String,
    }

    struct ConstProvider {
        response: serde_json::Value,
    }

    #[async_trait]
    impl ClassificationProvider for ConstProvider {
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

    #[tokio::test]
    async fn test_classification_result_deserialization() {
        let provider = ConstProvider {
            response: serde_json::json!({"category": "greeting"}),
        };
        let classifier = Classifier::<SampleOutput>::new(
            Arc::new(provider),
            ClassifierConfig {
                confidence_threshold: 0.0,
                ..Default::default()
            },
        );
        let schema = serde_json::json!({});
        let result = classifier
            .classify("system", "user", &schema)
            .await
            .unwrap();
        assert_eq!(result.value.category, "greeting");
        assert!(result.confidence.is_none());
    }

    #[tokio::test]
    async fn test_classification_extracts_confidence() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct WithConfidence {
            category: String,
            confidence: f64,
        }

        let provider = ConstProvider {
            response: serde_json::json!({"category": "greeting", "confidence": 0.9}),
        };
        let classifier = Classifier::<WithConfidence>::new(
            Arc::new(provider),
            ClassifierConfig {
                confidence_threshold: 0.5,
                ..Default::default()
            },
        );
        let result = classifier
            .classify("system", "user", &serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(result.confidence, Some(0.9));
    }

    struct CountingProvider {
        call_count: Arc<AtomicU32>,
        fail_times: u32,
    }

    #[async_trait]
    impl ClassificationProvider for CountingProvider {
        async fn classify_raw(
            &self,
            _system_prompt: &str,
            _user_prompt: &str,
            _schema: &serde_json::Value,
            _config: &ClassifierConfig,
        ) -> Result<serde_json::Value, Error> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
            if count <= self.fail_times {
                Err(Error::Provider {
                    status: Some(500),
                    message: "internal server error".into(),
                })
            } else {
                Ok(serde_json::json!({"category": "ok"}))
            }
        }
    }

    #[tokio::test]
    async fn test_retry_on_transient_error() {
        let call_count = Arc::new(AtomicU32::new(0));
        let provider = CountingProvider {
            call_count: Arc::clone(&call_count),
            fail_times: 1, // fail once, succeed on second attempt
        };
        let config = ClassifierConfig {
            max_retries: 1,
            retry_delay: Duration::from_millis(1), // fast for tests
            confidence_threshold: 0.0,
            ..Default::default()
        };
        let classifier = Classifier::<SampleOutput>::new(Arc::new(provider), config);
        let result = classifier
            .classify("s", "u", &serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(result.value.category, "ok");
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_no_retry_on_permanent_error() {
        let call_count = Arc::new(AtomicU32::new(0));
        let provider = CountingProvider {
            call_count: Arc::clone(&call_count),
            fail_times: 10, // always fail with 401
        };

        struct PermanentProvider {
            call_count: Arc<AtomicU32>,
        }

        #[async_trait]
        impl ClassificationProvider for PermanentProvider {
            async fn classify_raw(
                &self,
                _system_prompt: &str,
                _user_prompt: &str,
                _schema: &serde_json::Value,
                _config: &ClassifierConfig,
            ) -> Result<serde_json::Value, Error> {
                self.call_count.fetch_add(1, Ordering::SeqCst);
                Err(Error::Provider {
                    status: Some(401),
                    message: "unauthorized".into(),
                })
            }
        }

        drop(provider); // avoid unused warning
        let perm_count = Arc::new(AtomicU32::new(0));
        let perm_provider = PermanentProvider {
            call_count: Arc::clone(&perm_count),
        };
        let config = ClassifierConfig {
            max_retries: 3,
            retry_delay: Duration::from_millis(1),
            confidence_threshold: 0.0,
            ..Default::default()
        };
        let classifier = Classifier::<SampleOutput>::new(Arc::new(perm_provider), config);
        let result = classifier.classify("s", "u", &serde_json::json!({})).await;
        assert!(result.is_err());
        // Must not retry on permanent error — only 1 call
        assert_eq!(perm_count.load(Ordering::SeqCst), 1);
    }
}
