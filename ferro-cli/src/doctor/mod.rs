//! ferro doctor — health checks for a Ferro project (D-01..D-09).
//!
//! Single-command, machine-readable project health diagnostics.
//! Complementary to (not a replacement for) the `ferro:info` MCP tool.

pub mod check;
pub mod checks;
pub mod registry;
