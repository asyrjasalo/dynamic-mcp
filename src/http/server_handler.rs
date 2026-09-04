//! HTTP facade for dynamic-mcp.
//!
//! Implements an `rmcp::ServerHandler` that exposes dynamic-mcp's grouped-tool
//! API as a single Streamable HTTP MCP endpoint. All configured stdio upstream
//! groups are multiplexed behind a 3-tool surface:
//!
//! * `list_groups`       — discover available (and failed) MCP groups
//! * `get_dynamic_tools` — list tools of a specific group
//! * `call_dynamic_tool` — execute a tool in a specific group
//!
//! This mirrors the stdio meta-tool surface (`server.rs`) but is served over
//! the MCP Streamable HTTP transport, enabling remote Agents (Claude Desktop,
//! Cursor, etc.) to consume dynamic-mcp without a local stdio bridge.

use crate::proxy::ModularMcpClient;
use crate::singleton::dmcp_data_dir;
use rmcp::model::*;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ServerHandler;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP `ServerHandler` backed by the shared `ModularMcpClient`.
///
/// Instances are cheap to clone (they only hold an `Arc` to the client plus a
/// name/version pair), which is required because the Streamable HTTP server
/// creates one service instance per session via a factory closure.
#[derive(Clone)]
pub struct HttpFacadeHandler {
    client: Arc<RwLock<ModularMcpClient>>,
    name: String,
    version: String,
}

impl HttpFacadeHandler {
    pub fn new(client: Arc<RwLock<ModularMcpClient>>, name: String, version: String) -> Self {
        Self {
            client,
            name,
            version,
        }
    }

    async fn list_tools_inner(&self) -> Result<ListToolsResult, rmcp::ErrorData> {
        let client = self.client.read().await;
        let groups = client.list_groups();
        let failed_groups = client.list_failed_groups();

        let group_names: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();

        let groups_desc = groups
            .iter()
            .map(|g| format!("- {}: {}", g.name, g.description))
            .collect::<Vec<_>>()
            .join("\n");

        let failed_desc = if !failed_groups.is_empty() {
            let failed = failed_groups
                .iter()
                .map(|g| format!("- {}: {} (Error: {})", g.name, g.description, g.error))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n\nUnavailable groups (connection failed):\n{failed}")
        } else {
            String::new()
        };

        let get_tools_desc = build_get_tools_desc(&groups_desc, &failed_desc);

        Ok(ListToolsResult {
            tools: vec![
                Tool::new(
                    "list_groups",
                    LIST_GROUPS_DESC,
                    json_object(list_groups_schema()),
                ),
                Tool::new(
                    "get_dynamic_tools",
                    get_tools_desc,
                    json_object(get_tools_schema(&group_names)),
                ),
                Tool::new(
                    "call_dynamic_tool",
                    CALL_TOOL_DESC,
                    json_object(call_tool_schema(&group_names)),
                ),
            ],
            ..Default::default()
        })
    }

    async fn call_tool_inner(&self, name: &str, arguments: serde_json::Value) -> CallToolResult {
        let mut client = self.client.write().await;

        let text_result: Result<String, String> = match name {
            "list_groups" => {
                let groups = client.list_groups();
                let failed_groups = client.list_failed_groups();

                let mut all: Vec<serde_json::Value> = Vec::new();
                for g in &groups {
                    all.push(json!({
                        "name": g.name,
                        "description": g.description,
                        "status": "connected"
                    }));
                }
                for g in &failed_groups {
                    all.push(json!({
                        "name": g.name,
                        "description": g.description,
                        "status": "failed",
                        "error": g.error
                    }));
                }

                Ok(serde_json::to_string_pretty(&all).unwrap_or_else(|_| "[]".to_string()))
            }
            "get_dynamic_tools" => {
                let group = arguments.get("group").and_then(|v| v.as_str());
                match group {
                    None => Err("Missing required parameter: group".to_string()),
                    Some(g) => match client.list_tools(g).await {
                        Ok(tools) => {
                            let mode = arguments
                                .get("mode")
                                .and_then(|v| v.as_str())
                                .unwrap_or("full");
                            let include_schema = arguments
                                .get("include_schema")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true);
                            let page = arguments
                                .get("page")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(1)
                                .max(1) as usize;
                            let page_size = arguments
                                .get("page_size")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as usize;
                            let land_to_file = arguments
                                .get("land_to_file")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            let capabilities = arguments
                                .get("capabilities")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            let compact = mode == "compact";

                            // opt-in x-capabilities：取该组的 capability_tags（W5D）。
                            let tags: Vec<String> = if capabilities {
                                client
                                    .list_groups()
                                    .into_iter()
                                    .find(|gi| gi.name == g)
                                    .map(|gi| gi.capability_tags)
                                    .unwrap_or_default()
                            } else {
                                Vec::new()
                            };

                            let mut items: Vec<serde_json::Value> = tools
                                .iter()
                                .map(|tool| {
                                    let mut obj = json!({
                                        "name": tool.name,
                                        "description": tool.description,
                                    });
                                    if include_schema && !compact {
                                        let mut schema = tool.input_schema.clone();
                                        if let Some(o) = schema.as_object_mut() {
                                            o.remove("$schema");
                                        }
                                        obj["inputSchema"] = schema;
                                    }
                                    if capabilities {
                                        obj["x-capabilities"] = serde_json::Value::Array(
                                            tags.iter()
                                                .map(|t| serde_json::Value::String(t.clone()))
                                                .collect(),
                                        );
                                    }
                                    obj
                                })
                                .collect();

                            // 分页：page_size>0 返回包装 {tools,total,page,page_size,has_more}；
                            // page_size==0（默认）返回扁平数组，与旧输出逐字节一致。
                            let output: serde_json::Value = if page_size > 0 {
                                let total = items.len();
                                let start = (page - 1) * page_size;
                                let end = std::cmp::min(start + page_size, total);
                                let slice: Vec<serde_json::Value> = if start < total {
                                    items.drain(start..end).collect()
                                } else {
                                    Vec::new()
                                };
                                let has_more = end < total;
                                json!({
                                    "tools": slice,
                                    "total": total,
                                    "page": page,
                                    "page_size": page_size,
                                    "has_more": has_more
                                })
                            } else {
                                serde_json::Value::Array(items)
                            };

                            if land_to_file {
                                match write_dynamic_tools_file(&output) {
                                    Ok(path) => Ok(path),
                                    Err(e) => Err(format!("Failed to write tools file: {e}")),
                                }
                            } else {
                                Ok(serde_json::to_string_pretty(&output)
                                    .unwrap_or_else(|_| "[]".to_string()))
                            }
                        }
                        Err(e) => Err(format!("Failed to list tools: {e}")),
                    },
                }
            }
            "call_dynamic_tool" => {
                let group = arguments.get("group").and_then(|v| v.as_str());
                let tool_name = arguments.get("name").and_then(|v| v.as_str());
                let args = arguments.get("args").cloned().unwrap_or(json!({}));

                match (group, tool_name) {
                    (Some(g), Some(n)) => match client.call_tool(g, n, args).await {
                        Ok(result) => Ok(serde_json::to_string_pretty(&result).unwrap_or_default()),
                        Err(e) => Err(format!("Tool execution failed: {e}")),
                    },
                    _ => Err("Missing required parameters: group and name".to_string()),
                }
            }
            _ => Err(format!("Unknown tool: {name}")),
        };

        match text_result {
            Ok(text) => result_ok(text),
            // W5F: 结构化错误信封。仍为 CallToolResult{is_error:true}（非 JSON-RPC
            // error），宿主解析 content 文本即可拿到 {ok,code,message,cause}。
            // `code` 是信封内字符串字段，不是 JSON-RPC error.code（i32）。
            Err(msg) => {
                let code = if msg.contains("timed out") {
                    "timeout"
                } else if msg.contains("Tool execution failed") {
                    "upstream_error"
                } else if msg.contains("Missing required") {
                    "bad_request"
                } else {
                    "tool_error"
                };
                // cause 预留字段，暂固定为 null，后续可扩展为原始错误链。
                let envelope = serde_json::json!({
                    "ok": false,
                    "code": code,
                    "message": msg,
                    "cause": null
                });
                result_err(serde_json::to_string_pretty(&envelope).unwrap_or(msg))
            }
        }
    }
}

impl ServerHandler for HttpFacadeHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: self.name.clone(),
                title: None,
                version: self.version.clone(),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "dynamic-mcp exposes grouped MCP tools over Streamable HTTP. \
                 Use list_groups to discover groups, get_dynamic_tools to list a \
                 group's tools, and call_dynamic_tool to execute them."
                    .to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        self.list_tools_inner().await
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let name = request.name.as_ref();
        let arguments: serde_json::Value = match request.arguments {
            Some(map) => serde_json::Value::Object(map),
            None => serde_json::Value::Null,
        };
        Ok(self.call_tool_inner(name, arguments).await)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const LIST_GROUPS_DESC: &str = "Discover available MCP server groups. Returns each group's name, description, and connection status (connected/failed).\n\nUse this tool first when `enum` fields are not available (e.g., when behind an MCP proxy that strips them), then use get_dynamic_tools to list tools in a specific group, and call_dynamic_tool to execute them.\n\nNo parameters required.";

const CALL_TOOL_DESC: &str = "Execute a tool from a specific MCP group. Proxies the call to the appropriate upstream MCP server.\n\nUse get_dynamic_tools first to discover available tools and their input schemas in the specified group, then use this tool to execute them.\n\nThis maintains a clean separation between discovery (context-efficient) and execution phases, enabling effective management of large tool collections across multiple MCP servers.\n\nExample usage:\n  call_dynamic_tool(group=\"playwright\", name=\"browser_navigate\", args={\"url\": \"https://example.com\"})\n  -> Executes the browser_navigate tool from the playwright group with the specified arguments";

fn list_groups_schema() -> serde_json::Value {
    json!({ "type": "object", "properties": {} })
}

fn get_tools_schema(group_names: &[String]) -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "group": {
                "type": "string",
                "description": "The name of the MCP group to get tools from",
                "enum": group_names
            },
            "mode": {
                "type": "string",
                "description": "Output mode: 'full' (name + description + inputSchema, default) or 'compact' (name + description only, schema omitted).",
                "enum": ["full", "compact"]
            },
            "include_schema": {
                "type": "boolean",
                "description": "Include inputSchema in output. Default true. Has no effect in compact mode."
            },
            "page": {
                "type": "integer",
                "description": "1-based page number for pagination. Default 1."
            },
            "page_size": {
                "type": "integer",
                "description": "Items per page. 0 (default) returns all tools as a flat array (no pagination wrapper)."
            },
            "land_to_file": {
                "type": "boolean",
                "description": "If true, write the result to a JSON file and return its absolute path instead of inline JSON. Default false."
            },
            "capabilities": {
                "type": "boolean",
                "description": "If true, attach an 'x-capabilities' array (group-level tags such as http/sse/stdio/oauth) to each tool. Default false."
            }
        },
        "required": ["group"]
    })
}

fn call_tool_schema(group_names: &[String]) -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "group": {
                "type": "string",
                "description": "The name of the MCP group containing the tool",
                "enum": group_names
            },
            "name": {
                "type": "string",
                "description": "The name of the tool to execute"
            },
            "args": {
                "type": "object",
                "description": "Arguments to pass to the tool",
                "additionalProperties": true
            }
        },
        "required": ["group", "name"]
    })
}

fn json_object(value: serde_json::Value) -> Arc<JsonObject> {
    Arc::new(value.as_object().cloned().unwrap_or_default())
}

fn build_get_tools_desc(groups_desc: &str, failed_desc: &str) -> String {
    format!(
        "dynamic-mcp manages multiple MCP servers as organized groups, \
        providing only the necessary group's tool descriptions to the LLM \
        on demand instead of overwhelming it with all tool descriptions at once.\n\n\
        Use this tool to retrieve available tools in a specific group, \
        then use call_dynamic_tool to execute them.\n\n\
        Available groups:\n{}{}",
        groups_desc, failed_desc
    )
}

fn result_ok(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(text)],
        structured_content: None,
        is_error: Some(false),
        meta: None,
    }
}

fn result_err(text: String) -> CallToolResult {
    CallToolResult {
        content: vec![Content::text(text)],
        structured_content: None,
        is_error: Some(true),
        meta: None,
    }
}

/// Resolve the directory for on-demand tool-artifact files (e.g. land_to_file
/// output). Mirrors the log directory policy: next to the executable when
/// writable, otherwise `~/.dynamic-mcp/`.
fn artifact_dir() -> std::path::PathBuf {
    dmcp_data_dir()
}

/// Serialize `value` to a uniquely-named JSON file and return its absolute
/// path. Best-effort cleans up stale `dynamic-tools-*.json` (>72h) in the same
/// directory. Used by get_dynamic_tools `land_to_file`.
pub(crate) fn write_dynamic_tools_file(value: &serde_json::Value) -> std::io::Result<String> {
    let dir = artifact_dir();
    std::fs::create_dir_all(&dir)?;
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S%3f").to_string();
    let path = dir.join(format!("dynamic-tools-{}-{}.json", std::process::id(), ts));
    let content = serde_json::to_string_pretty(value).unwrap_or_else(|_| "[]".to_string());
    std::fs::write(&path, content)?;
    cleanup_dynamic_tools_artifacts(&dir);
    Ok(path.to_string_lossy().to_string())
}

/// Remove `dynamic-tools-*.json` artifacts older than 24h (best-effort).
fn cleanup_dynamic_tools_artifacts(dir: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let cutoff = std::time::SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(24 * 3600))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let p = entry.path();
        let is_json = p.extension().and_then(|s| s.to_str()) == Some("json");
        let is_prefixed = p
            .file_name()
            .and_then(|s| s.to_str())
            .map(|n| n.starts_with("dynamic-tools-"))
            .unwrap_or(false);
        if !(is_json && is_prefixed) {
            continue;
        }
        if let Ok(meta) = std::fs::metadata(&p) {
            if let Ok(m) = meta.modified() {
                if m < cutoff {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn make_handler() -> HttpFacadeHandler {
        HttpFacadeHandler::new(
            Arc::new(RwLock::new(ModularMcpClient::new())),
            "dynamic-mcp".to_string(),
            "0.0.0".to_string(),
        )
    }

    #[tokio::test]
    async fn list_tools_returns_three_facade_tools() {
        let h = make_handler();
        let res = h.list_tools_inner().await.unwrap();
        assert_eq!(res.tools.len(), 3);
        let names: Vec<&str> = res.tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(names.contains(&"list_groups"));
        assert!(names.contains(&"get_dynamic_tools"));
        assert!(names.contains(&"call_dynamic_tool"));
    }

    #[tokio::test]
    async fn call_list_groups_returns_empty_array() {
        let h = make_handler();
        let res = h
            .call_tool_inner("list_groups", serde_json::Value::Null)
            .await;
        assert_eq!(res.is_error, Some(false));
        assert_eq!(res.content.len(), 1);
    }

    #[tokio::test]
    async fn call_get_dynamic_tools_missing_group_errors() {
        let h = make_handler();
        let res = h.call_tool_inner("get_dynamic_tools", json!({})).await;
        assert_eq!(res.is_error, Some(true));
    }

    #[tokio::test]
    async fn call_call_dynamic_tool_missing_params_errors() {
        let h = make_handler();
        let res = h
            .call_tool_inner("call_dynamic_tool", json!({ "group": "x" }))
            .await;
        assert_eq!(res.is_error, Some(true));
    }

    #[tokio::test]
    async fn call_unknown_tool_errors() {
        let h = make_handler();
        let res = h.call_tool_inner("does-not-exist", json!({})).await;
        assert_eq!(res.is_error, Some(true));
    }
}
