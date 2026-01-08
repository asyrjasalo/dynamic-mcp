# Import Guide

This guide explains how to import MCP server configurations from AI coding tools into dynamic-mcp format.

## Why Import?

Standard MCP clients load all tool schemas from all servers upfront, consuming significant LLM context. Dynamic-mcp reduces context usage by:

1. Exposing only 2 proxy tools initially
2. Loading specific tool schemas on-demand
3. Grouping servers for organized access

## Quick Start

### Import from AI Coding Tools

**Project-level config** (run in project directory):
```bash
dmcp import cursor
dmcp import vscode
dmcp import cline
```

**Global/user-level config**:
```bash
dmcp import --global claude-desktop
dmcp import --global opencode
dmcp import --global codex
```

**Force overwrite**:
```bash
dmcp import cursor --force
```

### Supported Tools

The import command currently supports the following AI coding tools:

| Tool | Tool Name | Config Locations |
|------|-----------|------------------|
| **Cursor** | `cursor` | Project: `.cursor/mcp.json`<br>Global: `~/.cursor/mcp.json` |
| **OpenCode** | `opencode` | Project: `.opencode/mcp.json(c)`<br>Global: `~/.config/opencode/opencode.json(c)` |
| **Claude Desktop** | `claude-desktop` | Global only (OS-specific paths) |
| **Claude Code CLI** | `claude` | Project: `.mcp.json`<br>Global: `~/.claude/mcp.json` |
| **Visual Studio Code** | `vscode` | Project: `.vscode/mcp.json`<br>Global: OS-specific |
| **Cline** | `cline` | Project: `.cline/mcp.json`<br>Global: VS Code extension settings |
| **KiloCode** | `kilocode` | Project: `.kilocode/mcp.json`<br>Global: Extension settings |
| **Codex CLI** | `codex` | Global: `~/.codex/config.toml` (TOML format) |
| **Gemini CLI** | `gemini` | Project: `.gemini/settings.json`<br>Global: `~/.gemini/settings.json` |
| **Google Antigravity** | `antigravity` | Global: `~/.gemini/antigravity/mcp_config.json` |

**What the import command does**:
- Automatically detects the tool's config location
- Parses the config format (JSON, JSONC, or TOML)
- Normalizes environment variables to `${VAR}` format
- Prompts you for a description for each server
- Preserves all settings (commands, args, env, headers, OAuth)
- Generates `dynamic-mcp.json` in the current directory

**Example import session**:
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

If you prefer manual control:

**Before (Standard MCP)**:
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
    "playwright": {
      "url": "https://mcp.playwright.dev/sse",
      "oauth_client_id": "your-client-id",
      "oauth_scopes": ["read", "write"]
    }
  }
}
```

**After (Dynamic-MCP)**:
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
    "playwright": {
      "type": "sse",
      "description": "Browser automation with Playwright",
      "url": "https://mcp.playwright.dev/sse",
      "oauth_client_id": "your-client-id",
      "oauth_scopes": ["read", "write"]
    }
  }
}
```

**Changes required**:
1. Add `"type"` field for each server (http or sse) - stdio is default and optional
2. Add `"description"` field explaining what the server does
3. Keep all other fields unchanged

## Transport Type Detection

### Stdio Servers

**Indicators**: Has `command` field

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

**Indicators**: Has `url` field, no SSE endpoint

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

**Indicators**: Has `url` field with `/sse` endpoint or SSE protocol

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
✅ "Browser automation with Playwright"
✅ "Slack workspace integration for messaging and channels"
✅ "PostgreSQL database queries on production DB"

### Poor Descriptions

❌ "Filesystem server" (too vague)
❌ "MCP server for Brave" (redundant, LLM knows it's an MCP server)
❌ "Server that does playwright stuff" (informal, unclear)
❌ "" (empty, not helpful)

### Description Template

Use this format:
```
"[Primary capability] [using/with/on] [technology/resource]"
```

Examples:
- "File operations on /tmp directory"
- "Web search using Brave API"
- "Browser automation with Playwright"
- "Database queries on PostgreSQL"

## Common Import Scenarios

### Scenario 1: NPX-based Servers

**Before**:
```json
{
  "server": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-package"]
  }
}
```

**After**:
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

**Before**:
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

**After** (unchanged except for description):
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

**Before**:
```json
{
  "server": {
    "url": "https://api.example.com/mcp",
    "oauth_client_id": "client-id-123",
    "oauth_scopes": ["read", "write"]
  }
}
```

**After**:
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

**Before**:
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

**After**:
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
```
✅ MCP server config loaded successfully
✅ Successfully connected MCP Server: filesystem
✅ Successfully connected MCP Server: brave-search
...
Successfully connected N MCP groups. All groups are valid.
MCP server listening on stdio
```

If any servers fail:
```
❌ Failed to connect to server-name: [error details]
...
Some MCP groups failed to connect. success_groups=[...], failed_groups=[...]
```

## Troubleshooting

### Server Not Connecting

**Symptom**: `❌ Failed to connect to server-name`

**Solutions**:
1. Check server is running (for HTTP/SSE)
2. Verify command/URL is correct
3. Ensure environment variables are set
4. Check network connectivity (for remote servers)

### OAuth Failures

**Symptom**: Browser doesn't open for OAuth

**Solutions**:
1. Check `oauth_client_id` is correct
2. Ensure server supports `.well-known/oauth-authorization-server`
3. Manually open the URL shown in console
4. Check OAuth scopes are valid

### Environment Variable Issues

**Symptom**: Variables not substituted

**Solutions**:
1. Ensure vars are exported: `export VAR=value`
2. Use `${VAR}` syntax (not `$VAR`)
3. Check variable name matches exactly (case-sensitive)

## Rollback

If import causes issues, you can:

1. **Keep both configs**: Use original tool config with the original MCP client
2. **Revert**: Delete `dynamic-mcp.json`, use original config
3. **Fix forward**: Adjust descriptions or types in importd config

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

**Config Locations**:
- Project: `.cursor/mcp.json` (in project root)
- Global: `~/.cursor/mcp.json`

**Import**:
```bash
# From project config
cd /path/to/project
dmcp import cursor

# From global config
dmcp import --global cursor
```

**Environment Variables**: Cursor uses `${env:VAR}` format, automatically converted to `${VAR}`.

---

### OpenCode

**Config Locations**:
- Project: `.opencode/mcp.json` or `.opencode/mcp.jsonc`
- Global: `~/.config/opencode/opencode.json` or `~/.config/opencode/opencode.jsonc`

**Import**:
```bash
# From project config (auto-detects .json or .jsonc)
dmcp import opencode

# From global config (auto-detects .json or .jsonc)
dmcp import --global opencode
```

**Special Notes**:
- Supports both JSON and JSONC (JSON with comments) formats
- Auto-detects file extension (.json or .jsonc)
- Prefers .jsonc if both exist
- Uses `command` as array: `["npx", "-y", "package"]` instead of separate command/args
- Automatically parsed and normalized

---

### Claude Desktop

**Config Locations** (global only):
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

**Import**:
```bash
dmcp import --global claude-desktop
# or
dmcp import --global claude
```

**Environment Variables**: Uses `${VAR}` format (already compatible).

---

### Claude Code CLI

**Config Locations**:
- Project: `.mcp.json` (in project root - shared with team, version-controlled)
- Global: `~/.claude/mcp.json`

**Import**:
```bash
# From project config (.mcp.json in project root)
cd /path/to/project
dmcp import claude

# From global config
dmcp import --global claude
```

**Environment Variables**: Uses `${VAR}` format (already compatible).

**Special Notes**:
- Uses `.mcp.json` in project root (NOT `.claude/mcp.json`)
- Different from Claude Desktop (which uses `~/Library/Application Support/Claude/`)
- Supports multiple scopes: project (`.mcp.json`), local (`~/.claude.json` per project), user (global)
- Ideal for developers using Claude Code via CLI
- Project config can be version-controlled and shared with team

---

### Visual Studio Code

**Config Locations**:
- Project: `.vscode/mcp.json` (workspace-level)
- Global: 
  - macOS: `~/Library/Application Support/Code/User/mcp.json`
  - Windows: `%APPDATA%\Code\User\mcp.json`
  - Linux: `~/.config/Code/User/mcp.json`

**Import**:
```bash
# From project config
dmcp import vscode

# From global/user config
dmcp import --global vscode
```

**Special Notes**:
- Uses `servers` instead of `mcpServers`
- Supports `${input:ID}` for secure credential prompts (cannot auto-convert)
- VS Code specific `inputs` array not importd
- Can also use Command Palette: `MCP: Open User Configuration`
- Supports both dedicated `mcp.json` or settings in `settings.json`

**Manual Steps After Import**:
If your config used `${input:credential-id}`:
1. Replace with environment variable: `${API_KEY}`
2. Export the variable: `export API_KEY=your-key`

---

### Cline (VS Code Extension)

**Config Location**:
- Project: `.cline/mcp.json`

**Import**:
```bash
dmcp import cline
```

**Special Notes**:
- `alwaysAllow` field is not importd (Cline-specific)
- `disabled` field is not importd
- Environment variables use `${env:VAR}` format (auto-converted)

---

### KiloCode

**Config Location**:
- Project: `.kilocode/mcp.json`

**Import**:
```bash
dmcp import kilocode
```

**Similar to Cline**: Extension-specific fields (`alwaysAllow`, `disabled`) are not importd.

---

### Codex CLI

**Config Location**:
- Global: `~/.codex/config.toml`

**Import**:
```bash
dmcp import --global codex
```

**Special Notes**:
- Uses TOML format instead of JSON
- Format: `[mcp.server-name]` sections
- Environment variables: TOML string syntax automatically handled

**Example TOML**:
```toml
[mcp.github]
command = "docker"
args = ["run", "-i", "ghcr.io/github/github-mcp-server"]

[mcp.github.env]
GITHUB_TOKEN = "${GITHUB_TOKEN}"
```

---

### Google Antigravity

**Config Location**:
- UI-managed: `mcp_config.json`

**Import**:
```bash
# Locate the config file through Antigravity UI
# Then manually create dynamic-mcp.json following the format guide above
# or use one of the supported tool import commands
```

**Finding Config**:
1. Open Antigravity
2. Click "..." dropdown in agent panel
3. Select "Manage MCP Servers"
4. Click "View raw config"
5. Note the file location

---

### Gemini CLI

**Config Locations**:
- Project: `.gemini/settings.json` (in project root)
- Global: `~/.gemini/settings.json`

**Import**:
```bash
# From project config
cd /path/to/project
dmcp import gemini

# From global config
dmcp import --global gemini
```

**Environment Variables**: Uses standard environment variables (no special syntax).

**Special Notes**:
- Project config allows per-project MCP server configuration
- Useful for different contexts in different projects

---

## Getting Help

If you encounter issues:

1. Check [ARCHITECTURE.md](ARCHITECTURE.md) for system details
2. Review [README.md](../README.md) for configuration examples
3. Enable debug logging: `RUST_LOG=debug dmcp config.json`
4. Open an issue with error logs and config (redact secrets!)
