//! MCP tool rendering target for Ferro projections.
//!
//! `McpRenderer` implements the `Renderer` trait from `ferro-projections`,
//! mirroring how `JsonUiRenderer` lives in `ferro-json-ui`.

pub mod auth;
pub mod config;
pub mod dispatch;
pub mod error;
pub mod jsonrpc;
pub mod renderer;
pub mod schema;

pub use auth::resolve_tenant;
pub use config::McpServerConfig;
pub use dispatch::{dispatch, DispatchResult};
pub use error::{Error, Result};
pub use ferro_mcp_oauth::BearerCheck;
pub use jsonrpc::{handle_initialize, handle_tools_call, handle_tools_list};
pub use renderer::{render_exposed_tools, McpContext, McpRenderer};
