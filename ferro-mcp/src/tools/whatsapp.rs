//! MCP introspection tools for WhatsApp Business integration.
//!
//! Provides two tools:
//! - `whatsapp_config_status` — reports env var presence and scaffold existence
//! - `whatsapp_webhook_events` — lists listener implementations discovered from source

use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// whatsapp_config_status
// ---------------------------------------------------------------------------

/// Status of WhatsApp configuration in the current project.
#[derive(Debug, Serialize)]
pub struct WhatsAppConfigStatus {
    /// True when all required keys are present.
    pub configured: bool,
    /// Names of env vars that are set (values masked).
    pub keys_present: Vec<String>,
    /// Names of env vars that are missing.
    pub keys_missing: Vec<String>,
    /// True when src/whatsapp/ directory exists.
    pub scaffold_exists: bool,
    /// List of scaffold files found in src/whatsapp/.
    pub scaffold_files: Vec<String>,
}

/// Report WhatsApp configuration status for the project.
///
/// Reads env vars from the environment (and .env if present) and checks
/// whether the scaffold directory exists.
pub fn whatsapp_config_status(project_root: &Path) -> WhatsAppConfigStatus {
    // Load .env if present so values are available without a running server
    let env_path = project_root.join(".env");
    if env_path.exists() {
        let _ = dotenvy::from_path(&env_path);
    }

    let required_keys = [
        "WHATSAPP_APP_SECRET",
        "WHATSAPP_ACCESS_TOKEN",
        "WHATSAPP_PHONE_NUMBER_ID",
        "WHATSAPP_VERIFY_TOKEN",
    ];

    let mut keys_present = Vec::new();
    let mut keys_missing = Vec::new();

    for key in &required_keys {
        if std::env::var(key).is_ok() {
            keys_present.push(key.to_string());
        } else {
            keys_missing.push(key.to_string());
        }
    }

    let scaffold_dir = project_root.join("src/whatsapp");
    let scaffold_exists = scaffold_dir.is_dir();

    let scaffold_files = if scaffold_exists {
        match fs::read_dir(&scaffold_dir) {
            Ok(entries) => {
                let mut files: Vec<String> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                    .map(|e| format!("src/whatsapp/{}", e.file_name().to_string_lossy()))
                    .collect();
                files.sort();
                files
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    let configured = keys_missing.is_empty();

    WhatsAppConfigStatus {
        configured,
        keys_present,
        keys_missing,
        scaffold_exists,
        scaffold_files,
    }
}

// ---------------------------------------------------------------------------
// whatsapp_webhook_events
// ---------------------------------------------------------------------------

/// A discovered WhatsApp event listener in the listeners file.
#[derive(Debug, Serialize)]
pub struct WhatsAppWebhookEvent {
    /// The Ferro event type (e.g., "WhatsAppTextReceived").
    pub event_type: String,
    /// The listener struct name (e.g., "HandleInboundMessage").
    pub listener: String,
    /// Relative file path where the listener is defined.
    pub file: String,
}

/// Scan src/whatsapp/listeners.rs for Listener impl blocks.
pub fn whatsapp_webhook_events(project_root: &Path) -> Vec<WhatsAppWebhookEvent> {
    let listeners_path = project_root.join("src/whatsapp/listeners.rs");

    if !listeners_path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(&listeners_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Match: impl Listener<EventType> for StructName
    let re = Regex::new(r"impl\s+Listener<(\w+)>\s+for\s+(\w+)").unwrap();

    re.captures_iter(&content)
        .map(|cap| WhatsAppWebhookEvent {
            event_type: cap[1].to_string(),
            listener: cap[2].to_string(),
            file: "src/whatsapp/listeners.rs".to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- whatsapp_config_status tests ---

    #[test]
    fn test_config_status_scaffold_exists() {
        let tmp = TempDir::new().unwrap();
        let whatsapp_dir = tmp.path().join("src/whatsapp");
        fs::create_dir_all(&whatsapp_dir).unwrap();
        fs::write(whatsapp_dir.join("mod.rs"), "// mod").unwrap();
        fs::write(whatsapp_dir.join("webhook.rs"), "// webhook").unwrap();
        fs::write(whatsapp_dir.join("listeners.rs"), "// listeners").unwrap();

        let status = whatsapp_config_status(tmp.path());

        assert!(status.scaffold_exists);
        assert_eq!(status.scaffold_files.len(), 3);
        assert!(status.scaffold_files.iter().any(|f| f.ends_with("mod.rs")));
        assert!(status
            .scaffold_files
            .iter()
            .any(|f| f.ends_with("webhook.rs")));
        assert!(status
            .scaffold_files
            .iter()
            .any(|f| f.ends_with("listeners.rs")));
    }

    #[test]
    fn test_config_status_scaffold_missing() {
        let tmp = TempDir::new().unwrap();
        let status = whatsapp_config_status(tmp.path());

        assert!(!status.scaffold_exists);
        assert!(status.scaffold_files.is_empty());
    }

    #[test]
    fn test_config_status_serializes() {
        let status = WhatsAppConfigStatus {
            configured: false,
            keys_present: vec!["WHATSAPP_APP_SECRET".to_string()],
            keys_missing: vec!["WHATSAPP_ACCESS_TOKEN".to_string()],
            scaffold_exists: false,
            scaffold_files: Vec::new(),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("WHATSAPP_APP_SECRET"));
        assert!(json.contains("WHATSAPP_ACCESS_TOKEN"));
        assert!(json.contains("\"configured\":false"));
    }

    #[test]
    fn test_config_status_all_required_keys_tracked() {
        let tmp = TempDir::new().unwrap();
        let status = whatsapp_config_status(tmp.path());

        // All required keys appear in either present or missing lists
        let all_keys: Vec<&str> = status
            .keys_present
            .iter()
            .chain(status.keys_missing.iter())
            .map(|s| s.as_str())
            .collect();

        assert!(all_keys.contains(&"WHATSAPP_APP_SECRET"));
        assert!(all_keys.contains(&"WHATSAPP_ACCESS_TOKEN"));
        assert!(all_keys.contains(&"WHATSAPP_PHONE_NUMBER_ID"));
        assert!(all_keys.contains(&"WHATSAPP_VERIFY_TOKEN"));
    }

    // --- whatsapp_webhook_events tests ---

    #[test]
    fn test_webhook_events_not_found_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let result = whatsapp_webhook_events(tmp.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_webhook_events_parses_listeners() {
        let tmp = TempDir::new().unwrap();
        let whatsapp_dir = tmp.path().join("src/whatsapp");
        fs::create_dir_all(&whatsapp_dir).unwrap();

        let content = r#"
use ferro::{async_trait, EventError, Listener};
use ferro::{WhatsAppTextReceived, WhatsAppStatusUpdate};

pub struct HandleInboundMessage;

#[async_trait]
impl Listener<WhatsAppTextReceived> for HandleInboundMessage {
    async fn handle(&self, event: &WhatsAppTextReceived) -> Result<(), EventError> {
        Ok(())
    }
}

pub struct HandleDeliveryStatus;

#[async_trait]
impl Listener<WhatsAppStatusUpdate> for HandleDeliveryStatus {
    async fn handle(&self, event: &WhatsAppStatusUpdate) -> Result<(), EventError> {
        Ok(())
    }
}
"#;
        fs::write(whatsapp_dir.join("listeners.rs"), content).unwrap();

        let result = whatsapp_webhook_events(tmp.path());
        assert_eq!(result.len(), 2);

        let event_types: Vec<&str> = result.iter().map(|e| e.event_type.as_str()).collect();
        assert!(event_types.contains(&"WhatsAppTextReceived"));
        assert!(event_types.contains(&"WhatsAppStatusUpdate"));

        let listeners: Vec<&str> = result.iter().map(|e| e.listener.as_str()).collect();
        assert!(listeners.contains(&"HandleInboundMessage"));
        assert!(listeners.contains(&"HandleDeliveryStatus"));

        for event in &result {
            assert_eq!(event.file, "src/whatsapp/listeners.rs");
        }
    }

    #[test]
    fn test_webhook_events_serializes() {
        let event = WhatsAppWebhookEvent {
            event_type: "WhatsAppTextReceived".to_string(),
            listener: "HandleInboundMessage".to_string(),
            file: "src/whatsapp/listeners.rs".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("WhatsAppTextReceived"));
        assert!(json.contains("HandleInboundMessage"));
    }
}
