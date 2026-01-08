# Migration Guide

This guide explains how to migrate to dynamic-mcp from AI coding tools and standard MCP configurations.

## Why Migrate?

Standard MCP clients load all tool schemas from all servers upfront, consuming significant LLM context. Dynamic-mcp reduces context usage by:

1. Exposing only 2 proxy tools initially
2. Loading specific tool schemas on-demand
3. Grouping servers for organized access

## Quick Start

### Migrate from AI Coding Tools

**Project-level config** (run in project directory):
```bash
dmcp migrate cursor
dmcp migrate vscode
dmcp migrate cline
```

**Global/user-level config**:
```bash
dmcp migrate --global claude-desktop
dmcp migrate --global opencode
dmcp migrate --global codex
```

**Force overwrite**:
```bash
dmcp migrate cursor --force
```

### Migrate from Generic MCP Config

For backward compatibility with generic configs:

```bash
dmcp migrate config.json -o dynamic-mcp.json
```

**What it does**:
- Reads your existing MCP config
- Prompts you for a description for each server
- Transforms config to dynamic-mcp format
- Preserves all settings (commands, args, env, headers, OAuth)
- Writes output to specified file

**Example session**:
```
🔄 Starting migration from standard MCP config to dynamic-mcp format
📖 Reading config from: /Users/you/.config/mcp/config.json

✅ Found 3 MCP server(s) to migrate

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: filesystem
Type: stdio

Config details:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]

💬 Enter description for 'filesystem' (what this server does):
File operations on /tmp directory

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: brave-search
Type: http

Config details:
  url: "https://api.brave.com/mcp"

💬 Enter description for 'brave-search' (what this server does):
Web search using Brave Search API

...

✅ Migration complete!
📝 Output saved to: dynamic-mcp.json
```

### Method 2: Manual Migration

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

## Common Migration Scenarios

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

## Testing Your Migration

After migration, verify the config works:

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

If migration causes issues, you can:

1. **Keep both configs**: Use standard config with standard MCP client
2. **Revert**: Delete `dynamic-mcp.json`, use original config
3. **Fix forward**: Adjust descriptions or types in migrated config

## Next Steps

After successful migration:

1. Update your MCP client config to point to dynamic-mcp
2. Configure dynamic-mcp to start automatically
3. Test each group with `get_dynamic_tools`
4. Verify tool calls work as expected

## Example: Full Migration Workflow

```bash
# 1. Backup original config
cp ~/.config/mcp/config.json ~/.config/mcp/config.json.backup

# 2. Run migration
dmcp migrate ~/.config/mcp/config.json -o dynamic-mcp.json

# 3. Review migrated config
cat dynamic-mcp.json

# 4. Test the config
dmcp dynamic-mcp.json

# 5. If successful, update Claude/LLM config to use dmcp
# (Replace direct MCP server config with dmcp proxy)

# 6. Restart your LLM client
```

## Tool-Specific Migration Guides

### Cursor

**Config Locations**:
- Project: `.cursor/mcp.json` (in project root)
- Global: `~/.cursor/mcp.json`

**Migration**:
```bash
# From project config
cd /path/to/project
dmcp migrate cursor

# From global config
dmcp migrate --global cursor
```

**Environment Variables**: Cursor uses `${env:VAR}` format, automatically converted to `${VAR}`.

---

### OpenCode

**Config Locations**:
- Project: `.opencode/mcp.json` or `.opencode/mcp.jsonc`
- Global: `~/.config/opencode/opencode.json` or `~/.config/opencode/opencode.jsonc`

**Migration**:
```bash
# From project config (auto-detects .json or .jsonc)
dmcp migrate opencode

# From global config (auto-detects .json or .jsonc)
dmcp migrate --global opencode
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

**Migration**:
```bash
dmcp migrate --global claude-desktop
# or
dmcp migrate --global claude
```

**Environment Variables**: Uses `${VAR}` format (already compatible).

---

### Claude Code CLI

**Config Locations**:
- Project: `.mcp.json` (in project root - shared with team, version-controlled)
- Global: `~/.claude/mcp.json`

**Migration**:
```bash
# From project config (.mcp.json in project root)
cd /path/to/project
dmcp migrate claude

# From global config
dmcp migrate --global claude
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

**Migration**:
```bash
# From project config
dmcp migrate vscode

# From global/user config
dmcp migrate --global vscode
```

**Special Notes**:
- Uses `servers` instead of `mcpServers`
- Supports `${input:ID}` for secure credential prompts (cannot auto-convert)
- VS Code specific `inputs` array not migrated
- Can also use Command Palette: `MCP: Open User Configuration`
- Supports both dedicated `mcp.json` or settings in `settings.json`

**Manual Steps After Migration**:
If your config used `${input:credential-id}`:
1. Replace with environment variable: `${API_KEY}`
2. Export the variable: `export API_KEY=your-key`

---

### Cline (VS Code Extension)

**Config Location**:
- Project: `.cline/mcp.json`

**Migration**:
```bash
dmcp migrate cline
```

**Special Notes**:
- `alwaysAllow` field is not migrated (Cline-specific)
- `disabled` field is not migrated
- Environment variables use `${env:VAR}` format (auto-converted)

---

### KiloCode

**Config Location**:
- Project: `.kilocode/mcp.json`

**Migration**:
```bash
dmcp migrate kilocode
```

**Similar to Cline**: Extension-specific fields (`alwaysAllow`, `disabled`) are not migrated.

---

### Codex CLI

**Config Location**:
- Global: `~/.codex/config.toml`

**Migration**:
```bash
dmcp migrate --global codex
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

**Migration**:
```bash
# Locate the config file through Antigravity UI
# Then migrate using file path:
dmcp migrate /path/to/mcp_config.json -o dynamic-mcp.json
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

**Migration**:
```bash
# From project config
cd /path/to/project
dmcp migrate gemini

# From global config
dmcp migrate --global gemini
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
