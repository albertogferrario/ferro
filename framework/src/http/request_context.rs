//! Task-local request context accessible from any handler without a `Request` parameter.

tokio::task_local! {
    pub(crate) static REQUEST_HOST: String;
}

/// Return the `Host` header value (scheme-less, port stripped) for the current request.
///
/// Available inside any handler executed by the ferro server. Returns `None` outside
/// of a request context (e.g. during tests or background jobs).
pub fn request_host() -> Option<String> {
    REQUEST_HOST.try_with(|h| h.clone()).ok()
}
