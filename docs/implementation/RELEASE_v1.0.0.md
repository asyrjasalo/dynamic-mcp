# Release v1.0.0 - Production Release

**Release Date**: January 6, 2026  
**Status**: ✅ Published

## 🎉 Highlights

dynamic-mcp v1.0.0 is now available! This is the first production-ready release of the MCP proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading tool schemas on-demand.

## 📦 Installation

### From crates.io
```bash
cargo install dynamic-mcp
```

### Pre-built Binaries
Download from [GitHub Releases](https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.0.0):
- Linux x86_64: `dynamic-mcp-x86_64-unknown-linux-gnu.tar.gz`
- macOS ARM64: `dynamic-mcp-aarch64-apple-darwin.tar.gz` (Apple Silicon)
- Windows x86_64: `dynamic-mcp-x86_64-pc-windows-msvc.zip`

## ✨ Features

### Core Functionality
- **Dynamic Tool Loading**: Expose only 2 tools initially (`get_dynamic_tools`, `call_dynamic_tool`)
- **Multiple Transport Support**: stdio, HTTP, SSE
- **OAuth2 Authentication**: With PKCE flow and automatic token refresh
- **Live Configuration Reload**: Watch config file for changes and automatically reconnect
- **Automatic Retry**: Exponential backoff for failed connections
- **Migration Command**: Convert standard MCP configs to dynamic-mcp format

### Configuration
- Environment variable interpolation (`${VAR}` syntax)
- Server descriptions for LLM context
- Flexible transport configuration
- OAuth scopes and client configuration

### Development
- Comprehensive test suite (46 tests, 100% pass rate)
- Full Rust documentation
- CI/CD pipeline with automated testing and releases

## 🖥️ Platform Support

### Available Binary Releases
| Platform | Architecture | Target Triple | Status |
|----------|-------------|---------------|--------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | ✅ Native build |
| macOS | ARM64 | `aarch64-apple-darwin` | ✅ Native build (Apple Silicon) |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | ✅ Native build |

### Build from Source Required
| Platform | Architecture | Target Triple | Reason | Workaround |
|----------|-------------|---------------|--------|------------|
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | OpenSSL cross-compilation | `cargo install dynamic-mcp` |
| macOS | Intel | `x86_64-apple-darwin` | GitHub Actions runner retired | Use Rosetta 2 or build from source |

### Planned for Future Release
- **Windows ARM64** (`aarch64-pc-windows-msvc`) - Planned for v1.1.0+
- **Linux ARM64 binaries** - Planned for v1.1.0+ (with vendored OpenSSL)

## 🔧 Technical Details

### Dependencies
- **MCP Protocol**: rmcp v0.12
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **OAuth2**: oauth2 crate with PKCE
- **114 total crates**

### Metrics
- **Lines of Code**: ~2,900 (Rust)
- **Source Files**: 17
- **Tests**: 46 (37 unit + 9 integration)
- **Test Coverage**: Config (100%), Auth (100%), Server (100%), Transport (100%)

## 📝 Known Limitations

### Platform Availability
- ARM64 Linux pre-built binaries not available due to OpenSSL cross-compilation complexity
- Windows ARM64 not yet supported (planned for future)
- macOS Intel binaries not included (deprecated GitHub Actions runners)

### Runtime Limitations
- Live reload works for config changes only (binary updates require restart)
- OAuth tokens stored as plain text in `~/.dynamic-mcp/oauth-servers/` (see SECURITY.md)
- No built-in rate limiting for tool calls
- Child processes inherit full privileges (no sandboxing)

## 🚀 Getting Started

1. **Install dynamic-mcp**:
   ```bash
   cargo install dynamic-mcp
   ```

2. **Create configuration** (`dynamic-mcp.json`):
   ```json
   {
     "mcpServers": {
       "filesystem": {
         "description": "Use when you need to read, write, or search files.",
         "command": "npx",
         "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
       }
     }
   }
   ```

3. **Configure your AI agent** to use dynamic-mcp:
   ```json
   {
     "mcpServers": {
       "dynamic-mcp": {
         "command": "dynamic-mcp",
         "args": ["/path/to/dynamic-mcp.json"]
       }
     }
   }
   ```

## 📚 Documentation

- [README](../../README.md) - Quick start and configuration
- [CONTRIBUTING](../../CONTRIBUTING.md) - Development setup
- [SECURITY](../../SECURITY.md) - Security considerations
- [ARCHITECTURE](../ARCHITECTURE.md) - System design
- [MIGRATION](../MIGRATION.md) - Migrating from standard MCP

## 🙏 Acknowledgments

- TypeScript implementation: [modular-mcp](https://github.com/d-kimuson/modular-mcp)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP Ecosystem: [rust-mcp-stack](https://github.com/rust-mcp-stack)

## 🔗 Links

- **crates.io**: https://crates.io/crates/dynamic-mcp
- **GitHub**: https://github.com/asyrjasalo/dynamic-mcp
- **Releases**: https://github.com/asyrjasalo/dynamic-mcp/releases
- **Documentation**: https://docs.rs/dynamic-mcp

---

**Full Changelog**: https://github.com/asyrjasalo/dynamic-mcp/commits/v1.0.0
