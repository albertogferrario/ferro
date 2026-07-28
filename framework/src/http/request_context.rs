//! Task-local request context accessible from any handler without a `Request` parameter.

tokio::task_local! {
    pub(crate) static REQUEST_HOST: String;
    pub(crate) static FJUI_NAV_TARGET: Option<String>;
}

/// Return the `Host` header value (scheme-less, port stripped) for the current request.
///
/// Available inside any handler executed by the ferro server. Returns `None` outside
/// of a request context (e.g. during tests or background jobs).
pub fn request_host() -> Option<String> {
    REQUEST_HOST.try_with(|h| h.clone()).ok()
}

/// Return the `X-FJUI-Target` header value for the current request, if any.
///
/// Set by the server when the header is present. Used by the JSON-UI fragment
/// response path to skip full-page layout and return only the target subtree.
pub fn fjui_nav_target() -> Option<String> {
    FJUI_NAV_TARGET.try_with(|t| t.clone()).ok().flatten()
}
