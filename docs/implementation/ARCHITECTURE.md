# Architecture

Dynamic-MCP is a proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading schemas on-demand. It provides full MCP API proxying for Tools, Resources, and Prompts.

## Project Structure

```
dynamic-mcp/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── server.rs            # MCP server (exposes 2 tools)
│   ├── watcher.rs           # Config file watcher for live reload
│   ├── config/              # Configuration management
│   │   ├── mod.rs           # Module exports
│   │   ├── schema.rs        # Config data structures
│   │   ├── loader.rs        # File loading & validation
│   │   └── env_sub.rs       # Environment variable substitution
│   ├── proxy/               # Upstream server management
│   │   ├── mod.rs           # Module exports
│   │   ├── types.rs         # Shared types (Resource, Prompt, Tool types)
│   │   ├── client.rs        # Group state management
│   │   └── transport.rs     # Transport creation (stdio, HTTP, SSE)
│   ├── auth/                # OAuth2 authentication
│   │   ├── mod.rs           # Module exports
│   │   ├── oauth_client.rs  # OAuth2 PKCE flow
│   │   └── store.rs         # Token storage
│   └── cli/                 # CLI commands
│       ├── mod.rs           # Module exports
│       ├── import.rs        # Legacy import (deprecated)
│       ├── import_enhanced.rs # Enhanced import workflow
│       ├── tool_detector.rs # Tool detection & path resolution
│       └── config_parser.rs # Multi-format config parsing
├── docs/                    # Documentation
├── examples/                # Example configurations
└── Cargo.toml              # Dependencies
```

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         LLM Client                          │
│                  (Claude, ChatGPT, etc.)                    │
└──────────────────────┬────────────────────────────────────┘
                       │ JSON-RPC 2.0 (stdio)
                       │ MCP Protocol
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Dynamic-MCP Server                       │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              MCP Server (src/server.rs)              │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │ Tools API (2 proxy tools):                     │  │  │
│  │  │ - get_dynamic_tools  (list tools in a group)   │  │  │
│  │  │ - call_dynamic_tool  (execute a tool)          │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │ Resources API (proxied):                       │  │  │
│  │  │ - resources/list     (discover resources)      │  │  │
│  │  │ - resources/read     (retrieve content)        │  │  │
│  │  │ - resources/templates/list (URI templates)     │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  │  ┌────────────────────────────────────────────────┐  │  │
│  │  │ Prompts API (proxied):                         │  │  │
│  │  │ - prompts/list       (discover prompts)        │  │  │
│  │  │ - prompts/get        (retrieve prompt)         │  │  │
│  │  └────────────────────────────────────────────────┘  │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │                                       │
│  ┌──────────────────▼───────────────────────────────────┐  │
│  │       ModularMcpClient (src/proxy/client.rs)        │  │
│  │  Manages group state:                                │  │
│  │  - GroupState::Connected (name, tools, transport)    │  │
│  │  - GroupState::Failed    (name, error)               │  │
│  │  Proxy methods:                                      │  │
│  │  - proxy_resources_list/read/templates_list()        │  │
│  │  - proxy_prompts_list/get()                          │  │
│  │  Feature flags enforcement per server                │  │
│  └──────────────────┬───────────────────────────────────┘  │
│                     │                                       │
│  ┌──────────────────▼───────────────────────────────────┐  │
│  │      Transport Layer (src/proxy/transport.rs)       │  │
│  │  Creates appropriate transport for each group:       │  │
│  │  - StdioTransport    (child process)                 │  │
│  │  - HttpTransport     (rmcp HTTP client)              │  │
│  │  - SseTransport      (rmcp SSE client with resumption)│ │
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
│ Tools/Res/   │ │ Tools/Res/   │ │ Tools/Res/   │
│ Prompts      │ │ Prompts      │ │ Prompts      │
└──────────────┘ └──────────────┘ └──────────────┘
```

## MCP API Support

Dynamic-MCP provides full proxying support for all three MCP APIs:

### Tools API
- **Purpose**: Execute actions and commands
- **Proxy Tools**: `get_dynamic_tools`, `call_dynamic_tool`
- **On-demand loading**: Tool schemas loaded per group, reducing initial context
- **Caching**: Tools cached after first fetch for performance

### Resources API
- **Purpose**: Access files, documents, and data sources
- **Endpoints**: `resources/list`, `resources/read`, `resources/templates/list`
- **Features**:
  - Cursor-based pagination for large resource lists
  - Text and binary content support
  - Resource annotations (audience, priority, lastModified)
  - URI templates (RFC 6570) for dynamic resource URIs
  - Resource size field for context window estimation
- **Timeout**: 10s per operation

### Prompts API
- **Purpose**: Discover and retrieve prompt templates
- **Endpoints**: `prompts/list`, `prompts/get`
- **Features**:
  - Prompt metadata (name, description, arguments)
  - Multi-modal content (text, image, audio, embedded resources)
  - Argument substitution in prompt templates
  - Cursor-based pagination for prompt lists
- **Timeout**: 10s per operation

### Per-Server Feature Flags
- **Configuration**: Optional `features` field per server
- **Flags**: `tools`, `resources`, `prompts` (all default to `true`)
- **Opt-out design**: All APIs enabled unless explicitly disabled
- **Runtime enforcement**: Clear error messages when disabled features are accessed
- **Use case**: Disable unsupported APIs for specific servers

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

**Purpose**: Expose two-tool API and proxy full MCP protocol to LLM clients

**Tools API (2 proxy tools)**:

1. **`get_dynamic_tools`**
   - Input: `{ "group": "group_name" }`
   - Output: JSON array of tools with schemas
   - Purpose: On-demand schema loading (reduces initial context)

2. **`call_dynamic_tool`**
   - Input: `{ "group": "group_name", "name": "tool_name", "args": {...} }`
   - Output: Tool execution result
   - Purpose: Proxy calls to upstream servers

**Resources API (proxied)**:
- `resources/list`: Discover available resources from upstream servers
- `resources/read`: Retrieve resource content (text/binary)
- `resources/templates/list`: List resource URI templates (RFC 6570)

**Prompts API (proxied)**:
- `prompts/list`: Discover available prompts with metadata
- `prompts/get`: Retrieve prompt template with argument substitution

**JSON-RPC Methods**:
- `initialize`: Handshake with client (advertises tools/resources/prompts capabilities)
- `initialized`: Notification sent after initialize (MCP spec compliance)
- `tools/list`: Return the two proxy tools
- `tools/call`: Execute get_dynamic_tools or call_dynamic_tool
- `resources/list`, `resources/read`, `resources/templates/list`: Proxy to upstream
- `prompts/list`, `prompts/get`: Proxy to upstream

**Feature Flags**:
- Per-server `features` config field (tools, resources, prompts)
- Opt-out design: All features enabled by default
- Runtime enforcement with clear error messages

### 6. CLI & Import (`src/cli/`)

**Purpose**: Command-line interface and config import

**Commands**:
- `dynamic-mcp <config.json>`: Start server with config
- `dynamic-mcp import <tool-name>`: Import MCP configs from AI coding tools

**Import Process**:
```
Standard MCP Config          Dynamic-MCP Config
─────────────────           ──────────────────
{                           {
  "mcpServers": {             "mcpServers": {
    "server": {                 "server": {
      "command": "...",           "description": "...",  ← Added
      "args": [...]               "command": "...",
    }                             "args": [...]
  }                               }
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
    │  [get_dynamic_tools,      │                             │
    │   call_dynamic_tool]      │                             │
    │<──────────────────────────┤                             │
    │                           │                             │
    │  call: get_dynamic_tools  │                             │
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
    │  call: call_dynamic_tool  │                             │
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

### Resources API Flow

```
LLM Client                  Dynamic-MCP                 Upstream Server
    │                           │                             │
    │  resources/list           │                             │
    │  {group: "docs"}          │                             │
    ├──────────────────────────>│                             │
    │                           │  resources/list             │
    │                           ├────────────────────────────>│
    │                           │                             │
    │                           │  {resources: [...]}         │
    │                           │<────────────────────────────┤
    │  {resources: [...]}       │                             │
    │<──────────────────────────┤                             │
    │                           │                             │
    │  resources/read           │                             │
    │  {group: "docs",          │                             │
    │   uri: "file://..."}      │                             │
    ├──────────────────────────>│                             │
    │                           │  resources/read             │
    │                           │  {uri: "file://..."}        │
    │                           ├────────────────────────────>│
    │                           │                             │
    │                           │  {contents: [...]}          │
    │                           │<────────────────────────────┤
    │  {contents: [...]}        │                             │
    │<──────────────────────────┤                             │
```

### Prompts API Flow

```
LLM Client                  Dynamic-MCP                 Upstream Server
    │                           │                             │
    │  prompts/list             │                             │
    │  {group: "templates"}     │                             │
    ├──────────────────────────>│                             │
    │                           │  prompts/list               │
    │                           ├────────────────────────────>│
    │                           │                             │
    │                           │  {prompts: [...]}           │
    │                           │<────────────────────────────┤
    │  {prompts: [...]}         │                             │
    │<──────────────────────────┤                             │
    │                           │                             │
    │  prompts/get              │                             │
    │  {group: "templates",     │                             │
    │   name: "code-review",    │                             │
    │   arguments: {...}}       │                             │
    ├──────────────────────────>│                             │
    │                           │  prompts/get                │
    │                           │  {name: "code-review",      │
    │                           │   arguments: {...}}         │
    │                           ├────────────────────────────>│
    │                           │                             │
    │                           │  {messages: [...]}          │
    │                           │<────────────────────────────┤
    │  {messages: [...]}        │                             │
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
        env: Option<HashMap<String, String>>,
        features: Option<Features>  // Optional per-server feature flags
    },
    Http {
        description: String,
        url: String,
        headers: Option<HashMap<String, String>>,
        oauth_client_id: Option<String>,
        oauth_scopes: Option<Vec<String>>,
        features: Option<Features>  // Optional per-server feature flags
    },
    Sse {
        description: String,
        url: String,
        headers: Option<HashMap<String, String>>,
        oauth_client_id: Option<String>,
        oauth_scopes: Option<Vec<String>>,
        features: Option<Features>  // Optional per-server feature flags
    }
}

struct Features {
    tools: bool,      // Default: true
    resources: bool,  // Default: true
    prompts: bool,    // Default: true
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
- Resources/Prompts proxied without schema overhead
- Example: 2 tools initially, +20 tools only when needed

### Memory Usage

- Tools cached in memory after initial fetch
- No re-fetching on repeated use
- Failed groups tracked with minimal memory
- Resources/Prompts: Streamed, not cached (fresh data)

### Latency

- Initial connection: Parallel connection to all upstreams
- Tool discovery: Cached, no upstream call
- Tool execution: Single upstream call (no additional hop overhead)
- Resources/Prompts: Single upstream call with 10s timeout

### Feature Flags

- Disable unused APIs per server (tools, resources, prompts)
- Reduces connection overhead for servers that don't support all APIs
- Runtime enforcement prevents unnecessary API calls

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
- Resources API type definitions and serialization
- Prompts API type definitions and serialization
- Feature flags configuration

### Integration Tests
- End-to-end CLI workflows
- Import command with 10 AI coding tools
- Config schema validation
- Resources API proxying (list, read, templates/list)
- Prompts API proxying (list, get)
- Environment variable normalization across tool formats
- Feature selection during import

### Manual Testing
- Real MCP server connections
- OAuth flow with actual providers
- Multi-transport scenarios (stdio, HTTP, SSE)
- SSE stream resumption with Last-Event-ID

**See [TESTING.md](TESTING.md) for detailed test counts and coverage.**

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
