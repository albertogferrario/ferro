//! Static design-rule registry. Rules are populated in Plans 02–03.
use super::types::DesignRule;

/// The static rule registry. Iterated by [`super::lint`] and [`super::rules`].
pub(super) static RULE_REGISTRY: &[DesignRule] = &[];
