# dynamic-mcp Knowledge Base

**Generated:** 2026-02-14
**Commit:** addab19
**Branch:** main

## Overview

MCP proxy server written in Rust that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading tool schemas on-demand.

- **Language:** Rust 1.75+ (edition 2021)
- **MCP SDK:** rmcp v0.12
- **Transports:** stdio, HTTP, SSE
- **Distributions:** Binary (crates.io) + Python wheel (PyPI via maturin)

## Structure

```text
./
├── src/               # Rust source (auth, cli, config, proxy, server, watcher)
├── tests/             # Integration tests (8 files, ~120 tests)
├── docs/implementation/   # Architecture, status, testing docs
├── benches/           # Performance benchmarks
├── examples/          # Example config files
├── Cargo.toml         # Binary + benchmark targets
└── pyproject.toml     # Python wheel config
```

## Code Map

| Symbol | Type | Location | Refs | Role |
|--------|------|----------|------|------|
| main | fn | src/main.rs | - | CLI entry point |
| serve | fn | src/server.rs | - | MCP server core |
| run_import | fn | src/cli/import_enhanced.rs | - | Import command |
| load_config | fn | src/config/loader.rs | - | Config loading |

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Config schema | src/config/schema.rs | JsonSchema generation |
| OAuth flow | src/auth/ | OAuth2 client + token store |
| MCP proxy | src/proxy/ | Client, transport, types |
| CLI commands | src/cli/ | Import, config parsing |
| Tests | tests/ | Integration tests |
| Benchmarks | benches/performance.rs | Performance tests |

## Conventions

- **Formatting:** `cargo fmt` (default rustfmt)
- **Linting:** `cargo clippy -- -D warnings` (warnings = errors)
- **Tests:** Unit tests inline (`#[cfg(test)]`), integration in `tests/`
- **Naming:** `test_<category>_<feature>_<scenario>`
- **TDD:** Write failing tests BEFORE implementation
- **Docs:** Update CHANGELOG.md, STATUS.md, TESTING.md after changes

## Anti-Patterns (This Project)

- **NEVER** commit without explicit permission - ask first
- **NEVER** use `unwrap()` in production code
- **NEVER** skip tests - 100% pass required
- **NEVER** document test metrics in CHANGELOG

## Commands

```bash
# Build
cargo build

# Test
cargo test --all-features

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt -- --check

# Release build
cargo build --release

# Benchmarks
cargo bench --bench performance
```

## Unique Patterns

1. **Dual packaging:** Rust binary + Python wheel via maturin
2. **Multi-tool import:** Native import from 10+ AI coding tools
3. **JSON Schema sync:** config-schema.json must match src/config/schema.rs
4. **Feature flags:** Per-server control (tools, resources, prompts)
5. **OAuth token storage:** ~/.dynamic-mcp/oauth-servers/

## Notes

- No lib.rs - binary-only crate
- Uses rmcp (not official MCP SDK)
- Cross-compiles ARM64 Linux via cross-rs
- Pre-commit validates config JSON against schema
