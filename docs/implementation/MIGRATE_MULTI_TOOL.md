# Multi-Tool Migration Feature

**Date**: 2026-01-08
**Status**: 🚧 In Progress

## Overview

Extends the `migrate` command to support importing MCP server configurations from 10 different AI coding tools, handling their varied config formats, locations, and environment variable expansion patterns.

## Objectives

1. **Support 9 AI Tools** (priority order):
   - Cursor
   - OpenCode
   - Claude Desktop (Claude Code)
   - Visual Studio Code
   - Cline
   - KiloCode
   - Codex CLI
   - Gemini CLI
   - Google Antigravity

2. **Dual Config Support**:
   - Project-level: `dmcp migrate <tool-name>` (creates `dynamic-mcp.json` in current directory)
   - Global/user-level: `dmcp migrate --global <tool-name>` (creates in current directory from global config)

3. **Override Protection**:
   - Prompt user if `dynamic-mcp.json` exists
   - Allow force override with `--force` flag

4. **Clear Error Messages**:
   - Unknown tool name (show supported tools)
   - Config not found (show expected path)
   - Invalid config format (list invalid fields/values)

## Tool Configuration Research

### Supported Tools

| Priority | Tool | Global Config | Project Config | Env Var Pattern | Format |
|----------|------|--------------|----------------|-----------------|--------|
| 1 | **Cursor** | `~/.cursor/mcp.json` | `.cursor/mcp.json` | `${env:VAR}` | JSON |
| 2 | **OpenCode** | `~/.config/opencode/opencode.{json,jsonc}` | `.opencode/mcp.{json,jsonc}` | System env | JSON/JSONC |
| 3 | **Claude Desktop** | `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)<br>`%APPDATA%\Claude\claude_desktop_config.json` (Windows)<br>`~/.config/Claude/claude_desktop_config.json` (Linux) | N/A | `${VAR}` | JSON |
| 4 | **VS Code** | User `settings.json` | `.vscode/mcp.json` | `${input:ID}`, `${env:VAR}` | JSON |
| 5 | **Antigravity** | UI-managed `mcp_config.json` | N/A | System env | JSON |
| 6 | **Codex CLI** | `~/.codex/config.toml` | N/A | TOML syntax | TOML |
| 7 | **Gemini CLI** | `~/.gemini/settings.json` | N/A | System env | JSON |
| 8 | **Cline** | Extension settings | `.cline/mcp.json` | `${env:VAR}` | JSON |
| 9 | **KiloCode** | `mcp_settings.json` (via VS Code) | `.kilocode/mcp.json` | Standard | JSON |

### Common Schema Pattern

All tools use variations of:
```json
{
  "mcpServers": {  // or "mcp" (OpenCode), "servers" (VS Code)
    "server-name": {
      "command": "executable",  // or "command": ["array"] (OpenCode)
      "args": ["arg1", "arg2"],
      "env": {
        "KEY": "value"
      },
      "type": "stdio|http|sse",  // optional
      "url": "https://...",      // for HTTP/SSE
      "headers": {...},          // for HTTP/SSE
      "disabled": false,         // Cline, KiloCode
      "alwaysAllow": [...]       // Cline, KiloCode
    }
  }
}
```

### Environment Variable Patterns

| Tool | Pattern | Example |
|------|---------|---------|
| Cursor | `${env:VAR}` | `${env:GITHUB_TOKEN}` |
| OpenCode | System env | Direct variable name |
| Claude Desktop | `${VAR}` | `${GITHUB_TOKEN}` |
| VS Code | `${env:VAR}`, `${input:ID}` | `${env:GITHUB_TOKEN}` |
| Antigravity | System env | Direct variable name |
| Gemini CLI | System env | Direct variable name |
| Codex CLI | TOML syntax | `KEY = "${VAR}"` |
| Cline | `${env:VAR}` | `${env:GITHUB_TOKEN}` |
| KiloCode | Standard | Direct string value |

## Implementation Design

### CLI Interface

```bash
# Project-level migration (reads from current directory)
dmcp migrate <tool-name>

# Global/user-level migration (reads from home directory)
dmcp migrate --global <tool-name>

# Force override existing config
dmcp migrate <tool-name> --force

# Custom output path
dmcp migrate <tool-name> -o custom-path.json

# Examples
dmcp migrate cursor
dmcp migrate --global claude-desktop
dmcp migrate opencode --force
```

### Supported Tool Names

- `cursor`
- `opencode`
- `claude-desktop` (or `claude`)
- `vscode` (or `vs-code`)
- `antigravity`
- `gemini`
- `codex`
- `cline`
- `kilocode`

### Architecture Changes

#### 1. Config Schema Extensions

Add new structs to handle tool-specific variations:

```rust
// src/config/schema.rs

/// Tool-specific config source format
#[derive(Debug, Clone)]
pub enum ToolConfig {
    Cursor(CursorConfig),
    OpenCode(OpenCodeConfig),
    ClaudeDesktop(ClaudeDesktopConfig),
    VSCode(VSCodeConfig),
    // ... etc
}

/// Tool-agnostic intermediate representation
#[derive(Debug, Clone)]
pub struct IntermediateServerConfig {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub server_type: Option<String>,
}
```

#### 2. Tool Detection Module

```rust
// src/cli/tool_detector.rs

pub enum Tool {
    Cursor,
    OpenCode,
    ClaudeDesktop,
    VSCode,
    Antigravity,
    Gemini,
    Codex,
    Copilot,
    Cline,
    KiloCode,
}

impl Tool {
    pub fn from_name(name: &str) -> Result<Self> { ... }
    pub fn project_config_path(&self) -> Option<PathBuf> { ... }
    pub fn global_config_path(&self) -> Result<PathBuf> { ... }
    pub fn env_var_pattern(&self) -> EnvVarPattern { ... }
}

pub enum EnvVarPattern {
    EnvColon,        // ${env:VAR}
    CurlyBraces,     // ${VAR}
    SystemEnv,       // Direct system env
    InputPrompt,     // ${input:ID}
}
```

#### 3. Config Parser Module

```rust
// src/cli/config_parser.rs

pub struct ConfigParser {
    tool: Tool,
}

impl ConfigParser {
    pub fn parse(&self, content: &str) -> Result<HashMap<String, IntermediateServerConfig>> { ... }
    
    fn parse_json(&self, content: &str) -> Result<...> { ... }
    fn parse_jsonc(&self, content: &str) -> Result<...> { ... }
    fn parse_toml(&self, content: &str) -> Result<...> { ... }
    
    fn normalize_env_vars(&self, value: String) -> String { ... }
}
```

#### 4. Enhanced Migration Module

```rust
// src/cli/migrate.rs

pub struct MigrationConfig {
    pub tool: Tool,
    pub is_global: bool,
    pub output_path: String,
    pub force: bool,
}

pub async fn run_migration_from_tool(config: MigrationConfig) -> Result<()> {
    // 1. Determine input path (project vs global)
    // 2. Check if input exists, error if not
    // 3. Check if output exists, prompt unless --force
    // 4. Parse source config
    // 5. Convert to IntermediateServerConfig
    // 6. Prompt for descriptions
    // 7. Convert to McpServerConfig
    // 8. Write output
}
```

### Error Messages

#### Unknown Tool

```
Error: Unknown tool name 'unknown-tool'

Supported tools:
  - cursor
  - opencode
  - claude-desktop (or: claude)
  - vscode (or: vs-code)
  - antigravity
  - gemini
  - codex
  - cline
  - kilocode
  
Note: GitHub Copilot uses registry-based config and cannot be migrated this way.

Usage: dmcp migrate <tool-name>
```

#### Project Config Not Found

```
Error: Project config not found for 'cursor'

Expected location: .cursor/mcp.json
Current directory: /path/to/project

Suggestions:
  - Run this command from your project root directory
  - Or use --global flag to migrate from global config:
      dmcp migrate --global cursor
```

#### Global Config Not Found

```
Error: Global config not found for 'claude-desktop'

Expected location: ~/Library/Application Support/Claude/claude_desktop_config.json
(Platform: macOS)

Suggestions:
  - Verify Claude Desktop is installed
  - Check if config file exists: ls -la "~/Library/Application Support/Claude/"
  - Or omit --global to migrate from project config (if available)
```

#### Output File Exists

```
Warning: Output file already exists: dynamic-mcp.json

Overwrite existing file? [y/N]: _

(Or use --force flag to skip this prompt)
```

#### Invalid Config Format

```
Error: Invalid config format in cursor config

Location: ~/.cursor/mcp.json
Problem: Missing required field 'command' in server 'github'

Found in config:
  {
    "github": {
      "args": ["-y", "@modelcontextprotocol/server-github"]
      // Missing 'command' field
    }
  }

Expected format:
  {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"]
    }
  }
```

## Testing Strategy

### Unit Tests

**Location**: `src/cli/tool_detector.rs`, `src/cli/config_parser.rs`

Tests for each tool:
1. Tool name parsing (including aliases)
2. Path resolution (project and global)
3. Config parsing from fixture
4. Environment variable normalization
5. Invalid config handling

### Integration Tests

**Location**: `tests/integration_test.rs`

**Fixtures**: `tests/fixtures/migrate/<tool-name>/`
- `project.json` - Project-level config example
- `global.json` - Global-level config example
- `invalid.json` - Invalid config for error testing

Tests:
1. Migrate from each tool (project config)
2. Migrate from each tool (global config)
3. Force override existing output
4. Error on unknown tool
5. Error on missing config
6. Error on invalid config format

### Test Fixtures Structure

```
tests/fixtures/migrate/
├── cursor/
│   ├── project.json          # .cursor/mcp.json format
│   ├── global.json           # ~/.cursor/mcp.json format
│   └── invalid.json
├── opencode/
│   ├── project.jsonc         # .opencode/mcp.json format
│   ├── global.jsonc          # ~/.config/opencode/opencode.jsonc format
│   └── invalid.jsonc
├── claude-desktop/
│   ├── global.json           # claude_desktop_config.json format
│   └── invalid.json
├── vscode/
│   ├── project.json          # .vscode/mcp.json format
│   └── invalid.json
├── cline/
│   ├── project.json          # .cline/mcp.json format
│   └── invalid.json
├── kilocode/
│   ├── project.json          # .kilocode/mcp.json format
│   └── invalid.json
└── codex/
    ├── global.toml           # ~/.codex/config.toml format
    └── invalid.toml
```

## Documentation Updates

### README.md

Add section under "Migrate from an existing MCP config":

```markdown
### Migrate from AI Coding Tools

Dynamic-mcp can automatically import MCP server configurations from popular AI coding tools:

#### Supported Tools
- Cursor
- OpenCode
- Claude Desktop
- Visual Studio Code
- Cline (VS Code extension)
- KiloCode
- Google Antigravity
- Gemini CLI
- Codex CLI

#### Usage

**From project config** (run in project directory):
```bash
dmcp migrate cursor
dmcp migrate vscode
dmcp migrate cline
```

**From global/user config** (reads from home directory):
```bash
dmcp migrate --global claude-desktop
dmcp migrate --global cursor
```

**Force overwrite** (skip confirmation):
```bash
dmcp migrate cursor --force
```

The command will:
1. Detect your tool's config location
2. Parse the existing MCP servers
3. Interactively prompt for descriptions
4. Convert environment variable formats
5. Generate `dynamic-mcp.json`

#### Tool-Specific Notes

- **Cursor**: Supports both `.cursor/mcp.json` (project) and `~/.cursor/mcp.json` (global)
- **Claude Desktop**: Global config only (location varies by OS)
- **VS Code**: Reads from `.vscode/mcp.json` in project
- **OpenCode**: Supports JSONC format with comments
- **Codex CLI**: Uses TOML format
- **GitHub Copilot**: Registry-based config (not file-based, cannot migrate)

See [MIGRATION.md](docs/MIGRATION.md) for detailed tool-specific guides.
```

### docs/MIGRATION.md

Expand with tool-specific migration guides for each supported tool.

## Implementation Phases

### Phase 1: Foundation (Priority 1-3 tools)
- [ ] Implement tool detector for Cursor, OpenCode, Claude Desktop
- [ ] Create config parsers for JSON and JSONC
- [ ] Implement env var normalization
- [ ] Create test fixtures
- [ ] Write unit tests

### Phase 2: VS Code & Extensions (Priority 4, 9, 10)
- [ ] Add VS Code, Cline, KiloCode support
- [ ] Handle VS Code-specific input prompts
- [ ] Test with extension-specific configs

### Phase 3: CLI Tools (Priority 6, 7)
- [ ] Add Gemini CLI, Codex CLI support
- [ ] Implement TOML parser
- [ ] Handle TOML-specific patterns

### Phase 4: Edge Cases (Priority 5)
- [ ] Add Antigravity support (UI-managed config)
- [ ] Comprehensive error message testing

### Phase 5: Documentation & Polish
- [ ] Update README.md
- [ ] Update docs/MIGRATION.md
- [ ] Update docs/implementation/STATUS.md
- [ ] Update docs/implementation/TESTING.md
- [ ] Add examples for each tool

## Known Limitations

1. **VS Code Input Prompts**: `${input:ID}` syntax for secure credential prompts cannot be automatically converted. Will prompt user to manually configure these after migration.

2. **Antigravity UI Config**: Config is managed through UI, not direct file access. Users may need to manually locate the config file.

3. **Platform Differences**: Claude Desktop has different config paths per OS (macOS, Windows, Linux). Tool detector handles this automatically.

4. **TOML Complexity**: Codex CLI uses TOML format which requires additional parsing dependency.

## Dependencies

New dependencies to add:

```toml
# For JSONC (JSON with comments) parsing
jsonc-parser = "0.23"

# For TOML parsing (Codex CLI)
toml = "0.8"
```

## Success Criteria

- [ ] All 9 supported tools migrate successfully
- [ ] Project and global configs both supported
- [ ] Environment variables normalized correctly
- [ ] Clear error messages for all failure scenarios
- [ ] 100% test coverage for migration logic
- [ ] Documentation complete and accurate
- [ ] User can migrate in <30 seconds with clear prompts
