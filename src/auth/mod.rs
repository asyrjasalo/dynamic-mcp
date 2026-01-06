//! OAuth2 authentication for dynamic-mcp.
//!
//! This module provides OAuth2/PKCE authentication flow for HTTP and SSE transports.
//! It handles token storage, automatic refresh, and browser-based authorization.
//!
//! # Features
//!
//! - PKCE flow for secure authentication
//! - Automatic token discovery via .well-known endpoints
//! - Local token storage in `~/.dynamic-mcp/oauth-servers/`
//! - Automatic token refresh before expiry
//! - Browser-based authorization flow

pub mod oauth_client;
pub mod store;

pub use oauth_client::OAuthClient;
