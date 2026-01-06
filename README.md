# dynamic-mcp

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A Rust implementation of dynamic-mcp - an MCP proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading schemas on-demand.

## 📖 Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - System design, data flows, and component details
- **[Migration Guide](docs/MIGRATION.md)** - Migrate from standard MCP to dynamic-mcp
- **[Contributing](CONTRIBUTING.md)** - Development setup and contribution guidelines
- **[Development Status](docs/implementation/DEVELOPMENT.md)** - Project status, roadmap, and metrics
- **[Agent Guidelines](AGENTS.md)** - For AI coding agents: how to maintain documentation
- **[TypeScript Reference](https://github.com/d-kimuson/dynamic-mcp)** - Original implementation

## 🏗️ Architecture

```
dynamic-mcp/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── server.rs            # MCP server (exposes 2 tools)
│   ├── config/              # Configuration management
│   │   ├── schema.rs        # Config data structures
│   │   ├── loader.rs        # File loading & validation
│   │   └── env_sub.rs       # Environment variable substitution
│   ├── proxy/               # Upstream server management
│   │   ├── types.rs         # Shared types
│   │   ├── client.rs        # Group state management
│   │   └── transport.rs     # Transport creation
│   └── cli/                 # CLI commands
│       └── migrate.rs       # Config migration
├── docs/                    # Documentation
├── examples/                # Example configurations
└── Cargo.toml              # Dependencies
```

## 🚀 Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/yourusername/dynamic-mcp.git
cd dynamic-mcp
cargo build --release
```

For development setup and testing, see [CONTRIBUTING.md](CONTRIBUTING.md).

### Migration from Standard MCP Config

If you have an existing MCP config without descriptions, use the migration command:

```bash
# Migrate standard config to dynamic-mcp format
dynamic-mcp migrate mcp-config.json -o dynamic-mcp.json

# The tool will interactively prompt for descriptions for each server
```

**Example migration session:**
```
🔄 Starting migration from standard MCP config to dynamic-mcp format
📖 Reading config from: mcp-config.json

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

### Usage

```bash
# Run with config file
./target/release/dynamic-mcp examples/config.example.json

# Or use environment variable
export GATEWAY_MCP_CONFIG=examples/config.example.json
./target/release/dynamic-mcp
```

**Note**: Command line argument takes precedence over environment variable.

### Configuration

Create a `dynamic-mcp.json` file:

```json
{
  "mcpServers": {
    "filesystem": {
      "type": "stdio",
      "description": "Use when you need to read, write, or search files.",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

### Environment Variables

Supports `${VAR}` syntax for environment variable interpolation:

```json
{
  "mcpServers": {
    "example": {
      "type": "stdio",
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

## 🔧 Configuration Schema

### Server Types

#### stdio (Default)
```json
{
  "type": "stdio",
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

## ⚡ Performance

Run benchmarks to measure performance characteristics:

```bash
cargo bench --bench performance
```

**Key metrics:**
- Environment variable substitution: <1 µs per operation
- JSON config parsing: ~6 µs for typical configs
- Tool list caching: O(1) lookup performance
- Parallel connections: ~12ms for 10 servers

See `benches/performance.rs` for benchmark implementation.



## 🔧 Troubleshooting

### Server Connection Issues

**Problem**: `❌ Failed to connect to <server>`

**Solutions**:
- **Automatic retry**: System retries up to 3 times with exponential backoff (2s, 4s, 8s)
- **Periodic retry**: Failed servers are retried every 30 seconds in the background
- **Stdio servers**: Verify command exists (`which <command>`)
- **HTTP/SSE servers**: Check server is running and URL is correct
- **Environment variables**: Ensure all `${VAR}` references are defined
- **OAuth servers**: Complete OAuth flow when prompted

**Debug mode**:
```bash
RUST_LOG=debug dynamic-mcp config.json
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
- Ensure all required fields present (`type`, `description`)

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

## 🤝 Contributing

Contributions are welcome! Please see:
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development setup, testing, and contribution guidelines
- **[docs/implementation/DEVELOPMENT.md](docs/implementation/DEVELOPMENT.md)** - Project status, roadmap, and metrics

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

- Original TypeScript implementation: [d-kimuson/dynamic-mcp](https://github.com/d-kimuson/dynamic-mcp)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP Ecosystem: [rust-mcp-stack](https://github.com/rust-mcp-stack)
