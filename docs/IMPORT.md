# Import Guide

This guide explains how to import MCP server configurations from AI coding tools into dynamic-mcp format.

## Why Import?

Standard MCP clients load all tool schemas from all servers upfront, consuming significant LLM context. Dynamic-mcp reduces context usage by:

1. Exposing only 2 proxy tools initially
2. Loading specific tool schemas on-demand
3. Grouping servers for organized access

## Quick Start

### Import from AI Coding Tools

__Project-level config__ (run in project directory):

```bash
dmcp import cursor
dmcp import vscode
dmcp import cline
```

__Global/user-level config__:

```bash
dmcp import --global claude-desktop
dmcp import --global opencode
dmcp import --global codex
```

__Force overwrite__:

```bash
dmcp import cursor --force
```

### Supported Tools

The import command currently supports the following AI coding tools:

| Tool                   | Tool Name        | Config Locations                                                                       |
| ---------------------- | ---------------- | -------------------------------------------------------------------------------------- |
| __Cursor__             | `cursor`         | Project: `.cursor/mcp.json`<br>Global: `~/.cursor/mcp.json`                            |
| __OpenCode__           | `opencode`       | Project: `.opencode/opencode.json(c)`<br>Global: `~/.config/opencode/opencode.json(c)` |
| __Claude Desktop__     | `claude-desktop` | Global only (OS-specific paths)                                                        |
| __Claude Code CLI__    | `claude`         | Project: `.mcp.json`<br>User: `~/.claude.json`                                         |
| __Visual Studio Code__ | `vscode`         | Project: `.vscode/mcp.json`<br>Global: OS-specific                                     |
| __Cline__              | `cline`          | Project: `.cline/mcp.json`<br>Global: VS Code extension settings                       |
| __KiloCode__           | `kilocode`       | Project: `.kilocode/mcp.json`<br>Global: Extension settings                            |
| __Codex CLI__          | `codex`          | Global: `~/.codex/config.toml` (TOML format)                                           |
| __Gemini CLI__         | `gemini`         | Project: `.gemini/settings.json`<br>Global: `~/.gemini/settings.json`                  |
| __Google Antigravity__ | `antigravity`    | Global: `~/.gemini/antigravity/mcp_config.json`                                        |

__What the import command does__:

- Automatically detects the tool's config location
- Parses the config format (JSON, JSONC, or TOML)
- Normalizes environment variables to `${VAR}` format
- Prompts you for a description for each server
- Preserves all settings (commands, args, env, headers, OAuth)
- Generates `dynamic-mcp.json` in the current directory

__Example import session__:

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

💬 Enter description for 'filesystem' (what this server does):
File operations on /tmp directory

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: git
Type: stdio

Config details:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-git"]

💬 Enter description for 'git' (what this server does):
Git repository operations

✅ Import complete!
📝 Output saved to: dynamic-mcp.json
```

## Manual Import

If the import command doesn't support your tool or you prefer manual control:

__Before (Standard MCP)__:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {
        "ALLOWED_PATHS": "/tmp"
      }
    },
    "brave-search": {
      "url": "https://api.brave.com/mcp",
      "headers": {
        "Authorization": "Bearer ${BRAVE_API_KEY}"
      }
    },
    "context7": {
      "url": "https://mcp.context7.com/mcp",
      "headers": {
        "CONTEXT7_API_KEY": "${CONTEXT7_API_KEY}"
      }
    }
  }
}
```

__After (dynamic-mcp)__:

```json
{
  "mcpServers": {
    "filesystem": {
      "description": "File operations on /tmp directory",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {
        "ALLOWED_PATHS": "/tmp"
      }
    },
    "brave-search": {
      "type": "http",
      "description": "Web search using Brave Search API",
      "url": "https://api.brave.com/mcp",
      "headers": {
        "Authorization": "Bearer ${BRAVE_API_KEY}"
      }
    },
    "context7": {
      "type": "http",
      "description": "Documentation and code samples search",
      "url": "https://mcp.context7.com/mcp",
      "headers": {
        "CONTEXT7_API_KEY": "${CONTEXT7_API_KEY}"
      }
    }
  }
}
```

__Changes required__:

1. Add `"type"` field for each server (http or sse) - stdio is default and optional
2. Add `"description"` field explaining what the server does
3. Keep all other fields unchanged

## Transport Type Detection

### Stdio Servers

__Indicators__: Has `command` field

```json
{
  "server-name": {
    "description": "...",
    "command": "node",
    "args": ["server.js"],
    "env": {...}
  }
}
```

### HTTP Servers

__Indicators__: Has `url` field, no SSE endpoint

```json
{
  "server-name": {
    "type": "http",
    "description": "...",
    "url": "https://api.example.com/mcp",
    "headers": {...},
    "oauth_client_id": "...",
    "oauth_scopes": [...]
  }
}
```

### SSE Servers

__Indicators__: Has `url` field with `/sse` endpoint or SSE protocol

```json
{
  "server-name": {
    "type": "sse",
    "description": "...",
    "url": "https://api.example.com/sse",
    "headers": {...},
    "oauth_client_id": "...",
    "oauth_scopes": [...]
  }
}
```

## Writing Good Descriptions

Descriptions are shown to the LLM when listing tools. Write them from the LLM's perspective:

### Good Descriptions

✅ "File operations on the /tmp directory"
✅ "Web search using Brave Search API"
✅ "Documentation and code samples search"
✅ "Slack workspace integration for messaging and channels"
✅ "PostgreSQL database queries on production DB"

### Poor Descriptions

❌ "Filesystem server" (too vague)
❌ "MCP server for Brave" (redundant, LLM knows it's an MCP server)
❌ "Server that does context7 stuff" (informal, unclear)
❌ "" (empty, not helpful)

### Description Template

Use this format:

```text
"[Primary capability] [using/with/on] [technology/resource]"
```

Examples:

- "File operations on /tmp directory"
- "Web search using Brave API"
- "Documentation search using Context7"
- "Database queries on PostgreSQL"

## Common Import Scenarios

### Scenario 1: NPX-based Servers

__Before__:

```json
{
  "server": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-package"]
  }
}
```

__After__:

```json
{
  "server": {
    "description": "Package-specific functionality description",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-package"]
  }
}
```

### Scenario 2: Environment Variables

__Before__:

```json
{
  "server": {
    "command": "node",
    "args": ["server.js"],
    "env": {
      "API_KEY": "${MY_API_KEY}",
      "DATABASE_URL": "${DB_URL}"
    }
  }
}
```

__After__ (unchanged except for description):

```json
{
  "server": {
    "description": "Server with API and database access",
    "command": "node",
    "args": ["server.js"],
    "env": {
      "API_KEY": "${MY_API_KEY}",
      "DATABASE_URL": "${DB_URL}"
    }
  }
}
```

### Scenario 3: OAuth-Protected Servers

__Before__:

```json
{
  "server": {
    "url": "https://api.example.com/mcp",
    "oauth_client_id": "client-id-123",
    "oauth_scopes": ["read", "write"]
  }
}
```

__After__:

```json
{
  "server": {
    "type": "http",
    "description": "API access with OAuth authentication",
    "url": "https://api.example.com/mcp",
    "oauth_client_id": "client-id-123",
    "oauth_scopes": ["read", "write"]
  }
}
```

### Scenario 4: Custom Headers

__Before__:

```json
{
  "server": {
    "url": "https://api.example.com",
    "headers": {
      "X-API-Key": "${API_KEY}",
      "X-Custom-Header": "value"
    }
  }
}
```

__After__:

```json
{
  "server": {
    "type": "http",
    "description": "Custom API with authentication headers",
    "url": "https://api.example.com",
    "headers": {
      "X-API-Key": "${API_KEY}",
      "X-Custom-Header": "value"
    }
  }
}
```

## Testing Your Import

After import, verify the config works:

```bash
# Test the config
dmcp dynamic-mcp.json
```

You should see:

```text
✅ MCP server config loaded successfully
✅ Successfully connected MCP Server: filesystem
✅ Successfully connected MCP Server: brave-search
✅ Successfully connected MCP Server: context7
Successfully connected 3 MCP groups. All groups are valid.
MCP server listening on stdio
```

If any servers fail:

```text
❌ Failed to connect to server-name: [error details]
...
Some MCP groups failed to connect. success_groups=[...], failed_groups=[...]
```

## Troubleshooting

### Server Not Connecting

__Symptom__: `❌ Failed to connect to server-name`

__Solutions__:

1. Check server is running (for HTTP/SSE)
2. Verify command/URL is correct
3. Ensure environment variables are set
4. Check network connectivity (for remote servers)

### OAuth Failures

__Symptom__: Browser doesn't open for OAuth

__Solutions__:

1. Check `oauth_client_id` is correct
2. Ensure server supports `.well-known/oauth-authorization-server`
3. Manually open the URL shown in console
4. Check OAuth scopes are valid

### Environment Variable Issues

__Symptom__: Variables not substituted

__Solutions__:

1. Ensure vars are exported: `export VAR=value`
2. Use `${VAR}` syntax (not `$VAR`)
3. Check variable name matches exactly (case-sensitive)

## Rollback

If import causes issues, you can:

1. __Keep both configs__: Use original tool config with the original MCP client
2. __Revert__: Delete `dynamic-mcp.json`, use original config
3. __Fix forward__: Adjust descriptions or types in imported config

## Next Steps

After successful import:

1. Update your MCP client config to point to dynamic-mcp
2. Configure dynamic-mcp to start automatically
3. Test each group with `get_dynamic_tools`
4. Verify tool calls work as expected

## Example: Full Import Workflow

```bash
# 1. Navigate to your project directory
cd ~/my-project

# 2. Run import command for your tool
dmcp import cursor

# When prompted, provide descriptions for each server:
# 💬 Enter description for 'filesystem': File operations
# 💬 Enter description for 'git': Git repository operations

# 3. Review imported config
cat dynamic-mcp.json

# 4. Test the config
dmcp dynamic-mcp.json

# 5. If successful, update your MCP client to use dmcp
# (Replace direct MCP server config with dmcp proxy)

# 6. Restart your LLM client
```

## Tool-Specific Import Guides

### Cursor

__Config Locations__:

- Project: `.cursor/mcp.json` (in project root)
- Global: `~/.cursor/mcp.json`

__Import__:

```bash
# From project config
cd /path/to/project
dmcp import cursor

# From global config
dmcp import --global cursor
```

__Environment Variables__: Cursor uses `${env:VAR}` format, automatically converted to `${VAR}`.

______________________________________________________________________

### OpenCode

__Config Locations__:

- Project: `.opencode/opencode.json` or `.opencode/opencode.jsonc`
- Global: `~/.config/opencode/opencode.json` or `~/.config/opencode/opencode.jsonc`

__Import__:

```bash
# From project config (auto-detects .json or .jsonc)
dmcp import opencode

# From global config (auto-detects .json or .jsonc)
dmcp import --global opencode
```

__Special Notes__:

- Supports both JSON and JSONC (JSON with comments) formats
- Auto-detects file extension (.json or .jsonc)
- Prefers .jsonc if both exist
- Uses `command` as array: `["npx", "-y", "package"]` instead of separate command/args
- Automatically parsed and normalized

______________________________________________________________________

### Claude Desktop

__Config Locations__ (global only):

- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

__Import__:

```bash
dmcp import --global claude-desktop
# or
dmcp import --global claude
```

__Environment Variables__: Uses `${VAR}` format (already compatible).

______________________________________________________________________

### Claude Code CLI

__Config Locations__:

- Project: `.mcp.json` (in project root - shared with team, version-controlled)
- User: `~/.claude.json` (cross-project, private to you)

__Import__:

```bash
# From project config (.mcp.json in project root)
cd /path/to/project
dmcp import claude

# From global config
dmcp import --global claude
```

__Environment Variables__: Uses `${VAR}` format (already compatible).

__Special Notes__:

- Uses `.mcp.json` in project root (shared/version-controlled)
- User scope uses `~/.claude.json` (cross-project, private)
- Different from Claude Desktop (which uses `~/Library/Application Support/Claude/`)
- Supports multiple scopes per [official docs](https://code.claude.com/docs/en/mcp#mcp-installation-scopes):
  - Local: `~/.claude.json` under project path (private, project-specific)
  - Project: `.mcp.json` in project root (shared, version-controlled)
  - User: `~/.claude.json` (private, cross-project)
- Ideal for developers using Claude Code via CLI

______________________________________________________________________

### Visual Studio Code

__Config Locations__:

- Project: `.vscode/mcp.json` (workspace-level)
- Global:
  - macOS: `~/Library/Application Support/Code/User/mcp.json`
  - Windows: `%APPDATA%\Code\User\mcp.json`
  - Linux: `~/.config/Code/User/mcp.json`

__Import__:

```bash
# From project config
dmcp import vscode

# From global/user config
dmcp import --global vscode
```

__Special Notes__:

- Uses `servers` instead of `mcpServers`
- Supports `${input:ID}` for secure credential prompts (cannot auto-convert)
- VS Code specific `inputs` array not imported
- Can also use Command Palette: `MCP: Open User Configuration`
- Supports both dedicated `mcp.json` or settings in `settings.json`

__Manual Steps After Import__:
If your config used `${input:credential-id}`:

1. Replace with environment variable: `${API_KEY}`
2. Export the variable: `export API_KEY=your-key`

______________________________________________________________________

### Cline (VS Code Extension)

__Config Location__:

- Project: `.cline/mcp.json`

__Import__:

```bash
dmcp import cline
```

__Special Notes__:

- `alwaysAllow` field is not imported (Cline-specific)
- `disabled` field is not imported
- Environment variables use `${env:VAR}` format (auto-converted)

______________________________________________________________________

### KiloCode

__Config Location__:

- Project: `.kilocode/mcp.json`

__Import__:

```bash
dmcp import kilocode
```

__Similar to Cline__: Extension-specific fields (`alwaysAllow`, `disabled`) are not imported.

______________________________________________________________________

### Codex CLI

__Config Location__:

- Global: `~/.codex/config.toml`

__Import__:

```bash
dmcp import --global codex
```

__Special Notes__:

- Uses TOML format instead of JSON
- Format: `[mcp.server-name]` sections
- Environment variables: TOML string syntax automatically handled

__Example TOML__:

```toml
[mcp.github]
command = "docker"
args = ["run", "-i", "ghcr.io/github/github-mcp-server"]

[mcp.github.env]
GITHUB_TOKEN = "${GITHUB_TOKEN}"
```

______________________________________________________________________

### Google Antigravity

__Config Location__:

- Global: `~/.gemini/antigravity/mcp_config.json`

__Import__:

```bash
# From global config (standard location)
dmcp import --global antigravity

# Or manually specify path if non-standard location
dmcp import /path/to/mcp_config.json
```

__Environment Variables__:

- Uses system environment (no special conversion needed)

______________________________________________________________________

### Gemini CLI

__Config Locations__:

- Project: `.gemini/settings.json` (in project root)
- Global: `~/.gemini/settings.json`

__Import__:

```bash
# From project config
cd /path/to/project
dmcp import gemini

# From global config
dmcp import --global gemini
```

__Environment Variables__: Uses standard environment variables (no special syntax).

__Special Notes__:

- Project config allows per-project MCP server configuration
- Useful for different contexts in different projects

______________________________________________________________________

## Getting Help

If you encounter issues:

1. Check [ARCHITECTURE.md](implementation/ARCHITECTURE.md) for system details
2. Review [README.md](../README.md) for configuration examples
3. Enable debug logging: `RUST_LOG=debug dmcp config.json`
4. Open an issue with error logs and config (redact secrets!)
