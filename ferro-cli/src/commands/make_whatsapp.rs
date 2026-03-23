use console::style;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Template generators
// ---------------------------------------------------------------------------

fn whatsapp_mod_template() -> String {
    r#"pub mod listeners;
pub mod webhook;

use ferro::WhatsApp;

/// Initialize WhatsApp. Call from bootstrap.rs.
pub fn init() {
    let config = ferro::WhatsAppConfig::from_env(Box::new(|phone| {
        // TODO: Replace with your owner phone number check.
        // Phone numbers arrive in E.164 format without '+' (e.g. "393401234567").
        phone == std::env::var("OWNER_PHONE").as_deref().unwrap_or("")
    }))
    .expect("WhatsApp configuration missing. Set WHATSAPP_APP_SECRET, WHATSAPP_ACCESS_TOKEN, WHATSAPP_PHONE_NUMBER_ID, and WHATSAPP_VERIFY_TOKEN.");
    WhatsApp::init(config);
}
"#
    .to_string()
}

fn whatsapp_webhook_template() -> String {
    r#"use ferro::{handler, HttpResponse, Request, Response, WhatsApp};
use ferro::{verify_whatsapp_webhook, ProcessWhatsAppWebhook, queue_dispatch};

/// GET /whatsapp/webhook — Meta challenge verification endpoint.
///
/// Meta sends a GET request to verify the webhook endpoint before enabling it.
/// Must respond with hub.challenge as plain text.
#[handler]
pub async fn whatsapp_webhook_verify(req: Request) -> Response {
    let mode = req.query("hub.mode").unwrap_or_default();
    let token = req.query("hub.verify_token").unwrap_or_default();
    let challenge = req.query("hub.challenge").unwrap_or_default();

    if mode == "subscribe" && token == WhatsApp::config().verify_token {
        Ok(HttpResponse::text(challenge))
    } else {
        Err(HttpResponse::text("Forbidden").status(403))
    }
}

/// POST /whatsapp/webhook — Inbound message and status update handler.
///
/// Verifies HMAC-SHA256 signature, acknowledges immediately, then queues
/// ProcessWhatsAppWebhook job for async processing.
#[handler]
pub async fn whatsapp_webhook(req: Request) -> Response {
    let sig = req
        .header("x-hub-signature-256")
        .ok_or_else(|| HttpResponse::text("Missing x-hub-signature-256").status(400))?;
    let body = req
        .body_string()
        .await
        .map_err(|_| HttpResponse::text("Failed to read body").status(400))?;

    verify_whatsapp_webhook(body.as_bytes(), &sig, &WhatsApp::config().app_secret)
        .map_err(|_| HttpResponse::text("Invalid signature").status(400))?;

    let job = ProcessWhatsAppWebhook {
        payload_json: body,
    };
    queue_dispatch(job)
        .await
        .map_err(|e| HttpResponse::text(format!("Queue error: {e}")).status(500))?;

    Ok(HttpResponse::json(serde_json::json!({"received": true})))
}
"#
    .to_string()
}

fn whatsapp_listeners_template() -> String {
    r#"use ferro::{async_trait, EventError, Listener};
use ferro::{WhatsAppTextReceived, WhatsAppStatusUpdate};

pub struct HandleInboundMessage;

#[async_trait]
impl Listener<WhatsAppTextReceived> for HandleInboundMessage {
    async fn handle(&self, event: &WhatsAppTextReceived) -> Result<(), EventError> {
        // TODO: Handle inbound text message from sender.
        // event.sender_identity — Owner or Customer with phone number
        // event.text — the message text body
        // event.wamid — WhatsApp message ID for dedup/correlation
        println!("WhatsApp message received: {}", event.wamid);
        Ok(())
    }
}

pub struct HandleDeliveryStatus;

#[async_trait]
impl Listener<WhatsAppStatusUpdate> for HandleDeliveryStatus {
    async fn handle(&self, event: &WhatsAppStatusUpdate) -> Result<(), EventError> {
        // TODO: Handle delivery status update (Sent/Delivered/Read/Failed).
        // event.wamid — correlates with SendResult.wamid from WhatsApp::send()
        // event.status — DeliveryStatus enum variant
        println!("WhatsApp status update: {:?} for {}", event.status, event.wamid);
        Ok(())
    }
}
"#
    .to_string()
}

// ---------------------------------------------------------------------------
// File generation helpers
// ---------------------------------------------------------------------------

/// Write a file only if it does not already exist.
/// Returns true if the file was created, false if skipped.
fn write_if_not_exists(path: &Path, content: &str, label: &str) -> bool {
    if path.exists() {
        println!(
            "{} {} already exists, skipping",
            style("Skip:").yellow().bold(),
            label
        );
        return false;
    }
    if let Err(e) = fs::write(path, content) {
        eprintln!(
            "{} Failed to write {}: {}",
            style("Error:").red().bold(),
            label,
            e
        );
        return false;
    }
    println!("{} {}", style("Created:").green().bold(), label);
    true
}

fn ensure_dir(path: &Path) -> bool {
    if path.exists() {
        return true;
    }
    if let Err(e) = fs::create_dir_all(path) {
        eprintln!(
            "{} Failed to create directory {}: {}",
            style("Error:").red().bold(),
            path.display(),
            e
        );
        return false;
    }
    println!(
        "{} Created directory {}",
        style("Created:").green().bold(),
        path.display()
    );
    true
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Execute `ferro make:whatsapp`.
///
/// Generates the WhatsApp integration scaffold into the current Ferro project.
pub fn execute(project_root: &Path) {
    println!("Scaffolding WhatsApp integration...\n");

    let whatsapp_dir = project_root.join("src/whatsapp");

    if !ensure_dir(&whatsapp_dir) {
        std::process::exit(1);
    }

    // mod.rs
    write_if_not_exists(
        &whatsapp_dir.join("mod.rs"),
        &whatsapp_mod_template(),
        "src/whatsapp/mod.rs",
    );

    // webhook.rs
    write_if_not_exists(
        &whatsapp_dir.join("webhook.rs"),
        &whatsapp_webhook_template(),
        "src/whatsapp/webhook.rs",
    );

    // listeners.rs
    write_if_not_exists(
        &whatsapp_dir.join("listeners.rs"),
        &whatsapp_listeners_template(),
        "src/whatsapp/listeners.rs",
    );

    // Print env hints
    println!("\n{}", style("Add to your .env file:").bold());
    println!("  WHATSAPP_APP_SECRET=<from Meta Developer Dashboard -> App Settings -> Basic -> App Secret>");
    println!("  WHATSAPP_ACCESS_TOKEN=<from Meta Developer Dashboard -> WhatsApp -> API Setup -> Permanent Token>");
    println!("  WHATSAPP_PHONE_NUMBER_ID=<from Meta Developer Dashboard -> WhatsApp -> API Setup -> Phone Number ID>");
    println!("  WHATSAPP_VERIFY_TOKEN=<a secret string you choose for webhook verification>");

    // Print next steps
    print_next_steps();
}

fn print_next_steps() {
    println!("\n{}", style("Next steps:").bold());
    println!(
        "\n  {} Call WhatsApp::init() from your bootstrap.rs:",
        style("1.").dim()
    );
    println!("     {}", style("crate::whatsapp::init();").cyan());

    println!(
        "\n  {} Register webhook routes in src/routes.rs:",
        style("2.").dim()
    );
    println!(
        "     {}",
        style("use crate::whatsapp::webhook::{whatsapp_webhook, whatsapp_webhook_verify};").cyan()
    );
    println!(
        "     {}",
        style("get!(\"/whatsapp/webhook\", whatsapp_webhook_verify)").cyan()
    );
    println!(
        "     {}",
        style("post!(\"/whatsapp/webhook\", whatsapp_webhook)").cyan()
    );

    println!(
        "\n  {} Register event listeners in bootstrap.rs:",
        style("3.").dim()
    );
    println!(
        "     {}",
        style("use crate::whatsapp::listeners::{HandleInboundMessage, HandleDeliveryStatus};")
            .cyan()
    );
    println!(
        "     {}",
        style("ferro::register_listener::<WhatsAppTextReceived, HandleInboundMessage>();").cyan()
    );
    println!(
        "     {}",
        style("ferro::register_listener::<WhatsAppStatusUpdate, HandleDeliveryStatus>();").cyan()
    );

    println!(
        "\n  {} Configure the webhook URL in Meta Developer Dashboard:",
        style("4.").dim()
    );
    println!(
        "     {}",
        style("Meta Developer Dashboard -> Your App -> WhatsApp -> Configuration -> Webhook URL")
            .dim()
    );
    println!(
        "     {}",
        style("Set Callback URL to: https://yourdomain.com/whatsapp/webhook").dim()
    );
    println!(
        "     {}",
        style("Set Verify Token to the value of WHATSAPP_VERIFY_TOKEN").dim()
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Generate the scaffold files in a temp directory for testing.
#[cfg(test)]
pub fn generate_in_dir(base_dir: &Path) {
    let whatsapp_dir = base_dir.join("src/whatsapp");
    fs::create_dir_all(&whatsapp_dir).unwrap();

    fs::write(whatsapp_dir.join("mod.rs"), whatsapp_mod_template()).unwrap();
    fs::write(whatsapp_dir.join("webhook.rs"), whatsapp_webhook_template()).unwrap();
    fs::write(
        whatsapp_dir.join("listeners.rs"),
        whatsapp_listeners_template(),
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn read_file(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
    }

    // --- mod.rs template tests ---

    #[test]
    fn test_mod_template_has_correct_imports() {
        let tmpl = whatsapp_mod_template();
        assert!(tmpl.contains("pub mod listeners;"));
        assert!(tmpl.contains("pub mod webhook;"));
        assert!(tmpl.contains("use ferro::WhatsApp;"));
        assert!(tmpl.contains("pub fn init()"));
        assert!(tmpl.contains("ferro::WhatsAppConfig::from_env("));
        assert!(tmpl.contains("WhatsApp::init(config);"));
    }

    #[test]
    fn test_mod_template_has_is_owner_closure() {
        let tmpl = whatsapp_mod_template();
        // is_owner is a closure passed to from_env
        assert!(tmpl.contains("Box::new(|phone|"));
    }

    // --- webhook.rs template tests ---

    #[test]
    fn test_webhook_template_uses_queue_dispatch() {
        let tmpl = whatsapp_webhook_template();
        assert!(tmpl.contains("queue_dispatch(job)"));
        assert!(!tmpl.contains("dispatch_event"));
        assert!(tmpl.contains("verify_whatsapp_webhook("));
        assert!(tmpl.contains("x-hub-signature-256"));
        assert!(tmpl.contains(r#"{"received": true}"#));
    }

    #[test]
    fn test_webhook_template_has_verify_handler() {
        let tmpl = whatsapp_webhook_template();
        assert!(tmpl.contains("whatsapp_webhook_verify"));
        assert!(tmpl.contains("hub.mode"));
        assert!(tmpl.contains("hub.verify_token"));
        assert!(tmpl.contains("hub.challenge"));
        assert!(tmpl.contains("HttpResponse::text(challenge)"));
    }

    #[test]
    fn test_webhook_template_uses_ferro_imports() {
        let tmpl = whatsapp_webhook_template();
        assert!(tmpl.contains("use ferro::{"));
        assert!(tmpl.contains("ProcessWhatsAppWebhook"));
    }

    // --- listeners.rs template tests ---

    #[test]
    fn test_listeners_template_has_both_event_types() {
        let tmpl = whatsapp_listeners_template();
        assert!(tmpl.contains("WhatsAppTextReceived"));
        assert!(tmpl.contains("WhatsAppStatusUpdate"));
        assert!(tmpl.contains("impl Listener<WhatsAppTextReceived> for HandleInboundMessage"));
        assert!(tmpl.contains("impl Listener<WhatsAppStatusUpdate> for HandleDeliveryStatus"));
        assert!(tmpl.contains("async fn handle("));
        assert!(tmpl.contains("use ferro::{async_trait, EventError, Listener};"));
    }

    // --- file generation tests ---

    #[test]
    fn test_generates_three_required_files() {
        let tmp = TempDir::new().unwrap();
        generate_in_dir(tmp.path());

        let whatsapp_dir = tmp.path().join("src/whatsapp");
        assert!(
            whatsapp_dir.exists(),
            "src/whatsapp directory should be created"
        );
        assert!(
            whatsapp_dir.join("mod.rs").exists(),
            "mod.rs should be created"
        );
        assert!(
            whatsapp_dir.join("webhook.rs").exists(),
            "webhook.rs should be created"
        );
        assert!(
            whatsapp_dir.join("listeners.rs").exists(),
            "listeners.rs should be created"
        );
    }

    #[test]
    fn test_generated_files_have_correct_content() {
        let tmp = TempDir::new().unwrap();
        generate_in_dir(tmp.path());

        let whatsapp_dir = tmp.path().join("src/whatsapp");

        let mod_content = read_file(&whatsapp_dir.join("mod.rs"));
        assert!(mod_content.contains("use ferro::WhatsApp;"));

        let webhook_content = read_file(&whatsapp_dir.join("webhook.rs"));
        assert!(webhook_content.contains("queue_dispatch"));
        assert!(webhook_content.contains("verify_whatsapp_webhook"));

        let listeners_content = read_file(&whatsapp_dir.join("listeners.rs"));
        assert!(listeners_content.contains("WhatsAppTextReceived"));
        assert!(listeners_content.contains("WhatsAppStatusUpdate"));
    }

    #[test]
    fn test_does_not_overwrite_existing_files() {
        let tmp = TempDir::new().unwrap();

        // Test write_if_not_exists directly
        let out_path = tmp.path().join("test_file.txt");
        write_if_not_exists(&out_path, "original content", "test_file.txt");
        assert_eq!(fs::read_to_string(&out_path).unwrap(), "original content");

        // Second write should be skipped
        write_if_not_exists(&out_path, "overwritten content", "test_file.txt");
        assert_eq!(
            fs::read_to_string(&out_path).unwrap(),
            "original content",
            "write_if_not_exists must not overwrite existing files"
        );
    }

    #[test]
    fn test_generated_webhook_uses_queue_not_events() {
        let tmp = TempDir::new().unwrap();
        generate_in_dir(tmp.path());

        let webhook_path = tmp.path().join("src/whatsapp/webhook.rs");
        let content = read_file(&webhook_path);

        assert!(
            content.contains("queue_dispatch"),
            "webhook.rs must use queue_dispatch"
        );
        assert!(
            !content.contains("dispatch_event"),
            "webhook.rs must NOT use dispatch_event"
        );
    }
}
