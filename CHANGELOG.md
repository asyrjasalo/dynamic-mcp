# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-01-08

### Added
- **Prompts API Proxying** - Support for `prompts/list` and `prompts/get` endpoints
  - Proxy prompt discovery from upstream MCP servers
  - Retrieve prompt templates with argument substitution
  - Cursor-based pagination support for prompt lists
  - Multi-modal prompt content (text, image, audio, embedded resources)
  - Prompt metadata (name, title, description, arguments, icons)
- **Prompt Type Definitions** - Full support for MCP prompt schema
  - Prompt, PromptArgument, PromptMessage, PromptContent, PromptContentType types
  - Support for all content types: text, image, audio, resource
  - Proper serialization with optional field handling
- **Resources API Proxying** - Complete support for `resources/list`, `resources/read`, and `resources/templates/list` endpoints
  - Proxy resource discovery from upstream MCP servers
  - Retrieve resource contents (text and binary)
  - Cursor-based pagination support for resource lists
  - Resource annotations (audience, priority, lastModified)
  - **NEW**: Resource Templates with RFC 6570 URI template support
  - **NEW**: Resource size field for context window estimation
- **Resource Type Definitions** - Full support for MCP resource schema
  - Resource, ResourceContent, ResourceAnnotations, ResourceIcon, ResourceTemplate types
  - Proper serialization with optional field handling
  - Support for resource size and URI templates
- **SSE Stream Resumption** - Last-Event-ID tracking for interrupted SSE connections
  - Automatically extracts and stores event IDs from SSE responses
  - Sends Last-Event-ID header on reconnection to prevent event loss
  - Handles both `id: value` and compact formats

### Changed
- Updated `initialize` response to advertise `prompts` and `resources` capabilities
- Improved SSE transport reliability with event ID tracking
- Expanded Resources API from partial to full MCP specification compliance

### Fixed
- SSE connections now properly resume from last known event after network interruptions

## [1.2.0] - 2026-01-08

### Added
- **Multi-Tool Import Support** - Automatically import configs from 10 AI coding tools
  - Cursor, OpenCode, Claude Desktop, Claude Code CLI, VS Code
  - Cline, KiloCode, Codex CLI, Gemini CLI, Google Antigravity
- **Enhanced CLI** - `--global` flag for user-level configs, `--force` flag to skip prompts
- **Environment Variable Normalization** - Automatic conversion of tool-specific env var patterns
- **Config Parser Module** - Support for JSON, JSONC (with comments), and TOML formats
- **Tool Detection Module** - Smart path resolution for project/global configs per tool

### Changed
- Import command now uses tool names instead of file paths: `dmcp import cursor`
- Server processing order is now alphabetical (consistent interactive prompts)
- JSONC parsing improved with line comment stripping for better compatibility

### Documentation
- Updated README.md with tool-specific import examples and usage guides
- Updated IMPORT.md with comprehensive tool-specific import documentation

## [1.1.0] - 2026-01-07

### Added
- Python package distribution via PyPI with maturin bindings
- Windows ARM64 platform support in release binaries
- CHANGELOG.md included in GitHub release notes

### Changed
- Binary renamed from `dynamic-mcp` to `dmcp` for consistency with Python package
- Default logging level changed to `warn` (from `info`) for cleaner output
- Improved test reliability with better config fixtures and race condition handling

### Fixed
- Import command now respects `RUST_LOG` environment variable
- Removed duplicate wheel upload step in release workflow
- Updated dependencies: switched from native-tls to rustls for better ARM64 cross-compilation
- Snake_case tool names for better MCP protocol compliance
- Cross-platform process group handling for graceful shutdown

### Documentation
- Comprehensive AGENTS.md guide for AI-assisted development
- Expanded release process documentation
- Clearer installation instructions with uvx usage examples
- Updated README with restructured quick start and configuration sections

## [1.0.0] - 2026-01-06

### Added
- **Dynamic tool loading**: Expose only 2 proxy tools initially (`get_dynamic_tools`, `call_dynamic_tool`)
- **Multiple transport support**: stdio, HTTP, SSE for upstream MCP servers
- **OAuth2 authentication**: PKCE flow with automatic token refresh
- **Live configuration reload**: Watch config file changes and auto-reconnect
- **Automatic retry**: Exponential backoff for failed upstream connections
- **Import command**: Convert standard MCP configs to dynamic-mcp format (`dynamic-mcp import`)
- **Environment variable interpolation**: `${VAR}` syntax in configuration
- **Server descriptions**: Help LLMs understand when to use each tool group
- **Cross-platform binaries**: Linux x86_64, Linux ARM64, macOS ARM64, Windows x86_64

### Technical Details
- **Core**: Rust implementation using tokio async runtime
- **MCP Protocol**: rmcp v0.12 (official Rust MCP SDK)
- **HTTP Client**: reqwest with rustls-tls (pure Rust, no OpenSSL dependencies)
- **OAuth2**: oauth2 crate with PKCE support
- **File Watching**: notify crate for live reload
- **Testing**: 46 tests (37 unit + 9 integration), 100% pass rate
- **Lines of Code**: ~2,900 Rust

### Platform Support
- Linux x86_64 (`x86_64-unknown-linux-gnu`) - Native build
- Linux ARM64 (`aarch64-unknown-linux-gnu`) - Cross-compiled with rustls
- macOS ARM64 (`aarch64-apple-darwin`) - Native build for Apple Silicon
- Windows x86_64 (`x86_64-pc-windows-msvc`) - Native build

### Documentation
- Comprehensive README with quick start guide
- Architecture documentation explaining system design
- Import guide from standard MCP setup
- Security documentation for OAuth token storage
- Contributing guide with development setup
- Full API documentation via rustdoc

### Known Limitations
- Live reload works for config changes only (binary updates require restart)
- OAuth tokens stored as plain text in `~/.dynamic-mcp/oauth-servers/`
- No built-in rate limiting for tool calls
- Child processes inherit full privileges (no sandboxing)
- macOS Intel binaries are not released (build from source)
- Windows ARM64 binaries are not yet released (planned for future release)

### Installation
```bash
# From crates.io
cargo install dynamic-mcp

# Or download pre-built binaries from:
# https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.0.0
```

### Links
- **crates.io**: https://crates.io/crates/dynamic-mcp
- **GitHub**: https://github.com/asyrjasalo/dynamic-mcp
- **Documentation**: https://docs.rs/dynamic-mcp
- **Release Notes**: [docs/implementation/RELEASE_v1.0.0.md](docs/implementation/RELEASE_v1.0.0.md)

[1.2.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.2.0
[1.1.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.1.0
[1.0.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.0.0
