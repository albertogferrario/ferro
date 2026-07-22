//! Service projections — Nearly's domain described through Ferro's core
//! projection / intent abstraction. Each `ServiceDef` declares a service's
//! fields and their semantic meaning, from which the framework derives intents
//! and UI. They are surfaced via [`all`] and exercised by the
//! `projections_derive_intents` test; a future MCP endpoint can expose the same
//! set for agent introspection.

pub mod place;
pub mod presence;
pub mod profile;
pub mod trillo;

use ferro::ServiceDef;

/// All service projections for the application.
pub fn all() -> Vec<ServiceDef> {
    vec![
        profile::service_def(),
        presence::service_def(),
        trillo::service_def(),
        place::service_def(),
    ]
}
