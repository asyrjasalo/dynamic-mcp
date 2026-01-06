# Environment Variable Configuration Support

**Date**: January 6, 2026  
**Status**: ✅ Complete

## Overview

Added support for specifying the configuration file via the `GATEWAY_MCP_CONFIG` environment variable, with proper precedence handling when both environment variable and command line argument are provided.

## Implementation

### Precedence Rules

1. **Command line argument** (highest priority)
2. **GATEWAY_MCP_CONFIG environment variable**
3. **No config** → Error with helpful message

### Usage

#### Command Line (Explicit)
```bash
modular-mcp config.json
```

#### Environment Variable
```bash
export GATEWAY_MCP_CONFIG=config.json
modular-mcp
```

#### Override (CLI wins)
```bash
export GATEWAY_MCP_CONFIG=default.json
modular-mcp custom.json  # Uses custom.json, ignores env var
```

### Error Messages

When no configuration is provided:
```
Error: No configuration file specified

Usage: modular-mcp <config-file>
   or: GATEWAY_MCP_CONFIG=<config-file> modular-mcp

Example: modular-mcp config.example.json
     or: GATEWAY_MCP_CONFIG=config.example.json modular-mcp
```

## Code Changes

### Modified Files

**src/main.rs**:
- Added `get_config_path()` helper function
- Implemented precedence logic: CLI arg > env var > none
- Updated error messages to mention both options
- Added 4 unit tests for precedence handling

### New Tests

```rust
#[test]
fn test_cli_arg_takes_precedence()

#[test]
fn test_env_var_used_when_no_cli()

#[test]
fn test_no_config_returns_none()

#[test]
fn test_empty_env_var_is_invalid()
```

## Test Results

```
running 14 tests
test result: ok. 14 passed; 0 failed

Total: 17 tests passing (14 unit + 3 integration)
```

## Examples

### Docker/Container Usage
```dockerfile
ENV GATEWAY_MCP_CONFIG=/etc/modular-mcp/config.json
CMD ["/usr/local/bin/modular-mcp"]
```

### Systemd Service
```ini
[Service]
Environment="GATEWAY_MCP_CONFIG=/etc/modular-mcp/config.json"
ExecStart=/usr/local/bin/modular-mcp
```

### Development
```bash
# Use default config for dev
export GATEWAY_MCP_CONFIG=config.dev.json

# Switch to test config
modular-mcp config.test.json
```

### CI/CD
```yaml
env:
  GATEWAY_MCP_CONFIG: config.ci.json
run: modular-mcp
```

## Design Decisions

### 1. Environment Variable Name
**Choice**: `GATEWAY_MCP_CONFIG` (not `MCP_CONFIG` or `CONFIG_FILE`)

**Rationale**:
- Specific to this gateway application
- Avoids collision with other MCP tools
- Clear purpose from the name
- Follows convention of tool-specific env vars

### 2. Precedence Order
**Choice**: CLI arg > env var > error

**Rationale**:
- CLI arg is most explicit (user intent)
- Env var provides default/fallback
- No implicit config file search
- Clear error when nothing provided

### 3. Error Handling
**Choice**: Show both usage options in error message

**Rationale**:
- User learns about both methods
- Copy-paste ready examples
- Self-documenting behavior

### 4. Validation
**Choice**: Accept empty string from env var (will fail at file load)

**Rationale**:
- Consistent error handling (file loading stage)
- Simpler implementation
- Clear error message from file loader

## Logging

The application logs which method was used:

```
INFO Starting modular-mcp server with config: config.json (from command line argument)
```

or

```
INFO Starting modular-mcp server with config: config.json (from GATEWAY_MCP_CONFIG environment variable)
```

This helps with debugging configuration issues.

## Backward Compatibility

✅ **Fully backward compatible**

Existing usage continues to work:
```bash
modular-mcp config.json
```

The env var is purely additive functionality.

## Security Considerations

### Environment Variables in Production

**Good Practices**:
- ✅ Use env vars for deployment flexibility
- ✅ Set in systemd service files or container configs
- ✅ Don't commit `.env` files with secrets

**Cautions**:
- ⚠️ Env vars visible in process listings (`ps aux`)
- ⚠️ Config file path itself is not secret
- ⚠️ Actual secrets should use env var substitution in config

### Example: Secure Configuration

```bash
# Set config location via env var
export GATEWAY_MCP_CONFIG=/etc/modular-mcp/config.json

# Secrets in separate env vars (substituted in config)
export API_TOKEN=secret-token-here

# Config file uses substitution
{
  "mcpServers": {
    "remote": {
      "type": "http",
      "url": "https://api.example.com",
      "headers": {
        "Authorization": "Bearer ${API_TOKEN}"
      }
    }
  }
}
```

## Future Enhancements

Potential improvements (not implemented):

1. **Config Directory Search**
   - Check `~/.config/modular-mcp/config.json`
   - Check `/etc/modular-mcp/config.json`
   - Fall back to default locations

2. **Multiple Config Files**
   - Merge multiple configs
   - Layer: system → user → local → env → CLI

3. **Config Validation**
   - `--check` flag to validate without running
   - Better error messages for invalid configs

4. **Config Generation**
   - `--init` to create template config
   - Interactive config builder

## Documentation Updates

- ✅ README.md updated with env var usage
- ✅ Error messages show both options
- ✅ This document created

---

**Implementation Complete**: All 7 tasks finished  
**Test Coverage**: 4 new unit tests, all passing  
**Production Ready**: ✅
