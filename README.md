# Modular MCP (Rust Implementation)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

A Rust implementation of Modular MCP - an MCP proxy server that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading schemas on-demand.

## 🎯 Project Status

**Current Phase**: Phase 2 (HTTP/SSE Transport) ✅ **COMPLETE**  
**Next Phase**: Phase 3 (OAuth Authentication)

### ✅ Phase 1 Completed (100%)
- Project structure and build system
- Configuration schema with JSON support
- Environment variable substitution (`${VAR}` syntax)
- Module organization (config, proxy, server, cli)
- Type definitions for MCP protocol
- **MCP server with JSON-RPC 2.0 protocol**
- **Stdio transport for upstream servers**
- **Client connection management**
- **Two-tool API (get-modular-tools, call-modular-tool)**
- **Parallel upstream server connections**
- **Error handling and graceful degradation**
- Example configuration files
- Comprehensive documentation
- Integration tests

### ✅ Phase 2 Completed (100%)
- **HTTP transport support** using rmcp StreamableHttpClientTransport
- **SSE transport support** using rmcp StreamableHttpClientTransport
- **Unified Transport enum** supporting stdio, HTTP, and SSE
- **Native Rust implementation** (no npx/mcp-remote dependency)
- **Header support** for HTTP/SSE (Authorization, custom headers)
- **Async request/response** handling for all transport types
- Integration with rmcp v0.12 official MCP Rust SDK

### 📅 Roadmap
- [x] Phase 1: Core proxy with stdio transport ✅ **COMPLETE**
- [x] Phase 2: HTTP/SSE transport support ✅ **COMPLETE**
- [ ] Phase 3: OAuth authentication
- [ ] Phase 4: Migration command
- [ ] Phase 5: Tests & documentation
- [ ] Phase 6: Production release

## 📖 Documentation

- **[Implementation Plan](docs/PLAN.md)** - Complete 6-phase implementation roadmap
- **[Research](docs/RESEARCH.md)** - Rust MCP SDK ecosystem research
- **[TypeScript Reference](https://github.com/d-kimuson/dynamic-mcp)** - Original implementation

## 🏗️ Architecture

```
dynamic-mcp/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── server.rs            # MCP server (exposes 2 tools)
│   ├── config/              # Configuration management
│   │   ├── schema.rs        # Config data structures
│   │   ├── loader.rs        # File loading & validation
│   │   └── env_sub.rs       # Environment variable substitution
│   ├── proxy/               # Upstream server management
│   │   ├── types.rs         # Shared types
│   │   ├── client.rs        # Group state management
│   │   └── transport.rs     # Transport creation
│   └── cli/                 # CLI commands
│       └── migrate.rs       # Config migration
├── docs/                    # Documentation
├── config.example.json      # Example configuration
└── Cargo.toml              # Dependencies
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.75 or higher
- Cargo

### Build

```bash
cargo build --release
```

### Run

You can specify the configuration file in two ways:

**Option 1: Command line argument**
```bash
cargo run -- config.example.json
# or with the binary
./target/release/dynamic-mcp config.example.json
```

**Option 2: Environment variable**
```bash
export GATEWAY_MCP_CONFIG=config.example.json
cargo run
# or with the binary
./target/release/dynamic-mcp
```

**Note**: Command line argument takes precedence over environment variable if both are provided.

### Configuration

Create a `dynamic-mcp.json` file:

```json
{
  "mcpServers": {
    "filesystem": {
      "type": "stdio",
      "description": "Use when you need to read, write, or search files.",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    }
  }
}
```

### Environment Variables

Supports `${VAR}` syntax for environment variable interpolation:

```json
{
  "mcpServers": {
    "example": {
      "type": "stdio",
      "description": "Example with env vars",
      "command": "node",
      "args": ["${HOME}/.local/bin/server.js"],
      "env": {
        "API_KEY": "${MY_API_KEY}"
      }
    }
  }
}
```

## 🔧 Configuration Schema

### Server Types

#### stdio (Default)
```json
{
  "type": "stdio",
  "description": "Server description for LLM",
  "command": "npx",
  "args": ["-y", "package-name"],
  "env": {
    "KEY": "value"
  }
}
```

#### http
```json
{
  "type": "http",
  "description": "HTTP server",
  "url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### sse
```json
{
  "type": "sse",
  "description": "SSE server",
  "url": "https://api.example.com/sse",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

## 🧪 Testing

```bash
# Run tests
cargo test

# Run with test coverage
cargo test -- --test-threads=1

# Run specific test
cargo test test_substitute_env_vars
```

## 📝 Development

### Code Structure

- **config/**: Configuration loading, validation, and environment variable substitution
- **proxy/**: MCP client management, group state tracking, transport creation
- **server/**: MCP server that exposes the two-tool API
- **cli/**: Command-line interface and migration tools

### Key Features

1. **Environment Variable Substitution**
   - Supports `${VAR}` syntax only
   - Warns on undefined variables
   - Preserves placeholders for undefined vars

2. **Type Safety**
   - Serde-based JSON validation
   - JSON Schema support
   - Strongly-typed configurations

3. **Modular Design**
   - Clean separation of concerns
   - Easy to extend with new transports
   - Testable components

## 🤝 Contributing

This is a work in progress! Contributions are welcome.

### Development Setup

1. Clone the repository
2. Install Rust (1.75+)
3. Run `cargo build`
4. Make changes
5. Run `cargo test`
6. Submit PR

## 📄 License

MIT License - see [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

- Original TypeScript implementation: [d-kimuson/dynamic-mcp](https://github.com/d-kimuson/dynamic-mcp)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP Ecosystem: [rust-mcp-stack](https://github.com/rust-mcp-stack)

## 📊 Project Metrics

- **Lines of Code**: ~1,500 (Rust)
- **Dependencies**: 114 crates (including rmcp and HTTP/SSE stack)
- **Tests**: 7 passing (4 unit + 3 integration)
- **Test Coverage**: Config: 100%, Core: Working
- **Documentation**: Comprehensive

---

**Status**: ✅ Phase 2 Complete | Ready for Phase 3
