//! Email layout primitive for branded, table-based transactional emails.
//!
//! [`MailLayout`] is a content builder: callers compose an email from a small set
//! of [`ContentBlock`] components (heading, paragraph, detail rows, CTA button,
//! callout, divider) plus [`BrandParams`], and [`MailLayout::render`] emits an
//! email-client-safe HTML document together with a derived plain-text alternative.
//!
//! This is a content concern, distinct from the transport-layer
//! [`MailMessage`](crate::MailMessage): wire the two together with
//! `let (html, text) = layout.render(); MailMessage::new().html(html).body(text)`.
//!
//! All plain-text content is HTML-escaped at render time. The only raw-HTML path is
//! [`MailLayout::paragraph_html`], which is for trusted, code-constructed markup only.

/// Default accent colour used when [`BrandParams::accent_color`] is `None`.
///
/// Kept as the exact string `#0052cc` — downstream consumers assert on it.
const DEFAULT_ACCENT: &str = "#0052cc";

/// Per-tenant branding inputs for the email shell.
///
/// Only [`BrandParams::brand_name`] is used unconditionally; every other field is
/// optional and omitted cleanly (no empty markup) when `None`.
#[derive(Debug, Clone, Default)]
pub struct BrandParams {
    /// Sender/business name rendered in the header (as text when no logo) and footer.
    pub brand_name: String,
    /// Logo image URL. `None` → the brand name is rendered as text in the header.
    pub logo_url: Option<String>,
    /// Accent colour as a CSS hex string (e.g. `#0052cc`). `None` → [`DEFAULT_ACCENT`].
    pub accent_color: Option<String>,
    /// Footer copy line (e.g. contact/identity). `None` → omit the line.
    pub footer: Option<String>,
    /// Hidden inbox-preview preheader text. `None` → omit the hidden div.
    pub preheader: Option<String>,
}

/// A single composable block of email content.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    /// A section heading (`<h2>`); text is escaped at render time.
    Heading(String),
    /// A plain-text paragraph; text is escaped at render time.
    Paragraph(String),
    /// A paragraph carrying trusted raw HTML plus a plain-text fallback.
    ///
    /// The `html` is inserted verbatim (NOT escaped) — use only with
    /// code-constructed markup, never with un-sanitised caller input.
    ParagraphHtml {
        /// Trusted raw HTML for the HTML part.
        html: String,
        /// Plain-text equivalent for the text part.
        text: String,
    },
    /// A two-column `(label, value)` detail table; both columns escaped.
    DetailRows(Vec<(String, String)>),
    /// A call-to-action button; `label` escaped, `url` attribute-escaped.
    CtaButton {
        /// Visible button label.
        label: String,
        /// Destination URL.
        url: String,
    },
    /// A tinted callout box with an optional title; title/body escaped.
    Callout {
        /// Optional bold title line.
        title: Option<String>,
        /// Callout body text.
        body: String,
    },
    /// A horizontal rule separator.
    Divider,
}

/// Builder for a branded transactional email.
///
/// Accumulates [`ContentBlock`]s in order, then [`MailLayout::render`] produces the
/// `(html, text)` pair.
#[derive(Debug, Clone)]
pub struct MailLayout {
    brand: BrandParams,
    blocks: Vec<ContentBlock>,
}

impl MailLayout {
    /// Create a new layout for the given brand, with no content blocks yet.
    pub fn new(brand: BrandParams) -> Self {
        Self {
            brand,
            blocks: Vec::new(),
        }
    }

    /// Append a section heading (escaped at render time).
    pub fn heading(mut self, text: impl Into<String>) -> Self {
        self.blocks.push(ContentBlock::Heading(text.into()));
        self
    }

    /// Append a plain-text paragraph (escaped at render time).
    pub fn paragraph(mut self, text: impl Into<String>) -> Self {
        self.blocks.push(ContentBlock::Paragraph(text.into()));
        self
    }

    /// Append a paragraph of trusted raw HTML plus a plain-text fallback.
    ///
    /// The `html` argument is inserted verbatim — pass only code-constructed markup.
    pub fn paragraph_html(
        mut self,
        html: impl Into<String>,
        text_fallback: impl Into<String>,
    ) -> Self {
        self.blocks.push(ContentBlock::ParagraphHtml {
            html: html.into(),
            text: text_fallback.into(),
        });
        self
    }

    /// Append a two-column `(label, value)` detail table (both columns escaped).
    pub fn detail_rows(mut self, rows: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        self.blocks.push(ContentBlock::DetailRows(
            rows.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        ));
        self
    }

    /// Append an accent-coloured call-to-action button.
    pub fn cta_button(mut self, label: impl Into<String>, url: impl Into<String>) -> Self {
        self.blocks.push(ContentBlock::CtaButton {
            label: label.into(),
            url: url.into(),
        });
        self
    }

    /// Append a tinted callout box with no title.
    pub fn callout(mut self, body: impl Into<String>) -> Self {
        self.blocks.push(ContentBlock::Callout {
            title: None,
            body: body.into(),
        });
        self
    }

    /// Append a tinted callout box with a bold title.
    pub fn callout_with_title(mut self, title: impl Into<String>, body: impl Into<String>) -> Self {
        self.blocks.push(ContentBlock::Callout {
            title: Some(title.into()),
            body: body.into(),
        });
        self
    }

    /// Append a horizontal-rule divider.
    pub fn divider(mut self) -> Self {
        self.blocks.push(ContentBlock::Divider);
        self
    }

    /// Render the layout to `(html, text)`.
    ///
    /// The HTML is an email-client-safe, table-based document with inline styles
    /// only; the text part is derived from the same content model. Infallible.
    /// Wire into a message with
    /// `MailMessage::new().html(html).body(text)`.
    pub fn render(&self) -> (String, String) {
        let accent = self.brand.accent_color.as_deref().unwrap_or(DEFAULT_ACCENT);

        let preheader_html = match &self.brand.preheader {
            Some(p) => format!(
                r#"<div style="display:none;max-height:0;overflow:hidden">{}</div>"#,
                html_escape(p)
            ),
            None => String::new(),
        };

        let header_content = match &self.brand.logo_url {
            Some(url) => format!(
                r#"<img src="{}" height="40" alt="{}" style="display:block">"#,
                attr_escape(url),
                html_escape(&self.brand.brand_name)
            ),
            None => format!(
                r#"<span style="font-size:18px;font-weight:600;color:{}">{}</span>"#,
                accent,
                html_escape(&self.brand.brand_name)
            ),
        };

        let mut html_blocks = String::new();
        let mut text_blocks = String::new();
        for block in &self.blocks {
            let (h, t) = render_block(block, accent);
            html_blocks.push_str(&h);
            text_blocks.push_str(&t);
        }

        let footer_html = match &self.brand.footer {
            Some(f) => format!(
                r#"<p style="margin:0 0 4px;color:#666;font-size:12px">{}</p>"#,
                html_escape(f)
            ),
            None => String::new(),
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="it">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
{preheader_html}</head>
<body style="margin:0;padding:0;background:#f5f5f5;font-family:Arial,Helvetica,sans-serif">
<table width="100%" cellpadding="0" cellspacing="0" border="0" style="background:#f5f5f5"><tr>
<td align="center" style="padding:24px 16px">
<table width="600" cellpadding="0" cellspacing="0" border="0" style="max-width:600px;width:100%;background:#ffffff">
<tr><td style="padding:24px 32px;border-bottom:2px solid {accent}">{header_content}</td></tr>
<tr><td style="padding:32px">{html_blocks}</td></tr>
<tr><td style="padding:16px 32px;border-top:1px solid #eee;background:#fafafa">
{footer_html}<p style="margin:0;color:#999;font-size:11px">{brand_name}</p>
</td></tr>
</table></td></tr></table>
</body></html>"#,
            preheader_html = preheader_html,
            accent = accent,
            header_content = header_content,
            html_blocks = html_blocks,
            footer_html = footer_html,
            brand_name = html_escape(&self.brand.brand_name),
        );

        (html, text_blocks)
    }
}

/// Render a single content block to its `(html, text)` fragments.
fn render_block(block: &ContentBlock, accent: &str) -> (String, String) {
    match block {
        ContentBlock::Heading(s) => (
            format!(
                r#"<h2 style="margin:0 0 16px;font-size:20px;color:#333">{}</h2>"#,
                html_escape(s)
            ),
            format!("{}\n\n", s),
        ),
        ContentBlock::Paragraph(s) => (
            format!(
                r#"<p style="margin:0 0 16px;color:#444;font-size:15px;line-height:1.5">{}</p>"#,
                html_escape(s)
            ),
            format!("{}\n\n", s),
        ),
        ContentBlock::ParagraphHtml { html, text } => (
            format!(
                r#"<p style="margin:0 0 16px;color:#444;font-size:15px;line-height:1.5">{}</p>"#,
                html
            ),
            format!("{}\n\n", text),
        ),
        ContentBlock::DetailRows(rows) => {
            let mut html = String::from(
                r#"<table cellpadding="0" cellspacing="0" border="0" style="margin:0 0 16px;width:100%">"#,
            );
            let mut text = String::new();
            for (label, value) in rows {
                html.push_str(&format!(
                    r#"<tr><td style="padding:6px 0;color:#666;font-size:14px;width:140px">{}</td><td style="padding:6px 0;color:#333;font-size:14px;font-weight:600">{}</td></tr>"#,
                    html_escape(label),
                    html_escape(value)
                ));
                text.push_str(&format!("{}: {}\n", label, value));
            }
            html.push_str("</table>");
            text.push('\n');
            (html, text)
        }
        ContentBlock::CtaButton { label, url } => (
            format!(
                r#"<p style="margin:16px 0"><a href="{}" style="display:inline-block;padding:12px 24px;background:{};color:#fff;text-decoration:none;border-radius:6px;font-size:14px;font-weight:600">{}</a></p>"#,
                attr_escape(url),
                accent,
                html_escape(label)
            ),
            format!("{}: {}\n\n", label, url),
        ),
        ContentBlock::Callout { title, body } => {
            let title_html = match title {
                Some(t) => format!(
                    r#"<p style="margin:0 0 8px;font-weight:600;font-size:15px;color:#333">{}</p>"#,
                    html_escape(t)
                ),
                None => String::new(),
            };
            let html = format!(
                r#"<div style="margin:16px 0;padding:20px;background:#f0f4ff;border-radius:8px">{}<p style="margin:0;color:#444;font-size:14px;line-height:1.5">{}</p></div>"#,
                title_html,
                html_escape(body)
            );
            let text = match title {
                Some(t) => format!("[{}]\n{}\n\n", t, body),
                None => format!("{}\n\n", body),
            };
            (html, text)
        }
        ContentBlock::Divider => (
            r#"<hr style="border:none;border-top:1px solid #eee;margin:24px 0">"#.to_string(),
            "---\n\n".to_string(),
        ),
    }
}

/// Escape `&`, `<`, `>`, `"` for safe insertion into HTML text/attribute content.
///
/// `&` is replaced first to avoid double-escaping the entities introduced afterward.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Escape a URL for use inside an HTML attribute value (`href`/`src`).
fn attr_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_brand() -> BrandParams {
        BrandParams {
            brand_name: "Acme".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn render_html_contains_heading_text() {
        let (html, _) = MailLayout::new(default_brand())
            .heading("Benvenuto")
            .render();
        assert!(
            html.contains("Benvenuto"),
            "heading text must appear in HTML"
        );
    }

    #[test]
    fn render_text_mirrors_heading() {
        let (_, text) = MailLayout::new(default_brand())
            .heading("Benvenuto")
            .render();
        assert!(
            text.contains("Benvenuto"),
            "heading must appear in plain text"
        );
    }

    #[test]
    fn accent_none_uses_default_0052cc() {
        let brand = BrandParams {
            brand_name: "X".into(),
            accent_color: None,
            ..Default::default()
        };
        let (html, _) = MailLayout::new(brand)
            .cta_button("Go", "https://example.com")
            .render();
        assert!(html.contains("0052cc"), "default accent must be #0052cc");
    }

    #[test]
    fn logo_url_none_renders_brand_name_as_text() {
        let (html, _) = MailLayout::new(default_brand()).render();
        assert!(
            html.contains("Acme"),
            "brand name must appear in header when no logo"
        );
        assert!(!html.contains("<img"), "no img tag when logo_url is None");
    }

    #[test]
    fn logo_url_some_renders_img_tag() {
        let brand = BrandParams {
            brand_name: "Acme".into(),
            logo_url: Some("https://cdn.example.com/logo.png".into()),
            ..Default::default()
        };
        let (html, _) = MailLayout::new(brand).render();
        assert!(html.contains("<img"), "logo renders as img");
        assert!(
            html.contains("cdn.example.com/logo.png"),
            "logo URL present"
        );
    }

    #[test]
    fn preheader_none_produces_no_hidden_div() {
        let (html, _) = MailLayout::new(default_brand()).render();
        assert!(
            !html.contains("display:none"),
            "no hidden div without preheader"
        );
    }

    #[test]
    fn preheader_some_renders_hidden_div() {
        let brand = BrandParams {
            brand_name: "X".into(),
            preheader: Some("Your booking is confirmed".into()),
            ..Default::default()
        };
        let (html, _) = MailLayout::new(brand).render();
        assert!(
            html.contains("display:none"),
            "preheader hidden div present"
        );
        assert!(html.contains("Your booking is confirmed"));
    }

    #[test]
    fn detail_rows_render_label_and_value() {
        let (html, text) = MailLayout::new(default_brand())
            .detail_rows(vec![("Data".to_string(), "1 Jan 2027".to_string())])
            .render();
        assert!(html.contains("Data") && html.contains("1 Jan 2027"));
        assert!(text.contains("Data:") && text.contains("1 Jan 2027"));
    }

    #[test]
    fn cta_button_renders_url_and_label() {
        let (html, text) = MailLayout::new(default_brand())
            .cta_button("Clicca qui", "https://example.com/action")
            .render();
        assert!(html.contains("Clicca qui") && html.contains("https://example.com/action"));
        assert!(text.contains("Clicca qui") && text.contains("https://example.com/action"));
    }

    #[test]
    fn callout_renders_body_in_html_and_text() {
        let (html, text) = MailLayout::new(default_brand())
            .callout("Ricorda di portare un documento")
            .render();
        assert!(html.contains("Ricorda di portare un documento"));
        assert!(text.contains("Ricorda di portare un documento"));
    }

    #[test]
    fn divider_renders_hr_in_html_and_separator_in_text() {
        let (html, text) = MailLayout::new(default_brand()).divider().render();
        assert!(html.contains("<hr"), "divider renders as <hr> in HTML");
        assert!(text.contains("---"), "divider renders as --- in plain text");
    }

    #[test]
    fn empty_layout_renders_valid_html_document() {
        let (html, _) = MailLayout::new(default_brand()).render();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn html_escapes_brand_name_angle_brackets() {
        let brand = BrandParams {
            brand_name: "A&B <Inc>".into(),
            ..Default::default()
        };
        let (html, _) = MailLayout::new(brand).render();
        assert!(
            html.contains("A&amp;B &lt;Inc&gt;"),
            "brand name must be escaped"
        );
        assert!(
            !html.contains("A&B <Inc>"),
            "raw unescaped brand name must not appear"
        );
    }

    #[test]
    fn paragraph_escapes_plain_text() {
        let (html, _) = MailLayout::new(default_brand())
            .paragraph("5 < 10 & rising")
            .render();
        assert!(
            html.contains("5 &lt; 10 &amp; rising"),
            "paragraph text must be escaped"
        );
    }

    #[test]
    fn paragraph_html_emits_raw_html_and_text_fallback() {
        let (html, text) = MailLayout::new(default_brand())
            .paragraph_html(
                r#"<a href="https://x/y">Disdici</a>"#,
                "Disdici: https://x/y",
            )
            .render();
        assert!(
            html.contains(r#"<a href="https://x/y">Disdici</a>"#),
            "raw anchor markup must appear verbatim in HTML"
        );
        assert!(
            text.contains("Disdici: https://x/y"),
            "text fallback must appear in plain text"
        );
    }

    #[test]
    fn callout_with_title_renders_title() {
        let (html, text) = MailLayout::new(default_brand())
            .callout_with_title("Titolo", "Corpo del messaggio")
            .render();
        assert!(html.contains("Titolo") && html.contains("Corpo del messaggio"));
        assert!(text.contains("[Titolo]") && text.contains("Corpo del messaggio"));
    }

    #[test]
    fn footer_none_renders_no_footer_copy_line() {
        let (html, _) = MailLayout::new(default_brand()).render();
        // The brand-name footer line is always present; the optional copy line is not.
        assert!(
            !html.contains("color:#666;font-size:12px"),
            "no footer copy line without footer"
        );
    }

    #[test]
    fn footer_some_renders_footer_copy_line() {
        let brand = BrandParams {
            brand_name: "Acme".into(),
            footer: Some("Via Roma 1, Milano — info@acme.it".into()),
            ..Default::default()
        };
        let (html, _) = MailLayout::new(brand).render();
        assert!(
            html.contains("Via Roma 1, Milano"),
            "footer copy line must appear"
        );
    }

    #[test]
    fn cta_url_with_ampersand_is_attr_escaped() {
        let (html, _) = MailLayout::new(default_brand())
            .cta_button("Go", "https://x/y?a=1&b=2")
            .render();
        assert!(
            html.contains("a=1&amp;b=2"),
            "url ampersand must be attribute-escaped"
        );
    }
}
