# Testing Documentation

The test suite contains 266+ tests organized into logical layers, each testing a specific aspect of dynamic-mcp.

## Current Test Status

**Total Tests**: 266 (with enabled field and strict schema validation tests)

- **Unit Tests**: 146 (inline in src/ modules, +8 for enabled field)
- **Integration Tests**: 120
  - CLI Tests: 5
  - Config Tests: 9
  - Features Tests: 5
  - Import Tests: 20
  - Spec Compliance: 71 (Tools: 15, Prompts: 28, Resources: 28)
  - E2E Tests: 11

**Pass Rate**: 100% ✅

**Last Updated**: 2026-01-12

______________________________________________________________________

## Test Organization Overview

### Integration Test Files (8 files, 121 tests)

| File                             | Tests | What It Tests                                               |
| -------------------------------- | ----- | ----------------------------------------------------------- |
| `cli_integration_test.rs`        | 5     | Binary build, CLI flags (--version, --help), error handling |
| `config_integration_test.rs`     | 9     | Config parsing, live reload, schema validation              |
| `features_test.rs`               | 5     | Per-server feature flags (tools, resources, prompts)        |
| `tools_test.rs`                  | 15    | Tools API spec compliance (MCP protocol)                    |
| `prompts_test.rs`                | 28    | Prompts API spec compliance (MCP protocol)                  |
| `resources_test.rs`              | 28    | Resources API spec compliance (MCP protocol)                |
| `cli_import_integration_test.rs` | 20    | Import command (10 AI tools, env var conversion)            |
| `server_everything_e2e_test.rs`  | 11    | End-to-end with real MCP server                             |

### Unit Test Files (12 files, 120+ tests)

| Module            | Files                                  | What They Test                                        |
| ----------------- | -------------------------------------- | ----------------------------------------------------- |
| **Server & Core** | `server.rs`, `main.rs`, `watcher.rs`   | MCP protocol handling, CLI args, config watching      |
| **Config**        | `schema.rs`, `loader.rs`, `env_sub.rs` | Config structures, file loading, env var substitution |
| **Auth**          | `oauth_client.rs`, `store.rs`          | OAuth2 PKCE flow, token storage                       |
| **CLI**           | `config_parser.rs`, `tool_detector.rs` | Multi-format parsing, tool detection                  |
| **Proxy**         | `types.rs`, `transport.rs`             | MCP types, transport creation (stdio/HTTP/SSE)        |

______________________________________________________________________

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

______________________________________________________________________

### Layer 2: Configuration Tests

**File**: `config_integration_test.rs` (9 tests)

Tests configuration file parsing, schema validation, and live reload functionality.

#### config_integration_test.rs (9 tests)

- `test_config_file_with_server` - Validates basic config file loading with mcpServers structure
- `test_example_config_with_server_definition` - Tests example config with multiple server definitions
- `test_config_initialize_capabilities` - Validates capabilities declaration (tools, prompts, resources)
- `test_config_jsonrpc_error_codes` - Verifies standard JSON-RPC error codes
- `test_config_example_schema_validation` - Validates schema of `examples/config.example.json`
- `test_config_example_exists` - Checks example config file artifact presence
- `test_config_live_reload_file_modified` - Tests live reload detects file modifications
- `test_config_live_reload_add_server` - Tests live reload when new servers are added
- `test_config_live_reload_remove_server` - Tests live reload when servers are removed
- ✨ **NEW**: `test_load_config_rejects_unknown_field_in_server` - Strict validation rejects unknown server fields
- ✨ **NEW**: `test_load_config_rejects_unknown_top_level_field` - Strict validation rejects unknown top-level fields
- ✨ **NEW**: `test_load_config_rejects_unknown_field_in_features` - Strict validation rejects unknown features fields
- ✨ **NEW**: `test_load_http_config_rejects_unknown_field` - HTTP server strict validation
- ✨ **NEW**: `test_load_sse_config_rejects_unknown_field` - SSE server strict validation
- ✨ **NEW**: `test_load_config_with_optional_fields_valid` - Verifies all valid fields are accepted

**Purpose**: Ensures configuration files parse correctly, follow the expected schema (with strict field validation), and live reload works properly.

______________________________________________________________________

### Layer 2.5: Per-Server Feature Flags Tests

**File**: `features_test.rs` (5 tests)

Tests per-server feature flag configuration and parsing.

- `test_config_with_features_disabled_parses_successfully` - Config with disabled features parses correctly
- `test_config_without_features_parses_successfully` - Default behavior (all enabled) when features omitted
- `test_config_with_mixed_features` - Mix of enabled/disabled features per server
- `test_config_with_explicit_enables` - Explicit true values for all features
- `test_config_with_all_features_disabled` - All features disabled configuration

**Purpose**: Validates per-server feature flag configuration (tools, resources, prompts) added in v1.3.0.

______________________________________________________________________

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

______________________________________________________________________

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
- Polls for readiness with 60s timeout before test execution
- Pre-installed in CI to avoid download delays

______________________________________________________________________

### Layer 5: CLI Import Command Integration Tests

**File**: `cli_import_integration_test.rs` (20 tests)

Tests the CLI `import` command for importing MCP configurations from AI coding tools.

#### Import Success Scenarios (5 tests)

- `test_import_cursor_project_success` - Cursor project config import
- `test_import_opencode_jsonc_success` - OpenCode JSONC config import
- `test_import_claude_project_success` - Claude Code CLI config import
- `test_import_cline_success` - Cline config import
- `test_import_multiple_servers_interactive` - Multi-server import with interactive prompts

#### Environment Variable Handling (9 tests)

- `test_import_vscode_env_var_normalization` - VS Code env var normalization (general)
- `test_import_cursor_env_var_conversion` - Cursor `${env:VAR}` → `${VAR}` conversion
- `test_import_vscode_env_var_conversion_in_env` - VS Code env var normalization in env fields
- `test_import_vscode_env_var_conversion_in_headers` - VS Code env var normalization in headers
- `test_import_codex_env_var_passthrough` - Codex env var handling
- `test_import_claude_env_var_passthrough` - Claude env var handling
- `test_import_opencode_env_var_passthrough` - OpenCode env var handling
- `test_import_gemini_env_var_passthrough` - Gemini env var handling
- `test_import_kilocode_env_var_passthrough` - KiloCode env var handling

#### Feature Selection (2 tests)

- `test_import_custom_features_selection` - Custom feature flags during import
- `test_import_default_all_features_enabled` - Default behavior (all features enabled)

#### Error Handling & Flags (4 tests)

- `test_import_force_flag_skips_overwrite_prompt` - Force flag behavior
- `test_import_missing_config_file_error` - Error on missing source config
- `test_import_empty_description_error` - Error on empty description input
- `test_import_invalid_json_error` - Error on malformed JSON

**Purpose**: Ensures the import command correctly transforms configurations from all supported AI tools into dynamic-mcp format, with proper env var normalization and feature selection.

______________________________________________________________________

### Layer 6: Unit Tests

**Location**: `src/**/*.rs` (inline `#[cfg(test)]` modules) (120+ tests)

Core module testing across all source files. Each source file with `#[cfg(test)]` contains unit tests for its functionality.

#### Unit Test Files by Module (12 files)

**Server & Core** (3 files):

- **`src/server.rs`** - MCP server request handling
  - Tests: initialize, tools/list, tools/call, resources/list, prompts/list, unknown methods
  - Coverage: JSON-RPC protocol, capability negotiation, error handling
- **`src/main.rs`** - CLI argument parsing and config resolution
  - Tests: CLI args precedence, environment variable fallback, config path resolution
- **`src/watcher.rs`** - Configuration file watching
  - Tests: Watcher creation, invalid path handling

**Config Module** (3 files):

- **`src/config/schema.rs`** - Configuration data structures
  - Tests: Features default values, deserialization, per-server feature flags
  - ✨ **NEW**: Strict validation tests - Unknown field rejection for stdio/http/sse servers and features
  - Coverage: JSON schema validation, serde behavior, `deny_unknown_fields` attribute
- **`src/config/loader.rs`** - Config file loading
  - Tests: Valid config loading, env var substitution, nonexistent file errors
  - ✨ **NEW**: Integration tests for strict field validation across all server types
  - Coverage: File I/O, error handling, schema enforcement
- **`src/config/env_sub.rs`** - Environment variable substitution
  - Tests: `${VAR}` with/without braces, undefined vars, nested substitution
  - Coverage: Regex matching, env var expansion

**Auth Module** (2 files):

- **`src/auth/oauth_client.rs`** - OAuth2 PKCE flow
  - Tests: Callback server creation, OAuth client initialization
  - Coverage: OAuth endpoints, PKCE challenge generation
- **`src/auth/store.rs`** - Token storage
  - Tests: Save/load tokens, nonexistent token handling, token deletion
  - Coverage: File I/O, JSON serialization, token lifecycle

**CLI Module** (2 files):

- **`src/cli/config_parser.rs`** - Multi-format config parsing
  - Tests: Cursor JSON, OpenCode JSONC, Claude Desktop JSON parsing
  - Coverage: JSON/JSONC/TOML parsing, format detection
- **`src/cli/tool_detector.rs`** - Tool detection and path resolution
  - Tests: Tool name mapping, unknown tools, project/global config paths
  - Coverage: Path resolution, tool-specific config locations

**Proxy Module** (2 files):

- **`src/proxy/types.rs`** - MCP type definitions (Resource, Prompt, Tool)
  - Tests: Resource serialization with size field, optional fields omission
  - Coverage: JSON serialization, MCP spec compliance
- **`src/proxy/transport.rs`** - Transport layer (stdio, HTTP, SSE)
  - Tests: HTTP transport creation, custom headers, SSE transport
  - Coverage: Transport initialization, header injection

**Summary**: All core modules have comprehensive unit test coverage for their internal logic.

______________________________________________________________________

## Test Execution

### Run All Tests

```bash
cargo test
```

- **Result**: 259 tests passed
- **Coverage**: Unit + Integration + E2E tests
- **Speed**: Execution time depends on machine hardware and load

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

______________________________________________________________________

## Test Architecture Philosophy

The test suite is organized as a **verification pyramid**:

```
┌─────────────────────────────────┐
│   Unit Tests (120+ tests)       │  Core modules, internal logic
├─────────────────────────────────┤
│   E2E Tests (11 tests)          │  Real server, actual protocol
├─────────────────────────────────┤
│   Spec Tests (71 tests)         │  Format validation, no execution
├─────────────────────────────────┤
│   Config Tests (9 tests)        │  Configuration parsing & live reload
├─────────────────────────────────┤
│   Features Tests (5 tests)      │  Per-server feature flags
├─────────────────────────────────┤
│   CLI Tests (5 tests)           │  Binary & flags
├─────────────────────────────────┤
│   Import Tests (20 tests)       │  CLI import command
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

______________________________________________________________________

## Test Files Summary

| File                           | Type        | Count    | Purpose                                   |
| ------------------------------ | ----------- | -------- | ----------------------------------------- |
| src/\*\*/\*.rs (inline)        | Unit        | 120+     | Core modules, config, CLI, auth, watcher  |
| tools_test.rs                  | Integration | 15       | Tools API spec compliance                 |
| prompts_test.rs                | Integration | 28       | Prompts API spec compliance               |
| resources_test.rs              | Integration | 28       | Resources API spec compliance             |
| features_test.rs               | Integration | 5        | Per-server feature flags                  |
| config_integration_test.rs     | Integration | 9        | Config structure validation & live reload |
| cli_import_integration_test.rs | Integration | 20       | CLI import command from AI tools          |
| cli_integration_test.rs        | Integration | 5        | CLI build & artifact tests                |
| server_everything_e2e_test.rs  | E2E         | 11       | Real upstream server integration          |
| **TOTAL**                      |             | **242+** |                                           |

______________________________________________________________________

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

______________________________________________________________________

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

______________________________________________________________________

## Test Coverage

| Category                             | Count   | Coverage                                       |
| ------------------------------------ | ------- | ---------------------------------------------- |
| Unit Tests                           | 138     | Core modules, internal logic, edge cases       |
| Spec Tests (tools/prompts/resources) | 71      | MCP specification compliance (v2025-11-25)     |
| Features Tests                       | 5       | Per-server feature flag configuration          |
| Config Tests                         | 9       | Config parsing, schema validation, live reload |
| CLI Tests                            | 5       | Binary build, CLI flags, error handling        |
| Import Tests                         | 20      | Import from 10 AI tools, env var conversion    |
| E2E Tests                            | 11      | End-to-end workflows with real MCP server      |
| **Total**                            | **259** | **Comprehensive coverage**                     |

**Notes**:

- E2E tests use shared server instance with 60s readiness timeout
- Import tests validate real tool config fixtures
- Spec compliance tests verify MCP protocol adherence
- Unit tests run in parallel for efficiency
- First run may be slower if npm dependencies need to be downloaded

______________________________________________________________________

## Maintenance Notes

- Spec compliance tests (`tools_test.rs`, `prompts_test.rs`, `resources_test.rs`) test the [MCP specification v2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/). Update these if the spec changes.

- E2E tests depend on `@modelcontextprotocol/server-everything` being available via npx. The package is pre-installed in CI for faster test execution.

- Import tests use real tool config fixtures in `tests/fixtures/import/`. Fixture validation happens implicitly during test execution, not in separate tests.

- Total test count: **242+ tests** across 8 integration test files plus inline unit tests in src/ (120+ tests).

______________________________________________________________________

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

______________________________________________________________________

**Last Updated**: January 12, 2026

______________________________________________________________________

## Recent Updates

- **2026-01-12**: Added per-server enable/disable feature (8 new tests in schema.rs for enabled field). Total: 266 tests.
- **2026-01-12**: Added strict JSON schema validation tests (17 new tests across schema.rs and loader.rs). Total: 259 tests.
- **2026-01-10**: Documentation update - Added comprehensive test file listing and unit test breakdown by module. Total: 242+ tests.
- **2026-01-09**: Added live reload tests (3 tests) and watcher unit tests (2 tests) for ConfigWatcher.
