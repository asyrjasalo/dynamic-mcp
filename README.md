# dynamic-mcp

MCP proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading tool schemas on-demand.

Instead of you exposing all MCP servers upfront (which can consume thousands
of tokens), dynamic-mcp exposes only two MCP tools initially.

It maintains full functionality for upstream MCP servers and supports stdio, HTTP (and SSE) transports, handles OAuth, and automatically retries failed connections.

## Quick Start

### Installation

**Option 1: Python package**

It is available in [PyPI](https://pypi.org/project/dmcp/).

Use `uvx` to install and run that package in your agent's MCP settings:

```json
{
  "mcpServers": {
    "dynamic-mcp": {
      "command": "uvx",
      "args": ["dmcp", "/path/to/your/dynamic-mcp.json"]
    }
  }
}
```

You can also set `DYNAMIC_MCP_CONFIG=` environment variable and omit the path.

**Option 2: Native binary**

Download the binary for your operating system from the
[releases page](https://github.com/asyrjasalo/dynamic-mcp/releases)
and put it in your `PATH`:

```json
{
  "mcpServers": {
    "dynamic-mcp": {
      "command": "dmcp"
    }
  }
}
```

Set `DYNAMIC_MCP_CONFIG=` environment variable and omit the `args` altogether.

**Option 3: Compile from source**

Install it from [crates.io](https://crates.io/crates/dynamic-mcp):

    cargo install dynamic-mcp

The binary will be available at `~/.cargo/bin/dmcp`.

### Migrate from an existing MCP config

If you have an existing MCP config without descriptions, use `migrate` command.

**Note**: There is no standard MCP json format. Not all formats are supported.

Migrate from an existing mcp config to dynamic-mcp format:

    uvx dmcp migrate mcp.json -o dynamic-mcp.json

The command will interactively prompt for descriptions for each server.

Example migration session:
```
🔄 Starting migration from standard MCP config to dynamic-mcp format
📖 Reading config from: mcp.json

✅ Found 2 MCP server(s) to migrate

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: filesystem
Type: stdio

Config details:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]

💬 Enter description for 'filesystem' (what this server does): File operations on /tmp directory

[... prompts for other servers ...]

✅ Migration complete!
📝 Output saved to: dynamic-mcp.json
```

**Note**: The migrate command respects `RUST_LOG` for controlling verbosity (same as server mode).

## Config File

### Descriptions

Create a `dynamic-mcp.json` file with `description` field for each server:

```json
{
  "mcpServers": {
    "filesystem": {
      "description": "Use when you need to read, write, or search files.",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

### Environment Variables

It supports `${VAR}` syntax for environment variable interpolation:

```json
{
  "mcpServers": {
    "example": {
      "description": "Example with env vars",
      "command": "node",
      "args": ["${HOME}/.local/bin/server.js"],
      "env": {
        "API_KEY": "${MY_API_KEY}"
      }
    }
  }
}
```

### Server Types

#### stdio (Default)

```json
{
  "description": "Server description for LLM",
  "command": "npx",
  "args": ["-y", "package-name"],
  "env": {
    "KEY": "value"
  }
}
```

#### http

```json
{
  "type": "http",
  "description": "HTTP server",
  "url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### sse

```json
{
  "type": "sse",
  "description": "SSE server",
  "url": "https://api.example.com/sse",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### OAuth Authentication (HTTP/SSE)

```json
{
  "type": "http",
  "description": "OAuth-protected MCP server",
  "url": "https://api.example.com/mcp",
  "oauth_client_id": "your-client-id",
  "oauth_scopes": ["read", "write"]
}
```

**OAuth Flow:**
- On first connect, browser opens for authorization
- Access token stored in `~/.dynamic-mcp/oauth-servers/<server-name>.json`
- Automatic token refresh before expiry (with RFC 6749 token rotation support)
- Token injected as `Authorization: Bearer <token>` header

## Troubleshooting

### Server Connection Issues

**Problem**: `❌ Failed to connect to <server>`

**Solutions**:
- **Automatic retry**: System retries up to 3 times with exponential backoff (2s, 4s, 8s)
- **Periodic retry**: Failed servers are retried every 30 seconds in the background
- **Stdio servers**: Verify command exists (`which <command>`)
- **HTTP/SSE servers**: Check server is running and URL is correct
- **Environment variables**: Ensure all `${VAR}` references are defined
- **OAuth servers**: Complete OAuth flow when prompted

**Logging**:

By default, errors and warnings are logged to terminal. For more verbose output:

```bash
# Debug mode (all logs including debug-level details)
RUST_LOG=debug uvx dmcp config.json

# Info mode (includes informational messages)
RUST_LOG=info uvx dmcp config.json

# Default mode (errors and warnings only, no RUST_LOG needed)
uvx dmcp config.json
```

### OAuth Authentication Problems

**Problem**: Browser doesn't open for OAuth

**Solutions**:
- Manually open the URL shown in console
- Check firewall allows localhost connections
- Verify `oauth_client_id` is correct for the server

**Problem**: Token refresh fails

**Solutions**:
- Delete cached token: `rm ~/.dynamic-mcp/oauth-servers/<server-name>.json`
- Re-authenticate on next connection

### Environment Variable Not Substituted

**Problem**: Config shows `${VAR}` instead of value

**Solutions**:
- Use `${VAR}` syntax, not `$VAR`
- Export variable: `export VAR=value`
- Variable names are case-sensitive
- Check for typos in variable name

### Configuration Errors

**Problem**: `Invalid JSON in config file`

**Solutions**:
- Validate JSON syntax (use `jq . config.json`)
- Check for trailing commas
- Ensure all required fields present (`description`; `type` required only for http/sse, optional for stdio)

**Problem**: `Failed to resolve config path`

**Solutions**:
- Use absolute path or path relative to working directory
- Check file exists and has read permissions
- Try: `ls -la <config-path>`

### Tool Call Failures

**Problem**: Tool call returns error

**Debugging**:
1. Test tool directly with upstream server
2. Check tool name and arguments match schema
3. Verify group name is correct
4. Enable debug logging to see JSON-RPC messages

### Performance Issues

**Problem**: Slow startup

**Solutions**:
- Parallel connections already enabled
- Check network latency for HTTP/SSE servers
- Some servers may be slow to initialize (normal)

**Problem**: High memory usage

**Solutions**:
- Tools are cached in memory (expected)
- Failed groups use minimal memory
- Large tool schemas contribute to memory usage

## Building from source

To build `dynamic-mcp` from source:

```bash
git clone https://github.com/asyrjasalo/dynamic-mcp.git
cd dynamic-mcp
cargo build --release
```

The binary will be available at `./target/release/dmcp`.

For more details on development setup, testing, and contributing, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Release History

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

## Acknowledgments

- TypeScript implementation: [modular-mcp](https://github.com/d-kimuson/modular-mcp)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP Ecosystem: [rust-mcp-stack](https://github.com/rust-mcp-stack)
