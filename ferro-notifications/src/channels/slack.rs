//! Slack notification channel.

use serde::{Deserialize, Serialize};

/// A Slack message for webhook notifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlackMessage {
    /// Main message text.
    pub text: String,
    /// Optional channel override.
    pub channel: Option<String>,
    /// Optional username override.
    pub username: Option<String>,
    /// Optional icon emoji (e.g., ":rocket:").
    pub icon_emoji: Option<String>,
    /// Optional icon URL.
    pub icon_url: Option<String>,
    /// Message attachments.
    pub attachments: Vec<SlackAttachment>,
}

/// A Slack message attachment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlackAttachment {
    /// Attachment fallback text.
    pub fallback: Option<String>,
    /// Attachment color (hex or named).
    pub color: Option<String>,
    /// Pretext shown above attachment.
    pub pretext: Option<String>,
    /// Author name.
    pub author_name: Option<String>,
    /// Author link.
    pub author_link: Option<String>,
    /// Attachment title.
    pub title: Option<String>,
    /// Title link.
    pub title_link: Option<String>,
    /// Main attachment text.
    pub text: Option<String>,
    /// Attachment fields.
    pub fields: Vec<SlackField>,
    /// Footer text.
    pub footer: Option<String>,
    /// Timestamp (Unix epoch).
    pub ts: Option<i64>,
}

/// A field within a Slack attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackField {
    /// Field title.
    pub title: String,
    /// Field value.
    pub value: String,
    /// Whether to display inline.
    pub short: bool,
}

impl SlackMessage {
    /// Create a new Slack message with text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Set the channel.
    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    /// Set the username.
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set the icon emoji.
    pub fn icon_emoji(mut self, emoji: impl Into<String>) -> Self {
        self.icon_emoji = Some(emoji.into());
        self
    }

    /// Set the icon URL.
    pub fn icon_url(mut self, url: impl Into<String>) -> Self {
        self.icon_url = Some(url.into());
        self
    }

    /// Add an attachment.
    pub fn attachment(mut self, attachment: SlackAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }
}

impl SlackAttachment {
    /// Create a new empty attachment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the color.
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the title link.
    pub fn title_link(mut self, link: impl Into<String>) -> Self {
        self.title_link = Some(link.into());
        self
    }

    /// Set the text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Add a field.
    pub fn field(
        mut self,
        title: impl Into<String>,
        value: impl Into<String>,
        short: bool,
    ) -> Self {
        self.fields.push(SlackField {
            title: title.into(),
            value: value.into(),
            short,
        });
        self
    }

    /// Set the footer.
    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    /// Set the timestamp.
    pub fn timestamp(mut self, ts: i64) -> Self {
        self.ts = Some(ts);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_message_builder() {
        let msg = SlackMessage::new("Hello, Slack!")
            .channel("#general")
            .username("MyBot")
            .icon_emoji(":robot_face:");

        assert_eq!(msg.text, "Hello, Slack!");
        assert_eq!(msg.channel, Some("#general".into()));
        assert_eq!(msg.username, Some("MyBot".into()));
        assert_eq!(msg.icon_emoji, Some(":robot_face:".into()));
    }

    #[test]
    fn test_slack_attachment_builder() {
        let attachment = SlackAttachment::new()
            .color("good")
            .title("Order Placed")
            .text("Order #123 has been placed")
            .field("Customer", "John Doe", true)
            .field("Amount", "$99.99", true);

        assert_eq!(attachment.color, Some("good".into()));
        assert_eq!(attachment.title, Some("Order Placed".into()));
        assert_eq!(attachment.fields.len(), 2);
        assert_eq!(attachment.fields[0].title, "Customer");
        assert!(attachment.fields[0].short);
    }

    #[test]
    fn test_slack_message_with_attachment() {
        let msg = SlackMessage::new("New order received").attachment(
            SlackAttachment::new()
                .color("#36a64f")
                .title("Order Details"),
        );

        assert_eq!(msg.attachments.len(), 1);
    }
}
