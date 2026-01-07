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
- **Migration**: Interactive command to migrate from standard MCP configs

### Architecture
```
LLM Client → dynamic-mcp (2 tools) → Multiple Upstream MCP Servers
                                      ├─ stdio: Local processes
                                      ├─ HTTP: Remote HTTP servers
                                      └─ SSE: Server-sent events
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed system design.

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
- **Version**: 1.0.0 (Production Release)
- **Phase**: Phase 6 Complete ✅
- **LOC**: ~2,900 Rust
- **Tests**: 46 (37 unit + 9 integration), 100% pass rate
- **Platforms**: Linux (x86_64, ARM64), macOS (ARM64), Windows (x86_64, ARM64)
- **Published**: [crates.io](https://crates.io/crates/dynamic-mcp), [GitHub Releases](https://github.com/asyrjasalo/dynamic-mcp/releases)

---

## 🛠️ Development Workflow

### Setup
```bash
# Clone and build
git clone https://github.com/asyrjasalo/dynamic-mcp.git
cd dynamic-mcp
cargo build

# Run tests
cargo test

# Run with example config
cargo run -- examples/config.example.json

# Debug mode
RUST_LOG=debug cargo run -- examples/config.example.json
```

### Before You Start
1. **Read relevant docs**: Check [ARCHITECTURE.md](docs/ARCHITECTURE.md) for system design
2. **Understand the codebase**: Browse module structure in `src/`
3. **Check existing patterns**: Look at similar implementations before adding new code
4. **Review tests**: See `tests/` and module tests for examples

---

## ✨ Feature Development

### 1. Planning Phase

**Before writing code:**
- [ ] Clearly define the feature requirements
- [ ] Check if it requires changes to:
  - Configuration schema (`src/config/schema.rs`)
  - MCP protocol handling (`src/server.rs`)
  - Transport layer (`src/proxy/transport.rs`)
  - Authentication (`src/auth/`)
- [ ] Identify which tests need updating/adding
- [ ] Plan documentation updates (see Documentation section)

### 2. Implementation Phase

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
   cargo test              # All 46+ tests must pass
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

**⚠️ CRITICAL: Tests are MANDATORY for ALL new features. No exceptions.**

**A feature is NOT complete until it has comprehensive tests.**

**All features must include:**

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

| Module | Unit Tests | Integration Tests | Coverage |
|--------|-----------|-------------------|----------|
| `config/` | ✅ 100% | ✅ CLI | Config parsing, env vars |
| `auth/` | ✅ 100% | ✅ OAuth flow | OAuth2, token storage |
| `proxy/` | ✅ 100% | N/A | Transport, group state |
| `server.rs` | ✅ 100% | N/A | MCP protocol, tool calls |
| `cli/` | ✅ Basic | ✅ Full | Migrate command |

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

### When to Update Docs

**CRITICAL**: Documentation is code. Always update docs when making changes.

### Documentation Structure

**User-Facing Documentation (Root)**
- **README.md**: Quick start, usage, configuration examples
- **CONTRIBUTING.md**: Development setup, testing, PR workflow
- **AGENTS.md**: This file - AI agent guidelines

**Architecture Documentation (docs/)**
- **ARCHITECTURE.md**: System design, data flows, component details
- **MIGRATION.md**: Migration guide from standard MCP

**Implementation Documentation (docs/implementation/)**
- **STATUS.md**: Current metrics, features, limitations
- **TESTING.md**: Test coverage, running tests
- **DEVELOPMENT.md**: Project status, roadmap, metrics
- **Feature docs**: Complex features (ENV_VAR_CONFIG.md, LIVE_RELOAD.md)

### Documentation Update Matrix

| Change Type | README | ARCHITECTURE | STATUS | TESTING | CONTRIBUTING |
|-------------|--------|--------------|--------|---------|--------------|
| New user-facing feature | ✅ | ⚠️ | ✅ | ✅ | ❌ |
| New internal feature | ❌ | ⚠️ | ✅ | ✅ | ❌ |
| Bug fix | ⚠️ | ❌ | ⚠️ | ⚠️ | ❌ |
| Config schema change | ✅ | ❌ | ⚠️ | ❌ | ❌ |
| New tests | ❌ | ❌ | ✅ | ✅ | ⚠️ |
| Build process change | ❌ | ❌ | ⚠️ | ❌ | ✅ |
| New module | ❌ | ✅ | ✅ | ✅ | ❌ |
| Phase completion | ❌ | ❌ | ✅ | ⚠️ | ❌ |

Legend: ✅ Always | ⚠️ If applicable | ❌ Rarely

### Documentation Checklist

Before completing any feature, verify:

- [ ] User-facing changes reflected in README.md
- [ ] New features added to STATUS.md
- [ ] Test count accurate in TESTING.md and STATUS.md
- [ ] Architecture changes documented in ARCHITECTURE.md
- [ ] Build/test process changes in CONTRIBUTING.md
- [ ] Metrics updated if changed significantly (LOC, tests, dependencies)
- [ ] Cross-references between docs still valid
- [ ] Examples still work and are accurate

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
  - [ ] README.md updated with new features
  - [ ] CHANGELOG.md has entry for this version
  - [ ] STATUS.md metrics updated
  - [ ] API docs complete: `cargo doc --no-deps`
- [ ] **Version bumped** in `Cargo.toml`
- [ ] **Git tag prepared**: `git tag -a v1.x.x -m "Release v1.x.x"`

### Release Steps

**1. Update Version**
```bash
# Edit Cargo.toml
version = "1.x.x"

# Update STATUS.md
## 📊 Project Metrics
| **Version** | 1.x.x |
```

**2. Update CHANGELOG.md**
```markdown
## [1.x.x] - 2026-XX-XX

### Added
- New feature descriptions

### Changed
- Changed feature descriptions

### Fixed
- Bug fix descriptions
```

**3. Commit and Tag**
```bash
git add Cargo.toml pyproject.toml CHANGELOG.md docs/implementation/STATUS.md
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

- [ ] Create Phase Completion doc (if completing a phase):
  ```bash
  # Create docs/implementation/PHASEX_COMPLETE.md
  # Document objectives achieved, metrics, deployment details
  ```
- [ ] Update DEVELOPMENT.md roadmap if phase completed
- [ ] Announce release (if applicable)

---

## 🔄 Update Workflow by Change Type

## 📚 Documentation Structure

### User-Facing Documentation (Root)

| File | Purpose | Update When |
|------|---------|-------------|
| **README.md** | Quick start, usage, configuration examples | Adding/changing user-facing features, configuration schema changes |
| **CONTRIBUTING.md** | Development setup, testing, PR workflow | Changing build process, adding new test requirements, updating contribution guidelines |
| **AGENTS.md** | AI agent guidelines for documentation maintenance | Changing doc structure, adding new doc types |

### Architecture Documentation (docs/)

| File | Purpose | Update When |
|------|---------|-------------|
| **ARCHITECTURE.md** | System design, data flows, component details | Adding new modules, changing core architecture, modifying data flows |
| **MIGRATION.md** | Migration guide from standard MCP | Changing config format, adding new migration features |

### Implementation Documentation (docs/implementation/)

#### Current Status
| File | Purpose | Update When |
|------|---------|-------------|
| **DEVELOPMENT.md** | Project status, roadmap, metrics | Completing phases, major milestones, significant metric changes |
| **STATUS.md** | Current metrics, features, limitations | Phase completion, major feature additions, metric updates (LOC, tests) |
| **TESTING.md** | Test coverage, running tests | Adding test categories, changing test commands, significant coverage changes |

#### Feature Documentation
| File | Purpose | Update When |
|------|---------|-------------|
| **ENV_VAR_CONFIG.md** | Environment variable implementation | Changing env var behavior, adding new env vars |
| **LIVE_RELOAD.md** | Live reload feature details | Modifying file watching, changing reload behavior |

#### Historical Records (DO NOT MODIFY)
- **PHASE1_COMPLETE.md** through **PHASE5_COMPLETE.md** - Historical snapshots
- **PLAN.md** - Original implementation plan
- **RESEARCH.md** - Initial SDK research

## 🔄 Update Workflow by Change Type

### 1. Adding a New Feature

**Required Updates:**
1. ✅ **README.md** - Add usage example if user-facing
2. ✅ **STATUS.md** - Add to "Completed Features" section
3. ✅ **TESTING.md** - Add test coverage info if tests added
4. ⚠️ **ARCHITECTURE.md** - Only if adds new module or changes architecture

**Example:**
```markdown
Feature: WebSocket transport support

Update README.md:
- Add WebSocket config example under "Configuration Schema"
- Add troubleshooting section if needed

Update STATUS.md:
- Add "WebSocket transport" under Phase X features
- Update metrics (LOC, if significant)

Update TESTING.md:
- Add WebSocket transport tests to coverage table

Update ARCHITECTURE.md (if needed):
- Document WebSocket transport creation in proxy/ module
```

### 2. Fixing a Bug

**⚠️ CRITICAL: Regression tests are MANDATORY for ALL bug fixes. No exceptions.**

**A bug fix is NOT complete until it has tests that verify:**
- [ ] The bug is actually fixed (test that previously failed now passes)
- [ ] The fix doesn't break existing functionality (all existing tests still pass)
- [ ] The bug cannot easily regress (test covers the specific failure case)

**Required Updates:**
1. ⚠️ **README.md** - Only if changes user-facing behavior
2. ⚠️ **STATUS.md** - Only if removes known limitation
3. ✅ **TESTING.md** - Always when adding regression tests (which is MANDATORY)

**Example:**
```markdown
Fix: OAuth token refresh failing

Add regression test:
- Write test that reproduces the original bug
- Verify test fails without the fix
- Verify test passes with the fix

Update STATUS.md:
- Remove "OAuth refresh token rotation not implemented" from Known Limitations
- Update if fix adds new capability

Update TESTING.md:
- Document new OAuth refresh tests
```

### 3. Refactoring (No Behavior Change)

**Required Updates:**
1. ⚠️ **ARCHITECTURE.md** - Only if module structure changes
2. ⚠️ **STATUS.md** - Only if LOC changes significantly (>10%)

**Example:**
```markdown
Refactor: Split server.rs into server/ module

Update ARCHITECTURE.md:
- Update module structure diagram
- Document new file organization

Update STATUS.md:
- Update "Module Structure" section
```

### 4. Completing a Phase

**Required Updates:**
1. ✅ **docs/implementation/DEVELOPMENT.md** - Update phase status and roadmap
2. ✅ **docs/implementation/STATUS.md** - Update current phase, add completed features
3. ✅ **Create docs/implementation/PHASEx_COMPLETE.md** - Document completion details

**Example:**
```markdown
Complete: Phase 6 (Production Release)

Create docs/implementation/PHASE6_COMPLETE.md:
- Document all objectives achieved
- Include final metrics
- Note deployment details

Update docs/implementation/DEVELOPMENT.md:
- Mark Phase 6 as complete in roadmap
- Update "Current Phase" section

Update docs/implementation/STATUS.md:
- Update "Current Phase" header
- Add Phase 6 features to completed list
- Update project metrics
```

### 5. Changing Configuration Schema

**Required Updates:**
1. ✅ **README.md** - Update configuration examples
2. ✅ **MIGRATION.md** - If affects migration from standard MCP
3. ⚠️ **STATUS.md** - If adds significant new capability

**Example:**
```markdown
Change: Add timeout configuration for servers

Update README.md:
- Add timeout field to config schema examples
- Document timeout behavior

Update MIGRATION.md:
- Document how timeouts are handled during migration
- Update example output

Update STATUS.md (if major):
- Add timeout support to features list
```

### 6. Adding Tests

**Required Updates:**
1. ✅ **TESTING.md** - Update test counts and coverage
2. ✅ **STATUS.md** - Update test metrics
3. ⚠️ **CONTRIBUTING.md** - Only if adds new test category or requirements

**Example:**
```markdown
Add: WebSocket transport integration tests

Update TESTING.md:
- Add "WebSocket Module" to coverage table
- Update test counts (unit + integration)
- Document how to run WebSocket tests

Update STATUS.md:
- Update "Tests" metric (37 → 42)
- Update test coverage percentage if measured
```

### 7. Changing Build/Development Process

**Required Updates:**
1. ✅ **CONTRIBUTING.md** - Update build/test commands
2. ⚠️ **STATUS.md** - Only if affects dependencies count

**Example:**
```markdown
Change: Add Docker development environment

Update CONTRIBUTING.md:
- Add Docker setup instructions
- Document Docker-based testing

Update STATUS.md (if applicable):
- Update dependencies count if Dockerfile adds build deps
```

## 🎯 Quick Reference: Common Scenarios

| Scenario | README | CONTRIBUTING | DEVELOPMENT.md | STATUS.md | TESTING.md | ARCHITECTURE.md |
|----------|--------|--------------|----------------|-----------|------------|-----------------|
| New user-facing feature | ✅ | ❌ | ❌ | ✅ | ✅ | ⚠️ |
| New internal feature | ❌ | ❌ | ❌ | ✅ | ✅ | ⚠️ |
| Bug fix | ⚠️ | ❌ | ❌ | ⚠️ | ⚠️ | ❌ |
| Refactoring | ❌ | ❌ | ❌ | ⚠️ | ❌ | ⚠️ |
| Config schema change | ✅ | ❌ | ❌ | ⚠️ | ❌ | ❌ |
| Test additions | ❌ | ⚠️ | ❌ | ✅ | ✅ | ❌ |
| Phase completion | ❌ | ❌ | ✅ | ✅ | ⚠️ | ❌ |
| Build process change | ❌ | ✅ | ❌ | ⚠️ | ❌ | ❌ |
| New module | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ |

Note: DEVELOPMENT.md, STATUS.md, TESTING.md are in docs/implementation/

Legend: ✅ Always update | ⚠️ Update if applicable | ❌ Usually no update

## 📏 Update Guidelines

### When to Update Metrics

**STATUS.md** should be updated when:
- LOC changes by >10% (currently ~2,900)
- Test count changes (currently 46 total)
- New module added or removed
- Dependencies change significantly (currently 114 crates)
- Test coverage changes significantly

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
[Test coverage]

## Future Enhancements
[Known limitations or future work]
```

### What NOT to Update

**Do NOT modify:**
- **Historical phase completion docs** (PHASE1-5_COMPLETE.md)
- **PLAN.md** - Original planning document
- **RESEARCH.md** - Initial research document

These are historical records and should remain unchanged.

## 🔍 Verification Checklist

Before completing changes, verify:

- [ ] User-facing changes reflected in README.md
- [ ] New features added to STATUS.md
- [ ] Test count accurate in TESTING.md and STATUS.md
- [ ] Architecture changes documented in ARCHITECTURE.md
- [ ] Build/test process changes in CONTRIBUTING.md
- [ ] Metrics updated if changed significantly
- [ ] Cross-references between docs still valid

## 🎓 Best Practices

### 1. Keep Docs Synchronized
When updating one doc, check if related docs need updates. For example:
- README config example → STATUS.md features list
- New module → ARCHITECTURE.md structure + STATUS.md module list

### 2. Use Consistent Terminology
- **Transport**: stdio, HTTP, SSE, WebSocket
- **Module**: config, proxy, server, cli, auth
- **Phase**: Phase 1-6 (as per PLAN.md)
- **Tool**: get_dynamic_tools, call_dynamic_tool

### 3. Update Timestamps
Add "Last Updated: [Date]" when making significant updates to:
- STATUS.md
- TESTING.md
- Feature-specific docs

### 4. Preserve Examples
When updating README.md or MIGRATION.md:
- Keep examples functional and tested
- Update examples if config schema changes
- Add new examples for new features

### 5. Maintain Accuracy
- Test counts must match `cargo test` output
- LOC should match actual source code
- Feature lists should reflect implemented code, not planned features

## 📊 Current Project State (Reference)

**Phase**: Phase 6 Complete ✅
**Version**: 1.0.0 (Production Release)
**LOC**: ~2,900 Rust
**Tests**: 46 (37 unit + 9 integration)
**Modules**: config, proxy, server, cli, auth
**Transports**: stdio, HTTP, SSE
**Key Features**: OAuth2, Live Reload, Migration Command, CI/CD

**Where to find details:**
- Project status → **docs/implementation/DEVELOPMENT.md**
- Implementation status → **docs/implementation/STATUS.md**
- Testing → **docs/implementation/TESTING.md**
- Architecture → **docs/ARCHITECTURE.md**
- User guide → **README.md**
- Development setup → **CONTRIBUTING.md**

---

**Remember**: Documentation is code. Keep it accurate, up-to-date, and helpful for both humans and AI agents.
