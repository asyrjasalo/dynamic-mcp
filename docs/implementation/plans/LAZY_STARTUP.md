# Lazy Startup of Upstream MCP Servers

**Date**: January 12, 2026
**Status**: 📋 Planning Proposal (Not Implemented)

## Overview

**Problem**: Currently, dynamic-mcp connects to ALL configured upstream MCP servers immediately on startup, even if their tools/resources/prompts are never used during a session. This causes:

1. **Slow startup**: Spawning 10+ processes or establishing 10+ HTTP connections adds significant latency
2. **Unnecessary resource usage**: Memory/CPU allocated for servers that won't be used
3. **Connection storms**: If all servers are HTTP/SSE, this creates a burst of network traffic
4. **OAuth delays**: Servers requiring OAuth trigger browser flows immediately, blocking startup

**Goal**: Defer upstream server connection until their tools/resources/prompts are actually requested by the LLM (not just when schemas are loaded).

## Current Behavior (Eager Mode Only)

### Startup Flow (main.rs)

```
1. Load and validate config file
2. Spawn parallel connection tasks for ALL enabled servers
3. For each server:
   a. Create transport (stdio/HTTP/SSE) - 5s timeout
   b. OAuth authentication (if configured) - happens during transport creation
   c. MCP initialize handshake - 5s timeout
   d. tools/list request - 5s timeout (only if features.tools enabled)
   e. Store in GroupState::Connected or GroupState::Failed
4. Spawn periodic retry task: Failed servers retried every 30s
5. Spawn config reload handler: Monitor file, disconnect all, reconnect all
6. Start stdio listener
7. Shutdown handlers: SIGTERM (CTRL+C) and normal exit trigger disconnect_all()
```

### When OAuth Happens

**Transport::new()** (src/proxy/transport.rs, lines 540-552):

- Called during `connect()` in client.rs
- For HTTP/SSE servers with `oauth_client_id`:
  - Create OAuthClient
  - Call `authenticate()` (browser flow, cache tokens in `~/.dynamic-mcp/oauth-servers/`)
  - Insert Bearer token into HTTP headers
- **Timing**: Happens BEFORE `initialize` handshake
- **Result**: Token added to headers, then initialization proceeds

### When Tools Are Fetched

**connect()** (src/proxy/client.rs, lines 123-157):

- After `initialize` handshake completes successfully
- **Only** if `config.features().tools` is `true`:
  - Send `tools/list` request (5s timeout)
  - Parse response, extract tool info (name, description, inputSchema)
  - Store in `GroupState::Connected.tools` array
- **Resources and prompts**: NOT fetched during connection
  - Proxied fresh on each API call (no caching)
  - Resources: dynamic content (files, documents)
  - Prompts: fetched fresh for latest definitions

### What Happens on Config Reload

**Config reload handler** (src/main.rs, lines 188-252):

1. Detect file change via ConfigWatcher
2. Call `config::load_config()` to load new config
3. Call `client.disconnect_all()`:
   - Log: "Disconnecting {N} groups"
   - Drain all groups from `self.groups`
   - For each `GroupState::Connected`:
     - Log: "Closing transport for group: {name}"
     - Call `transport.close()` (terminates process or HTTP/SSE connection)
4. Reconnect all servers from new config:
   - Same parallel spawn logic as initial startup
   - Same flow: transport → OAuth → initialize → tools/list

### When Upstream Servers Disconnected and Shutdown

**Two shutdown paths** (src/main.rs):

**1. Graceful shutdown (CTRL+C signal)** (lines 266-272):

```
tokio::signal::ctrl_c().await.ok()
tracing::info!("Received shutdown signal, disconnecting all servers...")
let _ = client_lock.disconnect_all().await;
std::process::exit(0);
```

**2. Normal exit (stdin closed)** (lines 277-280):

```rust
let result = server.run_stdio().await;

{
    let mut client_lock = client.write().await;
    let _ = client_lock.disconnect_all().await;
}

result
```

**disconnect_all()** (src/proxy/client.rs, lines 599-608):

```rust
pub async fn disconnect_all(&mut self) -> Result<()> {
    tracing::info!("Disconnecting {} groups", self.groups.len());
    for (name, state) in self.groups.drain() {
        if let GroupState::Connected { mut transport, .. } = state {
            tracing::info!("Closing transport for group: {}", name);
            let _ = transport.close().await;
        }
    }
    Ok(())
}
```

### Summary of Current Eager Behavior

| Aspect                        | When                                          | How                                                                        |
| ----------------------------- | --------------------------------------------- | -------------------------------------------------------------------------- |
| **Servers started**           | On dynamic-mcp startup                        | Parallel spawn for ALL enabled servers                                     |
| **OAuth handled**             | During transport creation (before initialize) | OAuthClient::authenticate() called, token cached and inserted into headers |
| **Transport created**         | During connect()                              | 5s timeout, stdio process or HTTP/SSE connection                           |
| **Initialize handshake**      | After transport                               | 5s timeout, sends client info and protocol version                         |
| **Tools fetched**             | After initialize succeeds                     | 5s timeout, ONLY if features.tools enabled, cached in Connected state      |
| **Resources/prompts fetched** | NEVER during connection                       | Proxied fresh on each API call (no caching)                                |
| **Config reload**             | On file change                                | disconnect_all() → reload config → reconnect all                           |
| **Graceful shutdown**         | CTRL+C signal                                 | disconnect_all() → exit(0)                                                 |
| **Normal shutdown**           | Stdio closed                                  | disconnect_all() → return result                                           |
| **Failed servers**            | After initial connection fails                | Retried every 30s (max 3 retries)                                          |

### Group State (proxy/client.rs)

```rust
enum GroupState {
    Connected {
        name: String,
        description: String,
        tools: Vec<ToolInfo>,  // Cached after initial fetch
        transport: Transport,
        config: McpServerConfig,
    },
    Failed {
        name: String,
        description: String,
        error: String,
        retry_count: u32,  // Tracks retry attempts (0, 1, 2, ...)
        config: McpServerConfig,
    },
}
```

**Key Issue**: No "NotConnected" or "Pending" state - servers are either connected or failed. No lazy initialization, no on-demand connections.

## Proposed Design

### Two Approaches

**Approach A: Pure Lazy** (Fast startup, deferred discovery)

- No connections on startup
- Connect on first API call to group
- LLM discovers capabilities on-demand

**Approach B: Hybrid** (Discovery startup, disconnect lazy servers)

- Connect to ALL servers at startup for discovery
- Disconnect lazy servers immediately after fetching capabilities
- LLM sees full capability list upfront
- Reconnect lazy servers on first actual use

**Recommendation**: Implement both with configuration control (Approach B is default for backward compatibility with eager behavior, Approach A for maximum speed)

### New Group State

```rust
enum GroupState {
    /// Server is configured but not yet discovered/connected (pure lazy)
    Pending {
        name: String,
        description: String,
        config: McpServerConfig,
    },
    /// Server discovered (capabilities fetched) but disconnected (hybrid lazy)
    Discovered {
        name: String,
        description: String,
        tools: Vec<ToolInfo>,
        config: McpServerConfig,
        discovered_at: Instant,
    },
    /// Server successfully connected and ready
    Connected {
        name: String,
        description: String,
        tools: Vec<ToolInfo>,
        transport: Transport,
        config: McpServerConfig,
        last_activity: Instant,  // Track last API call for idle timeout
    },
    /// Connection attempt failed
    Failed {
        name: String,
        description: String,
        error: String,
        retry_count: u32,
        config: McpServerConfig,
    },
}
```

**Key Changes**:

- `Pending` for pure lazy startup (not yet discovered)
- `Discovered` for hybrid lazy startup (discovered but disconnected)

**Caching Strategy for Tools, Resources, Prompts**:

| API Type      | Cached in State? | Rationale                                                                         |
| ------------- | ---------------- | --------------------------------------------------------------------------------- |
| **Tools**     | ✅ Yes           | Tool schemas are static, cached in `Discovered` and `Connected` states            |
| **Resources** | ❌ No            | Resources are dynamic (files, docs), fetched fresh on each `resources/list` call  |
| **Prompts**   | ❌ No            | Prompts are static but typically small, fetched fresh on each `prompts/list` call |

**Design Decision**: Cache only tool schemas. Resources and prompts are fetched fresh when needed to ensure:

- Resources reflect current state (files may change)
- Prompts always use latest server-side definitions
- Simple state management (no need to cache complex nested structures)

### Startup Flow Changes

**Approach A: Pure Lazy**

```
1. Load config file
2. Create Pending state for ALL enabled servers (NO connections)
3. Start periodic retry task (for failed → pending → connected)
4. Start config reload handler
5. Start stdio listener
6. Defer all connections until first access
```

**Approach B: Hybrid (Recommended Default)**

```
1. Load config file
2. Connect to ALL servers (parallel)
3. For each server:
   a. Create transport - 5s timeout
   b. OAuth authentication (if configured)
   c. MCP initialize handshake - 5s timeout
   d. tools/list request - 5s timeout (cache tool schemas)
4. Disconnect lazy servers (marked as Discovered state with cached tools)
5. Keep non-lazy servers connected
6. Start periodic retry task
7. Start config reload handler
8. Start stdio listener
9. LLM sees tool capabilities upfront (from cached discovery)
10. Resources and prompts fetched fresh when LLM calls resources/list or prompts/list
11. Reconnect lazy servers on first API call
```

**Note**: Only tools are cached during discovery. Resources and prompts are fetched fresh on-demand to ensure:

- Resources reflect current state (files, documents may have changed)
- Prompts use latest server-side definitions
- Simpler state management

### On-Demand Connection Triggers

**Key Insight**: LLM needs to know what tools/resources/prompts are available before using them. Config file tells us which groups exist, but not their capabilities.

**Connections happen when** (for lazy servers):

1. **`get_dynamic_tools(group)` called** → Connect, fetch tools, **cache and return schemas**

   - This is the **PRIMARY trigger** (LLM asks "what tools does this group have?")
   - Tools are cached in `Discovered` or `Connected` state
   - After this call, `call_dynamic_tool` executes immediately (already connected)

2. **`call_dynamic_tool(group, tool)` called** → Edge case: connect if `get_dynamic_tools` was never called

   - Normally LLM calls `get_dynamic_tools` first to discover tools
   - This handles cases where LLM skips discovery

3. **`resources/list(group)` called** → Connect, list resources (**NOT cached**)

   - Resources are dynamic (files, documents), fetched fresh each time
   - No caching to ensure resources reflect current server state

4. **`prompts/list(group)` called** → Connect, list prompts (**NOT cached**)

   - Prompts are fetched fresh each time to use latest definitions
   - No caching for simplicity (prompts typically small)

**Important**: The initial `tools/list` (which returns `get_dynamic_tools` and `call_dynamic_tool`) does NOT trigger any upstream connections in pure lazy mode. In hybrid mode, tools are already discovered and cached from startup.

**Caching Summary**:

| API                 | Cached?                    | Trigger                             | State Management |
| ------------------- | -------------------------- | ----------------------------------- | ---------------- |
| `get_dynamic_tools` | ✅ Yes (tools)             | Connect, cache tools, return        |                  |
| `call_dynamic_tool` | ✅ Yes (uses cached tools) | Execute immediately if tools cached |                  |
| `resources/list`    | ❌ No                      | Connect, fetch fresh, return        |                  |
| `prompts/list`      | ❌ No                      | Connect, fetch fresh, return        |                  |

### Connection Laziness Levels

| Operation                        | Pure Lazy                                                              | Hybrid (Discovery on Startup)                                     |
| -------------------------------- | ---------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `tools/list` (proxy tools)       | Returns 2 tools + group list from config (no connections)              | Returns 2 tools + group list + tool capabilities (all discovered) |
| `get_dynamic_tools(group)`       | **TRIGGERS CONNECTION**: Connects, fetches tools, returns schemas      | Reconnects if lazy, returns **cached tools** from discovery       |
| `call_dynamic_tool(group, tool)` | Connects if `get_dynamic_tools` not called, executes tool              | Executes immediately if connected, reconnects if lazy/discovered  |
| `resources/list(group)`          | **TRIGGERS CONNECTION**: Connects, lists resources (**fetches fresh**) | Reconnects if lazy, lists resources (**fetches fresh**)           |
| `prompts/list(group)`            | **TRIGGERS CONNECTION**: Connects, lists prompts (**fetches fresh**)   | Reconnects if lazy, lists prompts (**fetches fresh**)             |

**Key Differences**:

- **Pure Lazy**: LLM learns capabilities on-demand, no discovery overhead
- **Hybrid**: LLM sees all **tool** capabilities upfront (cached), reconnects when actually using APIs
- **Resources/Prompts**: Never cached, fetched fresh in both modes (dynamic content)

### Connection Manager Logic

```rust
impl ModularMcpClient {
    /// Ensures server is connected, connecting on-demand if needed
    async fn ensure_connected(&mut self, group_name: &str) -> Result<()> {
        match self.groups.get(group_name) {
            Some(GroupState::Connected { .. }) => Ok(()), // Already connected
            Some(GroupState::Discovered { config, .. }) => {
                // Reconnect lazy server (was discovered at startup)
                self.connect(group_name.clone(), config.clone()).await
            }
            Some(GroupState::Pending { config, .. }) => {
                // Connect pure lazy server (never discovered)
                self.connect(group_name.clone(), config.clone()).await
            }
            Some(GroupState::Failed { config, retry_count, .. }) => {
                if *retry_count < MAX_RETRIES {
                    // Retry failed connection
                    self.connect(group_name.clone(), config.clone()).await
                } else {
                    // Return error (exceeded retries)
                    Err(anyhow::anyhow!("Server {} failed after {} retries", group_name, retry_count))
                }
            }
            None => Err(anyhow::anyhow!("Unknown group: {}", group_name)),
        }
    }

    /// Connect to server (handles Pending/Discovered/Failed → Connected transition)
    pub async fn connect(&mut self, group_name: String, config: McpServerConfig) -> Result<()> {
        let description = config.description().to_string();

        // ... existing connection logic ...
        // 1. Create transport (5s timeout)
        // 2. OAuth authentication (if configured)
        // 3. MCP initialize handshake (5s timeout)
        // 4. tools/list request (5s timeout)

        self.groups.insert(
            group_name.clone(),
            GroupState::Connected {
                name: group_name,
                description,
                tools,
                transport,
                config,
            },
        );

        Ok(())
    }

    /// Disconnect lazy server after discovery (hybrid mode)
    pub async fn disconnect_lazy(&mut self, group_name: &str) -> Result<()> {
        if let Some(GroupState::Connected { tools, config, .. }) = self.groups.remove(group_name) {
            // Close transport
            // ...

            // Move to Discovered state with cached tools
            // Note: Resources and prompts are NOT cached (fetched fresh on each call)
            self.groups.insert(
                group_name.to_string(),
                GroupState::Discovered {
                    name: group_name.to_string(),
                    description: config.description().to_string(),
                    tools,
                    config,
                    discovered_at: Instant::now(),
                },
            );
        }
        Ok(())
    }
}
```

````

### API Handler Changes

**get_dynamic_tools** (PRIMARY CONNECTION TRIGGER):
```rust
async fn handle_get_dynamic_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let group = extract_group(&request);

    // PURE LAZY: Ensure connected before returning tools
    // HYBRID: Reconnect if Discovered, return cached if Connected
    {
        let mut client_lock = self.client.write().await;
        if let Err(e) = client_lock.ensure_connected(&group).await {
            return error_response(e);
        }
        // Record activity for idle timeout tracking
        client_lock.record_activity(&group);
    }

    // Return tools from connected/discovered state
    let client_lock = self.client.read().await;
    let tools = client_lock.get_group_tools(&group).unwrap_or(vec![]);
    success_response(tools)
}
````

**call_dynamic_tool** (SECONDARY TRIGGER - edge case):

```rust
async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let group = extract_group(&request);
    let tool = extract_tool(&request);

    // Ensure connected (handles case where get_dynamic_tools wasn't called first)
    {
        let mut client_lock = self.client.write().await;
        if let Err(e) = client_lock.ensure_connected(&group).await {
            return error_response(e);
        }
        // Record activity for idle timeout tracking
        client_lock.record_activity(&group);
    }

    // Execute tool (server is now connected)
    let client_lock = self.client.read().await;
    client_lock.call_tool(&group, &tool, args).await
}
```

**resources/list** (NOT CACHED - fetched fresh):

```rust
async fn handle_resources_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let group = extract_group(&request);

    // Connect on-demand if needed
    {
        let mut client_lock = self.client.write().await;
        if let Err(e) = client_lock.ensure_connected(&group).await {
            return error_response(e);
        }
        // Record activity for idle timeout tracking
        client_lock.record_activity(&group);
    }

    // Proxy request to upstream (NOT cached - fetches fresh each time)
    // Resources are dynamic (files, documents), so we always fetch fresh
    let client_lock = self.client.read().await;
    client_lock.proxy_resources_list(&group).await
}
```

**prompts/list** (NOT CACHED - fetched fresh):

```rust
async fn handle_prompts_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let group = extract_group(&request);

    // Connect on-demand if needed
    {
        let mut client_lock = self.client.write().await;
        if let Err(e) = client_lock.ensure_connected(&group).await {
            return error_response(e);
        }
        // Record activity for idle timeout tracking
        client_lock.record_activity(&group);
    }

    // Proxy request to upstream (NOT cached - fetches fresh each time)
    // Prompts are fetched fresh to use latest server-side definitions
    let client_lock = self.client.read().await;
    client_lock.proxy_prompts_list(&group).await
}
```

**tools/list** (NO CONNECTION - just returns proxy tools):

```rust
async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let client = self.client.read().await;
    let groups = client.list_groups(); // Returns all groups (Pending, Discovered, Connected, Failed)
    let failed_groups = client.list_failed_groups();

    // Build response listing ALL groups from config
    // No upstream connections triggered here
}
```

## Configuration Schema Changes

Add optional `lazy_startup` field to `McpServerConfig` for per-server control:

```json
{
  "mcpServers": {
    "filesystem": {
      "description": "File operations",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "lazy_startup": false  // Eager: connect immediately and stay connected (no idle timeout)
    },
    "backup": {
      "description": "Backup operations (rarely used)",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-backup"],
      "lazy_startup": "pure",  // Pure lazy: connect on first API call
      "idle_timeout": 60  // Disconnect after 60 seconds of inactivity (optional, default: 300s)
    },
    "docs": {
      "description": "Documentation server (sometimes used)",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-docs"],
      "lazy_startup": "hybrid",  // Hybrid: discover at startup, disconnect, reconnect on use
      "idle_timeout": 300  // Disconnect after 5 minutes of inactivity (optional, default: 300s)
    }
  }
}
```

**Modes**:

- `"false"` or omitted: **Eager** - Connect immediately, stay connected (current behavior)
- `"pure"`: **Pure Lazy** - No connection until `get_dynamic_tools` called
- `"hybrid"` (recommended default): **Hybrid** - Discover at startup, disconnect, reconnect on use

**Idle timeout behavior** (applies to both pure lazy and hybrid modes):

- **Purpose**: Free memory for idle servers while allowing dynamic-mcp to manage their lifecycle
- **Applies to**: Pure lazy (`lazy_startup: "pure"`) and hybrid (`lazy_startup: "hybrid"`) modes
- **Behavior**: Disconnect after N seconds of inactivity since last API call
- **Default**: 5 minutes (300 seconds) if `idle_timeout` field not specified
- **Reconnection**: Dynamic-mcp will reconnect server when next API call is made (existing lifecycle management)
- **Eager mode** (`lazy_startup: false`): No idle timeout (stay connected forever)

**Key insight**: Dynamic-mcp already manages server lifetime:

- **Startup**: Starts all servers (eager) or discovers them (hybrid)
- **Config reload**: Disconnects all servers, reloads config, reconnects
- **Shutdown**: Gracefully or forcefully shuts down all servers when dynamic-mcp exits

**Idle timeout is additional lifecycle management**:

- Disconnects idle servers to free memory
- Dynamic-mcp handles reconnection on next API call (no user action needed)
- Maintains same lifecycle guarantees (startup, reload, shutdown still work)

### Recommended Idle Timeout Defaults

| Server Type                                    | Default Idle Timeout | Rationale                                                        |
| ---------------------------------------------- | -------------------- | ---------------------------------------------------------------- |
| **Frequently used** (filesystem, git, code)    | 10 minutes (600s)    | Keep connected, avoid reconnection overhead, high usage expected |
| **Occasionally used** (docs, search, analysis) | 5 minutes (300s)     | Balanced default, reasonable for mixed usage patterns            |
| **Rarely used** (backup, testing, staging)     | 1 minute (60s)       | Quick disconnect, maximize memory savings, reconnection is cheap |
| **Not specified**                              | 5 minutes (300s)     | Global default, applies to all lazy servers                      |

**CLI override**: `--idle-timeout SECONDS` to set global timeout for all lazy servers
**Per-server config**: `idle_timeout` field overrides global setting for specific servers

**Note**: Eager servers (`lazy_startup: false`) have no idle timeout regardless of config or CLI flags.

**Default behavior** (if field omitted): `"hybrid"` for backward compatibility with existing eager behavior

**CLI flags**:

- `--eager-startup`: Force all servers to eager mode (no idle timeout)
- `--pure-lazy`: Force all servers to pure lazy mode (idle timeout applies)
- `--hybrid-lazy`: Use hybrid mode (default)
- `--idle-timeout SECONDS`: Override default idle timeout (applies to all lazy and hybrid servers)

**Backward compatibility**: Default to `"hybrid"` to maintain LLM visibility of all capabilities while still saving memory for lazy servers.

## Performance Impact

### Benefits

| Metric                                         | Current (Eager)        | Pure Lazy                   | Hybrid (Recommended)                    |
| ---------------------------------------------- | ---------------------- | --------------------------- | --------------------------------------- |
| Startup time (10 servers)                      | ~15-20s                | ~0.5s                       | ~15-20s (discovery)                     |
| Initial memory usage                           | High (all processes)   | Low (none)                  | Low (lazy disconnected)                 |
| Network connections on startup                 | Burst of 10, keep all  | 0                           | Burst of 10, keep only non-lazy         |
| Time to `get_dynamic_tools(first_group)`       | 0s (already connected) | 5-15s (connect + init)      | 0s (cached from discovery)              |
| Time to `get_dynamic_tools(subsequent_groups)` | 0s (already connected) | 5-15s each (connect + init) | 0s (cached from discovery)              |
| Time to `call_dynamic_tool`                    | 0s (already connected) | 0s (already connected)      | 0s (already connected)                  |
| OAuth flows on startup                         | Yes (all servers)      | No (deferred)               | Yes (all servers, then disconnect lazy) |
| LLM visibility of all capabilities             | ✅ Yes (immediate)     | ❌ No (discover per-group)  | ✅ Yes (immediate)                      |

### Comparison Matrix

| Aspect                            | Eager      | Pure Lazy                        | Hybrid                                     |
| --------------------------------- | ---------- | -------------------------------- | ------------------------------------------ |
| Startup speed                     | ❌ Slow    | ✅ Fast (0.5s)                   | ⚠️ Slow (discovery)                        |
| LLM sees all capabilities upfront | ✅ Yes     | ❌ No                            | ✅ Yes                                     |
| First `get_dynamic_tools` latency | ✅ Instant | ⚠️ 5-15s                         | ✅ Instant                                 |
| First `call_dynamic_tool` latency | ✅ Instant | ✅ Instant (if discovered first) | ✅ Instant                                 |
| Memory usage after startup        | ❌ High    | ✅ Minimal                       | ✅ Low (lazy disconnected)                 |
| Double connection overhead        | ❌ No      | ❌ No                            | ⚠️ Yes (discovery + reconnect)             |
| OAuth flows on startup            | ✅ Yes     | ✅ No                            | ✅ Yes (all servers, then disconnect lazy) |
| Resource waste                    | ❌ High    | ✅ Minimal                       | ✅ Low                                     |

### Use Case Recommendations

| Scenario                                       | Best Approach                      | Rationale                                            |
| ---------------------------------------------- | ---------------------------------- | ---------------------------------------------------- |
| Critical servers (always used)                 | Eager (`lazy_startup: false`)      | No latency, always available                         |
| Rarely used servers (backup, testing)          | Pure Lazy (`lazy_startup: "pure"`) | Maximize startup speed, zero memory when unused      |
| Mixed usage (some critical, some optional)     | Hybrid (`lazy_startup: "hybrid"`)  | LLM sees all capabilities, saves memory on lazy ones |
| Many servers (10+) with unknown usage patterns | Pure Lazy                          | Avoid 15-20s startup overhead                        |
| Development/debugging                          | Eager or Hybrid                    | Immediate access to all servers                      |

**Default recommendation**: **Hybrid** (`lazy_startup: "hybrid"`)

- Maintains backward compatibility with LLM capability visibility
- Saves memory by disconnecting lazy servers after discovery
- No latency on first tool call (already discovered)
- Startup still slow due to discovery, but memory saved

## Implementation Plan

### Phase 1: Core Changes (Pure Lazy)

01. **Add Pending state** to `GroupState` enum
02. **Add Discovered state** to `GroupState` enum (for hybrid mode)
03. **Update initialization** to create `Pending` states (no connections in pure lazy)
04. **Add `ensure_connected()`** method to `ModularMcpClient`
05. **Add `disconnect_lazy()`** method to disconnect lazy servers after discovery
06. **Modify `get_dynamic_tools` handler** to call `ensure_connected()` before returning tools
07. **Modify `call_dynamic_tool` handler** to call `ensure_connected()` (edge case)
08. **Modify `resources/list` and `prompts/list` handlers** to call `ensure_connected()`
09. **Update `list_groups()`** to include `Pending`, `Discovered`, and `Connected` groups
10. **Add tests** for lazy connection behavior

### Phase 2: Hybrid Mode Implementation

1. **Implement startup discovery flow**: Connect to all servers, fetch tools
2. **Add tool caching**: Store tools in `Discovered` state (resources/prompts NOT cached)
3. **Implement lazy disconnect**: Call `disconnect_lazy()` for servers with `lazy_startup: "hybrid"`
4. **Modify `ensure_connected()`** to handle `Discovered` state (reconnect if needed)
5. **Add CLI flags**: `--eager-startup`, `--pure-lazy`, `--hybrid-lazy` (default)
6. **Add tests** for hybrid mode behavior

### Phase 3: Configuration Control

1. **Add `lazy_startup` field** to `McpServerConfig` (optional, enum: false/pure/hybrid)
2. **Update config schema** (`src/config/schema.rs`)
3. **Update JSON schema** (`config-schema.json`)
4. **Modify initialization** to respect lazy_startup flag
5. **Add config validation tests**

### Phase 4: UX Improvements

1. **Log lazy connections**: "Connecting to {group} on-demand..."
2. **Log discovery**: "Discovered {group}: {tool_count} tools"
3. **Log lazy disconnect**: "Disconnected {group} (lazy mode, will reconnect on use)"
4. **Log idle disconnect**: "Server {group} idle for {seconds}s, disconnecting..."
5. **Track connection stats**: Time saved by lazy startup
6. **Documentation updates**: README, ARCHITECTURE.md, examples/

### Phase 5: Edge Cases & Testing

1. **Concurrent connection requests**: Ensure only one connection attempt per group (use `Arc<Mutex>` or similar)
2. **Timeout handling**: Lazy connections respect same timeouts (5s transport, 5s init, 5s tools)
3. **Retry logic**: Pending → Failed → Retry, Discovered → Failed → Retry
4. **Config reload**: Preserve lazy/eager state across reloads
5. **OAuth flow**: Handle deferred OAuth in pure lazy mode, immediate in hybrid mode
6. **Idle timeout**: Implement monitoring task, handle concurrent disconnects, respect per-server timeouts
7. **Integration tests**: Full workflow with lazy connections
8. **Performance tests**: Benchmark startup time and memory usage

## Migration Path

### Backward Compatibility

- **Default behavior changes**: From eager → lazy (breaking change in behavior, not API)
- **Mitigation**: Provide `--eager-startup` CLI flag to restore old behavior
- **Documentation**: Clearly communicate the change and benefits

### Version Strategy

- **1.4.0**: Add lazy startup as default behavior
- **Deprecation**: Remove `--eager-startup` flag in 2.0.0 (if needed)

## Testing Strategy

### Unit Tests

1. `test_pending_state_created_on_init` - Verify Pending state created
2. `test_ensure_connected_transitions_pending_to_connected` - Lazy connection works
3. `test_ensure_connected_noop_if_already_connected` - Idempotency
4. `test_ensure_connected_returns_error_on_max_retries` - Retry exhaustion
5. `test_concurrent_ensure_connected_dedupes` - Only one connection attempt
6. `test_record_activity_updates_timestamp` - Activity tracking works
7. `test_idle_timeout_disconnects_lazy_server` - Disconnect after inactivity
8. `test_idle_timeout_preserves_eager_servers` - Eager servers never disconnect

### Integration Tests

1. `test_lazy_startup_no_connections_on_init` - Verify no initial connections
2. `test_get_dynamic_tools_triggers_connection` - First call connects
3. `test_call_tool_triggers_connection` - Tool execution connects
4. `test_lazy_startup_config_flag` - Per-server control
5. `test_eager_startup_cli_flag` - CLI flag works
6. `test_hybrid_mode_disconnects_on_idle` - Hybrid idle timeout works
7. `test_hybrid_mode_reconnects_after_idle_disconnect` - Reconnection after idle timeout
8. `test_idle_timeout_configurable_per_server` - Custom timeouts work

### Performance Tests

1. Benchmark startup time with 10 servers (eager vs lazy)
2. Measure memory usage differences
3. Profile first tool call latency
4. Measure memory savings from idle timeout disconnections

## Open Questions

**Resolved** (based on discussion):

1. ✅ **When should upstream servers connect?**

   - **Answer**: When `get_dynamic_tools(group)` is called (discovery phase)
   - **Rationale**: LLM needs to know what tools/resources/prompts exist before using them. Config tells us groups exist, but not their capabilities.

2. ✅ **Should `resources/list` and `prompts/list` be lazy?**

   - **Answer**: Yes, all APIs lazy (consistent behavior)
   - **Rationale**: Same principle as tools - connect on first API call to discover resources/prompts.

3. ✅ **How to handle `get_dynamic_tools` for Pending groups?**

   - **Answer**: Connect immediately, return real tools (non-blocking async)
   - **Rationale**: LLM calling `get_dynamic_tools` wants actual tools, not "try later". Async connection handles the delay gracefully.

4. ✅ **Should we support hybrid mode?**

   - **Answer**: Yes, as default behavior
   - **Rationale**: Maintains backward compatibility with LLM capability visibility while saving memory on lazy servers.

**Resolved** (based on discussion):

1. ✅ **When should upstream servers connect?**

   - **Answer**: Connect when `get_dynamic_tools(group)` is called
   - **Rationale**: Config file tells us which groups exist (no connection needed)
   - `get_dynamic_tools` is LLM's way of asking "what tools does this group have?"
   - LLM needs real tool schemas to make informed decisions
   - Subsequent `call_dynamic_tool` executes immediately (already connected)

2. ✅ **How to handle `get_dynamic_tools` for Pending groups?**

   - **Answer**: Connect immediately, return real tools (non-blocking async)
   - **Rationale**: LLM calling `get_dynamic_tools` wants actual tools, not "try later". Async connection handles delay gracefully.

3. ✅ **Should `resources/list` and `prompts/list` be lazy?**

   - **Answer**: Yes, all APIs lazy (consistent behavior)
   - **Rationale**: Same principle as tools - connect on first API call to discover resources/prompts
   - **Caching**: Tools cached, resources/prompts fetched fresh

4. ✅ **What about resources and prompts discovery?**

   - **Answer**: Fetch fresh on each API call (no caching)
   - **Rationale**: Resources are dynamic (files, documents), prompts fetched fresh for latest definitions
   - **Simplicity**: Only tools cached in state, resources/prompts proxied through active connection

5. ✅ **How long should hybrid servers stay connected after use?**

   - **Answer**: Configurable idle timeout, default 5 minutes
   - **Rationale**: Balance between memory savings and reconnection overhead
   - **Configuration**: `idle_timeout` field in seconds, per-server control
   - **Smart defaults**: 1 minute for rarely used, 10 minutes for frequently used
   - **Applies to**: Both pure lazy and hybrid modes (not just hybrid)

6. ✅ **Should idle timeout apply to pure lazy mode too?**

   - **Answer**: Yes, idle timeout applies to all lazy servers (pure and hybrid)
   - **Rationale**: Free memory for idle servers regardless of how they were initially connected
   - **Global default**: 5 minutes (300 seconds) for all lazy servers

**Resolved**: Idle timeout is additional lifecycle management, not replacing dynamic-mcp's existing startup/reload/shutdown management.

## Risks & Mitigations

| Risk                                                | Impact | Mode Affected  | Mitigation                                                                 |
| --------------------------------------------------- | ------ | -------------- | -------------------------------------------------------------------------- |
| First tool call latency surprises users             | Medium | Pure Lazy      | Document clearly, hybrid default provides better UX                        |
| Concurrent connection attempts                      | Low    | All modes      | Use `Arc<Mutex>` or similar to dedupe                                      |
| Config reload breaks lazy connections               | Medium | All modes      | Test thoroughly, preserve state                                            |
| Backward compatibility concerns                     | Medium | All modes      | Hybrid default, CLI flags for other modes                                  |
| Double connection overhead (discovery + reconnect)  | Medium | Hybrid         | Document trade-off, pure lazy option available                             |
| OAuth flows on startup for lazy servers             | Low    | Hybrid         | Document that OAuth still triggers at startup (discovery)                  |
| LLM cannot discover lazy server capabilities        | High   | Pure Lazy      | Use hybrid mode as default, document pure lazy behavior                    |
| State complexity (Pending + Discovered + Connected) | Medium | Hybrid         | Thorough testing, clear state transition documentation                     |
| **Idle timeout too aggressive**                     | Medium | All lazy modes | Configurable per-server, reasonable defaults (1-10min)                     |
| **Idle disconnect during active use**               | Low    | All lazy modes | Track activity on ALL API calls (not just connection), reasonable defaults |
| **Excessive reconnections**                         | Low    | All lazy modes | Reasonable default timeouts, longer for frequently used servers            |

## Key Decision Points

### 1. When to Connect?

**Decision**: Connect when `get_dynamic_tools(group)` is called

**Rationale**:

- Config file tells us which groups exist (no connection needed)
- `get_dynamic_tools` is the LLM's way of asking "what tools does this group have?"
- LLM needs real tool schemas to make informed decisions
- Subsequent `call_dynamic_tool` executes immediately (already connected)

**Alternative considered**: Connect on `call_dynamic_tool` only

- Rejected: LLM doesn't know what tools exist without calling `get_dynamic_tools` first
- Would require LLM to guess tool names and parameters (bad UX)

### 2. Pure Lazy vs Hybrid?

**Decision**: Support both, with hybrid as default

**Pure Lazy**:

- **Pros**: Maximum startup speed (0.5s), zero memory until used
- **Cons**: LLM can't see all capabilities upfront, must discover per-group
- **Use case**: Many servers (10+), unknown usage patterns, prioritize startup speed

**Hybrid** (recommended default):

- **Pros**: LLM sees all capabilities immediately (backward compatible), saves memory
- **Cons**: Startup still slow (15-20s for discovery), double connection overhead
- **Use case**: Most production use cases, mixed critical/lazy servers

### 3. How Does LLM Discover Groups?

**Current approach**: Config file provides group list (no connection needed)

```json
{
  "mcpServers": {
    "filesystem": { "description": "File operations", ... },
    "git": { "description": "Git operations", ... }
  }
}
```

**tools/list** returns 2 proxy tools with group list from config

- LLM knows groups exist from config
- LLM doesn't know tools/resources/prompts until connecting to each group

This is sufficient for most use cases - LLM discovers group-specific capabilities on-demand.

### 4. What About Resources and Prompts?

**Decision**: Lazy connection approach (connect on first API call), but different caching strategy

**Connection triggers**:

- `get_dynamic_tools(group)` → Connect, **fetch and cache** tools
- `resources/list(group)` → Connect, **fetch fresh** resources (no caching)
- `prompts/list(group)` → Connect, **fetch fresh** prompts (no caching)

**Caching strategy**:

- **Tools**: Cached in `Discovered` and `Connected` states (schemas are static)
- **Resources**: NOT cached (dynamic content, fetched fresh each time)
- **Prompts**: NOT cached (fetched fresh to use latest definitions)

**Rationale**:

- All MCP APIs use lazy connection (connect on first call)
- Tools cached for performance (schemas don't change frequently)
- Resources/prompts fetched fresh for accuracy (content may change dynamically)
- Simpler state management (no complex nested caching)

## Summary of Updates

This proposal has been updated to reflect design discussions:

**Original proposal** (pure lazy only):

- No connections on startup
- Connect only when `get_dynamic_tools` called
- LLM discovers capabilities on-demand

**Updated proposal** (pure lazy + hybrid):

- **Pure lazy mode**: No connections, discover on-demand
- **Hybrid mode** (recommended default): Discover all at startup, disconnect lazy servers
- **Connection trigger**: `get_dynamic_tools` (not `call_dynamic_tool`)
- **New state**: `Discovered` for hybrid mode (cached capabilities, disconnected)
- **Configuration**: `lazy_startup` enum with `false`, `"pure"`, `"hybrid"` options
- **Idle timeout**: Configurable per-server, tracks activity, disconnects lazy servers after inactivity

**Key insight**: Config file already tells us which groups exist. The question is when to discover their capabilities (tools/resources/prompts), not whether to groups exist.

**Design decision**: Hybrid mode as default balances:

- Backward compatibility (LLM sees all capabilities)
- Memory savings (lazy servers disconnected after discovery)
- No latency on first tool call (already discovered)
- Idle timeout prevents wasting resources on unused lazy servers

## Complete MCP API Behavior Summary

### Tools API

| Operation                        | Pure Lazy                                       | Hybrid                                                                  | Caching           |
| -------------------------------- | ----------------------------------------------- | ----------------------------------------------------------------------- | ----------------- |
| `tools/list`                     | Returns 2 proxy tools + group list from config  | Returns 2 proxy tools + group list + cached tool capabilities           | N/A (proxy only)  |
| `get_dynamic_tools(group)`       | Connects, fetches tools, **caches** and returns | Reconnects if lazy, returns **cached tools**                            | ✅ Yes (in state) |
| `call_dynamic_tool(group, tool)` | Connects if needed, executes tool               | Executes immediately (already connected), reconnects if lazy/discovered | Uses cached tools |

**Connection trigger**: `get_dynamic_tools(group)` (LLM asks "what tools does this group have?")

**Caching**: Tools are cached in `Discovered` and `Connected` states because schemas are static.

### Resources API

| Operation                         | Pure Lazy                                            | Hybrid                                                | Caching             |
| --------------------------------- | ---------------------------------------------------- | ----------------------------------------------------- | ------------------- |
| `resources/list(group)`           | Connects, fetches and returns resources              | Reconnects if lazy, fetches and returns resources     | ❌ No (fetch fresh) |
| `resources/read(group, uri)`      | Reuses existing connection, fetches resource content | Reuses existing connection, fetches resource content  | ❌ No (fetch fresh) |
| `resources/templates/list(group)` | Connects, fetches and returns URI templates          | Reconnects if lazy, fetches and returns URI templates | ❌ No (fetch fresh) |

**Connection trigger**: First resources API call to group

**Caching**: Resources are NOT cached. Fetched fresh each time because:

- Files, documents, and data sources are dynamic (may change)
- Must reflect current server state
- Simpler implementation

### Prompts API

| Operation                  | Pure Lazy                                           | Hybrid                                              | Caching             |
| -------------------------- | --------------------------------------------------- | --------------------------------------------------- | ------------------- |
| `prompts/list(group)`      | Connects, fetches and returns prompts               | Reconnects if lazy, fetches and returns prompts     | ❌ No (fetch fresh) |
| `prompts/get(group, name)` | Reuses existing connection, fetches prompt template | Reuses existing connection, fetches prompt template | ❌ No (fetch fresh) |

**Connection trigger**: First prompts API call to group

**Caching**: Prompts are NOT cached. Fetched fresh each time because:

- Prompt templates may change on server side
- Ensures latest definitions are used
- Simpler implementation

### State Transitions by API

```rust
// Pure Lazy: Initial state
groups.insert("filesystem", GroupState::Pending { ... });

// Pure Lazy: After get_dynamic_tools("filesystem")
groups.insert("filesystem", GroupState::Connected { tools: [...], transport: ... });

// Hybrid: After discovery (startup)
groups.insert("filesystem", GroupState::Discovered { tools: [...], ... });

// Hybrid: After get_dynamic_tools("filesystem") - already discovered, returns cached
// (no state change, returns cached tools from Discovered state)

// Hybrid: After call_dynamic_tool("filesystem", "read_file") - reconnects
groups.insert("filesystem", GroupState::Connected { tools: [...], transport: ... });

// All modes: resources/list or prompts/list always goes to Connected state
// (reuses connection if connected, reconnects if discovered/disconnected)
```

### Key Takeaways

1. **Tools**: Cached for performance, connection triggered by `get_dynamic_tools`
2. **Resources**: Never cached, fetched fresh each time, connection triggered by first resources API call
3. **Prompts**: Never cached, fetched fresh each time, connection triggered by first prompts API call
4. **Consistency**: All APIs use lazy connection (connect on first call), but only tools are cached
5. **Simplicity**: State management focused on tools only, resources/prompts are proxied through active connection

## Related Issues

- Discusses connection timeouts
- May benefit from lazy startup (fewer timeout errors)

## References

- Current initialization flow: `src/main.rs` lines 118-165
- Group state management: `src/proxy/client.rs` lines 9-24
- API handlers: `src/server.rs` lines 29-52
- Config schema: `src/config/schema.rs` lines 44-86
