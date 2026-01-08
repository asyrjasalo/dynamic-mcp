//! Configuration management for dynamic-mcp.
//!
//! This module handles loading, parsing, and validating MCP server configurations.
//! It supports environment variable substitution and multiple transport types (stdio, HTTP, SSE).
//!
//! # Examples
//!
//! ```no_run
//! use dynamic_mcp::config::load_config;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = load_config("config.json").await?;
//!     println!("Loaded {} MCP servers", config.mcp_servers.len());
//!     Ok(())
//! }
//! ```

pub mod env_sub;
pub mod loader;
pub mod schema;

pub use loader::load_config;
pub use schema::{IntermediateServerConfig, McpServerConfig, ServerConfig, StandardServerConfig};
