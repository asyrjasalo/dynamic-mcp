# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.9.2] - 2026-09-04

### Changed

- **数据目录本地化 `dmcp_data_dir()`** — 新增统一的数据目录解析函数：**优先 `dmcp.exe` 同目录**（便携化部署），目录不可写（如 Program Files 受限 ACL / EDR 拦截）时降级 `~/.dynamic-mcp/`。
- **四类运行时文件统一落位** — 端点锁（`locks/`）、登记文件（`instances/`）、日志（`logs/`）、工具产物（`artifacts/`）全部基于 `dmcp_data_dir()` 解析，去掉三处各自重复的探针逻辑。
- **`dmcp status` 提示改进** — 打印实际生效的 data_dir 路径，不再硬编码 `~/.dynamic-mcp/`。

### Fixed

- **rustfmt 格式修复** — `main.rs` 中 `println!` 行宽超限，按 `cargo fmt` 规范拆行。

### 不变

- OAuth token store 保持 `~/.dynamic-mcp/oauth-servers/`（认证凭据与运行时状态分离）。

## [1.9.1] - 2026-09-04

### Added

- **启动自修复 `cleanup_orphans()`** — 启动时清理 pid 已死的残留登记文件（`instances/stdio-*.json`）与锁文件（`locks/*.lock`），防止高频启停（Task Manager 强杀）场景下文件堆积。
- **测试覆盖** — 新增 9 个单元测试，覆盖清理逻辑、pid 复用判定、TOCTOU 防护、文件名解析等回归场景。

### Changed

- **Ctrl+C 优雅退出** — 移除 `exit(0)` 粗暴退出路径，改用 `tokio::select!` 同时等待 `run_stdio()` 与 `ctrl_c()`，确保 `_singleton_guard` 被正确 Drop、锁文件自动清理。
- **判定逻辑统一** — 残留文件判定改用 `is_same_binary`（pid 存活 + exe 路径校验），与 `try_acquire_lock` / `list_instances` 口径一致，防止 Windows pid 复用漏删。
- **TOCTOU 防护** — 登记文件改为从文件名读 pid，pid 活着直接跳过、不打开文件内容，消除读写竞态误删。
- **日志清理窗口** — 日志/产物文件清理窗口从 72h 缩短为 24h，适应高频启停场景。

### Fixed

- **`.gitignore` 补漏** — 补写 `/releases` 规则，防止 4.5MB 构建产物误提交。

## [1.9.0] - 2026-09-03

### Added

- **`dmcp status` 子命令** — 扫描 `~/.dynamic-mcp/locks/` 与 `~/.dynamic-mcp/instances/`，列出当前所有已知实例（HTTP 域端点锁 + STDIO 域登记文件），标记存活/死亡状态；无需配置文件即可调用。
- **STDIO 登记机制** — stdio-only 实例启动时写 `~/.dynamic-mcp/instances/stdio-<pid>.json`（进程退出时 RAII 删除），不参与仲裁、仅供可观测性查询。
- **配置一致性告警** — `InstanceLock` 新增 `config_path` 字段；两实例同端点但配置不同时，弹窗显式告警而非静默串味（`#[serde(default)]` 保障向后兼容，旧版锁文件仍可反序列化）。
- **弹窗自动关闭** — 通知弹窗 15 秒后自动关闭（Windows `FindWindowW`+`PostMessageW(WM_CLOSE)` / macOS `giving up after` / Linux `notify-send -t`），用户可随时手动提前关闭；`SelfTerminate` 退出前多等 1 秒确保弹窗读完。

### Changed

- **决策函数 `decide()` 重写为规则驱动** — 原硬编码 `match` 改为优先级数值比较（`Both=3 > Http=2 > Stdio=1`），辅以 `parse_transport()` / `transport_priority()` / `has_stdio()` 辅助函数，对应 R0–R4 规则集语义更清晰。
- **#12 降级提示文案修正** — 原 `_ =>` 兜底写"双开浪费"误导用户；新增 `keep_stdio_msg()` 三段式文案（旧实例身份 / 降级行为明确 / 强调"预期共存、非浪费"），并带出 `--no-evict` 退出路径。
- **`--no-evict` 帮助文案正名** — 改为"保护**本实例**不被驱逐"，明确"该标志保护的是本实例 — 不是让本实例驱逐别人"。
- **弹窗超时统一为 15 秒** — 常量 `POPUP_TIMEOUT_SECS = 15` 替代原硬编码 8 秒，README 同步更新。

### Fixed

- **`list_instances` 扩展名过滤 bug** — 原 `path.extension() != Some("json")` 会漏掉所有 `.lock` 文件（HTTP 域实例全部丢失）；修复为按来源类型分别过滤（`Lock => "lock"` / `Registry => "json"`）。

> 本批次为单例可观测性增强（#1–#7），对应分支 `feat/singleton-observability`，编译与测试由 GitHub Actions CI 回归验证。

## [1.8.4] - 2026-08-13

### Added

- **`timeout.initialize` 配置项** — 新增可选字段，覆盖后端 MCP 服务器的「连接 + 初始化」全过程（transport 创建 + `initialize` 握手 + 协议版本能力重试 + 首次 `tools/list`），默认值 **120 秒**。用于缓解冷启动慢的重后端（如 `codebase-memory-mcp` 单二进制约 296MB）在旧版 5 秒硬编码超时下反复连接失败（flapping）的问题。与 `tools` / `resources` / `prompts` 相互独立，向后兼容（`Timeout` 结构体在 `deny_unknown_fields` 下为 `initialize` 标注 `serde(default)`，缺失时不报错）。

### Fixed

- **cbm 初始化超时可配置（Path A 治本修复）** — 将 `src/proxy/client.rs` 中 transport 创建、`init_request`、`retry_request`、`list_tools_request` 三处硬编码 `Duration::from_secs(5)` 统一改为 `config.initialize_timeout()`（默认 120s，可经 `timeout.initialize` 配置）。
- **Failed 自愈重连** — `list_tools` / `call_tool` 在连接处于 `Failed` 状态时，自动触发一次 `connect` 自愈重连（受 `initialize` 超时包裹），并在错误中携带 `group_name` 便于定位；`call_tool` 由 `&self` 调整为 `&mut self` 以支持自愈。
- **周期重连上限放宽 + 有界退避** — `retry_failed_connections` 的 `MAX_RETRIES` 由 3 次放宽至 10 次，退避策略由无界 `2^n` 秒改为有界 `2 + 5×n` 秒（上限 30 秒），避免重后端因退避指数爆炸而长期不可达。
- **调用方锁调整** — `src/server.rs` 与 `src/http/server_handler.rs` 中 `list_tools` / `call_tool` 的调用点由读锁改为写锁并补 `.await`，与自愈逻辑（内部 `connect`）的 `&mut self` 借用保持一致。

> 本版本为 cbm 连接超时治本修复（Path A），对应分支 `fix/cbm-init-timeout`，编译与测试由 GitHub Actions CI 回归验证。

## [1.8.3] - 2026-07-25

### Fixed

- **B1（启动稳定性）** — 取消启动时对全部 group 连接的阻塞等待：各 group 连接改为在后台 `tokio::spawn` 任务中继续，`run_stdio` 立即进入 MCP 握手，不再 `await` 连接完成。避免 MCP 连接器在 dmcp 尚未就绪时因启动阶段握手/健康检查超时而将其杀掉重启，形成每 ~20–30 秒一次的慢速重启循环（冷启动 GNF 已在配置层 A2 + `mimo_mcp.py` 进程树清理处缓解）。
- **B3（响应正确性）** — `StdioTransport` 读取 `JsonRpcResponse` 时丢弃 `id` 与当前请求不匹配的陈旧响应，消除「请求 future 因超时丢弃、但上游随后回写其响应」导致的响应串扰（cross-talk）。`id` 为 null/缺失的通知与错误仍照常透传，保持原有行为。

> 本版本为连接器稳定性补丁，合并自 PR #8（`fix/mimo-stability-merge`），已剔除调试仪器化，对应版本号 bump 至 1.8.3。

## [1.8.2] - 2026-07-15

### Added

- **`--log` parameter (hybrid logging)** — New `--log <LEVEL>` CLI flag (`trace` / `debug` / `info` / `warn` / `error`; invalid values fall back to `warn`).
  - **Without `--log`**: `http` mode outputs WARN-level logs to stderr; `stdio` / `both` are silent; no log file. The `import` subcommand still initializes tracing as before.
  - **With `--log`**: all transport modes write a log file `dynamic-<pid>-<YYYYMMDD-HHMMSSmmm>.log` in the executable's directory (read-only fallback: `data_local_dir/dynamic-mcp/`). Additionally, `http` mode mirrors logs to stderr; `stdio` / `both` remain stderr-silent to protect JSON-RPC.
  - On startup, stale `dynamic-*.log` files older than 72 hours (excluding the current run) are cleaned up automatically.
  - No new dependencies added; uses `tracing-subscriber` with `env-filter` (already in `Cargo.toml`).

- **`get_dynamic_tools` enhanced parameters** — New optional parameters (defaults preserve byte-identical output with v1.8.1):
  - `mode` (`full` default / `compact`): `compact` returns tool name + full description, omits `inputSchema`.
  - `include_schema` (default `true`): set `false` to strip `inputSchema` from results.
  - `page` / `page_size` (defaults `1` / `0`): when `page_size > 0`, results are wrapped as `{tools, total, page, page_size, has_more}`; when `0`, flat array as before.
  - `land_to_file` (default `false`): writes results to a JSON file and returns the absolute path; files auto-cleaned after 72 hours.
  - `capabilities` (default `false`): attaches an `x-capabilities` array (from `GroupInfo.capability_tags`) to each tool entry.

- **`GroupInfo` metadata enrichment** — `list_groups` response now includes `tool_count` (number of tools in the group) and `capability_tags` (derived from transport type: `stdio` / `http` / `sse` / `oauth`). No `example` field (no source for it). `GroupInfo` has no `deny_unknown_fields`, so older clients remain compatible.

### Fixed

- **OAuth callback `localhost` → `127.0.0.1`** — The OAuth redirect URI used `localhost` while the callback listener bound to `127.0.0.1`; on some systems `localhost` resolves to `::1` (IPv6), causing the browser redirect to fail. Both sides now use `127.0.0.1` consistently.
- **OAuth discovery skipped when static `Authorization` header present** — Servers configured with a static `Authorization` header (i.e., `needs_oauth() == false`) no longer trigger an unnecessary OAuth discovery round-trip; they connect directly with the static header.
- **OAuth transport creation timeout 120s → 300s** — The timeout for creating the OAuth transport (browser authorization window) increased from 120s to 300s, giving users more time to complete browser-based authorization.

### Changed

- **Structured error envelope for `call_dynamic_tool`** — Tool call errors now return a structured JSON envelope (still via `CallToolResult` with `is_error: true`, not JSON-RPC error):
  ```json
  { "ok": false, "code": "<code>", "message": "<original message>", "cause": null }
  ```
  `code` mapping: `timed out` → `timeout` / `Tool execution failed` → `upstream_error` / `Missing required` → `bad_request` / other → `tool_error`. Existing tests asserting `is_error == Some(true)` remain unaffected.

## [1.8.1] - 2026-07-15

### ⚠️ Breaking Changes

- **Merged HTTP endpoint flags** — `--http-host` / `--http-port` / `--http-path` replaced by a single `--http-endpoint` (`host:port/path`, IPv6 `[host]:port/path`). Default `127.0.0.1:8082/dynamic-mcp`. Update startup scripts and LLM MCP configs.

### Fixed

- **IPv6 bind address** — binding to `::1` (or any IPv6 host) no longer fails to parse; the address is correctly bracketed before `SocketAddr` parsing (v1.8.0 crashed on IPv6 hosts).
- **Singleton conflict popup for IPv6** — the port-conflict popup now always shows the address (including IPv6) instead of being silently skipped.
- **macOS popup with newlines** — the `osascript` popup no longer fails silently when the message contains a newline (AppleScript string concatenation fix).

### Changed

- Singleton detection keys on the canonical `host:port/path` string (IPv6 brackets stripped), keeping cross-version lock-file compatibility with v1.8.0.
- Conflict / double-launch popup messages now reference `--http-endpoint` with a full example.

## [1.8.0] - 2026-07-14

### Added

- **Singleton / double-launch detection for HTTP endpoints** - When starting with `--transport http` or `both`, dynamic-mcp now detects whether another instance already owns the same HTTP endpoint (`--http-host` + `--http-port` + `--http-path`) and resolves the conflict automatically instead of failing silently or hitting a port clash.
  - A per-endpoint lock file under `~/.dynamic-mcp/locks/<sha256(endpoint)[..16]>.lock` records the owner's pid, transport, `--no-evict` flag, and executable path.
  - Decisions (pure, unit-tested `decide()`): a redundant `http` self-terminates after 8s; a `both` evicts an existing `http` (unless that http was started with `--no-evict`) and takes over the port 8s later with stdio available immediately; a `both` vs `both` (or vs a `--no-evict` http) keeps stdio only with HTTP off.
  - Stale locks are detected via pid liveness + executable-path comparison, so a reused pid cannot be mistaken for a live instance.
  - New `--no-evict` flag (valid only with `--transport http`): marks a plain http instance so a later `both` coexists (stdio only) instead of being evicted.
  - Two-layer user notification (double-launch `warn` + port-conflict `warn`) merged into a single popup: a real closable `MessageBoxW` on Windows, `notify-send` on Linux, `osascript` on macOS, plus a stderr line on every platform.
  - HTTP bind uses `SO_REUSEADDR` with a ~10s retry so an eviction can take over the port even while it is in `TIME_WAIT`.

## [1.6.0] - 2026-07-10

### Added

- **Streamable HTTP MCP transport (Route A)** - Expose the grouped-tool facade over a single Streamable HTTP endpoint, multiplexing multiple stdio upstream servers into three tools (`list_groups` / `get_dynamic_tools` / `call_dynamic_tool`).
  - New `--transport` CLI flag: `stdio` (default), `http`, or `both`
  - New `--http-host` (default `127.0.0.1`), `--http-port` (default `8082`), `--http-path` (default `/dynamic-mcp`) flags
  - `HttpFacadeHandler` implements `rmcp::ServerHandler` and serves the facade via `StreamableHttpService` mounted on an axum router (CORS-permissive)
- `list_groups` meta-tool now also exposed on the stdio surface (previously HTTP-facade only)

## [1.5.2] - 2026-07-09

### Fixed

- **list_groups Meta-Tool Implementation** - The `list_groups` tool was documented in v1.5.1 changelog but missing from the actual code
  - Added as the first built-in tool in `tools/list` response (before `get_dynamic_tools` and `call_dynamic_tool`)
  - Returns JSON array with group name, description, and status (`connected`/`failed`) for each MCP server
  - No input parameters required — works as a discovery tool independent of `enum` fields
  - Critical for interoperability with MCP proxies (e.g., mcp-bridge) that strip `enum` during JSON re-serialization

## [1.5.1] - 2026-07-09

### Added

- **list_groups Meta-Tool** - New built-in tool for MCP group discovery without enum dependency
  - LLMs can discover available MCP server groups when `enum` is stripped by proxy layers
  - Returns group name, description, and connection status (connected/failed) for each server
  - Resolves interoperability issue with MCP proxies (e.g., mcp-bridge using mcp-go) that lose `enum` fields during JSON re-serialization
  - No parameters required; works as a discovery tool before calling `get_dynamic_tools` or `call_dynamic_tool`

## [1.5.0] - 2026-02-14

### Added

- **Per-Server Timeout Configuration** - Custom timeouts per server for tools, resources, and prompts
  - Add `"timeout": { "tools": "30s", "resources": "30s", "prompts": "30s" }` to any server config
  - Supports formats: `30s`, `1min`, `1m`, `500ms`, plain numbers
  - Defaults: tools 30 seconds, resources 10 seconds, prompts 10 seconds
  - Useful for servers with long-running operations

## [1.4.0] - 2026-01-12

### Added

- **Per-Server Enable/Disable Control** - Optional `enabled` field to disable specific MCP servers
  - Add `"enabled": false` to any server config to skip connection
  - Defaults to `true` when omitted (all servers enabled by default)
  - Clean way to comment out servers without modifying config structure
- **Strict JSON Schema Validation** - Config files now enforce strict schema compliance
  - Only defined fields are allowed in config (denies unknown fields)
  - Catches typos and misspelled field names with clear error messages
  - Applies to all config levels: root, server objects, and feature flags
  - Helps prevent silent config errors and unexpected behavior
- **JSON Schema Reference Support** - Config files can now include `$schema` field for IDE validation

### Changed

- **Optional `type` Field for URL-Based Servers** - Simplified configuration for HTTP/SSE servers
  - `type` field is now optional when `url` is present (automatically defaults to `"http"`)
  - Automatic SSE detection per MCP spec when server responds with `text/event-stream`
  - Explicit `type: "sse"` still supported for SSE-only servers
  - Maintains full backwards compatibility with existing configs

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
- **Release Notes**: [docs/implementation/mvp/RELEASE_v1.0.0.md](docs/implementation/mvp/RELEASE_v1.0.0.md)

[1.0.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.0.0
[1.1.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.1.0
[1.2.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.2.0
[1.3.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.3.0
[1.4.0]: https://github.com/asyrjasalo/dynamic-mcp/releases/tag/v1.4.0
