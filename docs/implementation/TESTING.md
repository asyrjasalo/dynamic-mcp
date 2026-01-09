# Testing

## Current Test Status

**Total Tests**: 218
- **Unit Tests**: 107
- **Integration Tests**: 100 (Spec Compliance + Config + Import + CLI)
- **End-to-End Tests**: 11 (Real server interaction)

**Pass Rate**: 100% ✅

**Last Updated**: 2026-01-09

**Note**: Integration test count reduced from 105 to 100 by consolidating redundant fixture tests from config_integration_test.rs (removed 5 redundant fixture existence tests).

---

## 1. Unit Tests (107 tests)

Core module testing for server.rs and internal components.

**Location**: `src/main.rs` (inline `#[cfg(test)]` modules)

**Coverage**:
- Configuration parsing and validation
- Environment variable substitution (`${VAR}` format)
- Request/response handling (JSON-RPC)
- Error handling and edge cases
- Tool discovery and management
- Prompt handling
- Resource management
- CLI argument parsing
- Import command functionality

**Run Tests**:
```bash
cargo test src/
```

---

## 2. Integration Tests (34 tests)

Validation of API specifications and configuration without spawning upstream servers.

### 2.1 Tools API Spec Compliance (15 tests)
**File**: `tests/tools_test.rs`

**Purpose**: Validate Tools API compliance with MCP Specification v2025-11-25

- tools/list request/response format
- tools/call request/response format
- Tool input schemas (primitive types, objects, required/optional)
- Content types (text, image, audio, resource)
- Pagination (cursor-based)
- Error responses (JSON-RPC error codes)
- Capability declaration
- Complex arguments (nested objects, arrays)
- Tool input schema patterns (enum, pattern, min/max)

**Run Tests**:
```bash
cargo test --test tools_test
```

### 2.2 Prompts API Spec Compliance (28 tests)
**File**: `tests/prompts_test.rs`

**Purpose**: Combined spec compliance and integration tests for Prompts API v2025-11-25

**Spec Compliance Tests** (18 tests):
- prompts/list request/response format
- prompts/get request/response format
- Prompt messages (role, content types)
- Message content types (text, image, audio, resource)
- Pagination support
- Arguments (required/optional)
- Empty responses
- Special characters (UTF-8, emojis)
- Multiline text content
- JSON-RPC error responses

**Integration Tests** (10 tests):
- Dynamic MCP configuration with prompts support
- Prompts response format validation
- Prompt messages with content types
- Prompt arguments structure
- Pagination with cursor support
- Capability declaration

**Run Tests**:
```bash
cargo test --test prompts_test
```

### 2.3 Resources API Spec Compliance (28 tests)
**File**: `tests/resources_test.rs`

**Purpose**: Combined spec compliance and integration tests for Resources API v2025-11-25

**Spec Compliance Tests** (19 tests):
- resources/list request/response format
- resources/read request/response format
- resources/templates/list request/response format
- Resource size field (optional u64)
- Resource annotations (audience, priority, lastModified)
- Resource icons (src, mimeType, sizes)
- RFC 6570 URI template syntax
- Multiple URI schemes (file, https, git, custom)
- Pagination support
- Blob content (base64 encoding)
- Empty responses

**Integration Tests** (9 tests):
- Resource template structure validation
- Resource size field validation
- Resource annotations validation
- Resource icons with MIME types

**Run Tests**:
```bash
cargo test --test resources_test
```

### 2.4 Configuration Integration (6 tests)
**File**: `tests/config_integration_test.rs`

**Purpose**: Validate MCP server configuration and response format compliance

**Config Structure Tests** (6 tests):
- Configuration file format validation (mcpServers schema)
- Config file loading and parsing
- Multi-server configuration support
- Initialize capabilities declaration
- JSON-RPC error codes
- Example config file validation

**Note**: Import fixture validation is now consolidated in cli_import_integration_test.rs tests rather than tested separately.

**Run Tests**:
```bash
cargo test --test config_integration_test
```

### 2.5 CLI Import Command Integration (18 tests)
**File**: `tests/cli_import_integration_test.rs`

**Purpose**: Validate configuration import from other tools

- Import from Cursor, Claude Desktop, VS Code, Cline, OpenCode, Gemini, KiloCode
- Environment variable conversion (${env:VAR} → ${VAR})
- Configuration validation and error handling
- Global vs project config support
- Force flag and interactive prompts
- JSON/JSONC/TOML format handling
- Import fixture validation (implicit during test execution)

**Run Tests**:
```bash
cargo test --test cli_import_integration_test
```

### 2.6 CLI Integration (5 tests)
**File**: `tests/cli_integration_test.rs`

**Purpose**: Validate CLI functionality and build artifacts

- Binary build verification
- Binary artifact existence
- Help flag (--help)
- Version flag (--version)
- Invalid config path error handling

**Run Tests**:
```bash
cargo test --test cli_integration_test
```

---

## 3. End-to-End Tests (11 tests)

Real upstream server integration. Tests spawn actual MCP server instances and validate complete request/response cycles.

**File**: `tests/server_everything_e2e_test.rs`

**Tests**:
1. `test_e2e_initialize` - MCP protocol initialization
2. `test_e2e_tools_list` - Tool discovery from upstream server
3. `test_e2e_get_dynamic_tools_everything` - Tool schema verification
4. `test_e2e_call_dynamic_tool_get_dynamic_tools` - Dynamic tool listing
5. `test_e2e_tools_echo_execution` - Real tool execution (echo tool)
6. `test_e2e_prompts_list` - Prompt discovery
7. `test_e2e_prompts_get_simple` - Prompt retrieval
8. `test_e2e_resources_list` - Resource discovery
9. `test_e2e_resources_read` - Resource content retrieval
10. `test_e2e_resources_templates_list` - Resource template discovery
11. `test_e2e_error_handling_invalid_group` - Error handling

**Server Used**: `@modelcontextprotocol/server-everything` (via npx)

**Characteristics**:
- Spawns real upstream server instances
- Makes live JSON-RPC requests through proxy
- Tests complete request/response cycles
- Each test: ~15 seconds (server startup + request/response)
- Run in parallel: ~30 seconds total for all 11 tests
- First run downloads npm packages; subsequent runs cached

**Run Tests**:
```bash
# All E2E tests
cargo test --test server_everything_e2e_test

# Specific E2E test
cargo test --test server_everything_e2e_test test_e2e_tools_echo_execution
```

---

## Test Execution

### Run All Tests
```bash
cargo test
```
- **Result**: 218 tests passed in ~45 seconds
- **Coverage**: Unit + Integration + E2E tests

### Run by Category
```bash
# Unit tests only
cargo test src/

# Integration tests only (all spec compliance and integration tests)
cargo test --test tools_test --test prompts_test --test resources_test --test config_integration_test --test cli_import_integration_test --test cli_integration_test

# Spec compliance tests only
cargo test --test tools_test --test prompts_test --test resources_test

# E2E tests only (real server interaction)
cargo test --test server_everything_e2e_test
```

### Run Specific Tests
```bash
# Run test by name
cargo test test_e2e_tools_echo_execution
cargo test test_tools_list_response_format

# Run with output visible
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test

# Run single-threaded (slower, useful for debugging)
cargo test -- --test-threads=1
```

---

## Test Files Summary

| File | Type | Count | Purpose |
|------|------|-------|---------|
| src/main.rs | Unit | 107 | Core modules, config, CLI, auth |
| tools_test.rs | Integration | 15 | Tools API spec compliance |
| prompts_test.rs | Integration | 28 | Prompts API spec compliance |
| resources_test.rs | Integration | 28 | Resources API spec compliance |
| config_integration_test.rs | Integration | 6 | Config structure validation |
| cli_import_integration_test.rs | Integration | 18 | CLI import command from AI tools |
| cli_integration_test.rs | Integration | 5 | CLI build & artifact tests |
| server_everything_e2e_test.rs | E2E | 11 | Real upstream server integration |
| **TOTAL** | | **218** | |

---

## MCP Specification Compliance

All tests validate compliance with **MCP Specification v2025-11-25**.

**Validated Requirements**:
- ✅ Tools API (tools/list, tools/call)
- ✅ Prompts API (prompts/list, prompts/get)
- ✅ Resources API (resources/list, resources/read, resources/templates/list)
- ✅ JSON-RPC protocol (error codes, request/response format)
- ✅ Content types (text, image, audio, resource)
- ✅ Pagination (cursor-based)
- ✅ Capability declaration
- ✅ Resource annotations, icons, size field
- ✅ RFC 6570 URI template syntax

---

## Test Dependencies

### No Runtime Configuration Dependency
✅ **All tests are independent of `dynamic-mcp.json`**
- Tests create temporary configs or define configs inline
- Tests never read the real config file
- Can run in any environment without external config

### Example File Dependency
⚠️ **Some config integration tests depend on `examples/config.example.json`**

**Tests that depend on example config**:
- `test_config_example_exists` - Verifies example file exists
- `test_config_example_schema_validation` - Validates example config is valid JSON

**Why**: These tests ensure the documentation example is correct and present.

**Location**: `tests/config_integration_test.rs`

---

## Notes for Contributors

### Test Structure
- **Unit tests**: Inline in source files with `#[cfg(test)]`
- **Integration tests**: Separate files in `tests/` directory
- **E2E tests**: Single file for all end-to-end scenarios

### Test Naming Convention
```
test_<category>_<feature>_<scenario>
```

Examples:
- `test_tools_list_response_format`
- `test_e2e_tools_echo_execution`
- `test_prompts_get_with_optional_arguments`

### Adding New Tests

1. **For bug fixes**: Add regression test that reproduces bug, verify fix makes it pass
2. **For features**: Add tests BEFORE implementation (TDD approach)
3. **For spec changes**: Update tests before updating implementation
4. **For new APIs**: Create comprehensive integration test file

### Test Requirements

All code changes must include:
- ✅ Unit tests for new functions/logic
- ✅ Integration tests for public APIs
- ✅ Error case coverage
- ✅ Edge case coverage
- ✅ 100% pass rate on full test suite

### Debugging Failed Tests

```bash
# See full output
cargo test <test_name> -- --nocapture

# Single-threaded execution (no parallel interference)
cargo test <test_name> -- --test-threads=1

# With debug logging
RUST_LOG=debug cargo test <test_name>

# Run test in isolation (no other tests)
cargo test --test <file_name> <test_name>
```

---

## Performance

| Category | Count | Time | Per Test |
|----------|-------|------|----------|
| Unit Tests | 107 | ~0.5s | ~5ms |
| Integration Tests | 100 | ~10s | ~100ms |
| E2E Tests | 11 | ~30s | ~2.7s |
| **Total** | **218** | **~45s** | |

**Notes**:
- E2E tests are slower due to server startup overhead (~15s per test)
- Integration tests validate spec compliance without spawning servers
- Unit tests run in parallel for speed
- First run: E2E tests may take longer (npm package download)
