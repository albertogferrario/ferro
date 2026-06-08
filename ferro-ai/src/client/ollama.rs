// Tests only — OllamaClient not yet defined (RED phase).

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
