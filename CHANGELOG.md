# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-01-09

### Added
- **Per-Server Feature Flags** - Control which MCP APIs are exposed per server
  - Optional `features` field in server config (tools, resources, prompts)
  - Opt-out design: all features enabled by default for backward compatibility
  - Clear error messages when accessing disabled features
- **Interactive Feature Selection** - Import command now prompts for feature customization
  - Customize which features to enable during `dmcp import` workflow
  - Press Enter to keep all features (default) or type 'n' to customize
- **Resources API** - Full support for `resources/list`, `resources/read`, and `resources/templates/list`
  - Discover and retrieve file-like resources from upstream MCP servers
  - URI templates with RFC 6570 support for dynamic resource discovery
  - Resource size field for context window estimation
  - Cursor-based pagination and resource annotations
- **Prompts API** - Full support for `prompts/list` and `prompts/get`
  - Discover and retrieve prompt templates from upstream servers
  - Multi-modal prompt content (text, image, audio, embedded resources)
  - Argument substitution in prompt templates
  - Cursor-based pagination
- **SSE Stream Resumption** - Automatic recovery from interrupted SSE connections
  - Tracks Last-Event-ID to prevent event loss on reconnection
  - Handles both `id: value` and compact event ID formats

### Changed
- Resource and prompt operations now auto-discover server groups
- Config serialization omits default features for cleaner output
- SSE transport improved with event ID tracking

### Fixed
- Resource and prompt endpoints now work without explicit group parameter
- SSE connections properly resume from last known event after interruptions

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
- **Release Notes**: [docs/history/RELEASE_v1.0.0.md](docs/history/RELEASE_v1.0.0.md)

[1.3.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.3.0
[1.2.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.2.0
[1.1.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.1.0
[1.0.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.0.0
