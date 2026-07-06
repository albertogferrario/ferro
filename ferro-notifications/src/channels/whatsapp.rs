//! WhatsApp notification channel.

/// A WhatsApp message wrapping the typed shape from `ferro-whatsapp`.
///
/// Construct via [`WhatsAppMessage::text`] or [`WhatsAppMessage::template`].
/// The dispatcher passes `self.message` to `ferro_whatsapp::WhatsApp::send`
/// (per CONTEXT.md D-04 — static facade, not an injected client).
#[derive(Debug, Clone)]
pub struct WhatsAppMessage {
    /// The underlying WhatsApp Cloud API message variant.
    pub message: ferro_whatsapp::Message,
}

impl WhatsAppMessage {
    /// Build a plain-text WhatsApp message.
    pub fn text(body: impl Into<String>) -> Self {
        Self {
            message: ferro_whatsapp::Message::Text { body: body.into() },
        }
    }

    /// Build a template WhatsApp message.
    ///
    /// The `parameters` vector must contain typed parameter objects per Meta spec,
    /// e.g. `serde_json::json!({"type": "text", "text": "value"})`.
    pub fn template(
        name: impl Into<String>,
        language: impl Into<String>,
        parameters: Vec<serde_json::Value>,
    ) -> Self {
        Self {
            message: ferro_whatsapp::Message::Template {
                name: name.into(),
                language: language.into(),
                parameters,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whatsapp_message_text_builder() {
        let msg = WhatsAppMessage::text("hello");
        match msg.message {
            ferro_whatsapp::Message::Text { body } => assert_eq!(body, "hello"),
            _ => panic!("expected Text variant"),
        }
    }

    #[test]
    fn test_whatsapp_message_template_builder() {
        let params = vec![serde_json::json!({"type": "text", "text": "Alberto"})];
        let msg = WhatsAppMessage::template("order_confirmation", "it", params.clone());
        match msg.message {
            ferro_whatsapp::Message::Template {
                name,
                language,
                parameters,
            } => {
                assert_eq!(name, "order_confirmation");
                assert_eq!(language, "it");
                assert_eq!(parameters, params);
            }
            _ => panic!("expected Template variant"),
        }
    }
}
