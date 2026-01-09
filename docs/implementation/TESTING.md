# Testing Documentation

The test suite contains 218 tests organized into logical layers, each testing a specific aspect of dynamic-mcp.

## Current Test Status

**Total Tests**: 218
- **Unit Tests**: 107
- **Integration Tests**: 100 (Spec Compliance + Config + Import + CLI)
- **End-to-End Tests**: 11 (Real server interaction)

**Pass Rate**: 100% ✅

**Last Updated**: 2026-01-09

---

## Test Layers

### Layer 1: CLI/Binary Tests
**File**: `cli_integration_test.rs` (5 tests)

Tests the binary compilation and command-line interface.

- `test_project_builds` - Verifies `cargo build` succeeds
- `test_binary_exists_after_build` - Checks binary artifact exists after build
- `test_version_flag` - Tests `--version` CLI flag output
- `test_help_flag` - Tests `--help` CLI flag output
- `test_invalid_config_path` - Verifies error handling for missing config file

**Purpose**: Ensures the project builds and CLI works correctly.

---

### Layer 2: Configuration Tests
**File**: `config_integration_test.rs` (6 tests)

Tests configuration file parsing and schema validation.

- `test_config_file_with_server` - Validates basic config file loading with mcpServers structure
- `test_example_config_with_server_definition` - Tests example config with multiple server definitions
- `test_config_initialize_capabilities` - Validates capabilities declaration (tools, prompts, resources)
- `test_config_jsonrpc_error_codes` - Verifies standard JSON-RPC error codes
- `test_config_example_schema_validation` - Validates schema of `examples/config.example.json`
- `test_config_example_exists` - Checks example config file artifact presence

**Purpose**: Ensures configuration files parse correctly and follow the expected schema.

---

### Layer 3: API Specification Compliance Tests
**Files**:
- `tools_test.rs` (15 tests)
- `prompts_test.rs` (28 tests)
- `resources_test.rs` (28 tests)

Total: 71 tests

Tests compliance with the MCP specification v2025-11-25 for JSON-RPC message structure.

#### Tools API (15 tests)
- Request/response structure validation
- Input schema types (primitives, objects, arrays)
- Tool call request format
- Error responses
- Content types (text, image, audio, resource)
- Pagination (cursor-based)
- Capability declaration
- Complex arguments (nested objects, arrays)
- Tool input schema patterns (enum, pattern, min/max)

#### Prompts API (28 tests)
- List/get request and response formats
- Prompt arguments structure (required/optional)
- Argument metadata validation
- Pagination support
- Prompt messages (role, content types)
- Message content types (text, image, audio, resource)
- Special characters (UTF-8, emojis)
- Multiline text content

#### Resources API (28 tests)
- List/read request and response formats
- Resource metadata (uri, name, size, annotations)
- Annotation structure (audience, priority, lastModified)
- Template support
- Resource icons (src, mimeType, sizes)
- RFC 6570 URI template syntax
- Multiple URI schemes (file, https, git, custom)
- Blob content (base64 encoding)

**Purpose**: Validates that the server produces messages that conform to the MCP specification. These tests verify **format compliance**, not actual functionality.

**Important**: These tests validate JSON structure only, without executing actual protocol operations. For functional testing, see the E2E layer below.

---

### Layer 4: End-to-End Integration Tests
**File**: `server_everything_e2e_test.rs` (11 tests)

Tests the complete server lifecycle using the official `@modelcontextprotocol/server-everything` test server.

- `test_e2e_initialize` - Protocol initialization and capabilities declaration
- `test_e2e_tools_list` - Tool listing via actual MCP protocol
- `test_e2e_get_dynamic_tools_everything` - Dynamic tool loading from proxy
- `test_e2e_call_dynamic_tool_get_dynamic_tools` - Nested tool call execution
- `test_e2e_tools_echo_execution` - Tool invocation with parameters
- `test_e2e_prompts_list` - Prompt listing
- `test_e2e_prompts_get_simple` - Prompt retrieval with arguments
- `test_e2e_resources_list` - Resource listing
- `test_e2e_resources_read` - Resource content reading
- `test_e2e_resources_templates_list` - Template listing
- `test_e2e_error_handling_invalid_group` - Error handling for invalid groups

**Purpose**: Verifies the entire system works end-to-end with a real MCP server. These are functional tests that exercise actual protocol behavior.

**Server Used**: `@modelcontextprotocol/server-everything` (via npx)

**Characteristics**:
- Spawns real upstream server instance via subprocess
- Makes live JSON-RPC requests through dynamic-mcp proxy
- Tests complete request/response cycles
- Uses shared server instance (OnceLock) across all tests
- Polls for readiness with 200ms intervals (60s timeout)
- Pre-installed in CI to avoid download delays
- Each test: ~1s (after initial server startup)
- Total suite: ~12s (includes server startup and health check)

---

### Layer 5: CLI Import Command Integration Tests
**File**: `cli_import_integration_test.rs` (18 tests)

Tests the CLI `import` command for importing MCP configurations from AI coding tools.

#### Import Success Scenarios
- `test_import_cursor_project_success` - Cursor project config import
- `test_import_opencode_jsonc_success` - OpenCode JSONC config import
- `test_import_claude_project_success` - Claude Code CLI config import
- `test_import_cline_success` - Cline config import
- `test_import_multiple_servers_interactive` - Multi-server import with interactive prompts

#### Environment Variable Handling
- `test_import_cursor_env_var_conversion` - Cursor `${env:VAR}` → `${VAR}` conversion
- `test_import_vscode_env_var_conversion_in_env` - VS Code env var normalization
- `test_import_vscode_env_var_conversion_in_headers` - VS Code header env var normalization
- `test_import_codex_env_var_passthrough` - Codex env var handling
- `test_import_claude_env_var_passthrough` - Claude env var handling
- `test_import_opencode_env_var_passthrough` - OpenCode env var handling
- `test_import_gemini_env_var_passthrough` - Gemini env var handling
- `test_import_kilocode_env_var_passthrough` - KiloCode env var handling

#### Error Handling & Flags
- `test_import_force_flag_skips_overwrite_prompt` - Force flag behavior
- `test_import_missing_config_file_error` - Error on missing source config
- `test_import_empty_description_error` - Error on empty description input
- `test_import_invalid_json_error` - Error on malformed JSON

**Purpose**: Ensures the import command correctly transforms configurations from all supported AI tools into dynamic-mcp format.

---

### Layer 6: Unit Tests
**Location**: `src/**/*.rs` (inline `#[cfg(test)]` modules) (107 tests)

Core module testing across all source files including server, config, auth, CLI, and proxy modules.

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

---

## Test Execution

### Run All Tests
```bash
cargo test
```
- **Result**: 218 tests passed in ~35 seconds
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

### Run a Specific Layer
```bash
cargo test --test cli_integration_test                # CLI layer
cargo test --test config_integration_test             # Config layer
cargo test --test tools_test                          # Tools spec layer
cargo test --test prompts_test                        # Prompts spec layer
cargo test --test resources_test                      # Resources spec layer
cargo test --test server_everything_e2e_test          # E2E layer
cargo test --test cli_import_integration_test         # CLI import layer
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

# Run single-threaded (useful for debugging)
cargo test -- --test-threads=1
```

---

## Test Architecture Philosophy

The test suite is organized as a **verification pyramid**:

```
┌─────────────────────────────────┐
│   Unit Tests (107 tests)        │  Core modules, internal logic
├─────────────────────────────────┤
│   E2E Tests (11 tests)          │  Real server, actual protocol
├─────────────────────────────────┤
│   Spec Tests (71 tests)         │  Format validation, no execution
├─────────────────────────────────┤
│   Config Tests (6 tests)        │  Configuration parsing
├─────────────────────────────────┤
│   CLI Tests (5 tests)           │  Binary & flags
├─────────────────────────────────┤
│   Import Tests (18 tests)       │  CLI import command
└─────────────────────────────────┘
```

### Benefits of This Structure

1. **Clear Separation of Concerns**: Each layer tests one aspect
2. **Independent Execution**: Run layers separately without affecting others
3. **Fast Feedback**: CLI and config tests run instantly
4. **Comprehensive Coverage**: Spec tests catch format issues, E2E tests catch behavior issues
5. **Easy Maintenance**: New tests fit naturally into existing structure
6. **Scalability**: Can add new test layers without reorganizing existing tests

### Test Complementarity

- **Spec tests** (tools/prompts/resources) validate that messages *should* look like
- **E2E tests** (server_everything) validate that they *actually do* look right when running
- Together, they provide high confidence in both specification compliance and implementation correctness

---

## Test Files Summary

| File | Type | Count | Purpose |
|------|------|-------|---------|
| src/**/*.rs (inline) | Unit | 107 | Core modules, config, CLI, auth |
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

### E2E Test Dependencies
⚠️ **E2E tests require `@modelcontextprotocol/server-everything`**

The package is:
- Automatically installed via `npx -y` when tests run (self-contained)
- Pre-installed in CI to avoid download delays during tests
- Uses polling to wait for server readiness (60s timeout)

---

## Performance

| Category | Count | Time | Per Test |
|----------|-------|------|----------|
| Unit Tests | 107 | ~0.02s | ~0.2ms |
| Spec Tests (tools/prompts/resources) | 71 | ~0.5s | ~7ms |
| Config Tests | 6 | ~0.5s | ~83ms |
| CLI Tests | 5 | ~1.4s | ~280ms |
| Import Tests | 18 | ~11.4s | ~633ms |
| E2E Tests | 11 | ~12s | ~1.1s |
| **Total** | **218** | **~35s** | |

**Notes**:
- E2E tests use shared server instance with polling for readiness (~12s total including startup)
- Import tests are slower due to file I/O and fixture processing
- Spec compliance tests run instantly (no actual server interaction)
- Unit tests run in parallel for speed
- First run may be slower if npm package needs to be downloaded

---

## Maintenance Notes

- Spec compliance tests (`tools_test.rs`, `prompts_test.rs`, `resources_test.rs`) test the [MCP specification v2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/). Update these if the spec changes.

- E2E tests depend on `@modelcontextprotocol/server-everything` being available via npx. The package is pre-installed in CI for faster test execution.

- Import tests use real tool config fixtures in `tests/fixtures/import/`. Fixture validation happens implicitly during test execution, not in separate tests.

- Total test count: **218 tests** across 8 test files (~3,600 lines) plus inline unit tests in src/ (~107 tests).

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

**Last Updated**: January 2026
