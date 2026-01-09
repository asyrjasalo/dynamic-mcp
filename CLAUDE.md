# Agent Guidelines - dynamic-mcp

> **For AI Coding Agents**: Complete guide for developing, testing, documenting, and releasing features in dynamic-mcp.

## 📋 Project Overview

**dynamic-mcp** is an MCP proxy server written in Rust that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading tool schemas on-demand.

### What It Does
- **Problem**: Exposing all MCP servers upfront consumes thousands of tokens with tool schemas
- **Solution**: Exposes only 2 proxy tools initially, loads tool schemas on-demand per group
- **Result**: Minimal initial context, full functionality preserved

### Key Features
- **Transports**: stdio (child processes), HTTP, SSE (server-sent events)
- **Authentication**: OAuth2 with PKCE, automatic token refresh, RFC 6749 compliant
- **Reliability**: Automatic retry with exponential backoff, periodic reconnection for failed servers
- **Live Reload**: Configuration file watching with automatic reconnection
- **Import**: Interactive command to import from 10 AI coding tools
- **MCP APIs**: Full proxying support for Tools, Resources, and Prompts APIs
- **Per-Server Features**: Optional feature flags to disable tools/resources/prompts per server
- **MCP Compliance**: 98.8% spec-compliant (84/86 requirements), production-ready

### Architecture
```
LLM Client → dynamic-mcp → Multiple Upstream MCP Servers
             (2 proxy tools + full MCP API)
             ├─ Tools: get_dynamic_tools, call_dynamic_tool
             ├─ Resources: list, read, templates/list
             └─ Prompts: list, get
                    ↓
             ├─ stdio: Local processes
             ├─ HTTP: Remote HTTP servers
             └─ SSE: Server-sent events
```

See [docs/implementation/ARCHITECTURE.md](docs/implementation/ARCHITECTURE.md) for detailed system design.

### Tech Stack
- **Language**: Rust 1.75+
- **MCP Protocol**: rmcp v0.12 (official Rust SDK)
- **Async Runtime**: Tokio
- **HTTP**: reqwest with streaming
- **OAuth**: oauth2 crate with PKCE
- **File Watching**: notify crate
- **CLI**: clap v4
- **Testing**: cargo test + integration tests

### Current Status
- **Version**: 1.3.0 (Per-Server Feature Flags)
- **Platforms**: Linux (x86_64, ARM64), macOS (ARM64), Windows (x86_64, ARM64)
- **Published**: [crates.io](https://crates.io/crates/dynamic-mcp), [PyPI](https://pypi.org/project/dmcp/), [GitHub Releases](https://github.com/asyrjasalo/dynamic-mcp/releases)

See [docs/implementation/STATUS.md](docs/implementation/STATUS.md) for current metrics (LOC, test counts, dependencies).

---

## 🛠️ Development Workflow

### Setup
```bash
# Build
cargo build

# Run tests
cargo test

# Run with example config
cargo run -- examples/config.example.json

# Debug mode
RUST_LOG=debug cargo run -- examples/config.example.json
```

### Before You Start
1. **Read relevant docs**: Check [ARCHITECTURE.md](docs/implementation/ARCHITECTURE.md) for system design
2. **Understand the codebase**: Browse module structure in `src/`
3. **Check existing patterns**: Look at similar implementations before adding new code
4. **Review tests**: See `tests/` and module tests for examples

### ⚠️ CRITICAL: No Git Commits Without Asking

**You MUST NOT commit changes to the repository without explicit permission from the project owner.** Only the project owner commits and pushes to the main branch.

Your responsibilities:
- ✅ Write code in your local working directory
- ✅ Run tests to verify functionality
- ✅ Update documentation
- ✅ Verify build passes (`cargo build --release`)
- ✅ **ASK FIRST** before running `git commit`
- ❌ **DO NOT** run `git commit` or `git push` without explicit approval
- ❌ **DO NOT** stage changes with `git add` for committing
- ❌ **DO NOT** push to any branches

The project owner will:
- Review your changes
- Decide whether to create commits
- Create appropriate commits with proper messages if approved
- Manage all git operations and releases

**Why this matters**: Maintains clean git history, proper attribution, version control integrity, and ensures the project owner has full control over what gets committed.

---

## ✨ Feature Development

### 1. Planning

**Before writing code:**
- [ ] Clearly define the feature requirements
- [ ] Check if it requires changes to:
  - Configuration schema (`src/config/schema.rs`)
  - MCP protocol handling (`src/server.rs`)
  - Transport layer (`src/proxy/transport.rs`)
  - Authentication (`src/auth/`)
- [ ] Identify which tests need updating/adding
- [ ] Plan documentation updates (see Documentation section)

### 2. Implementation

**Step-by-step process:**

1. **Write failing tests first (TDD approach)**
   ```bash
   # Add test cases for your feature
   # in appropriate test module or tests/ directory
   cargo test <test_name>  # Should fail initially
   ```

2. **Implement the feature**
   - Follow existing code patterns in the module
   - Use descriptive variable/function names
   - Add doc comments (`///`) for public APIs
   - Handle errors properly (avoid `unwrap()` in production code)

3. **Make tests pass**
   ```bash
   cargo test <test_name>  # Should pass now
   ```

4. **Run full test suite**
   ```bash
   cargo test              # All tests must pass
   ```

5. **Check code quality**
    ```bash
    cargo fmt               # Format code
    cargo clippy            # Lint and catch issues
    ```

6. **Verify with LSP diagnostics**
    ```bash
    # If using rust-analyzer
    # Check for warnings/errors in your editor
    ```

### 3. Testing Requirements

**⚠️ CRITICAL: Tests are MANDATORY for ALL code changes. No exceptions.**

**Exception: Documentation-only changes do NOT require running tests.**

**A feature is NOT complete until it has comprehensive tests.**

**All code changes must include:**

| Test Type | Location | When Required |
|-----------|----------|---------------|
| **Unit Tests** | Same file as code (`#[cfg(test)]` module) | ✅ **MANDATORY** for new functions/logic |
| **Integration Tests** | `tests/` directory | ✅ **MANDATORY** for CLI commands, end-to-end workflows |
| **Error Cases** | With unit tests | ✅ **MANDATORY** - Always test failure scenarios |
| **Edge Cases** | With unit tests | ✅ **MANDATORY** - Boundary conditions, empty inputs, etc. |

**No Pull Request will be accepted without:**
- [ ] Unit tests for all new functions/logic
- [ ] Integration tests for user-facing features
- [ ] Error case coverage
- [ ] Edge case coverage
- [ ] All tests passing (`cargo test` 100% pass rate)

**Example test structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_success_case() {
        // Arrange
        let input = setup_test_input();

        // Act
        let result = your_function(input);

        // Assert
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_feature_error_case() {
        // Test error handling
        let result = your_function_with_invalid_input();
        assert!(result.is_err());
    }
}
```

### 4. Performance Considerations

**For performance-sensitive code:**
- Add benchmarks in `benches/performance.rs`
- Run benchmarks before and after changes
  ```bash
  cargo bench --bench performance
  ```
- Document performance characteristics in code comments

---

## 🧪 Testing Guide

### Running Tests

```bash
# All tests (unit + integration)
cargo test

# Specific module
cargo test config::

# Specific test
cargo test test_substitute_env_vars

# With output visible
cargo test -- --nocapture

# Integration tests only
cargo test --test integration_test

# Single-threaded (for debugging)
cargo test -- --test-threads=1
```

### Test Coverage by Module

| Module | Unit Tests | Integration Tests | What's Tested |
|--------|-----------|-------------------|---------------|
| `config/` | ✅ Yes | ✅ CLI | Config parsing, env vars |
| `auth/` | ✅ Yes | ✅ OAuth flow | OAuth2, token storage |
| `proxy/` | ✅ Yes | N/A | Transport, group state |
| `server.rs` | ✅ Yes | N/A | MCP protocol, tool calls |
| `cli/` | ✅ Yes | ✅ Full | Import command |

### Adding New Tests

**When adding tests:**
1. Place unit tests in same file as implementation (`#[cfg(test)]` module)
2. Place integration tests in `tests/` directory
3. Use descriptive test names: `test_<feature>_<scenario>_<expected_result>()`
4. Clean up test resources (temp files, env vars) in test teardown
5. Use `#[should_panic]` for tests that verify panics

**After adding tests:**
1. Update [docs/implementation/TESTING.md](docs/implementation/TESTING.md) with new test counts
2. Update [docs/implementation/STATUS.md](docs/implementation/STATUS.md) metrics

---

## 📚 Documentation Requirements

**CRITICAL**: Documentation is code. Always update docs when making changes.

### What to Update

**For any code change:**
1. **[CHANGELOG.md](CHANGELOG.md)** - Add user-facing changes to Unreleased section (features, bug fixes, breaking changes)
2. **[docs/implementation/STATUS.md](docs/implementation/STATUS.md)** - Add new features, update metrics if significant
3. **[docs/implementation/TESTING.md](docs/implementation/TESTING.md)** - Update test counts after adding tests
4. **[docs/implementation/ARCHITECTURE.md](docs/implementation/ARCHITECTURE.md)** - Update if adding new modules or changing architecture
5. **[README.md](README.md)** - Update if user-facing (new features, config changes)

**For documentation-only changes:**
- Skip running tests (`cargo test`, `cargo clippy`, `cargo build`)
- Only verify cross-references and formatting

---

## 🚀 Release Process

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):
- **Major (x.0.0)**: Breaking changes to API or config schema
- **Minor (1.x.0)**: New features, backward compatible
- **Patch (1.0.x)**: Bug fixes, backward compatible

### Pre-Release Checklist

Before releasing a new version:

- [ ] **All tests pass**: `cargo test` (100% pass rate required)
- [ ] **No clippy warnings**: `cargo clippy -- -D warnings`
- [ ] **Code formatted**: `cargo fmt --check`
- [ ] **Documentation complete**:
  - [ ] README.md updated with new user-facing features
  - [ ] CHANGELOG.md has entry with user-facing changes only (no metrics/internal details)
  - [ ] API docs complete: `cargo doc --no-deps`
- [ ] **Version bumped** in `Cargo.toml`
- [ ] **Git tag prepared**: `git tag -a v1.x.x -m "Release v1.x.x"`

### Release Steps

**1. Update Version**
```bash
# Edit Cargo.toml and pyproject.toml
version = "1.x.x"
```

**2. Update CHANGELOG.md**
```markdown
## [1.x.x] - 2026-XX-XX

### Added
- User-facing feature descriptions (e.g., "Added WebSocket transport support")

### Changed
- User-facing behavior changes (e.g., "Changed default timeout to 30s")

### Fixed
- User-facing bug fixes (e.g., "Fixed OAuth token refresh failing after expiry")
```

**Note**: Do NOT include technical metrics, internal refactorings, or implementation details in CHANGELOG.md.

**3. Commit and Tag**
```bash
git add Cargo.toml pyproject.toml CHANGELOG.md
git commit -m "chore: bump version to 1.x.x"
git tag -a v1.x.x -m "Release v1.x.x"
git push origin main --tags
```

**4. GitHub Actions Handles**
- Automated tests on all platforms
- Cross-platform binary builds (Linux x86_64/ARM64, macOS ARM64, Windows x86_64/ARM64)
- Cross-platform Python wheel builds (5 platforms via maturin)
- GitHub Release creation with binaries and wheels attached
- crates.io publication (Rust package)
- PyPI publication (Python package)

**5. Verify Release**
- [ ] GitHub Release created: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/vX.X.X
- [ ] Binaries available for download (5 platforms)
- [ ] Python wheels available (5 platforms)
- [ ] crates.io updated: https://crates.io/crates/dynamic-mcp
- [ ] PyPI updated: https://pypi.org/project/dmcp/
- [ ] Documentation rendered correctly

### Post-Release

- [ ] Announce release (if applicable)


## 📏 Update Guidelines

### When to Update Metrics

**STATUS.md** should be updated when:
- LOC changes by >10%
- New module added or removed
- Dependencies change significantly

### When to Create New Docs

**Create new feature doc** in `docs/implementation/` when:
- Feature is complex enough to warrant dedicated documentation
- Feature has non-obvious implementation details worth preserving
- Multiple future changes expected to the feature
- Follow naming: `FEATURE_NAME.md` (e.g., `WEBSOCKET_TRANSPORT.md`)

**Example new feature doc structure:**
```markdown
# Feature Name

**Date**: [Date implemented]
**Status**: ✅ Complete / 🚧 In Progress

## Overview
[What it does and why]

## Implementation
[Key technical details]

## Usage
[Code examples if applicable]

## Testing
[Test approach and key test cases]

## Future Enhancements
[Known limitations or future work]
```

### What NOT to Update

**Do NOT modify:**
- **Historical documentation** (PHASE*_COMPLETE.md, PLAN.md, RESEARCH.md)
- **Previous release entries in CHANGELOG.md** - Only add new releases at the top, never modify historical entries

These are historical records and should remain unchanged.

**CHANGELOG.md Guidelines:**
- ✅ Add new release entries at the top (after the header)
- ✅ Follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format
- ✅ Focus on **user-facing changes**: features, bug fixes, breaking changes
- ✅ **Order matters**: List user-facing changes FIRST, technical details LAST within each section
- ✅ **No repetition across sections**: Each change should appear in ONLY ONE section (Added, Changed, or Fixed)
- ✅ **Choose the primary aspect**: If a change fits multiple categories, pick the most significant one
- ❌ NEVER document technical metrics (LOC, test counts, dependencies)
- ❌ NEVER document internal implementation details (refactorings, module structure)
- ❌ NEVER modify entries for previous releases (1.0.0, 1.1.0, etc.)
- ❌ NEVER update historical descriptions even if they're outdated
- ❌ NEVER repeat the same change in multiple sections (e.g., don't list a new feature in both "Added" and "Changed")
- Historical accuracy is more important than current correctness for past releases

**What to include in CHANGELOG.md**:
- ✅ New user-facing features ("Added OAuth2 authentication support")
- ✅ Bug fixes affecting users ("Fixed token refresh failing after expiry")
- ✅ Breaking changes ("Removed support for legacy config format")
- ✅ Deprecation notices ("Deprecated `--old-flag`, use `--new-flag` instead")

**⚠️ CRITICAL: DO NOT INCLUDE TEST METRICS IN CHANGELOG**
- ❌ NEVER mention test metrics
- ✅ Tests are implementation details, not user-facing features
- Put test documentation in STATUS.md and TESTING.md instead

**Ordering within each section**:
1. **User-facing changes FIRST** (CLI flags, new commands, behavior changes)
2. **Technical modules LAST** (internal modules, parsers, detection logic)

**Example**:
```markdown
### Added
- **Multi-Tool Import Support** - Main user-facing feature
- **Enhanced CLI** - --global and --force flags
- **Config Parser Module** - Technical implementation detail (goes last)
```

**What NOT to include** (goes in STATUS.md instead):
- ❌ Technical metrics (LOC, test counts, dependency counts)
- ❌ Internal refactorings (module restructuring)
- ❌ Dependency updates
- ❌ Build process changes

## 🎓 Best Practices

### 1. Keep Docs Synchronized
When updating one doc, check if related docs need updates. For example:
- README config example → STATUS.md features list
- New module → docs/implementation/ARCHITECTURE.md structure + STATUS.md module list

### 2. Use Consistent Terminology
- **Transport**: stdio, HTTP, SSE, WebSocket
- **Module**: config, proxy, server, cli, auth
- **Tool**: get_dynamic_tools, call_dynamic_tool

### 3. Update Timestamps
Add "Last Updated: [Date]" when making significant updates to:
- STATUS.md
- TESTING.md
- Feature-specific docs

### 4. Preserve Examples
When updating README.md or IMPORT.md:
- Keep examples functional and tested
- Update examples if config schema changes
- Add new examples for new features

### 5. Maintain Accuracy
- Test counts are only tracked in TESTING.md
- LOC should match actual source code
- Feature lists should reflect implemented code, not planned features

## 📊 Current Project State (Reference)

**Modules**: config, proxy, server, cli, auth, watcher
**Transports**: stdio, HTTP, SSE
**MCP APIs**: Tools, Resources, Prompts
**Key Features**: OAuth2, Live Reload, Import (10 tools), Feature Flags, CI/CD

**Where to find details:**
- Implemented features → **docs/implementation/STATUS.md**
- Test coverage → **docs/implementation/TESTING.md**
- Architecture → **docs/implementation/ARCHITECTURE.md**
- User guide → **README.md**
- Development setup → **CONTRIBUTING.md**

---

**Remember**: Documentation is code. Keep it accurate, up-to-date, and helpful for both humans and AI agents.
