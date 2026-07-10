# dynamic-mcp

> In one sentence: it is a "tool relay station" that gathers your scattered AI tools into one place, so your AI assistant uses them more cheaply and more conveniently.

## Understand it in 3 minutes (for non-technical readers)

Let's explain it in plain language first; the technical details come later.

### 1. The two problems it solves

**Problem 1: Too many tools — the AI assistant can't keep track, and it costs money**
An AI assistant (like Claude or GPT) can't actually do work by itself; it has to borrow your various "tools" — looking things up, reading files, doing calculations… If you put 20 tools in front of it at once, it has to keep every tool's instruction on its "work desk". The desk gets cluttered, the assistant slows down, and every sentence you exchange costs more money.

dynamic-mcp's clever trick: **at first it shows the assistant only a 3-item "menu"** (list groups, see what's in a group, call a tool). When the assistant actually needs a tool, it fetches that tool's details on the spot. The desk stays tidy — cheaper and simpler.

**Problem 2: The assistant can't reach the tools on your computer**
Many great tools only "sit inside your computer and work when you're standing right next to it". But if the assistant is in a browser, on a phone, or on another machine in the cloud, it "can't walk over" to grab them.

dynamic-mcp can attach a "phone line" to those local tools so they can be reached from afar too (the technical term is "bridging local stdio into Streamable HTTP"). Once bridged, a distant assistant just makes a "call" (connects to a network address) and can use the tools on your computer.

### 2. Two "ways to open the door" (modes)

Think of the software as a "tool room" that can open a different number of doors:

| Mode | Front door (for the assistant standing at your computer) | Back window (for assistants in browser/phone/cloud) | For whom |
|---|---|---|---|
| Mode 1 · stdio | open | closed | Only the AI app installed on your computer |
| Mode 2 · http | closed | open | Only assistants in browser/phone/cloud |
| Mode 3 · both | open | open | Both at the same time |

All three modes keep the "only 3 menu items" design.

### 3. Mode 3 (both) — advantages / highlights

Mode 3 is "one room, two doors, one set of tools". Its biggest benefit is **one program does the job of two**:

1. **One program, usable from two places**: your desktop AI app (Claude Desktop / Cursor / VS Code) uses the "front door" (stdio), while browser/phone/cloud assistants use the "back window" (HTTP) — **one process serves both kinds of assistants**, no need to run two dmcp instances.
2. **Shares one set of tool connections and config**: both doors use the same upstream tools and the same config file. No need to maintain two dmcp instances or connect upstream twice — saves memory and resources, and you configure only once.
3. **Less hassle**: without Mode 3 you'd run two programs — one stdio for the desktop, one http for the browser — double the connections, double the memory, two configs to maintain. Mode 3 removes all that.

> ⚠️ **Who should open Mode 3 (important)**: let **your desktop AI app (Claude Desktop / Cursor / VS Code) open it for you** — don't launch it manually yourself.
> Why: whether the "room has power" depends on "whether an assistant walked in through the front door". When the AI app opens dmcp, it pushes open the front door and stands inside — the room powers on, the back window opens too, and the browser assistant can come in through it.
> If you launch it manually: the front door is open but nobody uses it (waste), and your AI app still can't use it, so it has to open another one → back to two programs, and Mode 3's benefit is gone.
> Correct approach: in your AI app's settings, tell it to open dmcp "with the both mode"; it does the rest.

> 💡 **Tip**: the room's power depends on "whether the front-door assistant is present". If you close the desktop AI app, dmcp is shut down and the back window closes too — the browser assistant loses access immediately. If you want "the browser assistant to keep working even after the desktop app is closed", split Mode 3 into two separate programs: one always-on for the outside (Mode 2 http, you keep it running), and one for the desktop app (Mode 1 or Mode 3, opened by the app itself).

### 4. How to pick the one switch (for those who want to try)

The switch that decides "how many doors to open" is called `--transport`:
- `stdio` → only the front door (for the desktop AI app)
- `http` → only the back window (for browser/phone/cloud)
- `both` → both doors (let the AI app open it for you)

The other parameters (address, port, path) can stay at their defaults; you only touch them when you want devices *other than your own computer* to reach it (see the "Parameters" section in the technical chapter below).

---

The full technical explanation for developers follows (you don't need it to get the gist above).

## Quick Start

### Installation

#### Option 1: Python package

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

#### Option 2: Native binary

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

#### Option 3: Compile from source

Install from [crates.io](https://crates.io/crates/dynamic-mcp):

```text
cargo install dynamic-mcp
```

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

| Tool            | Original Format       | Converted To      |
| --------------- | --------------------- | ----------------- |
| Cursor          | `${env:GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| Claude Desktop  | `${GITHUB_TOKEN}`     | `${GITHUB_TOKEN}` |
| Claude Code CLI | `${GITHUB_TOKEN}`     | `${GITHUB_TOKEN}` |
| VS Code         | `${env:GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| Codex           | `"${GITHUB_TOKEN}"`   | `${GITHUB_TOKEN}` |

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

It supports all [standard MCP transport mechanisms](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports).

**Note**: The `type` field is **optional** when `url` is present. If omitted, the server automatically uses HTTP transport with SSE detection per the MCP spec. This maintains backwards compatibility with tools like [OpenCode](https://opencode.ai/docs/mcp-servers/).

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
  "description": "HTTP server (type is optional)",
  "url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

Or with explicit type:

```json
{
  "type": "http",
  "description": "HTTP server with explicit type",
  "url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### sse

SSE servers are automatically detected when the server responds with `Content-Type: text/event-stream`. You can also explicitly specify `type: "sse"` if the server only supports SSE:

```json
{
  "type": "sse",
  "description": "SSE server (explicit type required only if server doesn't auto-detect)",
  "url": "https://api.example.com/sse",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### OAuth Authentication (HTTP/SSE)

```json
{
  "description": "OAuth-protected MCP server (type is optional)",
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
      "description": "HTTP server without prompt templates (type is optional)",
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

### Disabling Servers

Use the optional `enabled` field to disable a specific server without removing it from the config:

```json
{
  "mcpServers": {
    "filesystem": {
      "description": "File operations",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "disabled-server": {
      "description": "This server won't connect",
      "command": "some-command",
      "enabled": false
    }
  }
}
```

**Behavior:**

- If `enabled` is omitted, the server is enabled (default behavior)
- If `enabled: false`, the server is skipped during connection and won't appear in available groups
- Useful for temporarily disabling servers during testing or maintenance without editing config structure
- See `examples/config.features.example.json` for a complete example

### Timeout Configuration

Configure custom timeouts for tool, resource, and prompt calls per server using the optional `timeout` field. By default:

- Tool calls: 30 seconds
- Resource calls: 10 seconds
- Prompt calls: 10 seconds

You can customize these for servers that need more time:

```json
{
  "mcpServers": {
    "slow-server": {
      "description": "Server with slow operations",
      "command": "npx",
      "args": ["-y", "some-slow-mcp-server"],
      "timeout": {
        "tools": "1min",
        "resources": "30s",
        "prompts": "30s"
      }
    }
  }
}
```

**Supported duration formats:**

| Format       | Example               | Description                   |
| ------------ | --------------------- | ----------------------------- |
| Seconds      | `"30s"`, `"5s"`       | Simple seconds                |
| Minutes      | `"1min"`, `"2m"`      | Minutes (abbreviated or full) |
| Milliseconds | `"3000ms"`, `"500ms"` | Milliseconds                  |
| Plain number | `30`                  | Seconds (plain number)        |

**Behavior:**

- If `timeout` is omitted, defaults are used (tools: 30s, resources: 10s, prompts: 10s)
- Individual timeout fields default to their respective defaults if not specified
- Applies only to tool/resource/prompt call operations, not to connection or initialization
- Useful for servers with long-running operations (database queries, file processing, etc.)

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

**Problem**: Unknown field in config (e.g., `unknown field \`typo_field\`\`)

**Solutions**:

- dynamic-mcp uses strict JSON schema validation that only allows defined fields
- Check for typos in field names: `description`, `command`, `url`, `type`, `args`, `env`, `headers`, `oauth_client_id`, `oauth_scopes`, `features`, `enabled`, `timeout`
- Remove any extra or misspelled fields from your config
- Refer to the schema examples above to see valid fields for each server type

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

## Streamable HTTP Transport (v1.6.0)

In addition to the default stdio transport, dynamic-mcp can now expose its grouped-tool facade over a single **Streamable HTTP MCP** endpoint. This lets HTTP/SSE-based MCP clients (web UIs, remote agents, other MCP proxies, gateways) connect to dynamic-mcp without stdio.

The HTTP endpoint aggregates all configured upstream (stdio) servers into a 3-tool facade:

- `list_groups` — list all configured groups with connection status.
- `get_dynamic_tools` — fetch the tool schemas of a selected group (on demand).
- `call_dynamic_tool` — invoke a tool on a selected group through the proxy.

### Transport modes (stdio / http / both)

Since v1.6.0, `--transport` decides how dynamic-mcp serves clients. The single most important difference between the modes is **who starts the process**:

#### Mode 1 · stdio (default, `--transport stdio`)

- **What it is**: dynamic-mcp runs as a local subprocess on your machine; data goes over standard input/output (stdio).
- **Who starts it**: The **normal way to run stdio mode is for an LLM client to launch it** — you list `"command": "dmcp"` in the MCP config of a desktop client such as Claude Desktop, Cursor, or VS Code, and the client starts dmcp and owns its input/output; the process is born when the client starts and dies when the client exits. **You can also run `dmcp config.json` manually in a terminal**, but then its stdin/stdout are only attached to your terminal with no LLM client driving them — there is no "conversation partner", so it cannot actually be used. For this reason, stdio mode is only useful when launched by an LLM client.
- **When to use**: Only when you run a desktop AI client locally on your own machine.
- **Limitation**: An LLM running in a browser, the cloud, or on a phone cannot launch a local process on your machine, so **those environments cannot use stdio mode**.

#### Mode 2 · HTTP (`--transport http`)

- **What it is**: dynamic-mcp runs as a **long-lived HTTP service**, exposing a Streamable HTTP MCP endpoint (`http://<host>:<port><path>`) that any HTTP-capable client can connect to.
- **Who starts it**: **Only you (the user) can start it manually — an LLM cannot launch it.** The reason: Mode 2 was added precisely to fix stdio's shortcoming. An LLM app in a browser, the cloud, or on a phone has no way to spawn a local subprocess on your machine; so **you must first start it yourself in a terminal or as a service and keep it listening**, and only then can a remote LLM connect to it. **In Mode 2 the LLM is only a "connector", never the "launcher".**
- **When to use**: Browser extensions, cloud agents, mobile apps, remote / containerized environments (Docker, k8s), or when multiple clients need to share one backend.
- **Minimal start command:**

  ```bash
  dmcp --transport http /path/to/dynamic-mcp.json
  ```

#### Mode 3 · both (`--transport both`)

- **What it is**: stdio and HTTP run at the same time — one process, two entry points (the back window + the front door).
- **Advantages / highlights**:
  - **One program does the job of two**: the desktop AI app uses the stdio door, browser/phone/cloud assistants use the HTTP window — **one process serves both kinds of clients**, no need to run two dmcp instances.
  - **Shares one set of upstream connections and config**: both entry points use the same upstream tools and the same config file, saving memory and resources, configured only once.
  - **Less hassle**: without `both` you'd run two programs (one stdio for the desktop, one http for the browser) — double connections, double memory, two configs; `both` removes all that.
- **Who starts it (key)**: **let your desktop AI app (Claude Desktop / Cursor / VS Code) launch it**, not a manual terminal start. When the app launches dmcp it opens the stdio door and stands inside, the process stays alive, and the HTTP window opens too so the browser assistant can connect. A manual start leaves the stdio door idle and the desktop app can't use it — wasting `both`. See "Understand it in 3 minutes" above for the plain-language version.
- **When to use**: when you need both local desktop token savings AND browser/cloud/mobile remote use of the same upstream tools.

### Parameters

New command-line flags control HTTP exposure (the config file is **unchanged** — see below):

| Flag           | Default        | Description (plain language)                |
| -------------- | -------------- | ------------------------------------------- |
| `--transport`  | `stdio`        | Which doors to open: `stdio` for the desktop AI app only; `http` for browser/phone/cloud only; `both` for both (recommended: let the desktop AI app open it for you). |
| `--http-host`  | `127.0.0.1`    | Which machine the HTTP window "binds to". Default `127.0.0.1` = only your own computer can reach it (safest). Usually leave it; change only to let LAN/other devices connect (has security risk). |
| `--http-port`  | `8082`         | The window's "door number". Default 8082; change it (e.g. 9000) if another program already uses 8082. |
| `--http-path`  | `/dynamic-mcp` | The window's "room name". The address you type in the client must end with it, e.g. `http://127.0.0.1:8082/dynamic-mcp`. |

### Usage

```bash
# HTTP only (stdio disabled):
dmcp --transport http /path/to/dynamic-mcp.json

# Both stdio and HTTP at the same time:
dmcp --transport both /path/to/dynamic-mcp.json

# Bind on all interfaces, custom port/path:
dmcp --transport http --http-host 0.0.0.0 --http-port 9000 --http-path /mcp /path/to/dynamic-mcp.json
```

When `--transport http` or `both` is used, the facade is served at `http://<host>:<port><path>` (e.g. `http://127.0.0.1:8082/dynamic-mcp`).

### Configuration file (no change required)

v1.6.0 does **not** modify the `dynamic-mcp.json` schema. Your existing configuration works as-is; HTTP exposure is controlled entirely by the CLI flags above. The same `config-schema.json` applies.

Example `dynamic-mcp.json` (unchanged):

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

Then run with HTTP enabled:

```bash
dmcp --transport both /path/to/dynamic-mcp.json
```

### Application scenarios

- **Remote / containerized deployment** where stdio is unavailable (Docker, k8s, remote VM).
- **Reverse-proxy / gateway fronting** dynamic-mcp (nginx, Traefik) so multiple clients share one backend.
- **Web-based MCP clients and playgrounds** that speak Streamable HTTP.
- **Cascaded MCP proxies** — a second proxy or orchestrator connects to dynamic-mcp over HTTP instead of spawning subprocesses.
- **Single-endpoint multi-group access** — one HTTP endpoint serves every group; the client selects a group via `get_dynamic_tools` / `call_dynamic_tool`.

## Fork builds via GitHub Actions

> **Note:** The upstream repository has not been updated for an extended period. To use the v1.6.0 features (including the HTTP facade) without waiting for an upstream release, this fork builds its own binaries through **GitHub Actions** (the Release workflow triggered by pushing a `v*` tag). Cross-platform binaries — including Windows `dmcp.exe` — are produced as release assets. These builds are **not** published to crates.io / PyPI; download the binary directly from the fork's Releases page.

## Contributing

For instructions on development setup, testing, and contributing, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Release History

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

## Acknowledgments

- TypeScript implementation: [modular-mcp](https://github.com/d-kimuson/modular-mcp)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP Ecosystem: [rust-mcp-stack](https://github.com/rust-mcp-stack)
