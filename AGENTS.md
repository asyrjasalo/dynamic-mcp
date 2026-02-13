# Agent Guidelines - dynamic-mcp

> **For AI Coding Agents**: Complete guide for developing, testing, documenting, and releasing features in dynamic-mcp.

## 📋 Project Overview

**dynamic-mcp** is an MCP proxy server written in Rust that reduces LLM context overhead by grouping tools from multiple upstream MCP servers and loading tool schemas on-demand.

### Quick Summary

- **Problem**: Exposing all MCP servers upfront consumes thousands of tokens
- **Solution**: Exposes only 2 proxy tools initially, loads schemas on-demand
- **Result**: Minimal initial context, full functionality preserved

### Key Information

- **Current Version**: 1.3.0
- **Language**: Rust 1.75+
- **MCP SDK**: rmcp v0.12
- **Transports**: stdio, HTTP, SSE
- **MCP APIs**: Tools, Resources, Prompts (all fully proxied)

**For detailed information, see:**

- Architecture & Design: @docs/implementation/ARCHITECTURE.md
- Implementation Status: @docs/implementation/STATUS.md
- Test Coverage: @docs/implementation/TESTING.md

______________________________________________________________________

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

**⚠️ MANDATORY: Always read documentation BEFORE implementing, and update it AFTER.**

1. **Read relevant docs FIRST**: @docs/implementation/ARCHITECTURE.md, @docs/implementation/STATUS.md, @docs/implementation/TESTING.md
2. **Understand the codebase**: Browse module structure in `src/` (see ARCHITECTURE.md)
3. **Check existing patterns**: Look at similar implementations before adding new code
4. **Review tests**: See @docs/implementation/TESTING.md for test organization
5. **After implementing**: Update all affected documentation (see Documentation Requirements section)

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

______________________________________________________________________

## ✨ Feature Development

### 1. Planning

**Before writing code:**

- [ ] **Read documentation FIRST** - Review @docs/implementation/ARCHITECTURE.md, @docs/implementation/STATUS.md, @docs/implementation/TESTING.md
- [ ] Clearly define the feature requirements
- [ ] Identify affected modules (see @docs/implementation/ARCHITECTURE.md for module structure)
- [ ] Identify which tests need updating/adding (see @docs/implementation/TESTING.md)
- [ ] Plan documentation updates (see Documentation Requirements section below)

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

**⚠️ CRITICAL: Tests are MANDATORY for ALL code changes (changes to Rust source files).**

**Note: Documentation-only changes (changes to .md files) do NOT require running tests.**

**A feature is NOT complete until it has comprehensive tests.**

**All code changes must include:**

| Test Type             | Location                                  | When Required                                              |
| --------------------- | ----------------------------------------- | ---------------------------------------------------------- |
| **Unit Tests**        | Same file as code (`#[cfg(test)]` module) | ✅ **MANDATORY** for new functions/logic                   |
| **Integration Tests** | `tests/` directory                        | ✅ **MANDATORY** for CLI commands, end-to-end workflows    |
| **Error Cases**       | With unit tests                           | ✅ **MANDATORY** - Always test failure scenarios           |
| **Edge Cases**        | With unit tests                           | ✅ **MANDATORY** - Boundary conditions, empty inputs, etc. |

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

______________________________________________________________________

## 🧪 Testing Guide

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_substitute_env_vars

# With output visible
cargo test -- --nocapture

# Single-threaded (for debugging)
cargo test -- --test-threads=1
```

**For test organization, coverage, and detailed commands, see @docs/implementation/TESTING.md.**

### Adding New Tests

**When adding tests:**

1. Place unit tests in same file as implementation (`#[cfg(test)]` module)
2. Place integration tests in `tests/` directory
3. Use descriptive test names: `test_<feature>_<scenario>_<expected_result>()`
4. See @docs/implementation/TESTING.md for test structure and examples

**After adding tests:**

1. Update @docs/implementation/TESTING.md with new test counts
2. Update @docs/implementation/STATUS.md if significant

______________________________________________________________________

## 📚 Documentation Requirements

**CRITICAL**: Documentation is code. Always update docs when making changes.

**Required workflow:**

1. **BEFORE implementing** - Read @docs/implementation/ARCHITECTURE.md, @docs/implementation/STATUS.md, @docs/implementation/TESTING.md
2. **Implement your changes** - Write code and tests
3. **AFTER implementing** - Update all affected documentation listed below

### What to Update

**For any code change, you MUST update:**

1. **@CHANGELOG.md** - Add user-facing changes to Unreleased section (features, bug fixes, breaking changes)
2. **@docs/implementation/STATUS.md** - Add new features, update metrics if significant
3. **@docs/implementation/TESTING.md** - Update test counts after adding tests
4. **@docs/implementation/ARCHITECTURE.md** - Update if adding new modules or changing architecture
5. **@config-schema.json** - Keep in sync with `src/config/schema.rs` (JSON Schema Draft 7 format)
6. **@README.md** - Update if user-facing (new features, config changes)

### Planning Documents

**⚠️ IMPORTANT**: Do NOT update current implementation docs (STATUS.md, ARCHITECTURE.md) to reference planning documents.

- Documents in `docs/implementation/plans/` are **planning/research only**
- They are **not implemented features** and should not be checked off in STATUS.md
- They should **not** be referenced as implemented in ARCHITECTURE.md
- Only update implementation docs when features are actually implemented and merged

**Critical**: The `config-schema.json` file is the source of truth for config validation exposed to users. Any changes to `McpServerConfig` struct must be reflected in the JSON schema immediately.

**For documentation-only changes:**

- Skip running tests (`cargo test`, `cargo clippy`, `cargo build`)
- Only verify grammar, cross-references and formatting is correct

______________________________________________________________________

## 🚀 Release Process

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **Major (x.0.0)**: Breaking changes to API or config schema
- **Minor (1.x.0)**: New features, backward compatible
- **Patch (1.0.x)**: Bug fixes, backward compatible

### Pre-Release Checklist

Before releasing a new version:

- [ ] **Code formatted**: `cargo fmt --check`
- [ ] **All tests pass**: `cargo test` (100% pass rate required)
- [ ] **No clippy warnings**: `cargo clippy -- -D warnings`
- [ ] **Documentation complete**:
  - [ ] README.md updated with new user-facing features
  - [ ] CHANGELOG.md has entry with user-facing changes only (no metrics/internal details)
  - [ ] API docs complete: `cargo doc --no-deps`
- [ ] **Version bumped** in `Cargo.toml` and `pyproject.toml`
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

- Entirely new features (e.g., "Added WebSocket transport support")
- New capabilities that didn't exist before

### Changed

- Modifications to existing behavior (e.g., "Made `type` field optional for URL-based servers")
- Config format changes that remain backwards compatible
- Default value changes (e.g., "Changed default timeout from 10s to 30s")

### Fixed

- User-facing bug fixes (e.g., "Fixed OAuth token refresh failing after expiry")
```

**How to classify changes**:

- **Added** = Something that didn't exist before (new feature, new field, new command)
- **Changed** = Modification to something that already existed (optional field that was required, different default, behavior adjustment)
- **Fixed** = Correction of broken/incorrect behavior

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
- [ ] crates.io updated: https://crates.io/crates/dynamic-mcp
- [ ] PyPI updated: https://pypi.org/project/dmcp/
  - [ ] Python wheels available (5 platforms)

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

- **Historical documentation** (`docs/implementation/mvp` such as RELEASE_v1.0.0.md, PLAN.md, RESEARCH.md)
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

**How to classify changes (Added vs Changed vs Fixed)**:

- **Added** = Something that didn't exist before
  - Examples: "Added WebSocket transport support", "Added `--force` flag", "Added `enabled` field"
  - Test: If users couldn't do this at all before → Added
- **Changed** = Modification to something that already existed
  - Examples: "Made `type` field optional for URL-based servers", "Changed default timeout from 10s to 30s", "Updated error message format"
  - Test: If users could do this before but differently → Changed
- **Fixed** = Correction of broken/incorrect behavior
  - Examples: "Fixed OAuth token refresh failing after expiry", "Fixed panic on empty config"
  - Test: If it wasn't working as intended → Fixed

**What to include in CHANGELOG.md**:

- ✅ New user-facing features ("Added OAuth2 authentication support")
- ✅ Modifications to existing behavior ("Made `type` field optional for URL-based servers")
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

- **Transport**: stdio, HTTP, SSE
- **Module**: config, proxy, server, cli, auth, watcher
- **Proxy Tool**: get_dynamic_tools, call_dynamic_tool

### 3. Update Timestamps

Add or update "Last Updated: [Date]" when making updates to docs.

### 4. Preserve Examples

When updating README.md or IMPORT.md:

- Keep examples functional and tested
- Update examples if config schema changes
- Add new examples for new features

### 5. Maintain Accuracy

- Test counts are only tracked in TESTING.md
- LOC should match actual source code
- Feature lists should reflect implemented code, not planned features

## 📊 Documentation Reference

### Before Starting Work - Read These First

- **@docs/implementation/ARCHITECTURE.md** - **READ THIS FIRST** for module structure, data flows, and design patterns
- **@docs/implementation/STATUS.md** - Current features, metrics, and what's implemented
- **@docs/implementation/TESTING.md** - Test organization, structure, and where to add new tests

### When Working on Features

- **@CONTRIBUTING.md** - Development workflow, build commands, and setup instructions
- **@docs/implementation/MCP_SPEC_COMPLIANCE.md** - MCP protocol requirements and compliance details

### When Updating Documentation

- **@README.md** - User-facing features and getting started guide (update for new user-visible features)
- **@docs/IMPORT.md** - Multi-tool import functionality (update if adding new tool support)
- **@CLAUDE.md** - This document (agent guidelines)

______________________________________________________________________

**Remember**: Documentation is code. Keep it accurate, up-to-date, and helpful for both humans and AI agents.
