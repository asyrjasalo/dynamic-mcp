# dynamic-mcp

一个 MCP 代理服务器：把多个上游 MCP 服务器聚合到一个入口，让 AI 更省 token、随处可用。

普通做法是把所有 MCP 工具一次性交给大模型（LLM），几十个工具的 Schema 动辄消耗数千 token，既烧钱又挤占上下文。dynamic-mcp 用两个核心能力解决它：

- **只暴露 3 个工具，按需动态加载**：无论接入多少个上游 MCP 服务器，始终只向 LLM 暴露 3 个「元工具」——列分组、查看某分组内的工具、调用具体工具。用到哪个分组，才临时加载对应工具的 Schema。上游工具再多，初始上下文开销几乎恒定，实质性降低 token 消耗。
- **把本地 stdio 服务桥接成 HTTP 服务**：许多 MCP 服务器只能以本地 `stdio` 方式运行，浏览器、云端、手机都用不上。dynamic-mcp 能把它们桥接成标准的 `Streamable HTTP` MCP 服务——一次配置，浏览器插件、云端 Agent、移动端 App 都能远程调用同一套工具。

**两种运行模式：**

- **模式一 · 本地代理（stdio）**：作为本地 stdio MCP 代理运行，直接对接 Claude Desktop、Cursor 等桌面客户端，专注「聚合 + 省 token」。
- **模式二 · HTTP 网关（Streamable HTTP）**：以 Streamable HTTP 对外提供服务，把本地工具开放给浏览器、云端、移动端远程使用。两种模式也可同时开启。

它支持来自上游 MCP 服务器的 tools（工具）、resources（资源）与 prompts（提示模板），传输方式涵盖 stdio、HTTP 与 SSE，并能处理 OAuth 认证、自动重试失败的连接。

## 快速开始

### 安装

#### 方式一：Python 包

在你的智能体 MCP 设置中使用 `uvx` 运行 [PyPI 包](https://pypi.org/project/dmcp/)：

```json
{
  "mcpServers": {
    "dynamic-mcp": {
      "command": "uvx",
      "args": ["dmcp", "/path/to/your/dynamic-mcp.json"]
    }
  }
}
```

你也可以设置 `DYNAMIC_MCP_CONFIG` 环境变量，从而省略配置文件路径。

#### 方式二：原生二进制

从 [Releases](https://github.com/asyrjasalo/dynamic-mcp/releases) 下载对应操作系统的版本，把 `dmcp` 放入 `PATH`：

```json
{
  "mcpServers": {
    "dynamic-mcp": {
      "command": "dmcp"
    }
  }
}
```

设置 `DYNAMIC_MCP_CONFIG` 环境变量后可完全省略 `args`。

#### 方式三：从源码编译

从 [crates.io](https://crates.io/crates/dynamic-mcp) 安装：

```text
cargo install dynamic-mcp
```

安装后二进制位于 `~/.cargo/bin/dmcp`（`$CARGO_HOME/bin/dmcp`）。

### 从 AI 编码工具导入

Dynamic-mcp 可以自动从主流 AI 编码工具导入 MCP 服务器配置。

**支持的工具**（`<tool-name>`）：

- Cursor（`cursor`）
- OpenCode（`opencode`）
- Claude Desktop（`claude-desktop`）
- Claude Code CLI（`claude`）
- Visual Studio Code（`vscode`）
- Cline（`cline`）
- KiloCode（`kilocode`）
- Codex CLI（`codex`）
- Gemini CLI（`gemini`）
- Google Antigravity（`antigravity`）

#### 快速开始

**从项目配置导入**（在项目目录中运行）：

```bash
dmcp import <tool-name>
```

**从全局 / 用户配置导入**：

```bash
dmcp import --global <tool-name>
```

**强制覆盖**（跳过确认提示）：

```bash
dmcp import <tool-name> --force
```

该命令会：

1. 检测你的工具配置位置
2. 解析已有的 MCP 服务器
3. 交互式提示输入描述
4. 交互式提示选择功能（tools、resources、prompts）
5. 规范化环境变量格式
6. 生成 `dynamic-mcp.json`

#### 导入示例

```bash
$ dmcp import cursor

🔄 Starting import from cursor to dynamic-mcp format
📖 Reading config from: .cursor/mcp.json

✅ Found 2 MCP server(s) to import

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Server: filesystem
Type: stdio

Config details:
  command: "npx"
  args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]

💬 Enter description for 'filesystem' (what this server does): File operations on /tmp directory

🔧 Keep all features (tools, resources, prompts) for 'filesystem'? [Y/n]:
(press Enter to keep all features, or 'n' to customize)

[... prompts for other servers ...]

✅ Import complete!
📝 Output saved to: dynamic-mcp.json
```

**功能选择**：导入过程中，你可以自定义每个服务器启用哪些 MCP 功能：

- 按回车（或 Y）保留全部功能（tools、resources、prompts）
- 输入 `n` 来选择性启用 / 禁用单个功能
- 这样无需手动编辑配置文件即可实现细粒度控制

自定义功能选择示例：

```bash
🔧 Keep all features (tools, resources, prompts) for 'server'? [Y/n]: n

  Select features to enable (press Enter to accept default):
  Enable tools? [Y/n]: y
  Enable resources? [Y/n]: n
  Enable prompts? [Y/n]: n
```

#### 各工具注意事项

- **Cursor**：同时支持 `.cursor/mcp.json`（项目级）与 `~/.cursor/mcp.json`（全局级）
- **Claude Desktop**：仅全局配置，位置因系统而异：
  - macOS：`~/Library/Application Support/Claude/claude_desktop_config.json`
  - Windows：`%APPDATA%\Claude\claude_desktop_config.json`
  - Linux：`~/.config/Claude/claude_desktop_config.json`
- **Claude Code CLI**：同时支持 `.mcp.json`（项目根目录）与 `~/.claude.json`（用户 / 全局级）
- **Gemini CLI**：同时支持 `.gemini/settings.json`（项目级）与 `~/.gemini/settings.json`（全局级）
- **VS Code**：同时支持 `.vscode/mcp.json`（项目级）与用户级配置（各系统路径不同）
- **OpenCode**：同时支持 JSON 与 JSONC 格式（带注释的 JSON）
- **Codex CLI**：仅全局级 —— 使用 TOML 格式（`~/.codex/config.toml`）
- **Antigravity**：仅全局级 —— `~/.gemini/antigravity/mcp_config.json`

#### 环境变量转换

导入命令会自动把环境变量规范化为 dynamic-mcp 的 `${VAR}` 格式：

| 工具            | 原格式                 | 转换后             |
| --------------- | ---------------------- | ------------------ |
| Cursor          | `${env:GITHUB_TOKEN}`  | `${GITHUB_TOKEN}`  |
| Claude Desktop  | `${GITHUB_TOKEN}`      | `${GITHUB_TOKEN}`  |
| Claude Code CLI | `${GITHUB_TOKEN}`      | `${GITHUB_TOKEN}`  |
| VS Code         | `${env:GITHUB_TOKEN}`  | `${GITHUB_TOKEN}`  |
| Codex           | `"${GITHUB_TOKEN}"`    | `${GITHUB_TOKEN}`  |

**注意**：VS Code 的 `${input:ID}` 安全提示无法自动转换，导入后需手动配置。

详细的工具专属导入指南见 [docs/IMPORT.md](docs/IMPORT.md)。

## Dynamic MCP 配置格式

### 按需调用上游服务器

创建一个 `dynamic-mcp.json` 文件，为每个服务器填写 `description` 字段：

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

### 环境变量

支持使用 `${VAR}` 语法进行环境变量插值：

```json
{
  "mcpServers": {
    "example": {
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

### 服务器类型

支持所有[标准 MCP 传输机制](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports)。

**注意**：当 `url` 存在时，`type` 字段是**可选**的。若省略，服务器会根据 MCP 规范自动使用 HTTP 传输并做 SSE 探测。这保持了与 [OpenCode](https://opencode.ai/docs/mcp-servers/) 等工具的向后兼容。

#### stdio（默认）

```json
{
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
  "description": "HTTP server (type is optional)",
  "url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

或显式指定类型：

```json
{
  "type": "http",
  "description": "HTTP server with explicit type",
  "url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### sse

当服务器以 `Content-Type: text/event-stream` 响应时，SSE 服务器会被自动探测。若服务器仅支持 SSE，也可显式指定 `type: "sse"`：

```json
{
  "type": "sse",
  "description": "SSE server (explicit type required only if server doesn't auto-detect)",
  "url": "https://api.example.com/sse",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

#### OAuth 认证（HTTP / SSE）

```json
{
  "description": "OAuth-protected MCP server (type is optional)",
  "url": "https://api.example.com/mcp",
  "oauth_client_id": "your-client-id",
  "oauth_scopes": ["read", "write"]
}
```

**OAuth 流程：**

- 首次连接时，浏览器会打开以进行授权
- 访问令牌保存在 `~/.dynamic-mcp/oauth-servers/<server-name>.json`
- 在过期前自动刷新令牌（支持 RFC 6749 令牌轮换）
- 令牌以 `Authorization: Bearer <token>` 请求头形式注入

### 功能开关（Feature Flags）

使用可选的 `features` 字段，按服务器控制暴露哪些 MCP 功能。默认情况下全部功能（`tools`、`resources`、`prompts`）均启用。你可以选择性地禁用某些功能：

```json
{
  "mcpServers": {
    "server-with-tools-only": {
      "description": "Server that only exposes tools",
      "command": "npx",
      "args": ["-y", "some-mcp-server"],
      "features": {
        "resources": false,
        "prompts": false
      }
    },
    "server-without-prompts": {
      "description": "HTTP server without prompt templates (type is optional)",
      "url": "https://api.example.com",
      "features": {
        "prompts": false
      }
    }
  }
}
```

**行为：**

- 若省略 `features`，则全部功能启用（opt-out 设计）
- 若指定了 `features`，未提及的功能默认仍为 `true`（启用）
- 被禁用的功能通过代理访问时会返回错误
- 例如：若 `resources: false`，调用 `resources/list` 会返回错误

### 禁用服务器

使用可选的 `enabled` 字段，在不从配置中删除服务器的前提下将其禁用：

```json
{
  "mcpServers": {
    "filesystem": {
      "description": "File operations",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "disabled-server": {
      "description": "This server won't connect",
      "command": "some-command",
      "enabled": false
    }
  }
}
```

**行为：**

- 若省略 `enabled`，服务器为启用状态（默认行为）
- 若 `enabled: false`，则在连接阶段跳过该服务器，不会出现在可用分组中
- 适用于测试或维护期间临时禁用服务器而无需改动配置结构
- 完整示例见 `examples/config.features.example.json`

### 超时配置

使用可选的 `timeout` 字段，按服务器自定义工具、资源、提示调用的超时时间。默认值：

- 工具调用：30 秒
- 资源调用：10 秒
- 提示调用：10 秒

你可以为需要更长时间的服务器自定义：

```json
{
  "mcpServers": {
    "slow-server": {
      "description": "Server with slow operations",
      "command": "npx",
      "args": ["-y", "some-slow-mcp-server"],
      "timeout": {
        "tools": "1min",
        "resources": "30s",
        "prompts": "30s"
      }
    }
  }
}
```

**支持的时长格式：**

| 格式         | 示例             | 说明                 |
| ------------ | ---------------- | -------------------- |
| 秒           | `"30s"`、`"5s"`  | 简单秒数             |
| 分钟         | `"1min"`、`"2m"` | 分钟（缩写或完整）   |
| 毫秒         | `"3000ms"`、`"500ms"` | 毫秒           |
| 纯数字       | `30`             | 秒（纯数字）         |

**行为：**

- 若省略 `timeout`，使用默认值（tools: 30s，resources: 10s，prompts: 10s）
- 单个超时字段若未指定，默认取各自对应的默认值
- 仅适用于工具 / 资源 / 提示调用操作，不适用于连接或初始化
- 适用于存在长时间运行操作的服务器（数据库查询、文件处理等）

## v1.6.0 新增：Streamable HTTP 传输

除默认的 stdio 传输外，dynamic-mcp 现在可以把其「分组工具门面（grouped-tool facade）」通过一个**单一的 Streamable HTTP MCP 端点**暴露出来。这让基于 HTTP / SSE 的 MCP 客户端（Web UI、远程智能体、其他 MCP 代理、网关）无需 stdio 即可连接 dynamic-mcp。

该 HTTP 端点会把所有已配置的上游（stdio）服务器聚合为 3 个工具的门面：

- `list_groups` —— 列出所有已配置的分组及其连接状态。
- `get_dynamic_tools` —— 按需获取某个选定分组的工具 Schema。
- `call_dynamic_tool` —— 通过代理在选定分组上调用某个工具。

### 参数（命令行）

新增的命令行参数用于控制 HTTP 暴露方式（**配置文件无需改动**，见下文）：

| 参数            | 默认值          | 说明                                              |
| --------------- | --------------- | ------------------------------------------------- |
| `--transport`   | `stdio`         | 传输模式：`stdio`、`http` 或 `both`。             |
| `--http-host`   | `127.0.0.1`     | HTTP 服务器绑定的地址。                           |
| `--http-port`   | `8082`          | HTTP 服务器绑定的端口。                           |
| `--http-path`   | `/dynamic-mcp`  | Streamable HTTP MCP 端点的挂载路径。              |

### 使用方法

```bash
# 仅 HTTP（关闭 stdio）：
dmcp --transport http /path/to/dynamic-mcp.json

# stdio 与 HTTP 同时开启：
dmcp --transport both /path/to/dynamic-mcp.json

# 绑定到所有网卡，自定义端口与路径：
dmcp --transport http --http-host 0.0.0.0 --http-port 9000 --http-path /mcp /path/to/dynamic-mcp.json
```

当使用 `--transport http` 或 `both` 时，门面服务地址为 `http://<host>:<port><path>`（例如 `http://127.0.0.1:8082/dynamic-mcp`）。

### 配置文件（无需改动）

v1.6.0 **没有**修改 `dynamic-mcp.json` 的 Schema。你已有的配置原样可用；HTTP 暴露完全由上面的命令行参数控制，沿用同一个 `config-schema.json`。

示例 `dynamic-mcp.json`（保持不变）：

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

然后以开启 HTTP 的方式运行：

```bash
dmcp --transport both /path/to/dynamic-mcp.json
```

### 应用场景

- **远程 / 容器化部署**：在 Docker、k8s、远程虚拟机等无法使用 stdio 的环境。
- **反向代理 / 网关前置**：用 nginx、Traefik 等把 dynamic-mcp 前置，让多个客户端共享一个后端。
- **基于 Web 的 MCP 客户端与调试台**：直接以 Streamable HTTP 方式通信的网页工具。
- **级联 MCP 代理**：第二个代理 / 编排器通过 HTTP 连接 dynamic-mcp，而无需拉起子进程。
- **单端点多分组访问**：一个 HTTP 端点服务所有分组；客户端通过 `get_dynamic_tools` / `call_dynamic_tool` 选择分组。

## 从源码构建

### Rust 二进制

直接构建 Rust 二进制：

```bash
git clone https://github.com/asyrjasalo/dynamic-mcp.git
cd dynamic-mcp
cargo build --release
```

构建后二进制位于 `./target/release/dmcp`。

### Python 包

构建 Python 包（wheel）：

```bash
# 构建 wheel
uvx maturin build --release

# 本地安装
pip install target/wheels/dmcp-*.whl
```

Python 包使用 **maturin** 配合 `bindings = "bin"`，将 Rust 二进制直接编译进 wheel。

## 关于本分支（fork）使用 GitHub Actions 自行构建的说明

> **说明**：上游仓库（asyrjasalo/dynamic-mcp）较长时间未更新。为了不等待上游发版即可使用 v1.6.0 的新功能（含 HTTP 门面），本 fork 通过 **GitHub Actions** 自行构建二进制——具体由 Release 工作流在推送 `v*` 标签时触发。构建产物为跨平台二进制（含 Windows 的 `dmcp.exe`），作为 Release 资产（assets）发布。
>
> 这些构建**不会**发布到 crates.io / PyPI，请直接从本 fork 的 Releases 页面下载二进制使用。

## 故障排查

### 服务器连接问题

**问题**：`❌ Failed to connect to <server>`

**解决方案**：

- **连接超时**：每个服务器在传输创建、初始化与工具列举上各有 10 秒超时
- **自动重试**：失败服务器最多重试 3 次，采用指数退避（2s、4s、8s）
- **周期性重试**：失败服务器在后台每 30 秒重试一次
- **慢速 HTTP 服务器**：远程 HTTP / SSE 服务器若响应慢会超时并被自动重试
- **stdio 服务器**：确认命令存在（`which <command>`）
- **HTTP / SSE 服务器**：检查服务器是否在运行、URL 是否正确
- **环境变量**：确保全部 `${VAR}` 引用均已定义
- **OAuth 服务器**：按提示完成 OAuth 流程

**日志：**

默认情况下，错误与警告会记录到终端。如需更详细的输出：

```bash
# 调试模式（全部日志，含 debug 级别细节）
RUST_LOG=debug uvx dmcp config.json

# 信息模式（含信息级消息）
RUST_LOG=info uvx dmcp config.json

# 默认模式（仅错误与警告，无需 RUST_LOG）
uvx dmcp config.json
```

### OAuth 认证问题

**问题**：浏览器未打开进行 OAuth

**解决方案**：

- 手动打开控制台显示的 URL
- 检查防火墙是否允许 localhost 连接
- 确认服务器的 `oauth_client_id` 正确

**问题**：令牌刷新失败

**解决方案**：

- 删除缓存令牌：`rm ~/.dynamic-mcp/oauth-servers/<server-name>.json`
- 下次连接时重新认证

### 环境变量未被替换

**问题**：配置中显示的是 `${VAR}` 而非实际值

**解决方案**：

- 使用 `${VAR}` 语法，而非 `$VAR`
- 导出变量：`export VAR=value`
- 变量名区分大小写
- 检查变量名是否拼写错误

### 配置错误

**问题**：`Server missing 'description' field`

**解决方案**：

- 配置中的每个 MCP 服务器都必须有 `description` 字段
- 该描述用于向 LLM 解释服务器用途
- 示例：

  ```json
  {
    "description": "File system access - read, write, and search files",
    "command": "npx",
    "args": ["@modelcontextprotocol/server-filesystem"]
  }
  ```

**问题**：`Invalid JSON in config file`

**解决方案**：

- 校验 JSON 语法（使用 `jq . config.json`）
- 检查是否有尾随逗号
- 确保所有必需字段齐全（`description` 始终必需；`type` 仅 http/sse 服务器必需）

**问题**：配置中存在未知字段（如 `unknown field \`typo_field\`\`）

**解决方案**：

- dynamic-mcp 使用严格的 JSON Schema 校验，仅允许已定义的字段
- 检查字段名拼写：`description`、`command`、`url`、`type`、`args`、`env`、`headers`、`oauth_client_id`、`oauth_scopes`、`features`、`enabled`、`timeout`
- 从配置中移除任何多余或拼写错误的字段
- 参考上文各服务器类型的示例查看合法字段

**问题**：`Failed to resolve config path`

**解决方案**：

- 使用绝对路径或相对于工作目录的路径
- 检查文件是否存在且具有读权限
- 尝试：`ls -la <config-path>`

### 工具调用失败

**问题**：工具调用返回错误

**排查**：

1. 直接使用上游服务器测试该工具
2. 检查工具名与参数是否与 Schema 匹配
3. 确认分组名正确
4. 开启 debug 日志查看 JSON-RPC 消息

### 性能问题

**问题**：启动缓慢

**解决方案**：

- 已启用并行连接
- 检查 HTTP / SSE 服务器的网络延迟
- 部分服务器初始化较慢属正常现象

**问题**：内存占用高

**解决方案**：

- 工具会缓存在内存中（预期行为）
- 失败的分组占用内存极少
- 大型工具 Schema 会增加内存占用

## 贡献

关于开发环境搭建、测试与贡献的说明，见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 版本历史

版本历史与发版说明见 [CHANGELOG.md](CHANGELOG.md)。

## 致谢

- TypeScript 实现：[modular-mcp](https://github.com/d-kimuson/modular-mcp)
- MCP 规范：[Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP 生态：[rust-mcp-stack](https://github.com/rust-mcp-stack)
