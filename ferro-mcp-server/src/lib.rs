//! MCP tool rendering target for Ferro projections.
//!
//! `McpRenderer` implements the `Renderer` trait from `ferro-projections`,
//! mirroring how `JsonUiRenderer` lives in `ferro-json-ui`.

pub mod dispatch;
pub mod error;
pub mod renderer;
pub mod schema;

pub use dispatch::{dispatch, DispatchResult};
pub use error::{Error, Result};
pub use renderer::{McpContext, McpRenderer};
