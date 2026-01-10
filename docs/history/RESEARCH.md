# Rust MCP SDK Research

## Research Date

January 6, 2026

## Executive Summary

The Rust MCP ecosystem is mature and production-ready with an official SDK (`rmcp`) that supports all required features for building a dynamic-mcp proxy server.

## Official Rust SDK

**Repository**: https://github.com/modelcontextprotocol/rust-sdk
**Crates**: `rmcp` (v0.12.0), `rmcp-macros` (v0.12.0)
**Downloads**: 2.3M+ each
**Maintenance**: Active (December 2025)

### Key Features

- **Tokio-based async runtime**: Full async/await support
- **Declarative macros**: `#[tool]`, `#[prompt]`, `#[resource]` for easy server implementation
- **Multiple transports**: STDIO, HTTP/SSE, child process, in-process
- **OAuth2 authentication**: Built-in support via `oauth2` crate
- **Long-running tasks**: Polling system for async operations
- **JSON Schema generation**: Automatic schema generation for tool inputs

### Dependencies

```toml
[package]
name = "rmcp"
version = "0.12.0"
description = "Rust SDK for Model Context Protocol"

[dependencies]
tokio = { version = "1", features = ["sync", "macros", "rt", "time"] }
serde = { version = "1.0", features = ["derive", "rc"] }
schemars = { version = "1.0", optional = true }
oauth2 = { version = "5.0", optional = true }
```

## Transport Support

### Officially Supported

| Transport         | Crate Feature                                                                  | Status        |
| ----------------- | ------------------------------------------------------------------------------ | ------------- |
| **STDIO**         | Built-in                                                                       | ✅ Production |
| **HTTP/SSE**      | `transport-streamable-http-server`, `transport-streamable-http-client-reqwest` | ✅ Production |
| **Child Process** | `transport-child-process`                                                      | ✅ Production |
| **In-Process**    | Built-in                                                                       | ✅ Testing    |

### Community Transports

- **WebSocket**: `mcp-server-runner`
- **P2P/QUIC**: `saorsa-core`
- **SSH/Telnet**: `ptyctl`

## Core MCP Primitives

### 1. Tools (Callable Functions)

Tools are executable functions that the LLM can call. The SDK provides a `#[tool]` macro for declarative tool definition.

**Example**:

```rust
#[tool_router]
impl Counter {
    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
}
```

**Source**: [counter.rs#L84-L108](https://github.com/modelcontextprotocol/rust-sdk/blob/63d89b1a4edf0eab869079d6fee91e0ad871a33b/examples/servers/src/common/counter.rs#L84-L108)

### 2. Resources (Read-Only Data)

Resources provide read-only data access through URIs. Implement via `ServerHandler` trait.

**Example**:

```rust
impl ServerHandler for Counter {
    async fn list_resources(&self, ...) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource::new("file:///path", "File".into()).no_annotation(),
                RawResource::new("memo://insights", "Memo".into()).no_annotation(),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(&self, ReadResourceRequestParam { uri }, ...)
        -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "file:///path" => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text("content", uri)],
            }),
            _ => Err(McpError::resource_not_found("not_found", None)),
        }
    }
}
```

**Source**: [counter.rs#L215-L255](https://github.com/modelcontextprotocol/rust-sdk/blob/63d89b1a4edf0eab869079d6fee91e0ad871a33b/examples/servers/src/common/counter.rs#L215-L255)

### 3. Prompts (Reusable Templates)

Prompts are reusable message templates that can be parameterized.

**Example**:

```rust
#[prompt_router]
impl Counter {
    #[prompt(name = "counter_analysis")]
    async fn counter_analysis(
        &self,
        Parameters(args): Parameters<CounterAnalysisArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let messages = vec![
            PromptMessage::new_text(PromptMessageRole::User,
                format!("Analyze: {}", args.goal)),
        ];
        Ok(GetPromptResult {
            description: Some("Analysis".into()),
            messages,
        })
    }
}
```

**Source**: [counter.rs#L141-L196](https://github.com/modelcontextprotocol/rust-sdk/blob/63d89b1a4edf0eab869079d6fee91e0ad871a33b/examples/servers/src/common/counter.rs#L141-L196)

## Authentication & OAuth

### OAuth2 Support

The official SDK includes OAuth2 authentication support through the `auth` feature flag.

```toml
[dependencies]
rmcp = { version = "0.12.0", features = ["auth"] }
oauth2 = "5.0"
```

### Custom Authentication

You can implement custom authentication by inspecting HTTP request headers in the `initialize` handler:

```rust
impl ServerHandler for Counter {
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let headers = &http_request_part.headers;
            // Validate authentication headers
        }
        Ok(self.get_info())
    }
}
```

**Source**: [counter.rs#L269-L280](https://github.com/modelcontextprotocol/rust-sdk/blob/63d89b1a4edf0eab869079d6fee91e0ad871a33b/examples/servers/src/common/counter.rs#L269-L280)

## Proxy Server Pattern

### AgentGateway (Production Reference)

**Repository**: https://github.com/agentgateway/agentgateway
**Stars**: 1,526
**Purpose**: Production MCP gateway/proxy server

Features:

- Multi-tenant support
- RBAC (Role-Based Access Control)
- Dynamic configuration
- Security-first design
- Acts as both MCP server (downstream) and client (upstream)

### Custom Proxy Implementation Pattern

```rust
#[derive(Clone)]
pub struct MCPProxy {
    upstream_servers: HashMap<String, String>,
}

impl ServerHandler for MCPProxy {
    async fn list_tools(&self, ...) -> Result<ListToolsResult, McpError> {
        let mut all_tools = Vec::new();
        for (server_name, _) in &self.upstream_servers {
            if let Ok(client) = self.get_upstream_client(server_name).await {
                if let Ok(tools) = client.list_all_tools().await {
                    all_tools.extend(tools.tools);
                }
            }
        }
        Ok(ListToolsResult { tools: all_tools, ... })
    }
}
```

## Ecosystem Statistics

| Metric                      | Value                                           |
| --------------------------- | ----------------------------------------------- |
| **Total Rust MCP Projects** | 800+ on GitHub                                  |
| **Active Crates**           | 50+ on crates.io                                |
| **Official SDK Downloads**  | 2.3M+                                           |
| **Specialized Servers**     | 50+ (Elasticsearch, Terraform, SurrealDB, etc.) |
| **Proxy/Gateway Solutions** | 5+ (AgentGateway, mcp-guardian, etc.)           |
| **Maintenance Status**      | 90% updated in last 6 months                    |

## Alternative Libraries (Tier-1)

| Library                 | Downloads | Best For                              | GitHub                                                      |
| ----------------------- | --------- | ------------------------------------- | ----------------------------------------------------------- |
| **rust-mcp-schema**     | 188k+     | Type-safe schema definitions          | [link](https://github.com/rust-mcp-stack/rust-mcp-schema)   |
| **MCPR**                | 10k+      | Rapid development, project generation | [link](https://github.com/conikeec/mcpr)                    |
| **turul-mcp-framework** | 1.3k+     | High-performance, HTTP/SSE focus      | [link](https://github.com/aussierobots/turul-mcp-framework) |
| **turbomcp**            | 10k+      | Complete protocol with context mgmt   | [link](https://github.com/Epistates/turbomcp)               |

## Recommended Stack for MCP Proxy

```
Layer 1: Transport
├─ Downstream: transport-streamable-http-server (for clients)
└─ Upstream: transport-child-process (for servers)

Layer 2: Core
├─ rmcp (official SDK)
├─ tokio (async runtime)
└─ serde (serialization)

Layer 3: Features
├─ axum (HTTP server)
├─ oauth2 (authentication)
├─ schemars (JSON schema)
└─ tracing (logging)
```

## Getting Started Guide

1. **Start with Official SDK**: `rmcp` v0.12.0
2. **Choose Transport**: STDIO (development) → HTTP/SSE (production)
3. **Define Primitives**: Use `#[tool]`, `#[prompt]`, `#[resource]` macros
4. **Implement ServerHandler**: Handle requests/notifications
5. **Test**: Use `mcp-probe` (105 stars) or `mcp-discovery` (76 stars)
6. **Deploy**: With appropriate transport layer

## Key Resources

- **Official Spec**: https://modelcontextprotocol.io/specification/2025-11-25
- **Official SDK Docs**: https://docs.rs/rmcp/latest/rmcp/
- **OAuth Support**: https://github.com/modelcontextprotocol/rust-sdk/blob/main/docs/OAUTH_SUPPORT.md
- **Community Hub**: https://github.com/rust-mcp-stack

## Conclusion

The Rust MCP ecosystem is production-ready with:

- ✅ Mature, well-maintained official SDK
- ✅ Comprehensive transport support (stdio, HTTP, SSE)
- ✅ OAuth2 authentication built-in
- ✅ Strong ecosystem with 800+ projects
- ✅ Active community and regular updates

**Recommendation**: Use the official `rmcp` SDK as the foundation for building the dynamic-mcp proxy server.
