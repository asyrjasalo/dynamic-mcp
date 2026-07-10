# dynamic-mcp

> 一句话：它是个「工具中转站」——把你散落各处的 AI 工具收拢到一处，让 AI 助手用起来更省、更方便。

## 三分钟看懂：它到底是什么、怎么用（给不懂电脑的人）

先用人话讲清楚，技术细节放在后面。

### 一、它帮你解决两个麻烦

**麻烦 1：工具太多，AI 助手记不过来，还费钱**
AI 助手（比如 Claude、GPT）自己不会干活，得借用你准备好的各种「工具」——查资料、读文件、算数据……如果你有 20 个工具，全摆到助手面前，它得把每个工具说明都记在「工作桌面」上。桌面塞满，助手反应慢，而且每说一句话都要多花钱。

dynamic-mcp 的聪明做法：**一开始只给助手看一张写着 3 个选项的「菜单」**（列分组、看分组里有什么、调用某个工具）。助手真要用某个工具时，才临时把说明「调」出来。这样桌面始终干净，又省事又省钱。

**麻烦 2：助手够不着你电脑里的工具**
很多好用的工具只能「放在你电脑里、站你旁边才能用」。但助手如果在浏览器里、手机上、云端的另一台机器上，就「走不过去」拿。

dynamic-mcp 能把这些本地工具**接一根「电话线」，变成远处也能打的**（技术名词叫「把本地 stdio 桥接成 Streamable HTTP」）。接好后，远处的助手打一个「电话」（连上网络地址）就能用你电脑里的工具了。

### 二、两种「开门方式」（模式）

把软件想成一间「工具房」，它可以开不同数量的门：

| 开门方式 | 家门口那扇门（给站你电脑旁的助手） | 对外窗口（给浏览器/手机/云端的助手） | 适合谁 |
|---|---|---|---|
| 模式一 stdio | 开 | 关 | 只给你电脑上装的 AI 软件用 |
| 模式二 http | 关 | 开 | 只给浏览器/手机/云端的助手用 |
| 模式三 both | 开 | 开 | 两边同时用 |

三种方式都保留了「只给 3 个菜单选项」的省心设计。

### 三、模式三（both）的优点 / 亮点

模式三就是「一间房、两扇门、一套工具」，最大的好处是**一个程序顶两个用**：

1. **只开一个程序，两处都能用**：你电脑上的 AI 软件（Claude Desktop / Cursor / VS Code）走「家门口那扇门」（stdio），浏览器、手机、云端的 AI 助手走「对外窗口」（HTTP）——**一份程序同时服务两类助手**，不用开两个 dmcp。
2. **共享同一份工具连接和配置**：两种开门方式共用同一套上游工具和同一份配置文件，不需要维护两套 dmcp、连两次上游，省内存、省资源，配置只写一份，改一次两端同时生效。
3. **省心省力**：没有模式三的话，你要开两个程序——一个给桌面软件（stdio）、一个给浏览器（http），双倍连接、双倍内存、配置维护两处；有了模式三，这些都免了。

> ⚠️ **模式三该由谁来打开（很重要）**：应该**让你电脑上的 AI 软件（Claude Desktop / Cursor / VS Code）来帮你打开**，不要你自己手动点开。
> 原因：这间房「亮不亮灯」取决于「家门口那扇门有没有助手进来」。AI 软件把 dmcp 打开时，等于推开了家门口的门、站在房子里——房子通电，对外窗口也自然开了，浏览器里的助手就能从窗口进来。
> 如果你自己手动点开：家门口的门开了却没人用（浪费），你的 AI 软件还是用不了它，只能再去另开一个 → 又变两个程序，模式三的好处没了。
> 正确做法：在你的 AI 软件设置里写「用 both 方式打开 dmcp」，剩下的它自己会做。

> 💡 **小提醒**：房子亮灯靠「家门口那个助手在不在」。你关掉电脑上的 AI 软件，dmcp 会被关掉，对外窗口也关了——浏览器里的助手立刻用不了。如果你希望「哪怕电脑 AI 软件关了，浏览器助手照样能用」，就把模式三拆成两个独立程序：一个专门对外常亮的（模式二 http，你手动开着），一个给电脑 AI 软件自用的（模式一或模式三，由软件自己打开）。

### 四、一个参数怎么选（给想动手的人）

决定「开几扇门」的，是一个叫 `--transport` 的开关：
- 写 `stdio` → 只开家门口的门（给桌面 AI 软件）
- 写 `http` → 只开对外窗口（给浏览器/手机/云端）
- 写 `both` → 两扇门都开（让 AI 软件帮你打开）

其余几个参数（地址、端口、路径）一般不用改，保持默认即可；只有当你想让「本机以外」的设备也能访问时，才需要调整（详见下方技术章节的「参数」说明）。

---

下面是给开发者看的完整技术说明（看不懂不影响上面的大意）。

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

### 三种传输模式（stdio / http / both）

v1.6.0 起，`--transport` 决定 dynamic-mcp 以哪种方式对外服务。三种模式**最核心的区别是「由谁启动」**：

#### 模式一 · stdio（默认，`--transport stdio`）

- **是什么**：dynamic-mcp 作为你本机的一个子进程运行，数据通过标准输入/输出（stdio）收发。
- **由谁启动**：stdio 模式的**正常用法是由 LLM 客户端拉起**——在 Claude Desktop / Cursor / VS Code 等桌面客户端的 MCP 配置里写 `"command": "dmcp"`，由客户端自己把 dmcp 启动起来、并接管它的标准输入/输出；进程随客户端启动而生、随客户端退出而灭。**你当然也可以在终端里手动运行 `dmcp config.json`**，但那样它的标准输入/输出只连着你的终端、没有 LLM 客户端接管，等于没有「对话对象」，无法正常使用——所以 stdio 模式只有被 LLM 客户端拉起才有实际意义。
- **适用场景**：仅在你本机、使用桌面 AI 客户端时。
- **限制**：浏览器、云端、手机上的 LLM 没有能力在你电脑上拉起一个本地进程，因此**这些环境用不了 stdio 模式**。

#### 模式二 · HTTP（`--transport http`）

- **是什么**：dynamic-mcp 作为**常驻的 HTTP 服务**运行，对外暴露一个 Streamable HTTP MCP 端点（`http://<host>:<port><path>`），任何能发 HTTP 请求的客户端都能连。
- **由谁启动**：**只能由你（用户）手动启动，LLM 不能拉起它**。原因——模式二正是为了弥补 stdio 的短板而新增：浏览器、云端、移动端里的 LLM 应用，根本无法在你的机器上 spawn 一个本地子进程；所以必须**先由你自己在终端或服务里把它跑起来、让它一直监听端口**，远端的 LLM 才能连上来。**在模式二中，LLM 只是「连接者」，永远不是「启动者」**。
- **适用场景**：浏览器插件、云端 Agent、手机 App、远程 / 容器（Docker、k8s）环境，或需要被多个客户端共享同一个后端时。
- **最小启动命令**：

  ```bash
  dmcp --transport http /path/to/dynamic-mcp.json
  ```

#### 模式三 · both（`--transport both`）

- **是什么**：stdio 与 HTTP 同时开启——一个进程，两套入口（对外窗口 + 家门口的门）。
- **亮点 / 优点**：
  - **一个程序顶两个用**：桌面 AI 软件走 stdio 门、浏览器/手机/云端助手走 HTTP 窗口，**一份进程同时服务两类客户端**，不用开两个 dmcp。
  - **共享同一份上游连接和配置**：两种入口共用同一套上游工具与同一份配置文件，省内存、省资源，配置只写一份。
  - **省心**：没有 `both` 就得开两个程序（一个 stdio 给桌面、一个 http 给浏览器），双倍连接、双倍内存、配置维护两处；`both` 免了这些。
- **由谁启动（关键）**：**应由你的桌面 AI 软件（Claude Desktop / Cursor / VS Code）拉起**，不要自己手动在终端启动。软件拉起 dmcp 时推开 stdio 门、站在房子里，进程随之常驻，HTTP 窗口也自然开着，浏览器助手即可连入；若手动启动，stdio 门空转、桌面软件用不上，等于白开 `both`。通俗讲解见上方「三分钟看懂」。
- **适用场景**：本机桌面端「省 token」与浏览器/云端/手机远程使用同一套上游工具，二者同时需要。

### 参数（命令行）

新增的命令行参数用于控制 HTTP 暴露方式（**配置文件无需改动**，见下文）：

| 参数            | 默认值          | 说明（人话）                                      |
| --------------- | --------------- | ------------------------------------------------- |
| `--transport`   | `stdio`         | 决定开几扇门：`stdio` 只给桌面 AI 软件用；`http` 只给浏览器/手机/云端用；`both` 两者都要（推荐让桌面 AI 软件帮你打开）。 |
| `--http-host`   | `127.0.0.1`     | HTTP 对外窗口「绑在哪台机器」。默认 `127.0.0.1` = 只有你本机连得上（最安全）。一般别改；想让同局域网/其他设备也能连才改（有安全风险）。 |
| `--http-port`   | `8082`          | 对外窗口的「门牌号」。默认 8082；若被别的程序占用就换一个（如 9000）。 |
| `--http-path`   | `/dynamic-mcp`  | 窗口上的「房间名」。客户端连接时填的地址末尾要和它对上，例如 `http://127.0.0.1:8082/dynamic-mcp`。 |

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
