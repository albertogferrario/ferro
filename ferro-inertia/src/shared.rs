//! Shared props that are merged into every Inertia response.

use serde::Serialize;

/// Shared props that are merged into every Inertia response.
///
/// Use this struct to pass common data (auth, flash messages, CSRF tokens)
/// to all Inertia responses via middleware.
///
/// # Example
///
/// ```rust
/// use ferro_inertia::InertiaShared;
/// use serde_json::json;
///
/// let shared = InertiaShared::new()
///     .auth(json!({
///         "id": 1,
///         "name": "John Doe",
///         "email": "john@example.com"
///     }))
///     .csrf("token123")
///     .flash(json!({
///         "success": "Profile updated!"
///     }));
/// ```
#[derive(Clone, Default, Debug, Serialize)]
pub struct InertiaShared {
    /// Authenticated user data (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<serde_json::Value>,
    /// Flash messages from the session
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash: Option<serde_json::Value>,
    /// CSRF token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csrf: Option<String>,
    /// Additional custom shared props
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl InertiaShared {
    /// Create a new empty shared props container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set authenticated user data.
    pub fn auth(mut self, auth: impl Serialize) -> Self {
        self.auth = Some(serde_json::to_value(auth).unwrap_or_default());
        self
    }

    /// Set flash messages.
    pub fn flash(mut self, flash: impl Serialize) -> Self {
        self.flash = Some(serde_json::to_value(flash).unwrap_or_default());
        self
    }

    /// Set CSRF token.
    pub fn csrf(mut self, token: impl Into<String>) -> Self {
        self.csrf = Some(token.into());
        self
    }

    /// Set additional custom shared props.
    ///
    /// These will be flattened into the shared props object.
    pub fn with(mut self, extra: impl Serialize) -> Self {
        self.extra = Some(serde_json::to_value(extra).unwrap_or_default());
        self
    }

    /// Merge shared props into a props object.
    pub fn merge_into(&self, props: &mut serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = props {
            if let Some(auth) = &self.auth {
                map.insert("auth".to_string(), auth.clone());
            }
            if let Some(flash) = &self.flash {
                map.insert("flash".to_string(), flash.clone());
            }
            if let Some(csrf) = &self.csrf {
                map.insert("csrf".to_string(), serde_json::Value::String(csrf.clone()));
            }
            if let Some(serde_json::Value::Object(extra_map)) = &self.extra {
                for (k, v) in extra_map {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
    }
}
