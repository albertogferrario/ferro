//! Notification dispatcher for sending notifications through channels.

use crate::channel::Channel;
use crate::channels::{MailMessage, SlackMessage};
use crate::notifiable::Notifiable;
use crate::notification::Notification;
use crate::Error;
use std::env;
use std::sync::OnceLock;
use tracing::{error, info};

/// Global notification dispatcher configuration.
static CONFIG: OnceLock<NotificationConfig> = OnceLock::new();

/// Configuration for the notification dispatcher.
#[derive(Clone, Default)]
pub struct NotificationConfig {
    /// Mail configuration (supports SMTP and Resend drivers).
    pub mail: Option<MailConfig>,
    /// Slack webhook URL.
    pub slack_webhook: Option<String>,
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
                let api_key = env::var("RESEND_API_KEY")
                    .ok()
                    .filter(|s| !s.is_empty())?;

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

                let username =
                    env::var("MAIL_USERNAME").ok().filter(|s| !s.is_empty());
                let password =
                    env::var("MAIL_PASSWORD").ok().filter(|s| !s.is_empty());

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
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        let smtp = self.smtp.get_or_insert(SmtpConfig {
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
                Channel::Sms | Channel::Push => {
                    // Not implemented yet
                    info!(channel = %channel, "Channel not implemented");
                }
            }
        }

        Ok(())
    }

    /// Send a mail notification.
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

        let smtp = config
            .smtp
            .as_ref()
            .ok_or_else(|| Error::mail("SMTP config missing for SMTP driver".to_string()))?;

        // Build the email
        use lettre::message::{header::ContentType, Mailbox};
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

        let from: Mailbox = if let Some(ref name) = config.from_name {
            format!("{} <{}>", name, config.from)
                .parse()
                .map_err(|e| Error::mail(format!("Invalid from address: {}", e)))?
        } else {
            config
                .from
                .parse()
                .map_err(|e| Error::mail(format!("Invalid from address: {}", e)))?
        };

        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|e| Error::mail(format!("Invalid to address: {}", e)))?;

        let mut email_builder = Message::builder()
            .from(from)
            .to(to_mailbox)
            .subject(&message.subject);

        // Add reply-to if specified
        if let Some(ref reply_to) = message.reply_to {
            let reply_to_mailbox: Mailbox = reply_to
                .parse()
                .map_err(|e| Error::mail(format!("Invalid reply-to address: {}", e)))?;
            email_builder = email_builder.reply_to(reply_to_mailbox);
        }

        // Add CC recipients
        for cc in &message.cc {
            let cc_mailbox: Mailbox = cc
                .parse()
                .map_err(|e| Error::mail(format!("Invalid CC address: {}", e)))?;
            email_builder = email_builder.cc(cc_mailbox);
        }

        // Add BCC recipients
        for bcc in &message.bcc {
            let bcc_mailbox: Mailbox = bcc
                .parse()
                .map_err(|e| Error::mail(format!("Invalid BCC address: {}", e)))?;
            email_builder = email_builder.bcc(bcc_mailbox);
        }

        // Build the message body
        let email = if let Some(ref html) = message.html {
            email_builder
                .header(ContentType::TEXT_HTML)
                .body(html.clone())
                .map_err(|e| Error::mail(format!("Failed to build email: {}", e)))?
        } else {
            email_builder
                .header(ContentType::TEXT_PLAIN)
                .body(message.body.clone())
                .map_err(|e| Error::mail(format!("Failed to build email: {}", e)))?
        };

        // Build the transport
        let transport = if smtp.tls {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.host)
                .map_err(|e| Error::mail(format!("Failed to create transport: {}", e)))?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp.host)
        };

        let transport = transport.port(smtp.port);

        let transport =
            if let (Some(ref user), Some(ref pass)) = (&smtp.username, &smtp.password) {
                transport.credentials(Credentials::new(user.clone(), pass.clone()))
            } else {
                transport
            };

        let mailer = transport.build();

        // Send the email
        mailer
            .send(email)
            .await
            .map_err(|e| Error::mail(format!("Failed to send email: {}", e)))?;

        info!(to = %to, "Mail notification sent");
        Ok(())
    }

    /// Send a database notification.
    async fn send_database<N: Notifiable + ?Sized>(
        notifiable: &N,
        message: &crate::channels::DatabaseMessage,
    ) -> Result<(), Error> {
        let notifiable_id = notifiable.notifiable_id();
        let notifiable_type = notifiable.notifiable_type();

        info!(
            notifiable_id = %notifiable_id,
            notification_type = %message.notification_type,
            "Storing database notification"
        );

        // In a real implementation, this would store to the database.
        // For now, we just log it. The user should implement DatabaseNotificationStore.
        info!(
            notifiable_id = %notifiable_id,
            notifiable_type = %notifiable_type,
            notification_type = %message.notification_type,
            data = ?message.data,
            "Database notification stored (placeholder)"
        );

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
            .map_err(|e| Error::slack(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Slack webhook failed");
            return Err(Error::slack(format!("Slack returned {}: {}", status, body)));
        }

        info!("Slack notification sent");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let config = MailConfig::resend("re_123456", "noreply@example.com")
            .from_name("My App");

        assert!(matches!(config.driver, MailDriver::Resend));
        assert_eq!(config.from, "noreply@example.com");
        assert_eq!(config.from_name, Some("My App".to_string()));

        let resend = config.resend.as_ref().unwrap();
        assert_eq!(resend.api_key, "re_123456");
        assert!(config.smtp.is_none());
    }

    #[test]
    fn test_mail_config_no_tls() {
        let config = MailConfig::new("smtp.example.com", 587, "noreply@example.com")
            .no_tls();

        let smtp = config.smtp.as_ref().unwrap();
        assert!(!smtp.tls);
    }

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert!(config.mail.is_none());
        assert!(config.slack_webhook.is_none());
    }
}
