# dynamic-mcp

> In one sentence: it is a "tool relay station" that gathers your scattered AI tools into one place, so your AI assistant uses them more cheaply and more conveniently.

## Understand it in 3 minutes (for non-technical readers)

Let's explain it in plain language first; the technical details come later.

### 1. The two problems it solves

**Problem 1: Too many tools — the AI assistant can't keep track, and it costs money**
An AI assistant (like Claude or GPT) can't actually do work by itself; it has to borrow your various "tools" — looking things up, reading files, doing calculations… If you put 20 tools in front of it at once, it has to keep every tool's instruction on its "work desk". The desk gets cluttered, the assistant slows down, and every sentence you exchange costs more money.

dynamic-mcp's clever trick: **at first it shows the assistant only a 3-item "menu"** (list groups, see what's in a group, call a tool). When the assistant actually needs a tool, it fetches that tool's details on the spot. The desk stays tidy — cheaper and simpler.

**Problem 2: The assistant can't reach the tools on your computer**
Many great tools only "sit inside your computer and work when you're standing right next to it". But if the assistant is in a browser, on a phone, or on another machine in the cloud, it "can't walk over" to grab them.

dynamic-mcp can attach a "phone line" to those local tools so they can be reached from afar too (the technical term is "bridging local stdio into Streamable HTTP"). Once bridged, a distant assistant just makes a "call" (connects to a network address) and can use the tools on your computer.

### 2. Two concepts first: launch "mode" vs actual "function"

The content below is easy to get dizzy over, because people mix up "mode" and "function". Let's pin down these two words first:

- **Launch mode (how the door is opened)**: you use the `--transport` command-line switch to choose **how dmcp is started up**. It has three values, like "how many doors this tool room opens":
  - `--transport stdio` → only opens the "front door at your home"
  - `--transport http` → only opens the "external window"
  - `--transport both` → opens both doors
- **Actual function**: the service capability dmcp can actually provide in a given mode. There are only two kinds:
  - **stdio function**: talks to the "desktop AI app standing next to your computer" via standard input/output (through the front door).
  - **http function**: lets "browser / phone / cloud AI assistants" call remotely via an HTTP network endpoint (through the external window).

**Mode ↔ function mapping (the key point of this whole document)**:

| Launch mode (`--transport`) | Doors opened | Actual function | Who can connect |
|---|---|---|---|
| `stdio` | only the front door | **stdio function only** | local desktop AI app |
| `http` | only the window | **http function only** | browser / phone / cloud assistant |
| `both` | both doors | **stdio function + http function** | both kinds of clients at once |

> 📌 **Remember in one line**: `stdio` / `http` / `both` are "**how it starts**" (how the door is opened); stdio function / http function are "**what it can do once started**". Mode decides how many doors open; function is the capability behind the door.

> ⚠️ **One exception (singleton detection can auto-downgrade)**: in rare port-conflict cases, `both` may be auto-downgraded to "**stdio function only, http function off**" (see "HTTP endpoint singleton / double-launch detection" below). This is a conflict self-healing result, not the normal state; normally `both` has both functions on.

### 3. Mode 3 (both) — advantages / highlights

Mode 3 is "one room, two doors, one set of tools". Its biggest benefit is **one program does the job of two**:

1. **One program, usable from two places**: your desktop AI app (WorkBuddy / Claude Desktop / Cursor / VS Code) uses the "front door" (stdio), while browser/phone/cloud assistants use the "external window" (HTTP) — **one process serves both kinds of clients**, no need to run two dmcp instances.
2. **Shares one set of upstream connections and config**: both doors use the same upstream tools and the same config file. No need to maintain two dmcp, connect upstream twice — saves memory and resources; configure only once, and a change takes effect on both sides.
3. **Less hassle**: without Mode 3 you'd run two programs — one stdio for the desktop, one http for the browser — double connections, double memory, two configs to maintain. Mode 3 removes all that.

> ⚠️ **Who should open Mode 3 (important)**: let **your desktop AI app (WorkBuddy / Claude Desktop / Cursor / VS Code) open it for you** — don't launch it manually in a terminal yourself.
> Why: whether the "room has power" depends on "whether an assistant walked in through the front door". When the AI app opens dmcp, it pushes open the front door and stands inside — the room powers on, the external window opens too, and the browser assistant can come in through it.
> If you launch it manually: the front door is open but nobody uses it (waste), and your AI app still can't use it, so it has to open another one → back to two programs, and Mode 3's benefit is gone.
> Correct approach: in your AI app's settings, tell it to open dmcp "with the both mode"; it does the rest.

> 💡 **Tip**: the room's power depends on "whether the front-door assistant is present". If you close the desktop AI app, dmcp is shut down and the external window closes too — the browser assistant loses access immediately. If you want "the browser assistant to keep working even after the desktop app is closed", split Mode 3 into two separate programs: one always-on for the outside (Mode 2 http, you keep it running), and one for the desktop app (Mode 1 or Mode 3, opened by the app itself).

### 4. How to pick the one switch (for those who want to try)

The switch that decides "how many doors to open" is called `--transport`:
- `stdio` → only the front door (for the desktop AI app)
- `http` → only the external window (for browser/phone/cloud)
- `both` → both doors (let the AI app open it for you)

The other parameters (address, port, path) can stay at their defaults; you only touch them when you want devices *other than your own computer* to reach it (see the "Parameters" section in the technical chapter below).

---

The full technical explanation for developers follows (you don't need it to get the gist above).

## About this fork (builds via GitHub Actions)

> **Note**: the upstream repository ([asyrjasalo/dynamic-mcp](https://github.com/asyrjasalo/dynamic-mcp)) has not been updated for an extended period. To use this fork's new features (including the HTTP facade, endpoint singleton detection, hybrid logging, etc., up to **v1.8.2**) without waiting for an upstream release, this fork builds its own binaries through **GitHub Actions** — specifically, the Release workflow triggered when a `v*` tag is pushed. The build artifacts are cross-platform binaries (including Windows `dmcp.exe`), published as release assets on this fork's Releases page.
>
> These builds are **not** published to crates.io / PyPI. **Do not use `cargo install` / `pip install` / `uvx` to install dynamic-mcp**; download the binary directly from this fork's Releases page, or compile from source (see "Quick Start → Installation" below).

## Quick Start

### Installation

#### Option 1: Native binary

Download the executable for your platform from this fork's Releases page (no Rust toolchain, no Python needed):

- **Linux x86_64**: `dmcp-x86_64-unknown-linux-gnu.tar.gz`
- **Linux ARM64**: `dmcp-aarch64-unknown-linux-gnu.tar.gz`
- **Windows x86_64**: `dmcp-x86_64-pc-windows-msvc.zip` (unzip to get `dmcp.exe`)
- **Windows ARM64**: `dmcp-aarch64-pc-windows-msvc.zip`
- **macOS ARM64**: `dmcp-aarch64-apple-darwin.tar.gz`
- (No macOS x86_64 build yet; for other platforms use Option 2 — compile from source)

After downloading and unzipping, put `dmcp` (or `dmcp.exe`) on your `PATH`.

> Download: https://github.com/zhangweildlh/dynamic-mcp/releases

#### Option 2: Compile from source

You need the Rust toolchain installed locally (edition 2021, 1.75+):

```bash
git clone https://github.com/zhangweildlh/dynamic-mcp.git
cd dynamic-mcp
cargo build --release
# Artifact: target/release/dmcp (Windows: target/release/dmcp.exe)
```

> Note: the binaries in this repo are built and published to Releases automatically by GitHub Actions when a `v*` tag is pushed (see "About this fork" above). You normally don't need to compile it yourself.

### Dynamic configuration (dynamic-mcp.json)

#### Import from upstream AI coding tools

Dynamic-mcp can automatically import MCP server configurations from popular AI coding tools.

**Supported tools** (`<tool-name>`):

- Cursor (`cursor`)
- OpenCode (`opencode`)
- Claude Desktop (`claude-desktop`)
- Claude Code CLI (`claude`)
- Visual Studio Code (`vscode`)
- Cline (`cline`)
- KiloCode (`kilocode`)
- Codex CLI (`codex`)
- Gemini CLI (`gemini`)
- Google Antigravity (`antigravity`)

##### Quick Start

**Import from project config** (run in the project directory):

```bash
dmcp import <tool-name>
```

**Import from global / user config**:

```bash
dmcp import --global <tool-name>
```

**Force overwrite** (skip the confirmation prompt):

```bash
dmcp import <tool-name> --force
```

The command will:

1. Detect your tool's config location
2. Parse the existing MCP servers
3. Interactively prompt for a description
4. Interactively prompt for feature selection (tools, resources, prompts)
5. Normalize environment-variable formats
6. Generate `dynamic-mcp.json`

##### Example import

```bash
$ dmcp import cursor

🔄 Starting import from cursor to dynamic-mcp format
📖 Reading config from: .cursor/mcp.json

✅ Found 2 MCP server(s) to import

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
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

**Feature selection**: During import, you can customize which MCP features are enabled per server:

- Press Enter (or Y) to keep all features (tools, resources, prompts)
- Type `n` to selectively enable / disable individual features
- This allows fine-grained control without manually editing the config file

Example of custom feature selection:

```bash
🔧 Keep all features (tools, resources, prompts) for 'server'? [Y/n]: n

  Select features to enable (press Enter to accept default):
  Enable tools? [Y/n]: y
  Enable resources? [Y/n]: n
  Enable prompts? [Y/n]: n
```

##### Tool-specific notes

- **Cursor**: Supports both `.cursor/mcp.json` (project) and `~/.cursor/mcp.json` (global)
- **Claude Desktop**: Global config only, location varies by OS:
  - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
  - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
  - Linux: `~/.config/Claude/claude_desktop_config.json`
- **Claude Code CLI**: Supports both `.mcp.json` (project root) and `~/.claude.json` (user/global)
- **Gemini CLI**: Supports both `.gemini/settings.json` (project) and `~/.gemini/settings.json` (global)
- **VS Code**: Supports both `.vscode/mcp.json` (project) and user-level config (OS-specific paths)
- **OpenCode**: Supports both JSON and JSONC formats (JSON with comments)
- **Codex CLI**: Global only — uses TOML format (`~/.codex/config.toml`)
- **Antigravity**: Global only — `~/.gemini/antigravity/mcp_config.json`

##### Environment variable conversion

The import command automatically normalizes environment variables to dynamic-mcp's `${VAR}` format:

| Tool            | Original Format       | Converted To      |
| --------------- | --------------------- | ----------------- |
| Cursor          | `${env:GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| Claude Desktop  | `${GITHUB_TOKEN}`     | `${GITHUB_TOKEN}` |
| Claude Code CLI | `${GITHUB_TOKEN}`     | `${GITHUB_TOKEN}` |
| VS Code         | `${env:GITHUB_TOKEN}` | `${GITHUB_TOKEN}` |
| Codex           | `"${GITHUB_TOKEN}"`   | `${GITHUB_TOKEN}` |

**Note**: VS Code's `${input:ID}` secure prompts cannot be automatically converted. You'll need to manually configure these after import.

See [docs/IMPORT.md](docs/IMPORT.md) for detailed tool-specific import guides.

#### Write manually from upstream MCP servers

> 💡 The `command: "npx"` / `command: "node"` in the examples below refer to the **upstream MCP server being proxied** (e.g. the filesystem server) and how *that server* starts. `dmcp` itself is the binary you downloaded / compiled from the "Installation" section above — **do not use `npx` / `uvx` / `pip` to install dmcp**.

Create a `dynamic-mcp.json` file with a `description` field for each server:

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

##### Environment variables

It supports the `${VAR}` syntax for environment variable interpolation:

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

##### Server types

It supports all [standard MCP transport mechanisms](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports).

**Note**: The `type` field is **optional** when `url` is present. If omitted, the server automatically uses HTTP transport with SSE detection per the MCP spec. This maintains backwards compatibility with tools like [OpenCode](https://opencode.ai/docs/mcp-servers/).

###### stdio (default)

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

###### http

```json
{
  "description": "HTTP server (type is optional)",
  "url": "https://api.example.com",
  "headers": {
    "Authorization": "Bearer ${TOKEN}"
  }
}
```

Or with explicit type:

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

###### sse

SSE servers are automatically detected when the server responds with `Content-Type: text/event-stream`. You can also explicitly specify `type: "sse"` if the server only supports SSE:

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

###### OAuth authentication (HTTP / SSE)

```json
{
  "description": "OAuth-protected MCP server (type is optional)",
  "url": "https://api.example.com/mcp",
  "oauth_client_id": "your-client-id",
  "oauth_scopes": ["read", "write"]
}
```

**OAuth flow:**

- On first connection, a browser opens for authorization
- Access tokens are stored in `~/.dynamic-mcp/oauth-servers/<server-name>.json`
- Automatic token refresh before expiry (with RFC 6749 token rotation support)
- The token is injected as an `Authorization: Bearer <token>` header

##### Feature flags

Control which MCP features are exposed per server using the optional `features` field. By default, all features (`tools`, `resources`, `prompts`) are enabled. You can selectively disable features:

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

**Behavior:**

- If `features` is omitted, all features are enabled (opt-out design)
- If `features` is specified, unmentioned features default to `true` (enabled)
- Disabled features return an error if accessed via the proxy
- Example: If `resources: false`, calling `resources/list` returns an error

##### Disabling servers

Use the optional `enabled` field to disable a specific server without removing it from the config:

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

**Behavior:**

- If `enabled` is omitted, the server is enabled (default behavior)
- If `enabled: false`, the server is skipped during connection and won't appear in available groups
- Useful for temporarily disabling servers during testing or maintenance without editing config structure
- See `examples/config.features.example.json` for a complete example

##### Timeout configuration

Configure custom timeouts for tool, resource, and prompt calls per server using the optional `timeout` field. By default:

- Tool calls: 30 seconds
- Resource calls: 10 seconds
- Prompt calls: 10 seconds

You can customize these for servers that need more time:

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

**Supported duration formats:**

| Format       | Example               | Description                   |
| ------------ | --------------------- | ----------------------------- |
| Seconds      | `"30s"`, `"5s"`       | Simple seconds                |
| Minutes      | `"1min"`, `"2m"`      | Minutes (abbreviated or full) |
| Milliseconds | `"3000ms"`, `"500ms"` | Milliseconds                  |
| Plain number | `30`                  | Seconds (plain number)        |

**Behavior:**

- If `timeout` is omitted, defaults are used (tools: 30s, resources: 10s, prompts: 10s)
- Individual timeout fields default to their respective defaults if not specified
- Applies only to tool/resource/prompt call operations, not to connection or initialization
- Useful for servers with long-running operations (database queries, file processing, etc.)

### Dynamic launch (including command-line launch)

> 📌 The mapping below corresponds to the "two concepts" above: `stdio` mode → stdio function only; `http` mode → http function only; `both` mode → both functions. The three modes differ in **who starts the process**, which is the easy-to-get-wrong part — pay attention.

#### Mode 1 · stdio (default, `--transport stdio`)

- **What it is**: dynamic-mcp runs as a local subprocess on your machine; data goes over standard input/output (stdio).
- **Who starts it**: The **normal way to run stdio mode is for an LLM client to launch it** — you list `"command": "dmcp"` in the MCP config of a desktop client such as WorkBuddy / Claude Desktop / Cursor / VS Code, and the client starts dmcp and owns its input/output; the process is born when the client starts and dies when the client exits. **You can also run `dmcp config.json` manually in a terminal**, but then its stdin/stdout are only attached to your terminal with no LLM client driving them — there is no "conversation partner", so it cannot actually be used. For this reason, stdio mode is only useful when launched by an LLM client.
- **When to use**: Only when you run a desktop AI client locally on your own machine.
- **Limitation**: An LLM running in a browser, the cloud, or on a phone cannot launch a local process on your machine, so **those environments cannot use stdio mode**.

#### Mode 2 · HTTP (`--transport http`)

- **What it is**: dynamic-mcp runs as a **long-lived HTTP service**, exposing a Streamable HTTP MCP endpoint (`http://<host>:<port><path>`) that any HTTP-capable client can connect to.
- **Who starts it**: **Only you (the user) can start it manually — an LLM cannot launch it.** The reason: Mode 2 was added precisely to fix stdio's shortcoming. An LLM app in a browser, the cloud, or on a phone has no way to spawn a local subprocess on your machine; so **you must first start it yourself in a terminal or as a service and keep it listening**, and only then can a remote LLM connect to it. **In Mode 2 the LLM is only a "connector", never the "launcher".**
- **When to use**: Browser extensions, cloud agents, mobile apps, remote / containerized environments (Docker, k8s), or when multiple clients need to share one backend.
- **Minimal start command**:

  ```bash
  dmcp --transport http /path/to/dynamic-mcp.json
  ```

#### Mode 3 · both (`--transport both`)

- **What it is**: stdio and HTTP run at the same time — one process, two entry points (the external window + the front door).
- **Advantages / highlights**:
  - **One program does the job of two**: the desktop AI app uses the stdio door, browser/phone/cloud assistants use the HTTP window — **one process serves both kinds of clients**, no need to run two dmcp instances.
  - **Shares one set of upstream connections and config**: both entry points use the same upstream tools and the same config file, saving memory and resources, configured only once.
  - **Less hassle**: without `both` you'd run two programs (one stdio for the desktop, one http for the browser) — double connections, double memory, two configs; `both` removes all that.
- **Who starts it (key)**: **let your desktop AI app (WorkBuddy / Claude Desktop / Cursor / VS Code) launch it**, not a manual terminal start. When the app launches dmcp it opens the stdio door and stands inside, the process stays alive, and the HTTP window opens too so the browser assistant can connect. A manual start leaves the stdio door idle and the desktop app can't use it — wasting `both`. See "Understand it in 3 minutes" above for the plain-language version.
- **When to use**: when you need both local desktop token savings AND browser/cloud/mobile remote use of the same upstream tools.

### Parameters (command line)

New command-line flags control HTTP exposure (the config file is **unchanged** — see below):

| Flag            | Default                   | Description (plain language)                                      |
| --------------- | ------------------------- | ----------------------------------------------------------------- |
| `--transport`   | `stdio`                   | Which doors to open: `stdio` for the desktop AI app only; `http` for browser/phone/cloud only; `both` for both (recommended: let the desktop AI app open it for you). |
| `--http-endpoint` | `127.0.0.1:8082/dynamic-mcp` | The full HTTP "address" in `host:port/path` form (IPv6 uses `[host]:port/path`). The client's connection address must match this exactly, e.g. `http://127.0.0.1:8082/dynamic-mcp`. Keep the default; change it only if the port is taken (e.g. `127.0.0.1:9000/dynamic-mcp`). |
| `--no-evict`    | `false`                   | Only valid with `--transport http`. "Locks" the current plain http instance: it tells a future `both` started on the same port "don't kill me", so that `both` runs stdio only and leaves HTTP off — the two coexist peacefully. Passing it with `--transport both` or `stdio` errors out immediately. |
| `--log`         | (none)                    | Log level: `trace` / `debug` / `info` / `warn` / `error` (invalid falls back to `warn`). **When omitted**: http mode outputs `warn`-level logs to stderr; stdio/both are fully silent; no log file. **When passed**: all modes write a log file (`dynamic-<pid>-<timestamp>.log`, in the executable's directory, auto-cleaned after 72h); http mode also mirrors to stderr; stdio/both stay stderr-silent to protect JSON-RPC. |

### Usage

```bash
# stdio only (no HTTP function):
dmcp /path/to/dynamic-mcp.json
dmcp --transport stdio /path/to/dynamic-mcp.json

# HTTP only (no stdio function; for LLMs that can't launch `dynamic-mcp` themselves, e.g. web LLMs / mobile LLMs):
dmcp --transport http /path/to/dynamic-mcp.json

# both (for LLMs that can launch `dynamic-mcp`, e.g. WorkBuddy / OpenCode / Claude Desktop):
dmcp --transport both /path/to/dynamic-mcp.json

# Custom endpoint (host:port/path; IPv6 uses [host]:port/path):
dmcp --transport http --http-endpoint 0.0.0.0:9000/mcp /path/to/dynamic-mcp.json
```

When `--transport http` or `both` is used, the facade is served at `http://<host>:<port><path>` (e.g. `http://127.0.0.1:8082/dynamic-mcp`).

> 💡 **What URL to type in the web LLM (important)**:
> - The full endpoint = `http://<host>:<port><path>`, where `<path>` is exactly the `path` part after `host:port` in your `--http-endpoint` (e.g. `/dynamic-mcp`) — **do not add or remove a `/mcp` prefix or suffix**.
> - Example: with `--http-endpoint 127.0.0.1:8082/dynamic-mcp-server`, the client uses `http://127.0.0.1:8082/dynamic-mcp-server` (verified working).
> - ⚠️ Typing `http://127.0.0.1:8082/mcp/dynamic-mcp-server` or `.../dynamic-mcp-server/mcp` both return **HTTP 404** — this service has no built-in `/mcp` prefix; any extra segment is a wrong path.
> - If the console is completely empty after starting, don't panic: in v1.8.2 without `--log`, stdio/both modes are fully silent, while http mode outputs `warn`-and-above logs to stderr (see "Troubleshooting → Logging" below); as long as the browser doesn't say "connection refused", it is listening. Pass `--log debug` to write logs to a file and (in http mode) stderr.

> 💡 **What address to fill in WorkBuddy / OpenCode / Claude Desktop**:
> These desktop clients typically cooperate with dmcp in two ways:
> - **Launched by the client (stdio / both)**: in the client's MCP config, write `"command": "dmcp"`, `"args": ["--transport", "both", "/abs/path/dynamic-mcp.json"]`. The client handles starting and stdio communication; you don't need to fill in an HTTP address manually.
> - **Connect to dmcp's external window as an "HTTP MCP server"**: when the client needs to fill in an HTTP endpoint directly, use `http://<host>:<port><path>`:
>   - With default `--http-endpoint 127.0.0.1:8082/dynamic-mcp`, fill `http://127.0.0.1:8082/dynamic-mcp`.
>   - When the client and dmcp are on the same machine, `host` can be `127.0.0.1`; if dmcp runs on another machine, fill that machine's LAN IP (and dmcp must use `0.0.0.0` or that IP as host).
>   - Again **do not add a `/mcp` prefix** or you'll get 404.

## Features added in this fork vs upstream (v1.5.0), up to v1.8.2

> The upstream repository ([asyrjasalo/dynamic-mcp](https://github.com/asyrjasalo/dynamic-mcp)) has stayed around v1.5.0; this fork keeps iterating on top of it. The following summarizes, **by feature**, the capabilities this fork (v1.6.0 → v1.8.2) added beyond upstream. Where a capability evolved across versions, the final form is what counts (later versions override earlier ones; version numbers are not listed repeatedly).

### 1. Streamable HTTP facade and multi-mode transport

This fork lets dynamic-mcp do more than a "stdio proxy" — it can also expose the grouped tools through a single Streamable HTTP endpoint, so browsers, the cloud, and phones can call your local tools too.

- Added the `--transport` switch with three modes: `stdio` (default, desktop clients only), `http` (external window only), `both` (stdio + HTTP at once).
- The HTTP facade multiplexes multiple stdio upstream servers into 3 tools exposed externally: `list_groups` (list groups), `get_dynamic_tools` (fetch a group's tool schemas on demand), `call_dynamic_tool` (invoke a specific tool). Clients no longer need to load all tool schemas at once — the "tidy desk, save tokens" design now extends to remote scenarios.
- The `list_groups` meta-tool is also exposed on the stdio surface (previously only on the HTTP facade), helping proxy layers that don't support `enum` do service discovery.

**Application scenario**: You use Claude Desktop (stdio) locally, but also want a web LLM in the browser to call the same set of tools. Run `dmcp --transport both config.json`: the desktop side uses the stdio door, the browser side connects to `http://127.0.0.1:8082/dynamic-mcp` — one process serves both kinds of clients, configured only once.

### 2. HTTP endpoint singleton / double-launch detection and `--no-evict`

When starting `--transport http` / `both`, this fork automatically detects **whether another instance is already running on the same HTTP endpoint**, and resolves conflicts automatically — no more silent failures or port collisions.

- One lock file per endpoint: `~/.dynamic-mcp/locks/<sha256(endpoint) first 16 chars>.lock`, recording the owner's pid, transport mode, `--no-evict` flag, executable path, and (since this version) the config file path.
- Stale locks are identified via **pid liveness + executable-path comparison**, avoiding false "alive" judgments from pid reuse.
- Conflict decisions follow the **R0–R4 rule set** (pure function `decide()`, unit-tested):
  - **R0 (domain split)**: stdio-only instances never arbitrate — they have no HTTP channel to conflict with, so multiple stdio instances can coexist.
  - **R2/R3 (priority + action)**: `both` > `http` > `stdio`. New instance with higher priority evicts the old; otherwise the new instance yields.
  - **R1 (yield method)**: a yielding instance keeps only its stdio channel (HTTP off) — it does not die, it just becomes a stdio-only instance.
  - **R4 (sole exception)**: `--no-evict` exempts only "new `both` evicting old `http`" — the new `both` runs stdio only instead of killing the old `http`.
  - Concrete behavior:
    - A redundant `http` instance: **self-terminates after 15 seconds**, yielding the port to the earlier one.
    - A later `both`: **takes over (evicts)** an existing `http` (unless that http started with `--no-evict`), occupying the port ~8s later with stdio immediately usable.
    - `both` vs `both` (or vs an `http` with `--no-evict`): keep the earlier one; the **later one runs stdio only, HTTP off** — both coexist without conflict.
- Added the `--no-evict` flag (only valid for plain `http`): marks "**this** instance must not be evicted" — it does **not** let this instance evict others. A later `both` will run stdio only instead of killing this `http`.
- **Config-consistency warning**: when two instances share the same endpoint but use different config files, the popup explicitly warns about the mismatch (instead of staying silent). This prevents "project A's window accidentally serving project B's config" cross-contamination.
- **Popup auto-close**: the notification popup closes automatically after **15 seconds** (enough reading time), but the user can manually close it earlier. The 15-second window also governs `SelfTerminate` exit delay, ensuring the popup is fully read before the process exits.
- **STDIO registration**: each stdio-only instance writes a small registry file (`~/.dynamic-mcp/instances/stdio-<pid>.json`) at startup and deletes it on exit (RAII). These files do **not** participate in arbitration — they exist solely for observability.
- Added the `dmcp status` subcommand: scans `~/.dynamic-mcp/locks/` and `~/.dynamic-mcp/instances/` to list all known instances (HTTP-domain locks + STDIO-domain registrations), marking each as alive/dead. No config file needed.
- HTTP binding uses `SO_REUSEADDR` + ~10s retry, so takeover can complete while the port is in `TIME_WAIT`.

**Application scenarios**:
1. You double-click `dmcp --transport both` by accident — the second detects the first already holds the port, auto-runs stdio only with HTTP off, avoiding a "port occupied" crash.
2. You first start a long-lived `dmcp --transport http --no-evict config.json` (dedicated to the browser), then your desktop app launches a `both` — because the http has `--no-evict`, `both` doesn't kill it and runs stdio only; the two instances coexist peacefully.
3. You work on multiple projects in parallel, each with its own `dmcp --transport both projectN.json`. When two instances accidentally target the same endpoint, the popup now flags the config mismatch explicitly — you immediately see "this window is serving project A's config, not project B's", preventing silent cross-contamination. Run `dmcp status` to see all live instances at a glance.

### 3. Single `--http-endpoint` parameter (breaking change) + IPv6 / popup fixes

This fork merged the previously separate `--http-host` / `--http-port` / `--http-path` into a single `--http-endpoint` (`host:port/path`, IPv6 uses `[host]:port/path`), with the default `127.0.0.1:8082/dynamic-mcp` unchanged. The old three flags are removed; your startup scripts and the LLM's MCP config must switch to the single flag.

Three defects were fixed alongside this merge:
- **IPv6 endpoint binding crash**: the bind address now gets square brackets auto-added before parsing (v1.8.0 crashed on IPv6 hosts).
- **IPv6 singleton popup canonical-key error**: the singleton lock's canonical key is now `host:port/path` (IPv6 brackets stripped), so the popup always shows the address (including IPv6).
- **macOS popup multiline newline dropped**: `osascript` now joins multiple lines with `" & return & "`, avoiding silent popup failure from lost newlines.

**Application scenarios**:
- To let other devices on the LAN connect: `dmcp --transport http --http-endpoint 0.0.0.0:9000/mcp config.json`.
- Local over IPv6: `dmcp --transport http --http-endpoint "[::1]:8082/dynamic-mcp" config.json` (no longer crashes since v1.8.1).

### 4. Default config file name `dynamic-mcp.json`

This fork changed the default config file name from `dmcp_config.json` to `dynamic-mcp.json`, unifying it with the binary name `dmcp`. Lookup priority: **CLI positional argument** → **`DYNAMIC_MCP_CONFIG` environment variable** → **`dynamic-mcp.json` next to the executable**.

**Application scenario**: name your config `dynamic-mcp.json` and place it next to the `dmcp` binary; running `dmcp` loads it directly, no need to pass a path every time.

### 5. Fork builds and publishes binaries via GitHub Actions

Because upstream hasn't released for a long time, this fork builds on its own: when a `v*` tag is pushed, the GitHub Actions Release workflow compiles cross-platform binaries (Linux x86_64 / ARM64, Windows x86_64 / ARM64, macOS ARM64) and publishes them to this fork's Releases. **Not published to crates.io / PyPI.** Users get a usable binary without a local Rust toolchain (see "Quick Start → Installation" above).

**Application scenario**: Without installing a Rust toolchain, you can download `dmcp` directly from [Releases](https://github.com/zhangweildlh/dynamic-mcp/releases) and use it; the upstream repo has no corresponding up-to-date binary available.

### 6. `--log` hybrid logging (new in v1.8.2)

v1.8.0 simplified server-mode logging to "always silent", making debugging inconvenient. v1.8.2 introduces `--log <LEVEL>`, using a hybrid approach — file logging for all modes, plus stderr for http only — balancing diagnostics with JSON-RPC protocol safety:

- **Without `--log`**: http mode outputs `warn`-level logs to stderr (for quick connection debugging); stdio / both are fully silent; no log file.
- **With `--log <LEVEL>`**: all modes write `dynamic-<pid>-<timestamp>.log` (executable directory, auto-cleaned after 72h); http also mirrors to stderr; stdio/both stay stderr-silent.
- `LEVEL`: `trace` / `debug` / `info` / `warn` / `error` (invalid falls back to `warn`).

**Application scenario**: Debug http connection issues with `dmcp --transport http --log debug config.json` — real-time stderr plus file archival; debug stdio with `--log debug` — file only, no JSON-RPC pollution.

### 7. `get_dynamic_tools` enhanced parameters (new in v1.8.2)

The `get_dynamic_tools` meta-tool gains 6 optional parameters. **Defaults preserve byte-identical output with v1.8.1** — passing none of the new parameters returns exactly the same result as before, with zero compatibility risk.

#### Parameter overview

| Parameter | Type | Default | Description |
|---|---|---|---|
| `mode` | string | `full` | `compact` returns name + full description, omits `inputSchema` |
| `include_schema` | bool | `true` | Set `false` to strip `inputSchema`; has no effect in `compact` mode |
| `page` | int | `1` | Page number (1-based), used with `page_size` |
| `page_size` | int | `0` | `>0` wraps results as `{tools, total, page, page_size, has_more}`; `0` returns flat array (same as before) |
| `land_to_file` | bool | `false` | `true` writes results to a JSON file and returns the absolute path (72h auto-cleanup) |
| `capabilities` | bool | `false` | `true` attaches `x-capabilities` array (from `GroupInfo.capability_tags`) to each tool entry |

#### Detailed parameter descriptions

**`mode` (output mode)**:
- `full` (default): Returns complete name + description + inputSchema, byte-identical to v1.8.1.
- `compact`: Returns name + full description, **omits inputSchema**. When tools have large parameter schemas, data volume drops to 1/5 ~ 1/10. Ideal for LLMs to quickly scan available tools, then fetch specific schemas on demand.

**`include_schema` (include parameter schema)**:
- Fine-grained control independent of `mode`. `mode=full` + `include_schema=false` also strips inputSchema without triggering compact behavior.
- Typical workflow: first use `mode=compact` to browse the tool list, then use `mode=full` (default `include_schema=true`) to get the full parameter schema for the tool of interest.

**`page` + `page_size` (pagination)**:
- `page_size=0` (default): Returns all tools at once as a flat JSON array, identical to old behavior.
- `page_size>0`: Paginates, wrapping results as `{tools: [...], total: 120, page: 1, page_size: 20, has_more: true}`. LLMs can use `has_more` to decide whether to fetch the next page.
- Ideal when the total tool count is very large (e.g., 10+ MCP servers, hundreds of tools).

**`land_to_file` (file landing mode)**:
- `false` (default): Results returned inline in the JSON-RPC response.
- `true`: Results written to a JSON file (`dynamic-tools-<pid>-<timestamp>.json`, executable directory, 72h auto-cleanup); only the absolute file path is returned.
- Ideal when inline results might exceed JSON-RPC message size limits. The LLM receives a path and can read the file contents as needed.

**`capabilities` (capability tags)**:
- `false` (default): No capability tags attached.
- `true`: Attaches an `x-capabilities` array to each tool entry, sourced from the group's `capability_tags` (e.g., `["http", "oauth"]` or `["stdio"]`).
- Helps LLMs understand each tool's transport type and authentication requirements, adjusting call strategy accordingly (OAuth tools may be slower; stdio tools respond faster).

#### Usage examples

```jsonc
// 1. Quick browse (compact, saves ~80% data):
get_dynamic_tools({ "group": "firecrawl-mcp", "mode": "compact" })

// 2. Paginated retrieval (20 per page):
get_dynamic_tools({ "group": "big-server", "page": 1, "page_size": 20 })
// → { "tools": [...20 items...], "total": 95, "page": 1, "page_size": 20, "has_more": true }

// 3. File landing (when tools are extremely numerous):
get_dynamic_tools({ "group": "mega-server", "land_to_file": true })
// → "/path/to/dynamic-tools-12345-20260716-120000.json"

// 4. With capability tags:
get_dynamic_tools({ "group": "github-server", "capabilities": true })
// → Each tool gets "x-capabilities": ["http", "oauth"]
```

#### Impact on LLMs (core value)

- **Context window savings**: The biggest improvement. An LLM's memory is limited; returning hundreds of tools with full parameter schemas can consume tens of thousands of tokens. `compact` mode reduces this to 1/5~1/10; pagination allows batch retrieval.
- **Two-step tool discovery**: LLMs can first use `compact` to quickly scan all tool names + descriptions, then use `full` to fetch the parameter schema for the target tool — saving tokens without losing precision.
- **Ultra-large-scale fallback**: When tools number in the thousands, `land_to_file` writes results to a file and returns only the path, avoiding JSON-RPC response size limits.
- **Transport awareness**: `capabilities` lets LLMs know whether a tool is local stdio (fast, no auth) or remote http+oauth (may timeout, requires auth), adjusting timeout expectations and retry strategies.

### 8. `list_groups` metadata enrichment (new in v1.8.2)

Each group returned by `list_groups` (corresponding to one upstream MCP server) now includes two new fields:

| New field | Type | Description |
|---|---|---|
| `tool_count` | int | Number of tools exposed by this group |
| `capability_tags` | string[] | Capability tags, auto-derived from transport type |

#### `capability_tags` auto-generation rules

| Transport type | Tags |
|---|---|
| stdio (local command-line) | `["stdio"]` |
| http (remote HTTP) | `["http"]` |
| sse (Server-Sent Events) | `["sse"]` |
| Any + requires OAuth | Appends `"oauth"` to the above |

#### Example

```json
[
  {
    "name": "github-server",
    "description": "GitHub MCP server",
    "tool_count": 45,
    "capability_tags": ["http", "oauth"]
  },
  {
    "name": "local-fs",
    "description": "Filesystem tools",
    "tool_count": 8,
    "capability_tags": ["stdio"]
  }
]
```

#### Impact on LLMs

- **Smarter group selection**: LLMs call `list_groups` first to see the overview — "github-server has 45 tools, needs OAuth" vs "local-fs has 8 tools, local stdio" — and can prioritize exploring the more relevant group.
- **Fewer wasted round-trips**: Previously, LLMs didn't know how many tools a group had until they called `get_dynamic_tools`. Now `tool_count` gives this instantly.
- **Transport awareness**: `capability_tags` give LLMs a preview of tool "personality" — local stdio is fast but limited to the local environment; http+oauth is powerful but may timeout.
- **Fully backward compatible**: `GroupInfo` struct has no `deny_unknown_fields`, so older clients seeing new fields won't error.

### 9. Structured error envelope for tool calls (new in v1.8.2)

When `call_dynamic_tool` fails, the error response changes from plain text to a structured JSON envelope, making errors "programmable" — each error has a clear type code.

#### Error format

```json
{
  "ok": false,
  "code": "timeout",
  "message": "Tool execution timed out: github-search",
  "cause": null
}
```

#### Error code mapping

| Original error contains | code value | Meaning |
|---|---|---|
| `timed out` | `timeout` | Timeout — upstream server did not respond in time |
| `Tool execution failed` | `upstream_error` | Upstream error — the server itself reported an error |
| `Missing required` | `bad_request` | Bad request — missing a required parameter |
| Other | `tool_error` | Other type of error |

- `cause` is currently fixed at `null`, reserved for future expansion (e.g., original error chains).
- **Important**: This envelope is returned via `CallToolResult`'s `content` field (`is_error: true`), not as a JSON-RPC protocol-level `error`. The LLM receives a "tool execution failed" result, not a communication failure, and can process it normally.

#### Impact on LLMs (core value)

- **Programmable error handling**: Previously, LLMs had to "guess" the error type from natural language text. Now they read the `code` field for precise classification.
- **Smart retry strategies**:
  - `timeout` → Possibly a network hiccup; reasonable to retry once
  - `bad_request` → Wrong parameters; should not retry — fix parameters and call again
  - `upstream_error` → Upstream server problem; try a different tool or inform the user
  - `tool_error` → Unknown error; inform the user or try an alternative approach
- **Reduced wasteful retries**: Previously, LLMs might blindly retry all errors 3 times, wasting tokens and time. With error classification, LLMs retry only when it makes sense.

### 10. OAuth fixes (new in v1.8.2)

- **Callback `localhost` → `127.0.0.1`**: the OAuth callback listener binds `127.0.0.1`, but the redirect URI used `localhost`; on some systems `localhost` resolves to `::1` (IPv6), causing the browser redirect to fail. Both sides now use `127.0.0.1`.
- **Static `Authorization` header skips OAuth discovery**: servers with a static `Authorization` header no longer trigger an unnecessary OAuth discovery round-trip; they connect directly.
- **OAuth transport creation timeout 120s → 300s**: gives users a more generous window for browser-based authorization.

## Troubleshooting

### Server connection issues

**Problem**: `❌ Failed to connect to <server>`

**Solutions**:

- **Connection timeout**: Each server has 10-second timeout for transport creation, initialization, and tool listing
- **Automatic retry**: Failed servers are retried up to 3 times with exponential backoff (2s, 4s, 8s)
- **Periodic retry**: Failed servers are retried every 30 seconds in the background
- **Slow HTTP servers**: If remote HTTP / SSE servers are slow, they'll timeout and be retried automatically
- **Stdio servers**: Verify command exists (`which <command>`)
- **HTTP / SSE servers**: Check that the server is running and the URL is correct
- **Environment variables**: Ensure all `${VAR}` references are defined
- **OAuth servers**: Complete OAuth flow when prompted

### Logging and "no console output" (important)

In **v1.8.2**, when running the server (`--transport stdio` / `http` / `both`), logging behavior depends on whether `--log` is passed:

#### Without `--log` (default)

- **http mode**: outputs `warn`-and-above logs to stderr — this is not a crash; it is by design, helping you spot connection issues quickly.
- **stdio / both modes**: fully silent — no file, no stderr, keeping JSON-RPC clean.
- No log file is written. Regardless of `RUST_LOG`, the log level is controlled by `--log` (defaults to `warn` when omitted).
- How to tell it's running? When connecting from a browser / web LLM, if you get **`HTTP 404`** (rather than "connection refused"), the server is listening — you just used the wrong **path**.

> ⚠️ **Don't be fooled by the empty console**: in stdio/both modes, seeing nothing in the terminal doesn't mean it failed to start — it is quietly listening on `127.0.0.1:8082`. In http mode, a few `warn` lines on stderr are normal.

#### With `--log <LEVEL>`

- **All modes write a log file**: `dynamic-<pid>-<YYYYMMDD-HHMMSSmmm>.log` in the executable's directory (read-only fallback: `data_local_dir/dynamic-mcp/`).
- **http mode also mirrors to stderr**: for real-time terminal viewing.
- **stdio / both stay stderr-silent**: to protect the JSON-RPC protocol.
- **Auto-cleanup**: on startup, `dynamic-*.log` files older than 72 hours are removed (the current run is excluded).
- `LEVEL`: `trace` / `debug` / `info` / `warn` / `error`; invalid values fall back to `warn`.

```bash
# Debug http mode (logs to both file and stderr):
dmcp --transport http --log debug /path/to/dynamic-mcp.json

# Debug stdio mode (file only, stderr stays silent):
dmcp --transport stdio --log debug /path/to/dynamic-mcp.json
```

> 💡 **Why v1.8.0 removed the old `--log-level` flag**: stdio mode must keep stderr clean to avoid polluting JSON-RPC. v1.8.2's `--log` uses a hybrid approach — file logging for all modes, plus stderr for http only — balancing diagnostics with protocol safety.

### OAuth authentication issues

**Problem**: The browser doesn't open for OAuth

**Solutions**:

- Manually open the URL shown in the console
- Check that the firewall allows localhost connections
- Verify `oauth_client_id` is correct for the server

**Problem**: Token refresh fails

**Solutions**:

- Delete cached token: `rm ~/.dynamic-mcp/oauth-servers/<server-name>.json`
- Re-authenticate on next connection

### Environment variable not substituted

**Problem**: Config shows `${VAR}` instead of value

**Solutions**:

- Use `${VAR}` syntax, not `$VAR`
- Export variable: `export VAR=value`
- Variable names are case-sensitive
- Check for typos in variable name

### Configuration errors

**Problem**: `Server missing 'description' field`

**Solutions**:

- Every MCP server in your config must have a `description` field
- The description explains what the server does to the LLM
- Example:

  ```json
  {
    "description": "File system access - read, write, and search files",
    "command": "npx",
    "args": ["@modelcontextprotocol/server-filesystem"]
  }
  ```

**Problem**: `Invalid JSON in config file`

**Solutions**:

- Validate JSON syntax (use `jq . config.json`)
- Check for trailing commas
- Ensure all required fields are present (`description` is always required; `type` is required only for http/sse servers)

**Problem**: Unknown field in config (e.g., `unknown field \`typo_field\`\`)

**Solutions**:

- dynamic-mcp uses strict JSON schema validation that only allows defined fields
- Check for typos in field names: `description`, `command`, `url`, `type`, `args`, `env`, `headers`, `oauth_client_id`, `oauth_scopes`, `features`, `enabled`, `timeout`
- Remove any extra or misspelled fields from your config
- Refer to the schema examples above to see valid fields for each server type

**Problem**: `Failed to resolve config path`

**Solutions**:

- Use an absolute path or a path relative to the working directory
- Check that the file exists and has read permissions
- Try: `ls -la <config-path>`

### Tool call failures

**Problem**: Tool call returns error

**Debugging**:

1. Test the tool directly with the upstream server
2. Check that the tool name and arguments match the schema
3. Verify the group name is correct
4. Enable debug logging to see JSON-RPC messages

### Performance issues

**Problem**: Slow startup

**Solutions**:

- Parallel connections already enabled
- Check network latency for HTTP / SSE servers
- Some servers may be slow to initialize (normal)

**Problem**: High memory usage

**Solutions**:

- Tools are cached in memory (expected)
- Failed groups use minimal memory
- Large tool schemas contribute to memory usage

## Contributing

For instructions on development setup, testing, and contributing, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Release History

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.

## Acknowledgments

- TypeScript implementation: [modular-mcp](https://github.com/d-kimuson/modular-mcp)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)
- Rust MCP Ecosystem: [rust-mcp-stack](https://github.com/rust-mcp-stack)
- Upstream repository: [asyrjasalo/dynamic-mcp](https://github.com/asyrjasalo/dynamic-mcp) (this fork iterates on top of this repo's v1.5.0 branch)
