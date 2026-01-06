# Implementation Plan: dynamic-mcp in Rust

## Project Overview

**Goal**: Create a Rust-based MCP proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading schemas on-demand.

**Architecture**: Single binary application using the official `rmcp` SDK that acts as:
- **MCP Server** (downstream): Exposes 2 tools to LLM clients
- **MCP Client** (upstream): Connects to multiple configured MCP servers

**Reference Implementation**: https://github.com/d-kimuson/modular-mcp (TypeScript)

## Design Principles

1. **Single Binary**: Simpler distribution, follows TypeScript model
2. **Official SDK**: Leverage `rmcp` for MCP compliance
3. **JSON Config**: Keep compatibility with TypeScript version
4. **Phased Approach**: Incremental delivery, testable at each phase
5. **Error Handling**: Use `anyhow` for application errors, `thiserror` for library errors
6. **Async**: Tokio for all I/O operations
7. **Logging**: `tracing` for structured logging
8. **Testing**: Unit tests + integration tests + example configs

---

## Phase 1: Core Proxy Functionality (stdio transport only)

**Goal**: Implement basic proxy that works with stdio-based MCP servers

**Duration**: 2-3 days

### 1.1 Project Setup

```bash
cargo new dynamic-mcp
cd dynamic-mcp
```

**Dependencies** (`Cargo.toml`):
```toml
[package]
name = "dynamic-mcp"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[[bin]]
name = "dynamic-mcp"
path = "src/main.rs"

[dependencies]
# MCP SDK
rmcp = "0.12"
rmcp-macros = "0.12"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration validation
schemars = { version = "1.0", features = ["preserve_order"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
regex = "1"
```

### 1.2 Project Structure

```
dynamic-mcp/
├── Cargo.toml
├── README.md
├── LICENSE (MIT)
├── config.example.json
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── config/
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   ├── loader.rs
│   │   └── env_sub.rs
│   ├── proxy/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── transport.rs
│   │   └── types.rs
│   └── cli/
│       ├── mod.rs
│       └── migrate.rs
├── tests/
│   ├── config_test.rs
│   └── integration_test.rs
└── docs/
    ├── RESEARCH.md
    └── PLAN.md
```

### 1.3 Configuration Module

**File**: `src/config/schema.rs`

Define configuration structures that mirror the TypeScript version:

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio {
        description: String,
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<HashMap<String, String>>,
    },
    Http {
        description: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
    Sse {
        description: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
    },
}

impl McpServerConfig {
    pub fn description(&self) -> &str {
        match self {
            McpServerConfig::Stdio { description, .. } => description,
            McpServerConfig::Http { description, .. } => description,
            McpServerConfig::Sse { description, .. } => description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}
```

**File**: `src/config/env_sub.rs`

Environment variable substitution (${VAR} syntax only):

```rust
use regex::Regex;
use std::collections::HashMap;
use crate::config::schema::McpServerConfig;

/// Substitutes ${VAR} syntax only (NOT $VAR)
pub fn substitute_env_vars(value: &str) -> String {
    let pattern = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();

    pattern.replace_all(value, |caps: &regex::Captures| {
        let var_name = &caps[1];
        match std::env::var(var_name) {
            Ok(val) => val,
            Err(_) => {
                tracing::warn!(
                    "Environment variable '{}' not defined, keeping placeholder",
                    var_name
                );
                caps[0].to_string() // Keep original ${VAR}
            }
        }
    }).to_string()
}

pub fn substitute_in_object(obj: HashMap<String, String>) -> HashMap<String, String> {
    obj.into_iter()
        .map(|(k, v)| (k, substitute_env_vars(&v)))
        .collect()
}

pub fn substitute_in_array(arr: Vec<String>) -> Vec<String> {
    arr.into_iter()
        .map(|s| substitute_env_vars(&s))
        .collect()
}

pub fn substitute_in_config(config: McpServerConfig) -> McpServerConfig {
    match config {
        McpServerConfig::Stdio { description, command, args, env } => {
            McpServerConfig::Stdio {
                description,
                command,
                args: args.map(substitute_in_array),
                env: env.map(substitute_in_object),
            }
        },
        McpServerConfig::Http { description, url, headers } => {
            McpServerConfig::Http {
                description,
                url: substitute_env_vars(&url),
                headers: headers.map(substitute_in_object),
            }
        },
        McpServerConfig::Sse { description, url, headers } => {
            McpServerConfig::Sse {
                description,
                url: substitute_env_vars(&url),
                headers: headers.map(substitute_in_object),
            }
        },
    }
}
```

**File**: `src/config/loader.rs`

Configuration file loading and validation:

```rust
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use crate::config::schema::ServerConfig;
use crate::config::env_sub::substitute_in_config;

pub async fn load_config(path: &str) -> Result<ServerConfig> {
    let absolute_path = Path::new(path).canonicalize()
        .with_context(|| format!("Failed to resolve config path: {}", path))?;

    // Read file
    let content = fs::read_to_string(&absolute_path).await
        .with_context(|| format!("Failed to read config file: {:?}", absolute_path))?;

    // Parse JSON
    let mut config: ServerConfig = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in config file: {:?}", absolute_path))?;

    // Apply environment variable substitution
    config.mcp_servers = config.mcp_servers
        .into_iter()
        .map(|(name, server_config)| {
            (name, substitute_in_config(server_config))
        })
        .collect();

    tracing::info!("✅ MCP server config loaded successfully");

    Ok(config)
}
```

### 1.4 Proxy Client Module

**File**: `src/proxy/types.rs`

Shared types:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedGroupInfo {
    pub name: String,
    pub description: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}
```

**File**: `src/proxy/client.rs`

Group state management and upstream client coordination:

```rust
use std::collections::HashMap;
use anyhow::{Context, Result};
use rmcp::{Client, ClientOptions};
use crate::config::schema::McpServerConfig;
use crate::proxy::types::{GroupInfo, FailedGroupInfo, ToolInfo};
use crate::proxy::transport::create_transport;

#[derive(Debug)]
pub enum GroupState {
    Connected {
        name: String,
        description: String,
        client: Client,
        tools: Vec<ToolInfo>,
    },
    Failed {
        name: String,
        description: String,
        error: String,
    },
}

pub struct ModularMcpClient {
    groups: HashMap<String, GroupState>,
}

impl ModularMcpClient {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub async fn connect(
        &mut self,
        group_name: String,
        config: McpServerConfig,
    ) -> Result<()> {
        if self.groups.contains_key(&group_name) {
            return Ok(());
        }

        let description = config.description().to_string();

        // Create transport
        let transport = create_transport(&config).await?;

        // Create client
        let client_options = ClientOptions {
            name: "dynamic-mcp-client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let mut client = Client::new(client_options);
        client.connect(transport).await?;

        // Initialize and list tools
        client.initialize().await?;
        let tools_result = client.list_tools().await?;

        let tools: Vec<ToolInfo> = tools_result.tools.iter().map(|tool| {
            ToolInfo {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.input_schema.clone(),
            }
        }).collect();

        self.groups.insert(
            group_name.clone(),
            GroupState::Connected {
                name: group_name,
                description,
                client,
                tools,
            },
        );

        Ok(())
    }

    pub fn record_failed_connection(
        &mut self,
        group_name: String,
        config: McpServerConfig,
        error: anyhow::Error,
    ) {
        self.groups.insert(
            group_name.clone(),
            GroupState::Failed {
                name: group_name,
                description: config.description().to_string(),
                error: error.to_string(),
            },
        );
    }

    pub fn list_groups(&self) -> Vec<GroupInfo> {
        self.groups.values()
            .filter_map(|state| match state {
                GroupState::Connected { name, description, .. } => {
                    Some(GroupInfo {
                        name: name.clone(),
                        description: description.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn list_failed_groups(&self) -> Vec<FailedGroupInfo> {
        self.groups.values()
            .filter_map(|state| match state {
                GroupState::Failed { name, description, error } => {
                    Some(FailedGroupInfo {
                        name: name.clone(),
                        description: description.clone(),
                        error: error.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub async fn list_tools(&self, group_name: &str) -> Result<Vec<ToolInfo>> {
        let group = self.groups.get(group_name)
            .context("Group not found")?;

        match group {
            GroupState::Connected { tools, .. } => Ok(tools.clone()),
            GroupState::Failed { error, .. } => {
                Err(anyhow::anyhow!("Group failed to connect: {}", error))
            }
        }
    }

    pub async fn call_tool(
        &self,
        group_name: &str,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let group = self.groups.get(group_name)
            .context("Group not found")?;

        match group {
            GroupState::Connected { client, .. } => {
                let result = client.call_tool(tool_name, args).await?;
                Ok(serde_json::to_value(result)?)
            }
            GroupState::Failed { error, .. } => {
                Err(anyhow::anyhow!("Group failed to connect: {}", error))
            }
        }
    }

    pub async fn disconnect_all(&mut self) -> Result<()> {
        for (_, state) in self.groups.drain() {
            if let GroupState::Connected { mut client, .. } = state {
                let _ = client.close().await;
            }
        }
        Ok(())
    }
}
```

**File**: `src/proxy/transport.rs`

Transport creation (Phase 1: stdio only):

```rust
use anyhow::{Result, bail};
use rmcp::transport::Transport;
use crate::config::schema::McpServerConfig;

pub async fn create_transport(config: &McpServerConfig) -> Result<Box<dyn Transport>> {
    match config {
        McpServerConfig::Stdio { command, args, env, .. } => {
            use std::process::Command;
            use rmcp::transport::ChildProcessTransport;

            let mut cmd = Command::new(command);

            if let Some(args) = args {
                cmd.args(args);
            }

            if let Some(env_vars) = env {
                cmd.envs(env_vars);
            }

            Ok(Box::new(ChildProcessTransport::new(cmd)?))
        }
        McpServerConfig::Http { .. } | McpServerConfig::Sse { .. } => {
            bail!("HTTP/SSE transport not implemented in Phase 1")
        }
    }
}
```

### 1.5 MCP Server Implementation

**File**: `src/server.rs`

Main MCP server that exposes two tools:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use rmcp::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::proxy::client::ModularMcpClient;

#[derive(Clone)]
pub struct ModularMcpServer {
    client: Arc<Mutex<ModularMcpClient>>,
}

impl ModularMcpServer {
    pub fn new(client: ModularMcpClient) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct GetToolsArgs {
    group: String,
}

#[derive(Deserialize, Serialize)]
struct CallToolArgs {
    group: String,
    name: String,
    #[serde(default)]
    args: serde_json::Value,
}

#[async_trait::async_trait]
impl ServerHandler for ModularMcpServer {
    async fn list_tools(
        &self,
        _request: ListToolsRequest,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let client = self.client.lock().await;
        let groups = client.list_groups();
        let failed_groups = client.list_failed_groups();

        let group_names: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();

        let groups_desc = groups.iter()
            .map(|g| format!("- {}: {}", g.name, g.description))
            .collect::<Vec<_>>()
            .join("\n");

        let failed_desc = if !failed_groups.is_empty() {
            let failed = failed_groups.iter()
                .map(|g| format!("- {}: {} (Error: {})", g.name, g.description, g.error))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\nUnavailable groups (connection failed):\n{}", failed)
        } else {
            String::new()
        };

        let get_tools_desc = format!(
            "dynamic-mcp manages multiple MCP servers as organized groups, \
            providing only the necessary group's tool descriptions to the LLM \
            on demand instead of overwhelming it with all tool descriptions at once.\n\n\
            Use this tool to retrieve available tools in a specific group, \
            then use call-modular-tool to execute them.\n\n\
            Available groups:\n{}{}",
            groups_desc, failed_desc
        );

        let call_tool_desc = r#"Execute a tool from a specific MCP group. Proxies the call to the appropriate upstream MCP server.

Use get-modular-tools first to discover available tools and their input schemas in the specified group, then use this tool to execute them.

This maintains a clean separation between discovery (context-efficient) and execution phases, enabling effective management of large tool collections across multiple MCP servers.

Example usage:
  call-modular-tool(group="playwright", name="browser_navigate", args={"url": "https://example.com"})
  → Executes the browser_navigate tool from the playwright group with the specified arguments"#;

        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: "get-modular-tools".to_string(),
                    description: Some(get_tools_desc),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "group": {
                                "type": "string",
                                "description": "The name of the MCP group to get tools from",
                                "enum": group_names
                            }
                        },
                        "required": ["group"]
                    }),
                },
                Tool {
                    name: "call-modular-tool".to_string(),
                    description: Some(call_tool_desc.to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "group": {
                                "type": "string",
                                "description": "The name of the MCP group containing the tool",
                                "enum": group_names
                            },
                            "name": {
                                "type": "string",
                                "description": "The name of the tool to execute"
                            },
                            "args": {
                                "type": "object",
                                "description": "Arguments to pass to the tool",
                                "additionalProperties": true
                            }
                        },
                        "required": ["group", "name"]
                    }),
                },
            ],
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequest,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match request.params.name.as_str() {
            "get-modular-tools" => {
                let args: GetToolsArgs = serde_json::from_value(request.params.arguments)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

                let client = self.client.lock().await;
                let tools = client.list_tools(&args.group).await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                // Strip $schema from input_schema
                let tools_json: Vec<_> = tools.iter().map(|tool| {
                    let mut schema = tool.input_schema.clone();
                    if let Some(obj) = schema.as_object_mut() {
                        obj.remove("$schema");
                    }
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": schema
                    })
                }).collect();

                Ok(CallToolResult::success(vec![
                    Content::text(serde_json::to_string(&tools_json).unwrap())
                ]))
            }
            "call-modular-tool" => {
                let args: CallToolArgs = serde_json::from_value(request.params.arguments)
                    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

                let client = self.client.lock().await;
                let result = client.call_tool(&args.group, &args.name, args.args).await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

                Ok(CallToolResult::success(vec![
                    Content::text(serde_json::to_string(&result).unwrap())
                ]))
            }
            _ => Err(McpError::method_not_found(
                format!("Unknown tool: {}", request.params.name),
                None
            )),
        }
    }
}
```

### 1.6 Main Entry Point

**File**: `src/main.rs`

```rust
mod config;
mod proxy;
mod server;
mod cli;

use clap::Parser;
use tracing_subscriber::EnvFilter;
use rmcp::prelude::*;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "dynamic-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "dynamic-mcp Proxy Server - Reduce context overhead with on-demand tool loading")]
struct Cli {
    /// Path to configuration file
    #[arg(value_name = "CONFIG_FILE")]
    config_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    let cli = Cli::parse();

    // Load configuration
    let config = config::load_config(&cli.config_path).await?;

    // Initialize modular client
    let mut modular_client = proxy::client::ModularMcpClient::new();

    // Connect to all upstream servers in parallel
    let connection_futures: Vec<_> = config.mcp_servers.into_iter()
        .map(|(name, server_config)| {
            async move {
                (name.clone(), server_config.clone(), {
                    let mut client = proxy::client::ModularMcpClient::new();
                    client.connect(name.clone(), server_config.clone()).await
                })
            }
        })
        .collect();

    let connection_results = futures::future::join_all(connection_futures).await;

    for (name, server_config, result) in connection_results {
        match result {
            Ok(_) => {
                tracing::info!("✅ Successfully connected MCP Server: {}", name);
                let _ = modular_client.connect(name, server_config).await;
            }
            Err(e) => {
                tracing::error!("❌ Failed to connect to {}: {}", name, e);
                modular_client.record_failed_connection(name, server_config, e);
            }
        }
    }

    // Log summary
    let groups = modular_client.list_groups();
    let failed = modular_client.list_failed_groups();

    if failed.is_empty() {
        tracing::info!(
            "Successfully connected {} MCP groups. All groups are valid.",
            groups.len()
        );
    } else {
        tracing::warn!(
            "Some MCP groups failed to connect. success_groups=[{}], failed_groups=[{}]",
            groups.iter().map(|g| &g.name).cloned().collect::<Vec<_>>().join(", "),
            failed.iter().map(|g| &g.name).cloned().collect::<Vec<_>>().join(", ")
        );
    }

    // Create and start server
    let server_handler = server::ModularMcpServer::new(modular_client);

    let server_info = ServerInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Start stdio transport
    let transport = StdioTransport::new();
    let mut mcp_server = Server::new(server_info, server_handler);

    // Setup signal handlers for graceful shutdown
    tokio::select! {
        result = mcp_server.serve(transport) => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal, cleaning up...");
        }
    }

    Ok(())
}
```

### 1.7 Example Configuration

**File**: `config.example.json`

```json
{
  "$schema": "https://raw.githubusercontent.com/d-kimuson/dynamic-mcp/refs/heads/main/config-schema.json",
  "mcpServers": {
    "filesystem": {
      "description": "Use when you need to read, write, or search files on the local filesystem.",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "context7": {
      "description": "Use when you need to search library documentation.",
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp@latest"],
      "env": {
        "CONTEXT7_API_KEY": "${CONTEXT7_API_KEY}"
      }
    }
  }
}
```

### 1.8 Deliverables

- ✅ Working stdio-based proxy server
- ✅ Configuration loading with env var substitution
- ✅ Parallel upstream server connections
- ✅ Two-tool API (get-modular-tools, call-modular-tool)
- ✅ Error handling and graceful degradation
- ✅ Logging with tracing

---

## Phase 2: Full Transport Support (HTTP/SSE)

**Goal**: Add support for HTTP and SSE transports

**Duration**: 1-2 days

### 2.1 Add Dependencies

```toml
# Add to Cargo.toml
rmcp = { version = "0.12", features = ["transport-http", "transport-sse"] }
reqwest = { version = "0.11", features = ["json"] }
```

### 2.2 Update Transport Creation

**File**: `src/proxy/transport.rs`

```rust
pub async fn create_transport(config: &McpServerConfig) -> Result<Box<dyn Transport>> {
    match config {
        McpServerConfig::Stdio { command, args, env, .. } => {
            // ... existing stdio code ...
        }
        McpServerConfig::Http { url, headers, .. } => {
            use rmcp::transport::HttpTransport;

            let mut request_builder = reqwest::Client::new().post(url);

            if let Some(headers) = headers {
                for (key, value) in headers {
                    request_builder = request_builder.header(key, value);
                }
            }

            Ok(Box::new(HttpTransport::new(request_builder)?))
        }
        McpServerConfig::Sse { url, headers, .. } => {
            use rmcp::transport::SseTransport;

            Ok(Box::new(SseTransport::new(url, headers.clone())?))
        }
    }
}
```

### 2.3 Fallback to mcp-remote

Add fallback logic similar to TypeScript implementation:

```rust
pub fn convert_to_mcp_remote_if_needed(config: McpServerConfig) -> McpServerConfig {
    // Find npx path
    let npx_path = which::which("npx")
        .unwrap_or_else(|_| std::path::PathBuf::from("npx"))
        .to_string_lossy()
        .to_string();

    match config {
        McpServerConfig::Http { description, url, headers } |
        McpServerConfig::Sse { description, url, headers } => {
            let transport_type = match &config {
                McpServerConfig::Sse { .. } => "sse-only",
                _ => "http-only",
            };

            let mut args = vec![
                "-y".to_string(),
                "mcp-remote".to_string(),
                url,
            ];

            if let Some(headers) = headers {
                for (key, value) in headers {
                    args.push("--header".to_string());
                    args.push(format!("{}: {}", key, value));
                }
            }

            args.push("--transport".to_string());
            args.push(transport_type.to_string());

            McpServerConfig::Stdio {
                description,
                command: npx_path,
                args: Some(args),
                env: None,
            }
        }
        _ => config,
    }
}
```

### 2.4 Deliverables

- ✅ HTTP transport support
- ✅ SSE transport support
- ✅ Fallback to mcp-remote for compatibility
- ✅ Updated tests for all transports

---

## Phase 3: OAuth Authentication

**Goal**: Implement OAuth2 authentication for remote MCP servers

**Duration**: 2-3 days

### 3.1 Add Dependencies

```toml
# Add to Cargo.toml
oauth2 = "5.0"
url = "2.5"
open = "5.0"
dirs = "5.0"
chrono = "0.4"
```

### 3.2 Auth Store

**File**: `src/auth/store.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use anyhow::Result;

#[derive(Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

pub struct AuthStore {
    base_path: PathBuf,
}

impl AuthStore {
    pub fn new() -> Result<Self> {
        let base_path = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".dynamic-mcp")
            .join("oauth-servers");

        Ok(Self { base_path })
    }

    pub async fn save_token(&self, server_name: &str, tokens: &OAuthTokens) -> Result<()> {
        fs::create_dir_all(&self.base_path).await?;
        let path = self.base_path.join(format!("{}.json", server_name));
        let json = serde_json::to_string_pretty(tokens)?;
        fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load_token(&self, server_name: &str) -> Result<Option<OAuthTokens>> {
        let path = self.base_path.join(format!("{}.json", server_name));
        if !tokio::fs::try_exists(&path).await? {
            return Ok(None);
        }
        let json = fs::read_to_string(path).await?;
        let tokens = serde_json::from_str(&json)?;
        Ok(Some(tokens))
    }
}
```

### 3.3 OAuth Client

**File**: `src/auth/oauth_client.rs`

```rust
use oauth2::*;
use anyhow::Result;

pub async fn authenticate_with_oauth(server_url: &str) -> Result<OAuthTokens> {
    // Implementation similar to TypeScript version
    // 1. Discover OAuth endpoints
    // 2. Create OAuth client
    // 3. Generate PKCE challenge
    // 4. Open browser for auth
    // 5. Start local callback server
    // 6. Exchange code for token
    todo!("OAuth implementation")
}
```

### 3.4 Integration with Transport

Update `create_transport` to handle OAuth when needed.

### 3.5 Deliverables

- ✅ OAuth token storage (~/.dynamic-mcp/)
- ✅ OAuth flow with browser
- ✅ Token refresh logic
- ✅ Integration with HTTP/SSE transports

---

## Phase 4: Migration Command

**Goal**: CLI command to migrate standard MCP configs

**Duration**: 1 day

### 4.1 CLI Structure

**File**: `src/cli/mod.rs`

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dynamic-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Config file (when running as server)
    pub config_path: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Migrate standard MCP config to dynamic-mcp format
    Migrate {
        /// Path to standard MCP config file
        mcp_config_path: String,

        /// Output path for dynamic-mcp.json
        #[arg(short, long, default_value = "dynamic-mcp.json")]
        output: String,
    },
}
```

### 4.2 Migration Logic

**File**: `src/cli/migrate.rs`

Prompt user for descriptions and transform config.

### 4.3 Deliverables

- ✅ `dynamic-mcp migrate` command
- ✅ Interactive description prompts
- ✅ Config transformation
- ✅ Original config replacement

---

## Phase 5: Testing & Documentation

**Goal**: Comprehensive tests and documentation

**Duration**: 2-3 days

### 5.1 Unit Tests

- Configuration parsing
- Environment variable substitution
- Group state management
- Tool listing/calling

### 5.2 Integration Tests

- End-to-end proxy workflow
- Multiple transport types
- Error handling scenarios

### 5.3 Documentation

- README.md with usage examples
- API documentation (cargo doc)
- Migration guide
- Architecture overview

### 5.4 Deliverables

- ✅ >80% test coverage
- ✅ Complete README
- ✅ Example configurations
- ✅ Architecture diagrams

---

## Phase 6: Build & Distribution

**Goal**: Production-ready builds and CI/CD

**Duration**: 1 day

### 6.1 Build Optimization

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### 6.2 GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check

  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
```

### 6.3 Release

- Publish to crates.io
- GitHub releases with binaries
- Optional: Homebrew, cargo-binstall

### 6.4 Deliverables

- ✅ CI/CD pipeline
- ✅ Cross-platform binaries
- ✅ Published to crates.io
- ✅ Release process documented

---

## Implementation Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| **Phase 1** | 2-3 days | Core proxy with stdio transport |
| **Phase 2** | 1-2 days | HTTP/SSE transport support |
| **Phase 3** | 2-3 days | OAuth authentication |
| **Phase 4** | 1 day | Migration command |
| **Phase 5** | 2-3 days | Tests + documentation |
| **Phase 6** | 1 day | Build system + CI/CD |
| **Total** | **9-13 days** | Production-ready v1.0.0 |

---

## Success Criteria

1. ✅ Feature parity with TypeScript version
2. ✅ All three transport types working (stdio, HTTP, SSE)
3. ✅ OAuth authentication functional
4. ✅ Environment variable substitution
5. ✅ Migration command working
6. ✅ >80% test coverage
7. ✅ Complete documentation
8. ✅ Published to crates.io
9. ✅ Cross-platform binaries available

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| rmcp API differences | Study examples, fallback to lower-level APIs |
| OAuth complexity | Start with basic flow, add refresh later |
| Transport compatibility | Thorough testing with real MCP servers |
| Build time | Optimize dependencies, use cargo-chef |
| Distribution | Multiple channels (crates.io, GitHub, Homebrew) |

---

## Next Steps

Ready to begin Phase 1 implementation. The plan provides a clear roadmap from basic functionality to production-ready release.
