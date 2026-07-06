//! # ferro-api-mcp
//!
//! Standalone MCP server that bridges OpenAPI specifications to Model Context
//! Protocol tools.
//!
//! Loads an OpenAPI v3 document and exposes each operation as an MCP tool,
//! translating JSON Schema parameters, constructing HTTP requests, and
//! forwarding responses to the MCP client over stdio transport.

pub mod error;
pub mod http;
pub mod schema;
pub mod server;
pub mod service;
pub mod spec;
pub mod types;
