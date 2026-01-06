# Architecture

Dynamic-MCP is a proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading schemas on-demand.

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         LLM Client                          │
│                  (Claude, ChatGPT, etc.)                    │
└──────────────────────┬──────────────────────────────────────┘
                       │ JSON-RPC 2.0
                       │ (stdio)
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Dynamic-MCP Server                       │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              MCP Server (src/server.rs)              │  │
│  │  Exposes 2 tools:                                    │  │
│  │  - get-modular-tools  (list tools in a group)        │  │
│  │  - call-modular-tool  (execute a tool)               │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │                                       │
│  ┌──────────────────▼───────────────────────────────────┐  │
│  │       ModularMcpClient (src/proxy/client.rs)        │  │
│  │  Manages group state:                                │  │
│  │  - GroupState::Connected (name, tools, transport)    │  │
│  │  - GroupState::Failed    (name, error)               │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │                                       │
│  ┌──────────────────▼───────────────────────────────────┐  │
│  │      Transport Layer (src/proxy/transport.rs)       │  │
│  │  Creates appropriate transport for each group:       │  │
│  │  - StdioTransport    (child process)                 │  │
│  │  - HttpTransport     (rmcp HTTP client)              │  │
│  │  - SseTransport      (rmcp SSE client)               │  │
│  └──────────────────┬───────────────────────────────────┘  │
└────────────────────┬┼┬────────────────────────────────────┘
                     │││
        ┌────────────┘│└────────────┐
        │             │             │
        ▼             ▼             ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│   Upstream   │ │   Upstream   │ │   Upstream   │
│ MCP Server 1 │ │ MCP Server 2 │ │ MCP Server 3 │
│   (stdio)    │ │    (HTTP)    │ │    (SSE)     │
└──────────────┘ └──────────────┘ └──────────────┘
```

## Key Components

### 1. Configuration System (`src/config/`)

**Purpose**: Load and validate server configurations

**Components**:
- `schema.rs`: Defines config structures (`ServerConfig`, `McpServerConfig`)
- `loader.rs`: Loads JSON config from disk
- `env_sub.rs`: Substitutes `${VAR}` environment variables

**Flow**:
```
config.json → load_config() → substitute_env_vars() → ServerConfig
```

### 2. Proxy Client (`src/proxy/`)

**Purpose**: Manage connections to upstream MCP servers

**State Machine**:
```
                    ┌─────────────┐
                    │   Initial   │
                    └──────┬──────┘
                           │ connect()
                ┌──────────┴───────────┐
                │                      │
                ▼                      ▼
         ┌─────────────┐      ┌──────────────┐
         │  Connected  │      │    Failed    │
         │             │      │              │
         │ - transport │      │ - error msg  │
         │ - tools[]   │      │              │
         └─────────────┘      └──────────────┘
```

**Group State**:
- `Connected`: Active connection, tools cached, transport ready
- `Failed`: Connection attempt failed, error recorded, group unavailable

### 3. Transport Layer (`src/proxy/transport.rs`)

**Purpose**: Abstract communication with upstream servers

**Transport Types**:

| Type    | Use Case              | Implementation                    |
|---------|-----------------------|-----------------------------------|
| stdio   | Local processes       | `StdioTransport` (tokio process)  |
| HTTP    | Remote HTTP servers   | rmcp HTTP client                  |
| SSE     | Server-sent events    | rmcp SSE client                   |

**Protocol**: All transports use JSON-RPC 2.0 over their respective channels

### 4. OAuth Authentication (`src/auth/`)

**Purpose**: Handle OAuth2 authentication for remote servers

**Flow**:
```
1. Check ~/.dynamic-mcp/oauth-servers/{server}.json for cached token
2. If missing/expired:
   a. Discover OAuth endpoints via /.well-known/oauth-authorization-server
   b. Generate PKCE challenge
   c. Open browser for user authorization
   d. Start local callback server (http://localhost:random/oauth/callback)
   e. Exchange authorization code for access token
   f. Save token to disk
3. Inject token as Authorization: Bearer {token} header
```

**Token Refresh**:
- Tokens checked before each connection
- Auto-refresh if expiring within 5 minutes
- Refresh token used if available

### 5. MCP Server (`src/server.rs`)

**Purpose**: Expose two-tool API to LLM clients

**Tools**:

1. **`get-modular-tools`**
   - Input: `{ "group": "group_name" }`
   - Output: JSON array of tools with schemas
   - Purpose: On-demand schema loading (reduces initial context)

2. **`call-modular-tool`**
   - Input: `{ "group": "group_name", "name": "tool_name", "args": {...} }`
   - Output: Tool execution result
   - Purpose: Proxy calls to upstream servers

**JSON-RPC Methods**:
- `initialize`: Handshake with client
- `tools/list`: Return the two proxy tools
- `tools/call`: Execute get-modular-tools or call-modular-tool

### 6. CLI & Migration (`src/cli/`)

**Purpose**: Command-line interface and config migration

**Commands**:
- `dynamic-mcp <config.json>`: Start server with config
- `dynamic-mcp migrate <input.json> -o <output.json>`: Migrate standard MCP config

**Migration Process**:
```
Standard MCP Config          Dynamic-MCP Config
─────────────────           ──────────────────
{                           {
  "mcpServers": {             "mcpServers": {
    "server": {                 "server": {
      "command": "...",           "type": "stdio",
      "args": [...]               "description": "...",  ← Added
    }                             "command": "...",
  }                               "args": [...]
}                               }
                              }
                            }
```

## Data Flow

### Initialization Flow

```
1. main.rs reads config file
2. load_config() parses JSON and substitutes env vars
3. ModularMcpClient created
4. For each configured server:
   a. Create transport (stdio/HTTP/SSE)
   b. If OAuth required, authenticate
   c. Send initialize JSON-RPC request
   d. Send tools/list request
   e. Cache tools in GroupState::Connected
   f. On error, record GroupState::Failed
5. ModularMcpServer wraps client
6. Server starts listening on stdio
```

### Tool Discovery Flow

```
LLM Client                  Dynamic-MCP                 Upstream Server
    │                           │                             │
    │  tools/list               │                             │
    ├──────────────────────────>│                             │
    │                           │                             │
    │  [get-modular-tools,      │                             │
    │   call-modular-tool]      │                             │
    │<──────────────────────────┤                             │
    │                           │                             │
    │  call: get-modular-tools  │                             │
    │  args: {group: "fs"}      │                             │
    ├──────────────────────────>│                             │
    │                           │  (from cached tools)        │
    │  [read_file, write_file,  │                             │
    │   list_directory, ...]    │                             │
    │<──────────────────────────┤                             │
```

### Tool Execution Flow

```
LLM Client                  Dynamic-MCP                 Upstream Server
    │                           │                             │
    │  call: call-modular-tool  │                             │
    │  args: {                  │                             │
    │    group: "fs",           │                             │
    │    name: "read_file",     │                             │
    │    args: {path: "..."}    │                             │
    │  }                        │                             │
    ├──────────────────────────>│                             │
    │                           │  tools/call                 │
    │                           │  {name: "read_file",        │
    │                           │   arguments: {path: "..."}} │
    │                           ├────────────────────────────>│
    │                           │                             │
    │                           │  {result: "file contents"}  │
    │                           │<────────────────────────────┤
    │  {result: "file contents"}│                             │
    │<──────────────────────────┤                             │
```

## Error Handling

### Connection Failures

- Failed connections recorded in `GroupState::Failed` with retry count
- **Automatic retry with exponential backoff**:
  - Retry attempts: Up to 3 times per server
  - Backoff strategy: 2^n seconds (2s, 4s, 8s)
  - Periodic retry: Every 30 seconds for failed groups
- Failed groups included in tool descriptions with error info
- LLM aware of unavailable groups, can inform user
- Server continues operating with available groups

### Runtime Errors

- Transport errors: Logged, error returned to LLM
- Tool call errors: Upstream error message forwarded to LLM
- Parse errors: JSON-RPC error response with code -32700

### OAuth Failures

- Token refresh failures trigger re-authentication
- Browser fails to open: URL printed to console
- Callback timeout: Clear error message

## Configuration Schema

### ServerConfig

```rust
struct ServerConfig {
    mcp_servers: HashMap<String, McpServerConfig>
}
```

### McpServerConfig (Tagged Enum)

```rust
enum McpServerConfig {
    Stdio {
        description: String,
        command: String,
        args: Option<Vec<String>>,
        env: Option<HashMap<String, String>>
    },
    Http {
        description: String,
        url: String,
        headers: Option<HashMap<String, String>>,
        oauth_client_id: Option<String>,
        oauth_scopes: Option<Vec<String>>
    },
    Sse {
        description: String,
        url: String,
        headers: Option<HashMap<String, String>>,
        oauth_client_id: Option<String>,
        oauth_scopes: Option<Vec<String>>
    }
}
```

## Performance Considerations

### Context Efficiency

**Without dynamic-mcp**:
- LLM receives ALL tools from ALL servers upfront
- Large context window consumed by tool schemas
- Example: 10 servers × 20 tools = 200 tool schemas in initial context

**With dynamic-mcp**:
- LLM receives only 2 proxy tools initially
- Tool schemas loaded on-demand per group
- Example: 2 tools initially, +20 tools only when needed

### Memory Usage

- Tools cached in memory after initial fetch
- No re-fetching on repeated use
- Failed groups tracked with minimal memory

### Latency

- Initial connection: Parallel connection to all upstreams
- Tool discovery: Cached, no upstream call
- Tool execution: Single upstream call (no additional hop overhead)

## Security

### OAuth Token Storage

- Tokens stored in `~/.dynamic-mcp/oauth-servers/`
- File permissions: User-only read/write
- JSON format for debuggability
- Automatic cleanup on re-authentication

### Environment Variables

- Secrets in env vars, not config files
- `${VAR}` substitution at load time
- Undefined vars preserved (not replaced with empty string)

### Process Isolation

- stdio servers run in separate process groups (Unix)
- Clean termination on shutdown (SIGTERM to process group)

## Testing Strategy

### Unit Tests
- Config parsing and validation
- Environment variable substitution
- OAuth token management
- Server request handling

### Integration Tests
- End-to-end CLI workflows
- Migration command
- Config schema validation

### Manual Testing
- Real MCP server connections
- OAuth flow with actual providers
- Multi-transport scenarios

## Extension Points

### Adding New Transports

1. Implement transport in `transport.rs`
2. Add variant to `McpServerConfig` enum
3. Add case in `Transport::new()`
4. Update config schema documentation

### Adding CLI Commands

1. Add variant to `Commands` enum in `cli/mod.rs`
2. Implement handler in `cli/` submodule
3. Update help text
4. Add integration test

### Custom Authentication

1. Implement auth module similar to `auth/oauth_client.rs`
2. Integrate in `Transport::new()` 
3. Add config fields to `McpServerConfig`
