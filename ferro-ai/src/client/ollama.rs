//! Ollama local LLM client.
//!
//! Implements [`LlmClient`] against a local (or operator-configured) Ollama instance.
//! No API key is required. The default base URL is `http://localhost:11434`.
//!
//! Streaming uses NDJSON line-delimited JSON over `bytes_stream()` — NOT SSE.
//! Never call `.eventsource()` on Ollama requests (Pitfall 2).

use crate::client::{CompletionRequest, LlmClient, Role, TokenStream};
use crate::error::Error;
use async_stream::try_stream;
use async_trait::async_trait;
use futures::StreamExt;

/// Ollama chat + embeddings client.
///
/// Uses `{base_url}/api/chat` for completions and `{base_url}/api/embed` for
/// embeddings. No authentication required.
///
/// The internal `reqwest::Client` uses a 60-second timeout (T-165-04).
pub struct OllamaClient {
    client: reqwest::Client,
    model: Option<String>,
    /// Base URL of the Ollama server. Defaults to `http://localhost:11434`.
    pub(crate) base_url: String,
}

impl OllamaClient {
    /// Create a new client.
    ///
    /// - `model`: optional model override; `None` resolves to `default_model()` (`"llama3.1"`).
    /// - `base_url`: optional base URL override; `None` defaults to `"http://localhost:11434"`.
    pub fn new(model: Option<String>, base_url: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");
        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
        Self {
            client,
            model,
            base_url,
        }
    }

    /// Build the request body for `/api/chat`.
    ///
    /// Ollama ignores `max_tokens` and structured-output schema fields — only
    /// `model`, `messages`, and `stream` are sent. No auth header needed.
    pub(crate) fn build_body(
        &self,
        request: &CompletionRequest,
        stream: bool,
    ) -> serde_json::Value {
        let model = request
            .model_override
            .as_deref()
            .unwrap_or_else(|| self.default_model());

        let mut messages: Vec<serde_json::Value> = Vec::new();

        if let Some(system) = &request.system {
            messages.push(serde_json::json!({
                "role": "system",
                "content": system,
            }));
        }

        for m in &request.messages {
            messages.push(serde_json::json!({
                "role": match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                },
                "content": m.content,
            }));
        }

        serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": stream,
        })
    }
}

/// Parse a single NDJSON line from an Ollama `/api/chat` stream.
///
/// Returns `(token, done)`:
/// - `token` is `Some(text)` when the line carries a non-empty content chunk.
/// - `done` is `true` when `"done": true` signals the end of the stream.
pub(crate) fn parse_ollama_line(line: &str) -> Result<(Option<String>, bool), Error> {
    let v: serde_json::Value =
        serde_json::from_str(line).map_err(|e| Error::Deserialization(e.to_string()))?;

    let done = v["done"].as_bool().unwrap_or(false);
    let token = v["message"]["content"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Ok((token, done))
}

/// Parse an Ollama `/api/embed` response, extracting `embeddings[0]` as `Vec<f32>`.
pub(crate) fn parse_ollama_embedding(json: &serde_json::Value) -> Result<Vec<f32>, Error> {
    json["embeddings"][0]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        })
        .ok_or_else(|| Error::Deserialization("no embeddings in response".into()))
}

#[async_trait]
impl LlmClient for OllamaClient {
    fn default_model(&self) -> &str {
        self.model.as_deref().unwrap_or("llama3.1")
    }

    async fn complete(&self, request: CompletionRequest) -> Result<String, Error> {
        let body = self.build_body(&request, false);

        let resp = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::Timeout
                } else {
                    Error::Provider {
                        status: None,
                        message: e.to_string(),
                    }
                }
            })?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider {
                status: Some(status),
                message: text,
            });
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        json["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| Error::Deserialization("no content in response".into()))
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<TokenStream, Error> {
        let body = self.build_body(&request, true);

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::Timeout
                } else {
                    Error::Provider {
                        status: None,
                        message: e.to_string(),
                    }
                }
            })?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Provider {
                status: Some(status),
                message: text,
            });
        }

        // NDJSON streaming: each newline is a complete JSON object, NOT SSE.
        // Never use .eventsource() here — Ollama returns application/x-ndjson (Pitfall 2).
        let stream = Box::pin(try_stream! {
            let mut bytes = response.bytes_stream();
            let mut buf = String::new();
            while let Some(chunk) = bytes.next().await {
                let chunk = chunk.map_err(|e| Error::Provider {
                    status: None,
                    message: e.to_string(),
                })?;
                buf.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(newline_pos) = buf.find('\n') {
                    let line = buf[..newline_pos].trim().to_string();
                    buf = buf[newline_pos + 1..].to_string();
                    if line.is_empty() {
                        continue;
                    }
                    let (token, done) = parse_ollama_line(&line)?;
                    if let Some(text) = token {
                        yield text;
                    }
                    if done {
                        return;
                    }
                }
            }
        });

        Ok(stream)
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, Error> {
        let model = self.default_model().to_string();
        let body = serde_json::json!({
            "model": model,
            "input": text,
        });

        let resp = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    Error::Timeout
                } else {
                    Error::Provider {
                        status: None,
                        message: e.to_string(),
                    }
                }
            })?;

        let status = resp.status().as_u16();
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Provider {
                status: Some(status),
                message: text,
            });
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| Error::Deserialization(e.to_string()))?;

        parse_ollama_embedding(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_ollama_embedding, parse_ollama_line, OllamaClient};
    use crate::client::LlmClient;
    use crate::error::Error;

    #[test]
    fn test_ollama_default_model() {
        let client = OllamaClient::new(None, None);
        assert_eq!(client.default_model(), "llama3.1");
    }

    #[test]
    fn test_ollama_model_override() {
        let client = OllamaClient::new(Some("mistral".into()), None);
        assert_eq!(client.default_model(), "mistral");
    }

    #[test]
    fn test_ollama_default_base_url() {
        let client = OllamaClient::new(None, None);
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_parse_ollama_line_token() {
        let line = r#"{"message":{"content":"The"},"done":false}"#;
        let (token, done) = parse_ollama_line(line).unwrap();
        assert_eq!(token, Some("The".to_string()));
        assert!(!done);
    }

    #[test]
    fn test_parse_ollama_line_done() {
        let line = r#"{"message":{"content":""},"done":true}"#;
        let (token, done) = parse_ollama_line(line).unwrap();
        assert_eq!(token, None);
        assert!(done);
    }

    #[test]
    fn test_parse_ollama_embedding() {
        let json = serde_json::json!({
            "embeddings": [[0.1_f64, -0.2_f64]],
            "total_duration": 12345
        });
        let result = parse_ollama_embedding(&json).unwrap();
        assert_eq!(result.len(), 2);
        assert!((result[0] - 0.1f32).abs() < 1e-6);
        assert!((result[1] - (-0.2f32)).abs() < 1e-6);
    }

    #[test]
    fn test_parse_ollama_embedding_missing() {
        let json = serde_json::json!({"embeddings": []});
        assert!(matches!(
            parse_ollama_embedding(&json),
            Err(Error::Deserialization(_))
        ));
    }

    #[test]
    fn test_ollama_is_object_safe() {
        let _: Box<dyn LlmClient> = Box::new(OllamaClient::new(None, None));
    }
}
