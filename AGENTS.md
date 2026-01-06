# Agent Guidelines - dynamic-mcp

> **For AI Coding Agents**: This document explains the project documentation structure and when/how to update docs when making changes.

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

**Required Updates:**
1. ⚠️ **README.md** - Only if changes user-facing behavior
2. ⚠️ **STATUS.md** - Only if removes known limitation
3. ✅ **TESTING.md** - If adds regression tests

**Example:**
```markdown
Fix: OAuth token refresh failing

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

**Phase**: Phase 5 Complete ✅
**LOC**: ~2,900 Rust
**Tests**: 46 (37 unit + 9 integration)
**Modules**: config, proxy, server, cli, auth
**Transports**: stdio, HTTP, SSE
**Key Features**: OAuth2, Live Reload, Migration Command

**Where to find details:**
- Project status → **docs/implementation/DEVELOPMENT.md**
- Implementation status → **docs/implementation/STATUS.md**
- Testing → **docs/implementation/TESTING.md**
- Architecture → **docs/ARCHITECTURE.md**
- User guide → **README.md**
- Development setup → **CONTRIBUTING.md**

---

**Remember**: Documentation is code. Keep it accurate, up-to-date, and helpful for both humans and AI agents.
