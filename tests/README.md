# Test Organization

This directory contains 111 integration tests organized into 5 logical layers, each testing a specific aspect of dynamic-mcp.

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

#### Tools API
- Request/response structure validation
- Input schema types (primitives, objects, arrays)
- Tool call request format
- Error responses

#### Prompts API
- List/get request and response formats
- Prompt arguments structure (required/optional)
- Argument metadata validation
- Pagination support

#### Resources API
- List/read request and response formats
- Resource metadata (uri, name, size, annotations)
- Annotation structure (audience, priority, lastModified)
- Template support

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

**Note**: These tests spawn a subprocess running `cargo run` with a live MCP server, so they take longer (~15-20 seconds per test due to npx package initialization).

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

## Test Execution

Run all tests:
```bash
cargo test
```

Run a specific layer:
```bash
cargo test --test cli_integration_test                # CLI layer
cargo test --test config_integration_test             # Config layer
cargo test --test tools_test                          # Tools spec layer
cargo test --test prompts_test                        # Prompts spec layer
cargo test --test resources_test                      # Resources spec layer
cargo test --test server_everything_e2e_test          # E2E layer
cargo test --test cli_import_integration_test         # CLI import layer
```

Run with output visible:
```bash
cargo test -- --nocapture
```

Run in single-threaded mode (useful for debugging):
```bash
cargo test -- --test-threads=1
```

---

## Test Architecture Philosophy

The test suite is organized as a **verification pyramid**:

```
┌─────────────────────────────────┐
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

## Maintenance Notes

- Spec compliance tests (`tools_test.rs`, `prompts_test.rs`, `resources_test.rs`) test the [MCP specification v2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/). Update these if the spec changes.

- E2E tests depend on `@modelcontextprotocol/server-everything` being available via npx. Tests are resilient to network delays (15-20 second startup time).

- Import tests use real tool config fixtures in `tests/fixtures/import/`. Fixture validation happens implicitly during test execution, not in separate tests.

- Total test count: **111 tests** across 7 files, ~3,500 lines of test code.

---

## Related Documentation

For detailed test specifications, coverage details, and performance metrics, see:
- **[docs/implementation/TESTING.md](../docs/implementation/TESTING.md)** - Comprehensive test documentation with spec details and performance analysis

---

**Last Updated**: January 2026
