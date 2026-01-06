use crate::proxy::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::proxy::ModularMcpClient;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

pub struct ModularMcpServer {
    client: Arc<tokio::sync::RwLock<ModularMcpClient>>,
    name: String,
    version: String,
}

impl ModularMcpServer {
    pub fn new(
        client: Arc<tokio::sync::RwLock<ModularMcpClient>>,
        name: String,
        version: String,
    ) -> Self {
        Self {
            client,
            name,
            version,
        }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        }
    }

    async fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": self.name,
                    "version": self.version
                }
            })),
            error: None,
        }
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
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
            format!("\n\nUnavailable groups (connection failed):\n{}", failed)
        } else {
            String::new()
        };

        let get_tools_desc = format!(
            "dynamic-mcp manages multiple MCP servers as organized groups, \
            providing only the necessary group's tool descriptions to the LLM \
            on demand instead of overwhelming it with all tool descriptions at once.\n\n\
            Use this tool to retrieve available tools in a specific group, \
            then use call-modular-tool to execute them.\n\n\
            Available groups:\n{}{}",
            groups_desc, failed_desc
        );

        let call_tool_desc = r#"Execute a tool from a specific MCP group. Proxies the call to the appropriate upstream MCP server.

Use get-modular-tools first to discover available tools and their input schemas in the specified group, then use this tool to execute them.

This maintains a clean separation between discovery (context-efficient) and execution phases, enabling effective management of large tool collections across multiple MCP servers.

Example usage:
  call-modular-tool(group="playwright", name="browser_navigate", args={"url": "https://example.com"})
  → Executes the browser_navigate tool from the playwright group with the specified arguments"#;

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "get-modular-tools",
                        "description": get_tools_desc,
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "group": {
                                    "type": "string",
                                    "description": "The name of the MCP group to get tools from",
                                    "enum": group_names
                                }
                            },
                            "required": ["group"]
                        }
                    },
                    {
                        "name": "call-modular-tool",
                        "description": call_tool_desc,
                        "inputSchema": {
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
                        }
                    }
                ]
            })),
            error: None,
        }
    }

    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = request.params.clone().unwrap_or(json!({}));
        let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        match tool_name {
            "get-modular-tools" => {
                let group = arguments.get("group").and_then(|v| v.as_str());

                if group.is_none() {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing required parameter: group".to_string(),
                            data: None,
                        }),
                    };
                }

                let client = self.client.read().await;
                match client.list_tools(group.unwrap()) {
                    Ok(tools) => {
                        let tools_json: Vec<_> = tools
                            .iter()
                            .map(|tool| {
                                let mut schema = tool.input_schema.clone();
                                if let Some(obj) = schema.as_object_mut() {
                                    obj.remove("$schema");
                                }
                                json!({
                                    "name": tool.name,
                                    "description": tool.description,
                                    "inputSchema": schema
                                })
                            })
                            .collect();

                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: Some(json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": serde_json::to_string_pretty(&tools_json).unwrap_or_else(|_| "[]".to_string())
                                    }
                                ]
                            })),
                            error: None,
                        }
                    }
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: format!("Failed to list tools: {}", e),
                            data: None,
                        }),
                    },
                }
            }
            "call-modular-tool" => {
                let group = arguments.get("group").and_then(|v| v.as_str());
                let name = arguments.get("name").and_then(|v| v.as_str());
                let args = arguments.get("args").cloned().unwrap_or(json!({}));

                if group.is_none() || name.is_none() {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing required parameters: group and name".to_string(),
                            data: None,
                        }),
                    };
                }

                let client = self.client.read().await;
                match client.call_tool(group.unwrap(), name.unwrap(), args).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(result),
                        error: None,
                    },
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: format!("Tool execution failed: {}", e),
                            data: None,
                        }),
                    },
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Unknown tool: {}", tool_name),
                    data: None,
                }),
            },
        }
    }

    pub async fn run_stdio(&self) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        tracing::info!("MCP server listening on stdio");

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                Ok(request) => {
                    let is_notification = matches!(request.id, serde_json::Value::Null);

                    if is_notification {
                        tracing::debug!(
                            "Received notification: {} (no response needed)",
                            request.method
                        );
                        continue;
                    }

                    tracing::debug!("Received request: {}", request.method);
                    let response = self.handle_request(request).await;
                    let response_json = serde_json::to_string(&response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
                Err(e) => {
                    tracing::error!("Failed to parse request: {}. Raw input: {}", e, trimmed);
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: serde_json::Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    let response_json = serde_json::to_string(&error_response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
        }

        Ok(())
    }
}
