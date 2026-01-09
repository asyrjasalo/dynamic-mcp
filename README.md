# dynamic-mcp

MCP proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading tool schemas on-demand.

Instead of requiring you to expose all MCP servers upfront (which can consume thousands of tokens), dynamic-mcp exposes only two MCP tools initially.

It supports tools, resources, and prompts from upstream MCP servers with stdio, HTTP, and SSE transports, handles OAuth, and automatically retries failed connections.

## Quick Start

### Installation

**Option 1: Python package**

Use `uvx` to run the [PyPI package](https://pypi.org/project/dmcp/) in your agent's MCP settings:

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

You can set the `DYNAMIC_MCP_CONFIG` environment variable and omit the config path.

**Option 2: Native binary**

Download a [release](https://github.com/asyrjasalo/dynamic-mcp/releases) for
your operating system and put `dmcp` in your `PATH`:

```json
{
  "mcpServers": {
    "dynamic-mcp": {
      "command": "dmcp"
    }
  }
}
```

Set the `DYNAMIC_MCP_CONFIG` environment variable and omit the `args` altogether.

**Option 3: Compile from source**

Install from [crates.io](https://crates.io/crates/dynamic-mcp):

    cargo install dynamic-mcp

The binary is then available at `~/.cargo/bin/dmcp` (`$CARGO_HOME/bin/dmcp`).

### Import from AI Coding Tools

Dynamic-mcp can automatically import MCP server configurations from popular AI coding tools.

**Supported Tools** (`<tool-name>`):
- Cursor (`cursor`)
- OpenCode (`opencode`)
- Claude Desktop (`claude-desktop`)
- Claude Code CLI (`claude`)
- Visual Studio Code (`vscode`)
- Cline (`cline`)
- KiloCode (`kilocode`)
- Codex CLI (`codex`)
- Gemini CLI (`gemini`)
- Google Antigravity (`antigravity`)

#### Quick Start

**Import from project config** (run in project directory):
```bash
dmcp import <tool-name>
```

**Import from global/user config**:
```bash
dmcp import --global <tool-name>
```

**Force overwrite** (skip confirmation prompt):
```bash
dmcp import <tool-name> --force
```

The command will:
1. Detect your tool's config location
2. Parse the existing MCP servers
3. Interactively prompt for descriptions
4. Interactively prompt for feature selection (tools, resources, prompts)
5. Normalize environment variable formats
6. Generate `dynamic-mcp.json`

#### Example Import

```bash
$ dmcp import cursor

🔄 Starting import from cursor to dynamic-mcp format
📖 Reading config from: .cursor/mcp.json

✅ Found 2 MCP server(s) to import

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: filesystem
Type: stdio

Config details:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]

💬 Enter description for 'filesystem' (what this server does): File operations on /tmp directory

🔧 Keep all features (tools, resources, prompts) for 'filesystem'? [Y/n]:
(press Enter to keep all features, or 'n' to customize)

[... prompts for other servers ...]

✅ Import complete!
📝 Output saved to: dynamic-mcp.json
```

**Feature Selection**: During import, you can customize which MCP features are enabled per server:
- Press Enter (or Y) to keep all features (tools, resources, prompts)
- Type 'n' to selectively enable/disable individual features
- This allows fine-grained control without manually editing the config file

Example of custom feature selection:
```bash
🔧 Keep all features (tools, resources, prompts) for 'server'? [Y/n]: n

  Select features to enable (press Enter to accept default):
  Enable tools? [Y/n]: y
  Enable resources? [Y/n]: n
  Enable prompts? [Y/n]: n
```

#### Tool-Specific Notes

- **Cursor**: Supports both `.cursor/mcp.json` (project) and `~/.cursor/mcp.json` (global)
- **Claude Desktop**: Global config only, location varies by OS:
  - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
  - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
  - Linux: `~/.config/Claude/claude_desktop_config.json`
- **Claude Code CLI**: Supports both `.mcp.json` (project root) and `~/.claude.json` (user/global)
- **Gemini CLI**: Supports both `.gemini/settings.json` (project) and `~/.gemini/settings.json` (global)
- **VS Code**: Supports both `.vscode/mcp.json` (project) and user-level config (OS-specific paths)
- **OpenCode**: Supports both JSON and JSONC formats (JSON with comments)
- **Codex CLI**: Global only - uses TOML format (`~/.codex/config.toml`)
- **Antigravity**: Global only - `~/.gemini/antigravity/mcp_config.json`

#### Environment Variable Conversion

The import command automatically normalizes environment variables to dynamic-mcp's `${VAR}` format:

| Tool | Original Format | Converted To |
|------|----------------|--------------|
| Cursor | `${env:GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| Claude Desktop | `${GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| Claude Code CLI | `${GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| VS Code | `${env:GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| Codex | `"${GITHUB_TOKEN}"` | `${GITHUB_TOKEN}` |

**Note**: VS Code's `${input:ID}` secure prompts cannot be automatically converted. You'll need to manually configure these after import.

See [docs/IMPORT.md](docs/IMPORT.md) for detailed tool-specific import guides.

## Dynamic MCP format

### Calling upstream servers on demand

Create a `dynamic-mcp.json` file with a `description` field for each server:

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

It supports the `${VAR}` syntax for environment variable interpolation:

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

It supports all [standard MCP transport mechanisms](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports).

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

- On first connection, a browser opens for authorization
- Access tokens are stored in `~/.dynamic-mcp/oauth-servers/<server-name>.json`
- Automatic token refresh before expiry (with RFC 6749 token rotation support)
- The token is injected as an `Authorization: Bearer <token>` header

### Feature Flags

Control which MCP features are exposed per server using the optional `features` field. By default, all features (`tools`, `resources`, `prompts`) are enabled. You can selectively disable features:

```json
{
  "mcpServers": {
    "server-with-tools-only": {
      "description": "Server that only exposes tools",
      "command": "npx",
      "args": ["-y", "some-mcp-server"],
      "features": {
        "resources": false,
        "prompts": false
      }
    },
    "server-without-prompts": {
      "type": "http",
      "description": "HTTP server without prompt templates",
      "url": "https://api.example.com",
      "features": {
        "prompts": false
      }
    }
  }
}
```

**Behavior:**

- If `features` is omitted, all features are enabled (opt-out design)
- If `features` is specified, unmentioned features default to `true` (enabled)
- Disabled features return an error if accessed via the proxy
- Example: If `resources: false`, calling `resources/list` returns an error

## Troubleshooting

### Server Connection Issues

**Problem**: `❌ Failed to connect to <server>`

**Solutions**:

- **Connection timeout**: Each server has 10-second timeout for transport creation, initialization, and tool listing
- **Automatic retry**: Failed servers are retried up to 3 times with exponential backoff (2s, 4s, 8s)
- **Periodic retry**: Failed servers are retried every 30 seconds in the background
- **Slow HTTP servers**: If remote HTTP/SSE servers are slow, they'll timeout and be retried automatically
- **Stdio servers**: Verify command exists (`which <command>`)
- **HTTP/SSE servers**: Check that the server is running and the URL is correct
- **Environment variables**: Ensure all `${VAR}` references are defined
- **OAuth servers**: Complete OAuth flow when prompted

**Logging**:

By default, errors and warnings are logged to the terminal. For more verbose output:

```bash
# Debug mode (all logs including debug-level details)
RUST_LOG=debug uvx dmcp config.json

# Info mode (includes informational messages)
RUST_LOG=info uvx dmcp config.json

# Default mode (errors and warnings only, no RUST_LOG needed)
uvx dmcp config.json
```

### OAuth Authentication Problems

**Problem**: The browser doesn't open for OAuth

**Solutions**:

- Manually open the URL shown in the console
- Check that the firewall allows localhost connections
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

**Problem**: `Server missing 'description' field`

**Solutions**:

- Every MCP server in your config must have a `description` field
- The description explains what the server does to the LLM
- Example:
  ```json
  {
    "description": "File system access - read, write, and search files",
    "command": "npx",
    "args": ["@modelcontextprotocol/server-filesystem"]
  }
  ```

**Problem**: `Invalid JSON in config file`

**Solutions**:

- Validate JSON syntax (use `jq . config.json`)
- Check for trailing commas
- Ensure all required fields are present (`description` is always required; `type` is required only for http/sse servers)

**Problem**: `Failed to resolve config path`

**Solutions**:

- Use an absolute path or a path relative to the working directory
- Check that the file exists and has read permissions
- Try: `ls -la <config-path>`

### Tool Call Failures

**Problem**: Tool call returns error

**Debugging**:

1. Test the tool directly with the upstream server
2. Check that the tool name and arguments match the schema
3. Verify the group name is correct
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

### Rust Binary

To build the Rust binary directly:

```bash
git clone https://github.com/asyrjasalo/dynamic-mcp.git
cd dynamic-mcp
cargo build --release
```

The binary is then available at `./target/release/dmcp`.

### Python Package

To build the Python package (wheel):

```bash
# Build wheel
uvx maturin build --release

# Install locally
pip install target/wheels/dmcp-*.whl
```

The Python package uses **maturin** with `bindings = "bin"` to compile the Rust binary directly into the wheel.

## Contributing

For instructions on development setup, testing, and contributing, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Release History

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

## Acknowledgments

- TypeScript implementation: [modular-mcp](https://github.com/d-kimuson/modular-mcp)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP Ecosystem: [rust-mcp-stack](https://github.com/rust-mcp-stack)
