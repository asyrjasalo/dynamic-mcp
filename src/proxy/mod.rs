//! Proxy client for managing upstream MCP servers.
//!
//! This module handles connections to multiple upstream MCP servers (called "groups"),
//! manages their state, and proxies tool calls to the appropriate server.
//!
//! # Architecture
//!
//! - Each upstream MCP server is treated as a "group"
//! - Groups can be in Connected or Failed state
//! - Failed groups are tracked with error information for debugging
//! - Supports stdio, HTTP, and SSE transports

pub mod client;
pub mod transport;
pub mod types;

pub use client::ModularMcpClient;
