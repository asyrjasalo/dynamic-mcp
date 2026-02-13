# CLI Module

**Parent:** ./AGENTS.md

## Overview

CLI module handling commands and configuration import from AI coding tools.

## Files

- `mod.rs` - CLI module exports
- `import_enhanced.rs` - Multi-tool import command
- `config_parser.rs` - Config file parsing
- `tool_detector.rs` - AI tool config detection

## Where to Look

| Task | File | Notes |
|------|------|-------|
| Import command | import_enhanced.rs | run_import function |
| Config parsing | config_parser.rs | Config file loading |
| Tool detection | tool_detector.rs | Detect cursor, claude, etc. |

## Conventions

Same as root AGENTS.md. Tests go in `tests/cli_integration_test.rs` and `tests/cli_import_integration_test.rs`.
