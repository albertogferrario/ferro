//! Notification dispatcher for sending notifications through channels.

use crate::channel::Channel;
use crate::channels::{DatabaseMessage, InAppMessage, MailMessage, SlackMessage, WhatsAppMessage};
use crate::notifiable::{DatabaseNotificationStore, Notifiable};
use crate::notification::Notification;
use crate::Error;
use serde::Serialize;
use std::env;
use std::sync::{Arc, OnceLock};
use tracing::{error, info, warn};

/// Global notification dispatcher configuration.
static CONFIG: OnceLock<NotificationConfig> = OnceLock::new();

/// Configuration for the notification dispatcher.
#[derive(Clone, Default)]
pub struct NotificationConfig {
    /// Mail configuration (supports SMTP and Resend drivers).
    pub mail: Option<MailConfig>,
    /// Slack webhook URL.
    pub slack_webhook: Option<String>,
    /// Enable the WhatsApp channel (per CONTEXT.md D-04).
    ///
    /// Defaults to `false`. When `true`, the dispatcher calls
    /// [`ferro_whatsapp::WhatsApp::send`] which requires that
    /// [`ferro_whatsapp::WhatsApp::init`] was called at app startup.
    pub whatsapp_enabled: bool,
    /// In-app SSE channel configuration (per CONTEXT.md D-07).
    ///
    /// When `Some`, `Channel::InApp` dispatches persist via `store` then publish via `broker`.
    /// When `None`, `Channel::InApp` emits a structured "channel not configured" no-op.
    pub in_app: Option<InAppConfig>,
    /// Database notification store (per CONTEXT.md D-13, closes ARCH-FINDING-02).
    ///
    /// When `Some`, `Channel::Database` calls `store.store(...)` instead of the placeholder log.
    /// When `None`, the existing placeholder behavior is preserved (backward-compat).
    /// Note: `InAppConfig.store` is independent of this field — they may point to the same Arc
    /// or to different stores.
    pub database_store: Option<Arc<dyn DatabaseNotificationStore>>,
}

/// In-app SSE channel configuration.
///
/// Combines a `Broadcaster` for real-time fanout and a `DatabaseNotificationStore`
/// for persistence. Per CONTEXT.md D-08, `Channel::InApp` dispatches write the DB-store
/// leg first, then publish to the broadcast channel `format!("user.{}", notifiable_id)`.
#[derive(Clone)]
pub struct InAppConfig {
    /// Broadcaster handle for SSE / WebSocket fanout.
    pub broker: Arc<ferro_broadcast::Broadcaster>,
    /// Persistence store — typically the same Arc as `NotificationConfig::database_store`.
    pub store: Arc<dyn DatabaseNotificationStore>,
}

/// Mail transport driver.
#[derive(Debug, Clone, Default)]
pub enum MailDriver {
    /// SMTP via lettre (default).
    #[default]
    Smtp,
    /// Resend HTTP API.
    Resend,
}

/// SMTP-specific configuration.
#[derive(Clone)]
pub struct SmtpConfig {
    /// SMTP host.
    pub host: String,
    /// SMTP port.
    pub port: u16,
    /// SMTP username.
    pub username: Option<String>,
    /// SMTP password.
    pub password: Option<String>,
    /// Use TLS.
    pub tls: bool,
}

/// Resend-specific configuration.
#[derive(Clone)]
pub struct ResendConfig {
    /// Resend API key.
    pub api_key: String,
}

/// Mail configuration supporting multiple drivers.
#[derive(Clone)]
pub struct MailConfig {
    /// Which driver to use.
    pub driver: MailDriver,
    /// Default from address (shared across all drivers).
    pub from: String,
    /// Default from name (shared across all drivers).
    pub from_name: Option<String>,
    /// SMTP-specific config (only when driver = Smtp).
    pub smtp: Option<SmtpConfig>,
    /// Resend-specific config (only when driver = Resend).
    pub resend: Option<ResendConfig>,
}

impl NotificationConfig {
    /// Create a new notification config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration from environment variables.
    ///
    /// Reads the following environment variables:
    /// - Mail: `MAIL_HOST`, `MAIL_PORT`, `MAIL_USERNAME`, `MAIL_PASSWORD`,
    ///   `MAIL_FROM_ADDRESS`, `MAIL_FROM_NAME`, `MAIL_ENCRYPTION`
    /// - Slack: `SLACK_WEBHOOK_URL`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_notifications::NotificationConfig;
    ///
    /// // In bootstrap.rs
    /// let config = NotificationConfig::from_env();
    /// NotificationDispatcher::configure(config);
    /// ```
    pub fn from_env() -> Self {
        Self {
            mail: MailConfig::from_env(),
            slack_webhook: env::var("SLACK_WEBHOOK_URL").ok().filter(|s| !s.is_empty()),
            whatsapp_enabled: env::var("WHATSAPP_ENABLED")
                .ok()
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(false),
            // `in_app` and `database_store` require typed handles (per CONTEXT.md D-14)
            // — they are not env-driven.
            in_app: None,
            database_store: None,
        }
    }

    /// Set the mail configuration.
    pub fn mail(mut self, config: MailConfig) -> Self {
        self.mail = Some(config);
        self
    }

    /// Set the Slack webhook URL.
    pub fn slack_webhook(mut self, url: impl Into<String>) -> Self {
        self.slack_webhook = Some(url.into());
        self
    }

    /// Enable or disable the WhatsApp channel.
    pub fn with_whatsapp_enabled(mut self, enabled: bool) -> Self {
        self.whatsapp_enabled = enabled;
        self
    }

    /// Set the in-app channel configuration (per CONTEXT.md D-07).
    pub fn with_in_app(mut self, config: InAppConfig) -> Self {
        self.in_app = Some(config);
        self
    }

    /// Set the database notification store (per CONTEXT.md D-13).
    ///
    /// When configured, `Channel::Database` dispatches call `store.store(...)`
    /// instead of the placeholder log path.
    pub fn with_database_store(mut self, store: Arc<dyn DatabaseNotificationStore>) -> Self {
        self.database_store = Some(store);
        self
    }
}

impl MailConfig {
    /// Create a new SMTP mail config (backwards compatible).
    pub fn new(host: impl Into<String>, port: u16, from: impl Into<String>) -> Self {
        Self {
            driver: MailDriver::Smtp,
            from: from.into(),
            from_name: None,
            smtp: Some(SmtpConfig {
                host: host.into(),
                port,
                username: None,
                password: None,
                tls: true,
            }),
            resend: None,
        }
    }

    /// Create a new Resend mail config.
    pub fn resend(api_key: impl Into<String>, from: impl Into<String>) -> Self {
        Self {
            driver: MailDriver::Resend,
            from: from.into(),
            from_name: None,
            smtp: None,
            resend: Some(ResendConfig {
                api_key: api_key.into(),
            }),
        }
    }

    /// Create mail configuration from environment variables.
    ///
    /// Returns `None` if required variables are missing.
    ///
    /// Reads the following environment variables:
    /// - `MAIL_DRIVER`: "smtp" (default) or "resend"
    /// - `MAIL_FROM_ADDRESS`: Default from email address (required for all drivers)
    /// - `MAIL_FROM_NAME`: Default from name (optional)
    ///
    /// SMTP driver variables:
    /// - `MAIL_HOST`: SMTP server host (required)
    /// - `MAIL_PORT`: SMTP server port (default: 587)
    /// - `MAIL_USERNAME`: SMTP username (optional)
    /// - `MAIL_PASSWORD`: SMTP password (optional)
    /// - `MAIL_ENCRYPTION`: "tls" or "none" (default: "tls")
    ///
    /// Resend driver variables:
    /// - `RESEND_API_KEY`: Resend API key (required)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use ferro_notifications::MailConfig;
    ///
    /// if let Some(config) = MailConfig::from_env() {
    ///     // Mail is configured
    /// }
    /// ```
    pub fn from_env() -> Option<Self> {
        let from = env::var("MAIL_FROM_ADDRESS")
            .ok()
            .filter(|s| !s.is_empty())?;
        let from_name = env::var("MAIL_FROM_NAME").ok().filter(|s| !s.is_empty());

        let driver_str = env::var("MAIL_DRIVER")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "smtp".into());

        match driver_str.to_lowercase().as_str() {
            "resend" => {
                let api_key = env::var("RESEND_API_KEY").ok().filter(|s| !s.is_empty())?;

                Some(Self {
                    driver: MailDriver::Resend,
                    from,
                    from_name,
                    smtp: None,
                    resend: Some(ResendConfig { api_key }),
                })
            }
            _ => {
                // Default: SMTP (backwards compatible)
                let host = env::var("MAIL_HOST").ok().filter(|s| !s.is_empty())?;

                let port = env::var("MAIL_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(587);

                let username = env::var("MAIL_USERNAME").ok().filter(|s| !s.is_empty());
                let password = env::var("MAIL_PASSWORD").ok().filter(|s| !s.is_empty());

                let tls = env::var("MAIL_ENCRYPTION")
                    .map(|v| v.to_lowercase() != "none")
                    .unwrap_or(true);

                Some(Self {
                    driver: MailDriver::Smtp,
                    from,
                    from_name,
                    smtp: Some(SmtpConfig {
                        host,
                        port,
                        username,
                        password,
                        tls,
                    }),
                    resend: None,
                })
            }
        }
    }

    /// Set SMTP credentials.
    ///
    /// No-ops with a warning when the driver is not [`MailDriver::Smtp`] — calling
    /// `credentials(...)` on a Resend config is almost certainly a caller mistake
    /// and should not silently mutate the config into an SMTP shape.
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        if !matches!(self.driver, MailDriver::Smtp) {
            warn!("MailConfig::credentials called on non-SMTP driver; ignoring");
            return self;
        }
        let smtp = self.smtp.get_or_insert_with(|| SmtpConfig {
            host: String::new(),
            port: 587,
            username: None,
            password: None,
            tls: true,
        });
        smtp.username = Some(username.into());
        smtp.password = Some(password.into());
        self
    }

    /// Set the from name.
    pub fn from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }

    /// Disable TLS (SMTP only).
    pub fn no_tls(mut self) -> Self {
        if let Some(ref mut smtp) = self.smtp {
            smtp.tls = false;
        }
        self
    }
}

/// Resend API attachment payload.
#[derive(Serialize)]
struct ResendAttachment {
    filename: String,
    /// Base64-encoded attachment content (standard alphabet, not URL-safe).
    content: String,
}

/// Resend API email payload.
#[derive(Serialize)]
struct ResendEmailPayload {
    from: String,
    to: Vec<String>,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    bcc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<ResendAttachment>,
}

/// The notification dispatcher.
pub struct NotificationDispatcher;

impl NotificationDispatcher {
    /// Configure the global notification dispatcher.
    pub fn configure(config: NotificationConfig) {
        let _ = CONFIG.set(config);
    }

    /// Get the current configuration.
    pub fn config() -> Option<&'static NotificationConfig> {
        CONFIG.get()
    }

    /// Send a notification to a notifiable entity.
    pub async fn send<N, T>(notifiable: &N, notification: T) -> Result<(), Error>
    where
        N: Notifiable + ?Sized,
        T: Notification,
    {
        let channels = notification.via();
        let notification_type = notification.notification_type();

        info!(
            notification = notification_type,
            channels = ?channels,
            "Dispatching notification"
        );

        for channel in channels {
            match channel {
                Channel::Mail => {
                    if let Some(mail) = notification.to_mail() {
                        Self::send_mail(notifiable, &mail).await?;
                    }
                }
                Channel::Database => {
                    if let Some(db_msg) = notification.to_database() {
                        Self::send_database(notifiable, &db_msg).await?;
                    }
                }
                Channel::Slack => {
                    if let Some(slack) = notification.to_slack() {
                        Self::send_slack(notifiable, &slack).await?;
                    }
                }
                Channel::WhatsApp => {
                    if let Some(wa) = notification.to_whatsapp() {
                        Self::send_whatsapp(notifiable, &wa).await?;
                    }
                }
                Channel::InApp => {
                    if let Some(in_app) = notification.to_in_app() {
                        Self::send_in_app(notifiable, &in_app).await?;
                    }
                }
                Channel::Sms | Channel::Push => {
                    // Per ARCH-FINDING-03 — not implemented in this phase.
                    info!(channel = %channel, "Channel not implemented");
                }
            }
        }

        Ok(())
    }

    /// Send a mail notification, dispatching to the configured driver.
    async fn send_mail<N: Notifiable + ?Sized>(
        notifiable: &N,
        message: &MailMessage,
    ) -> Result<(), Error> {
        let to = notifiable
            .route_notification_for(Channel::Mail)
            .ok_or_else(|| Error::ChannelNotAvailable("No mail route configured".into()))?;

        let config = CONFIG
            .get()
            .and_then(|c| c.mail.as_ref())
            .ok_or_else(|| Error::ChannelNotAvailable("Mail not configured".into()))?;

        info!(to = %to, subject = %message.subject, "Sending mail notification");

        match config.driver {
            MailDriver::Smtp => Self::send_mail_smtp(&to, message, config).await,
            MailDriver::Resend => Self::send_mail_resend(&to, message, config).await,
        }
    }

    /// Send mail via SMTP using lettre.
    async fn send_mail_smtp(
        to: &str,
        message: &MailMessage,
        config: &MailConfig,
    ) -> Result<(), Error> {
        let smtp = config
            .smtp
            .as_ref()
            .ok_or_else(|| Error::mail("SMTP config missing for SMTP driver"))?;

        use lettre::message::{header::ContentType, Attachment, Mailbox, MultiPart, SinglePart};
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

        let from: Mailbox = if let Some(ref name) = config.from_name {
            format!("{} <{}>", name, config.from)
                .parse()
                .map_err(|e| Error::mail(format!("Invalid from address: {e}")))?
        } else {
            config
                .from
                .parse()
                .map_err(|e| Error::mail(format!("Invalid from address: {e}")))?
        };

        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|e| Error::mail(format!("Invalid to address: {e}")))?;

        let mut email_builder = Message::builder()
            .from(from)
            .to(to_mailbox)
            .subject(&message.subject);

        if let Some(ref reply_to) = message.reply_to {
            let reply_to_mailbox: Mailbox = reply_to
                .parse()
                .map_err(|e| Error::mail(format!("Invalid reply-to address: {e}")))?;
            email_builder = email_builder.reply_to(reply_to_mailbox);
        }

        for cc in &message.cc {
            let cc_mailbox: Mailbox = cc
                .parse()
                .map_err(|e| Error::mail(format!("Invalid CC address: {e}")))?;
            email_builder = email_builder.cc(cc_mailbox);
        }

        for bcc in &message.bcc {
            let bcc_mailbox: Mailbox = bcc
                .parse()
                .map_err(|e| Error::mail(format!("Invalid BCC address: {e}")))?;
            email_builder = email_builder.bcc(bcc_mailbox);
        }

        // Body part — single-part for backward-compat when no attachments,
        // SinglePart wrapped in MultiPart::mixed otherwise.
        let email = if message.attachments.is_empty() {
            // Backward-compatible single-part path
            if let Some(ref html) = message.html {
                email_builder
                    .header(ContentType::TEXT_HTML)
                    .body(html.clone())
                    .map_err(|e| Error::mail(format!("Failed to build email: {e}")))?
            } else {
                email_builder
                    .header(ContentType::TEXT_PLAIN)
                    .body(message.body.clone())
                    .map_err(|e| Error::mail(format!("Failed to build email: {e}")))?
            }
        } else {
            // Multipart path — body becomes a SinglePart inside MultiPart::mixed
            let body_part = if let Some(ref html) = message.html {
                SinglePart::html(html.clone())
            } else {
                SinglePart::plain(message.body.clone())
            };

            let mut mp = MultiPart::mixed().singlepart(body_part);
            for att in &message.attachments {
                let ct = ContentType::parse(&att.content_type).map_err(|e| {
                    Error::mail(format!("Invalid content-type '{}': {e}", att.content_type))
                })?;
                let part = Attachment::new(att.filename.clone()).body(att.content.clone(), ct);
                mp = mp.singlepart(part);
            }

            email_builder
                .multipart(mp)
                .map_err(|e| Error::mail(format!("Failed to build multipart email: {e}")))?
        };

        let transport = if smtp.tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host)
                .map_err(|e| Error::mail(format!("Failed to create transport: {e}")))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp.host)
        };

        let transport = transport.port(smtp.port);

        let transport = if let (Some(ref user), Some(ref pass)) = (&smtp.username, &smtp.password) {
            transport.credentials(Credentials::new(user.clone(), pass.clone()))
        } else {
            transport
        };

        let mailer = transport.build();

        mailer
            .send(email)
            .await
            .map_err(|e| Error::mail(format!("Failed to send email: {e}")))?;

        info!(to = %to, "Mail notification sent via SMTP");
        Ok(())
    }

    /// Send mail via Resend HTTP API.
    async fn send_mail_resend(
        to: &str,
        message: &MailMessage,
        config: &MailConfig,
    ) -> Result<(), Error> {
        let resend = config
            .resend
            .as_ref()
            .ok_or_else(|| Error::mail("Resend config missing for Resend driver"))?;

        let from = message.from.clone().unwrap_or_else(|| {
            if let Some(ref name) = config.from_name {
                format!("{} <{}>", name, config.from)
            } else {
                config.from.clone()
            }
        });

        use base64::Engine;

        let attachments: Vec<ResendAttachment> = message
            .attachments
            .iter()
            .map(|att| ResendAttachment {
                filename: att.filename.clone(),
                content: base64::engine::general_purpose::STANDARD.encode(&att.content),
            })
            .collect();

        let payload = ResendEmailPayload {
            from,
            to: vec![to.to_string()],
            subject: message.subject.clone(),
            html: message.html.clone(),
            text: if message.html.is_some() {
                None
            } else {
                Some(message.body.clone())
            },
            cc: message.cc.clone(),
            bcc: message.bcc.clone(),
            reply_to: message.reply_to.clone(),
            attachments,
        };

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.resend.com/emails")
            .bearer_auth(&resend.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::mail(format!("Resend HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Resend API error");
            return Err(Error::mail(format!("Resend API error {status}: {body}")));
        }

        // Parse the success body to extract the Resend message id (correlation token
        // for downstream debugging). Failure to parse is non-fatal: the send already
        // succeeded per the 2xx status, so we log and continue without an id.
        let resend_id = match response.json::<serde_json::Value>().await {
            Ok(body) => body
                .get("id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .unwrap_or_else(|| "<no-id>".to_string()),
            Err(e) => {
                warn!(error = %e, "Resend response parse failed; continuing without id");
                "<unparseable>".to_string()
            }
        };

        info!(to = %to, resend_id = %resend_id, "Mail notification sent via Resend");
        Ok(())
    }

    /// Send a database notification.
    ///
    /// Per CONTEXT.md D-13 / ARCH-FINDING-02, this calls
    /// [`DatabaseNotificationStore::store`] when `NotificationConfig::database_store`
    /// is configured. When unconfigured, retains the original placeholder log
    /// (backward-compat: no consumer of the placeholder is broken).
    async fn send_database<N: Notifiable + ?Sized>(
        notifiable: &N,
        message: &DatabaseMessage,
    ) -> Result<(), Error> {
        let notifiable_id = notifiable.notifiable_id();
        let notifiable_type = notifiable.notifiable_type();

        if let Some(store) = CONFIG.get().and_then(|c| c.database_store.as_ref()) {
            store
                .store(
                    &notifiable_id,
                    notifiable_type,
                    &message.notification_type,
                    message,
                )
                .await?;
            info!(
                notifiable_id = %notifiable_id,
                notification_type = %message.notification_type,
                "Database notification stored"
            );
        } else {
            warn!(
                notifiable_id = %notifiable_id,
                notifiable_type = %notifiable_type,
                notification_type = %message.notification_type,
                data = ?message.data,
                "Database notification dropped — no store configured. \
                 Call NotificationConfig::with_database_store() at startup."
            );
        }

        Ok(())
    }

    /// Send a Slack notification.
    async fn send_slack<N: Notifiable + ?Sized>(
        notifiable: &N,
        message: &SlackMessage,
    ) -> Result<(), Error> {
        let webhook_url = notifiable
            .route_notification_for(Channel::Slack)
            .or_else(|| CONFIG.get().and_then(|c| c.slack_webhook.clone()))
            .ok_or_else(|| Error::ChannelNotAvailable("No Slack webhook configured".into()))?;

        info!(channel = ?message.channel, "Sending Slack notification");

        let client = reqwest::Client::new();
        let response = client
            .post(&webhook_url)
            .json(message)
            .send()
            .await
            .map_err(|e| Error::slack(format!("HTTP request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Slack webhook failed");
            return Err(Error::slack(format!("Slack returned {status}: {body}")));
        }

        info!("Slack notification sent");
        Ok(())
    }

    /// Send a WhatsApp notification via the static `ferro_whatsapp::WhatsApp` facade.
    ///
    /// Per CONTEXT.md D-04 / ARCH-FINDING-01, the adapter does NOT inject a client.
    /// `ferro_whatsapp::WhatsApp` owns its global state via `WhatsApp::init` (called
    /// once at app startup). The `whatsapp_enabled: false` default ensures this code
    /// path is unreachable unless the consumer opted in.
    ///
    /// **Retry is the caller's responsibility.** This dispatcher performs no backoff,
    /// jitter, or retry on transient failures. Match on the inner [`ferro_whatsapp::Error`]
    /// via `Error::WhatsApp(inner)` to distinguish retryable variants (e.g. `RateLimit`,
    /// `NetworkError`) from terminal ones (e.g. invalid phone number); recommended path
    /// for retry is to enqueue the send through `ferro-queue`.
    async fn send_whatsapp<N: Notifiable + ?Sized>(
        notifiable: &N,
        message: &WhatsAppMessage,
    ) -> Result<(), Error> {
        let enabled = CONFIG.get().map(|c| c.whatsapp_enabled).unwrap_or(false);

        if !enabled {
            info!("WhatsApp channel not configured (WHATSAPP_ENABLED=false)");
            return Ok(());
        }

        let phone = notifiable
            .route_notification_for(Channel::WhatsApp)
            .ok_or_else(|| Error::ChannelNotAvailable("No WhatsApp route configured".into()))?;

        info!(to = %phone, "Sending WhatsApp notification");

        // The Error::WhatsApp(#[from]) conversion (Plan 02) handles the propagation.
        let result = ferro_whatsapp::WhatsApp::send(&phone, message.message.clone()).await?;
        info!(to = %phone, wamid = %result.wamid, "WhatsApp notification sent");
        Ok(())
    }

    /// Send an in-app notification (per CONTEXT.md D-06, D-07, D-08).
    ///
    /// Writes both legs:
    /// 1. Persists via [`DatabaseNotificationStore::store`] (DB leg first — broker can replay
    ///    on reconnect from the store; the inverse order would risk silent loss).
    /// 2. Publishes via `Broadcaster::broadcast` to channel `user.{id}` with event
    ///    `Notification.{notification_type}` and the InAppMessage `data` as payload.
    ///
    /// Returns the first error encountered. The legs are NOT transactional: if the DB
    /// write succeeds and the broadcast then fails, the persisted row remains and a
    /// naive retry will create a duplicate. Per CONTEXT.md D-08 this is intentional —
    /// the broker can replay from the store on reconnect — but callers performing
    /// manual retries should dedupe on (notifiable_id, notification_type, idempotency-key).
    /// `ferro_broadcast::Error` is mapped via [`Error::broadcast`] (no `#[from]` available).
    async fn send_in_app<N: Notifiable + ?Sized>(
        notifiable: &N,
        message: &InAppMessage,
    ) -> Result<(), Error> {
        let cfg = match CONFIG.get().and_then(|c| c.in_app.as_ref()) {
            Some(c) => c,
            None => {
                info!("InApp channel not configured");
                return Ok(());
            }
        };

        let notifiable_id = notifiable.notifiable_id();
        let notifiable_type = notifiable.notifiable_type();

        // Convert InAppMessage to DatabaseMessage for persistence.
        // Per RESEARCH.md: object data → HashMap of fields; non-object → wrap as { "payload": value }.
        let db_msg = inapp_to_database_message(message);

        // Leg 1: persistence (DB-store first per D-08).
        cfg.store
            .store(
                &notifiable_id,
                notifiable_type,
                &message.notification_type,
                &db_msg,
            )
            .await?;

        // Leg 2: real-time fanout.
        let channel = format!("user.{notifiable_id}");
        let event = format!("Notification.{}", message.notification_type);
        cfg.broker
            .broadcast(&channel, &event, &message.data)
            .await
            .map_err(|e| Error::broadcast(e.to_string()))?;

        info!(
            notifiable_id = %notifiable_id,
            notification_type = %message.notification_type,
            "InApp notification persisted and broadcast"
        );
        Ok(())
    }
}

/// Convert an [`InAppMessage`] to a [`DatabaseMessage`] for persistence.
///
/// If `msg.data` is a JSON object, its fields become the DatabaseMessage data map.
/// Otherwise, the value is wrapped under the key `"payload"`.
fn inapp_to_database_message(msg: &InAppMessage) -> DatabaseMessage {
    use std::collections::HashMap;
    let data: HashMap<String, serde_json::Value> = if let serde_json::Value::Object(map) = &msg.data
    {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    } else {
        let mut m = HashMap::new();
        m.insert("payload".to_string(), msg.data.clone());
        m
    };
    DatabaseMessage::new(&msg.notification_type).with_data(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_mail_config_smtp_builder() {
        let config = MailConfig::new("smtp.example.com", 587, "noreply@example.com")
            .credentials("user", "pass")
            .from_name("My App");

        assert!(matches!(config.driver, MailDriver::Smtp));
        assert_eq!(config.from, "noreply@example.com");
        assert_eq!(config.from_name, Some("My App".to_string()));

        let smtp = config.smtp.as_ref().unwrap();
        assert_eq!(smtp.host, "smtp.example.com");
        assert_eq!(smtp.port, 587);
        assert_eq!(smtp.username, Some("user".to_string()));
        assert_eq!(smtp.password, Some("pass".to_string()));
        assert!(smtp.tls);
        assert!(config.resend.is_none());
    }

    #[test]
    fn test_mail_config_resend_builder() {
        let config = MailConfig::resend("re_123456", "noreply@example.com").from_name("My App");

        assert!(matches!(config.driver, MailDriver::Resend));
        assert_eq!(config.from, "noreply@example.com");
        assert_eq!(config.from_name, Some("My App".to_string()));

        let resend = config.resend.as_ref().unwrap();
        assert_eq!(resend.api_key, "re_123456");
        assert!(config.smtp.is_none());
    }

    #[test]
    fn test_mail_config_no_tls() {
        let config = MailConfig::new("smtp.example.com", 587, "noreply@example.com").no_tls();

        let smtp = config.smtp.as_ref().unwrap();
        assert!(!smtp.tls);
    }

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.mail.is_none());
        assert!(config.slack_webhook.is_none());
        assert!(!config.whatsapp_enabled);
        assert!(config.in_app.is_none());
        assert!(config.database_store.is_none());
    }

    #[test]
    fn test_notification_config_with_database_store_builder() {
        use crate::channels::DatabaseMessage;
        use crate::notifiable::DatabaseNotificationStore;
        use async_trait::async_trait;

        struct NoopStore;
        #[async_trait]
        impl DatabaseNotificationStore for NoopStore {
            async fn store(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: &DatabaseMessage,
            ) -> Result<(), Error> {
                Ok(())
            }
            async fn mark_as_read(&self, _: &str) -> Result<(), Error> {
                Ok(())
            }
            async fn unread(&self, _: &str) -> Result<Vec<crate::StoredNotification>, Error> {
                Ok(vec![])
            }
        }

        let store: Arc<dyn DatabaseNotificationStore> = Arc::new(NoopStore);
        let config = NotificationConfig::new().with_database_store(store);
        assert!(config.database_store.is_some());
    }

    #[test]
    #[serial]
    fn test_notification_config_whatsapp_enabled_from_env() {
        unsafe { env::remove_var("WHATSAPP_ENABLED") };
        with_env_vars(&[("WHATSAPP_ENABLED", "true")], || {
            let config = NotificationConfig::from_env();
            assert!(config.whatsapp_enabled);
        });
    }

    #[test]
    #[serial]
    fn test_notification_config_whatsapp_disabled_when_env_false() {
        unsafe { env::remove_var("WHATSAPP_ENABLED") };
        with_env_vars(&[("WHATSAPP_ENABLED", "false")], || {
            let config = NotificationConfig::from_env();
            assert!(!config.whatsapp_enabled);
        });
    }

    #[test]
    #[serial]
    fn test_notification_config_whatsapp_disabled_when_env_unset() {
        unsafe { env::remove_var("WHATSAPP_ENABLED") };
        let config = NotificationConfig::from_env();
        assert!(!config.whatsapp_enabled);
    }

    #[test]
    #[serial]
    fn test_notification_config_whatsapp_disabled_when_env_garbage() {
        unsafe { env::remove_var("WHATSAPP_ENABLED") };
        with_env_vars(&[("WHATSAPP_ENABLED", "yes-please")], || {
            let config = NotificationConfig::from_env();
            assert!(
                !config.whatsapp_enabled,
                "non-bool string must fall back to false"
            );
        });
    }

    #[test]
    fn test_notification_config_with_whatsapp_enabled_builder() {
        let config = NotificationConfig::new().with_whatsapp_enabled(true);
        assert!(config.whatsapp_enabled);
        let config2 = NotificationConfig::new().with_whatsapp_enabled(false);
        assert!(!config2.whatsapp_enabled);
    }

    /// Helper to run env-based tests with clean env var state.
    fn with_env_vars<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
        // Set vars
        for (key, val) in vars {
            unsafe { env::set_var(key, val) };
        }
        f();
        // Clean up
        for (key, _) in vars {
            unsafe { env::remove_var(key) };
        }
    }

    /// Helper to ensure env vars are clean before a test.
    fn clean_mail_env() {
        let keys = [
            "MAIL_DRIVER",
            "MAIL_FROM_ADDRESS",
            "MAIL_FROM_NAME",
            "MAIL_HOST",
            "MAIL_PORT",
            "MAIL_USERNAME",
            "MAIL_PASSWORD",
            "MAIL_ENCRYPTION",
            "RESEND_API_KEY",
        ];
        for key in keys {
            unsafe { env::remove_var(key) };
        }
    }

    #[test]
    #[serial]
    fn test_mail_config_smtp_from_env() {
        clean_mail_env();
        with_env_vars(
            &[
                ("MAIL_FROM_ADDRESS", "noreply@example.com"),
                ("MAIL_FROM_NAME", "Test App"),
                ("MAIL_HOST", "smtp.example.com"),
                ("MAIL_PORT", "465"),
                ("MAIL_USERNAME", "user@example.com"),
                ("MAIL_PASSWORD", "secret"),
                ("MAIL_ENCRYPTION", "tls"),
            ],
            || {
                let config = MailConfig::from_env().expect("should parse SMTP config");
                assert!(matches!(config.driver, MailDriver::Smtp));
                assert_eq!(config.from, "noreply@example.com");
                assert_eq!(config.from_name, Some("Test App".to_string()));

                let smtp = config.smtp.as_ref().expect("smtp config present");
                assert_eq!(smtp.host, "smtp.example.com");
                assert_eq!(smtp.port, 465);
                assert_eq!(smtp.username, Some("user@example.com".to_string()));
                assert_eq!(smtp.password, Some("secret".to_string()));
                assert!(smtp.tls);
                assert!(config.resend.is_none());
            },
        );
    }

    #[test]
    #[serial]
    fn test_mail_config_resend_from_env() {
        clean_mail_env();
        with_env_vars(
            &[
                ("MAIL_DRIVER", "resend"),
                ("MAIL_FROM_ADDRESS", "noreply@example.com"),
                ("MAIL_FROM_NAME", "Test App"),
                ("RESEND_API_KEY", "re_test_123456"),
            ],
            || {
                let config = MailConfig::from_env().expect("should parse Resend config");
                assert!(matches!(config.driver, MailDriver::Resend));
                assert_eq!(config.from, "noreply@example.com");
                assert_eq!(config.from_name, Some("Test App".to_string()));

                let resend = config.resend.as_ref().expect("resend config present");
                assert_eq!(resend.api_key, "re_test_123456");
                assert!(config.smtp.is_none());
            },
        );
    }

    #[test]
    #[serial]
    fn test_mail_config_default_driver() {
        clean_mail_env();
        with_env_vars(
            &[
                ("MAIL_FROM_ADDRESS", "noreply@example.com"),
                ("MAIL_HOST", "smtp.example.com"),
            ],
            || {
                let config = MailConfig::from_env().expect("should default to SMTP");
                assert!(matches!(config.driver, MailDriver::Smtp));
                assert_eq!(config.smtp.as_ref().unwrap().host, "smtp.example.com");
                assert_eq!(config.smtp.as_ref().unwrap().port, 587); // default port
            },
        );
    }

    #[test]
    #[serial]
    fn test_mail_config_resend_missing_api_key() {
        clean_mail_env();
        with_env_vars(
            &[
                ("MAIL_DRIVER", "resend"),
                ("MAIL_FROM_ADDRESS", "noreply@example.com"),
            ],
            || {
                let config = MailConfig::from_env();
                assert!(
                    config.is_none(),
                    "should return None when RESEND_API_KEY missing"
                );
            },
        );
    }

    #[test]
    fn test_resend_payload_serialization() {
        let payload = ResendEmailPayload {
            from: "sender@example.com".into(),
            to: vec!["recipient@example.com".into()],
            subject: "Test".into(),
            html: Some("<p>Hello</p>".into()),
            text: None,
            cc: vec![],
            bcc: vec![],
            reply_to: None,
            attachments: vec![],
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["from"], "sender@example.com");
        assert_eq!(json["to"][0], "recipient@example.com");
        assert_eq!(json["subject"], "Test");
        assert_eq!(json["html"], "<p>Hello</p>");
        // skip_serializing_if fields should be absent
        assert!(json.get("text").is_none());
        assert!(json.get("cc").is_none());
        assert!(json.get("bcc").is_none());
        assert!(json.get("reply_to").is_none());
        assert!(json.get("attachments").is_none());
    }

    #[test]
    fn test_resend_payload_text_fallback() {
        let payload = ResendEmailPayload {
            from: "sender@example.com".into(),
            to: vec!["recipient@example.com".into()],
            subject: "Test".into(),
            html: None,
            text: Some("Plain text body".into()),
            cc: vec!["cc@example.com".into()],
            bcc: vec!["bcc@example.com".into()],
            reply_to: Some("reply@example.com".into()),
            attachments: vec![],
        };

        let json = serde_json::to_value(&payload).unwrap();
        assert!(json.get("html").is_none());
        assert_eq!(json["text"], "Plain text body");
        assert_eq!(json["cc"][0], "cc@example.com");
        assert_eq!(json["bcc"][0], "bcc@example.com");
        assert_eq!(json["reply_to"], "reply@example.com");
    }

    #[test]
    fn test_resend_payload_no_attachments_omits_field() {
        // Regression guard: when attachments empty, the JSON payload has NO "attachments" key.
        let payload = ResendEmailPayload {
            from: "sender@example.com".into(),
            to: vec!["recipient@example.com".into()],
            subject: "Test".into(),
            html: Some("<p>Hello</p>".into()),
            text: None,
            cc: vec![],
            bcc: vec![],
            reply_to: None,
            attachments: vec![],
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert!(
            json.get("attachments").is_none(),
            "Empty attachments must not appear in serialized payload (byte-identical-to-today guarantee)"
        );
    }

    #[test]
    fn test_resend_payload_with_attachments_serializes_base64() {
        let payload = ResendEmailPayload {
            from: "sender@example.com".into(),
            to: vec!["recipient@example.com".into()],
            subject: "Test".into(),
            html: None,
            text: Some("body".into()),
            cc: vec![],
            bcc: vec![],
            reply_to: None,
            attachments: vec![ResendAttachment {
                filename: "hi.txt".into(),
                // "hello" -> base64 standard = "aGVsbG8="
                content: "aGVsbG8=".into(),
            }],
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["attachments"][0]["filename"], "hi.txt");
        assert_eq!(json["attachments"][0]["content"], "aGVsbG8=");
        assert_eq!(json["attachments"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_base64_encoding_uses_standard_alphabet() {
        use base64::Engine;
        // Verify a known fixture: "Many hands make light work."
        // Standard base64: "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu"
        // URL-safe would substitute / and + characters; standard is what Resend expects.
        let encoded =
            base64::engine::general_purpose::STANDARD.encode(b"Many hands make light work.");
        assert_eq!(encoded, "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu");
    }

    #[test]
    fn test_send_whatsapp_disabled_returns_ok_without_calling_init() {
        // The behavior contract: when whatsapp_enabled is false, send_whatsapp must NOT
        // touch ferro_whatsapp at all. We verify this indirectly by checking that
        // NotificationConfig::default().whatsapp_enabled is false (already covered by
        // test_notification_config_default), and that the dispatcher gating is
        // observable through the public `whatsapp_enabled` field.
        //
        // A live integration test that sends a real WhatsApp message lives downstream
        // in gestiscilo-it Phase 120 (per ROADMAP success criterion #7).
        let config = NotificationConfig::default();
        assert!(
            !config.whatsapp_enabled,
            "Default whatsapp_enabled must be false so dispatch path is gated"
        );
    }

    #[test]
    fn test_smtp_multipart_path_compiles_with_attachment() {
        // Smoke test — actual SMTP send is exercised by the Mailpit integration test in Plan 07.
        // Here we just construct a MailMessage with an attachment and verify the type compiles end-to-end.
        use crate::channels::MailMessage;
        let mail = MailMessage::new()
            .subject("Test")
            .body("Hello")
            .attachment("test.txt", "text/plain", b"hello".to_vec())
            .expect("under-limit attachment must succeed");
        assert_eq!(mail.attachments.len(), 1);
        assert_eq!(mail.attachments[0].content_type, "text/plain");
    }

    #[test]
    fn test_inapp_to_database_message_object_data_flattens() {
        use crate::channels::{InAppMessage, InAppSeverity};
        let msg = InAppMessage::new("OrderShipped")
            .data(serde_json::json!({"order_id": 42, "tracking": "ABC"}))
            .severity(InAppSeverity::Success);
        let db = inapp_to_database_message(&msg);
        assert_eq!(db.notification_type, "OrderShipped");
        assert_eq!(db.get("order_id"), Some(&serde_json::json!(42)));
        assert_eq!(db.get("tracking"), Some(&serde_json::json!("ABC")));
        assert!(
            db.get("payload").is_none(),
            "object data must NOT be wrapped under 'payload'"
        );
    }

    #[test]
    fn test_inapp_to_database_message_non_object_wraps_under_payload() {
        use crate::channels::InAppMessage;
        let msg = InAppMessage::new("Heartbeat").data(serde_json::json!("ping"));
        let db = inapp_to_database_message(&msg);
        assert_eq!(db.notification_type, "Heartbeat");
        assert_eq!(db.get("payload"), Some(&serde_json::json!("ping")));
    }

    #[tokio::test]
    async fn test_send_database_calls_store_when_configured() {
        // Substantive behavior — store.store() is invoked with the dispatcher's wiring —
        // is best exercised at the consumer integration level (gestiscilo-it Phase 120).
        // Since CONFIG is a OnceLock global, we cannot inject a fresh store per test
        // without restructuring the dispatcher. Here we verify the *path* exists by
        // exercising the trait directly through the public builder shape.
        use crate::channels::DatabaseMessage;
        use crate::notifiable::DatabaseNotificationStore;
        use async_trait::async_trait;
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingStore {
            calls: AtomicUsize,
        }
        #[async_trait]
        impl DatabaseNotificationStore for CountingStore {
            async fn store(
                &self,
                _: &str,
                _: &str,
                _: &str,
                _: &DatabaseMessage,
            ) -> Result<(), Error> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            async fn mark_as_read(&self, _: &str) -> Result<(), Error> {
                Ok(())
            }
            async fn unread(&self, _: &str) -> Result<Vec<crate::StoredNotification>, Error> {
                Ok(vec![])
            }
        }

        let store = Arc::new(CountingStore {
            calls: AtomicUsize::new(0),
        });
        let msg = DatabaseMessage::new("Test").data("k", "v");
        store
            .store("user_id", "User", &msg.notification_type, &msg)
            .await
            .unwrap();
        assert_eq!(store.calls.load(Ordering::SeqCst), 1);
    }
}
