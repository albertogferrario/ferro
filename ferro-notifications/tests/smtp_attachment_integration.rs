//! Mailpit-backed SMTP attachment integration test (per ROADMAP Phase 149 criterion #5).
//!
//! This test sends a real multipart/mixed email through ferro-notifications' SMTP path
//! to a local Mailpit instance, then fetches the message via Mailpit's HTTP API and
//! verifies the attachment bytes round-trip intact.
//!
//! ## Running
//!
//! 1. Start Mailpit: `docker run -d -p 1025:1025 -p 8025:8025 axllent/mailpit`
//! 2. Run: `MAILPIT_SMTP_HOST=localhost MAILPIT_API_HOST=localhost cargo test \
//!         -p ferro-notifications --features integration-tests \
//!         --test smtp_attachment_integration -- --nocapture`
//!
//! When `MAILPIT_SMTP_HOST` is unset, the test skips silently (exits 0). This keeps the
//! default `cargo test` green in CI without requiring Mailpit to be running.

#![cfg(feature = "integration-tests")]

use ferro_notifications::{
    Channel, MailConfig, MailMessage, Notifiable, Notification, NotificationConfig,
    NotificationDispatcher,
};
use std::env;
use std::time::{Duration, Instant};

const FIXTURE_FILENAME: &str = "phase149-fixture.bin";
const FIXTURE_CONTENT_TYPE: &str = "application/octet-stream";
const FIXTURE_SUBJECT: &str = "Phase 149 attachment integration test";

/// Deterministic 1KB pattern: 0x00..0xff repeating, rolled to 1024.
fn fixture_bytes() -> Vec<u8> {
    (0..1024).map(|i| (i % 256) as u8).collect()
}

struct TestRecipient {
    email: String,
}

impl Notifiable for TestRecipient {
    fn route_notification_for(&self, channel: Channel) -> Option<String> {
        match channel {
            Channel::Mail => Some(self.email.clone()),
            _ => None,
        }
    }
}

struct AttachmentNotification;

impl Notification for AttachmentNotification {
    fn via(&self) -> Vec<Channel> {
        vec![Channel::Mail]
    }

    fn to_mail(&self) -> Option<MailMessage> {
        let bytes = fixture_bytes();
        Some(
            MailMessage::new()
                .subject(FIXTURE_SUBJECT)
                .body("This email carries a 1KB binary attachment. See attachments[0].")
                .attachment(FIXTURE_FILENAME, FIXTURE_CONTENT_TYPE, bytes)
                .expect("under-limit fixture must succeed"),
        )
    }
}

#[tokio::test]
async fn smtp_attachment_round_trip_via_mailpit() {
    let smtp_host = match env::var("MAILPIT_SMTP_HOST") {
        Ok(h) if !h.is_empty() => h,
        _ => {
            eprintln!("SKIP: MAILPIT_SMTP_HOST not set (start Mailpit and re-run)");
            return;
        }
    };
    let api_host = env::var("MAILPIT_API_HOST").unwrap_or_else(|_| smtp_host.clone());

    // Configure ferro-notifications to point at Mailpit (no-auth, no-TLS).
    NotificationDispatcher::configure(
        NotificationConfig::new().mail(
            MailConfig::new(&smtp_host, 1025, "phase149@example.com")
                .from_name("Phase 149 test")
                .no_tls(),
        ),
    );

    let recipient = TestRecipient {
        email: "test-recipient@example.com".into(),
    };

    // Send.
    recipient
        .notify(AttachmentNotification)
        .await
        .expect("send must succeed");

    // Poll Mailpit's HTTP API for the message.
    let api_base = format!("http://{api_host}:8025");
    let client = reqwest::Client::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut found_message_id: Option<String> = None;
    while Instant::now() < deadline {
        let resp: serde_json::Value = client
            .get(format!("{api_base}/api/v1/messages"))
            .send()
            .await
            .expect("api list")
            .json()
            .await
            .expect("api list json");

        if let Some(messages) = resp.get("messages").and_then(|m| m.as_array()) {
            for m in messages {
                let subject = m.get("Subject").and_then(|s| s.as_str()).unwrap_or("");
                if subject == FIXTURE_SUBJECT {
                    if let Some(id) = m.get("ID").and_then(|i| i.as_str()) {
                        found_message_id = Some(id.to_string());
                        break;
                    }
                }
            }
        }
        if found_message_id.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let message_id = found_message_id.expect("message must arrive in Mailpit within 5s");

    // Fetch the attachment list and verify metadata.
    let detail: serde_json::Value = client
        .get(format!("{api_base}/api/v1/message/{message_id}"))
        .send()
        .await
        .expect("api detail")
        .json()
        .await
        .expect("api detail json");

    let attachments = detail
        .get("Attachments")
        .and_then(|a| a.as_array())
        .expect("Attachments array present");
    assert_eq!(attachments.len(), 1, "exactly one attachment expected");
    let att = &attachments[0];
    assert_eq!(
        att.get("FileName").and_then(|s| s.as_str()),
        Some(FIXTURE_FILENAME)
    );
    assert_eq!(
        att.get("ContentType").and_then(|s| s.as_str()),
        Some(FIXTURE_CONTENT_TYPE)
    );

    // Fetch raw attachment bytes and assert byte-equality with the source fixture.
    let part_id = att
        .get("PartID")
        .and_then(|s| s.as_str())
        .expect("PartID present");
    let raw = client
        .get(format!(
            "{api_base}/api/v1/message/{message_id}/part/{part_id}"
        ))
        .send()
        .await
        .expect("api part")
        .bytes()
        .await
        .expect("api part bytes");

    let expected = fixture_bytes();
    assert_eq!(
        raw.as_ref(),
        expected.as_slice(),
        "attachment bytes must round-trip intact"
    );

    eprintln!("OK: 1KB binary attachment round-tripped through SMTP via Mailpit");
}
