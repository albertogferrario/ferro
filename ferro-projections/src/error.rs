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
    /// An action selected for transition derivation declares no
    /// `transition_trigger`, so no event-to-target mapping exists.
    #[error("action '{0}' has no transition_trigger")]
    NoTransitionTrigger(String),
    /// Transition derivation was requested for a service with no state machine.
    #[error("service '{0}' has no state machine")]
    NoStateMachine(String),
    /// An action's `transition_trigger` names an event that no declared
    /// transition carries — the EXEC-04 drift condition, by construction.
    #[error("action '{action}' transition_trigger '{event}' matches no declared transition")]
    UndeclaredTransition { action: String, event: String },
    /// A single event fans out to more than one target state across its
    /// transitions; a `TransitionPlan` cannot pick one unambiguously.
    #[error("event '{event}' fans out to multiple target states — ambiguous transition plan")]
    AmbiguousTransition { event: String },
    /// A CRUD verb was requested for a service that has not opted into it
    /// (`.creatable(false)`, `.updatable(false)`, or `.deletable(false)`).
    #[error("crud verb not enabled for service: {0}")]
    VerbNotEnabled(String),
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
