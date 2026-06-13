use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("service definition error: {0}")]
    Definition(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Caller supplied an empty intents slice — no render target exists.
    /// Returned by render entry points instead of a silent `"unknown"`;
    /// defined here so non-visual renderers can consume it. (D-08)
    #[error("cannot render service with no intents")]
    NoIntents,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_intents_error_message() {
        let err = Error::NoIntents;
        assert_eq!(err.to_string(), "cannot render service with no intents");
    }
}
