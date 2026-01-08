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
            "resources/list" => self.handle_resources_list(request).await,
            "resources/read" => self.handle_resources_read(request).await,
            "resources/templates/list" => self.handle_resources_templates_list(request).await,
            "prompts/list" => self.handle_prompts_list(request).await,
            "prompts/get" => self.handle_prompts_get(request).await,
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
                    "tools": {},
                    "resources": {
                        "subscribe": false,
                        "listChanged": false
                    },
                    "prompts": {
                        "listChanged": false
                    }
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
            then use call_dynamic_tool to execute them.\n\n\
            Available groups:\n{}{}",
            groups_desc, failed_desc
        );

        let call_tool_desc = r#"Execute a tool from a specific MCP group. Proxies the call to the appropriate upstream MCP server.

Use get_dynamic_tools first to discover available tools and their input schemas in the specified group, then use this tool to execute them.

This maintains a clean separation between discovery (context-efficient) and execution phases, enabling effective management of large tool collections across multiple MCP servers.

Example usage:
  call_dynamic_tool(group="playwright", name="browser_navigate", args={"url": "https://example.com"})
  → Executes the browser_navigate tool from the playwright group with the specified arguments"#;

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "get_dynamic_tools",
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
                        "name": "call_dynamic_tool",
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
            "get_dynamic_tools" => {
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
            "call_dynamic_tool" => {
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
                        result: Some(json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Tool execution failed: {}", e),
                                "isError": true
                            }]
                        })),
                        error: None,
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

    async fn handle_resources_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let client = self.client.read().await;

        let group_name = match request
            .params
            .as_ref()
            .and_then(|p| p.get("group"))
            .and_then(|g| g.as_str())
        {
            Some(name) => name.to_string(),
            None => {
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
        };

        let cursor = request
            .params
            .as_ref()
            .and_then(|p| p.get("cursor"))
            .and_then(|c| c.as_str())
            .map(String::from);

        match client.proxy_resources_list(&group_name, cursor).await {
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
                    message: format!("Failed to list resources: {}", e),
                    data: None,
                }),
            },
        }
    }

    async fn handle_resources_read(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let client = self.client.read().await;

        let (group_name, uri) = match request.params.as_ref() {
            Some(params) => {
                let group = params
                    .get("group")
                    .and_then(|g| g.as_str())
                    .map(String::from);
                let uri = params.get("uri").and_then(|u| u.as_str()).map(String::from);

                match (group, uri) {
                    (Some(g), Some(u)) => (g, u),
                    _ => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Missing required parameters: group, uri".to_string(),
                                data: None,
                            }),
                        };
                    }
                }
            }
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params object".to_string(),
                        data: None,
                    }),
                };
            }
        };

        match client.proxy_resources_read(&group_name, uri).await {
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
                    message: format!("Failed to read resource: {}", e),
                    data: None,
                }),
            },
        }
    }

    async fn handle_resources_templates_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let client = self.client.read().await;

        let group_name = match request
            .params
            .as_ref()
            .and_then(|p| p.get("group"))
            .and_then(|g| g.as_str())
        {
            Some(name) => name.to_string(),
            None => {
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
        };

        match client.proxy_resources_templates_list(&group_name).await {
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
                    message: format!("Failed to list resource templates: {}", e),
                    data: None,
                }),
            },
        }
    }

    async fn handle_prompts_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let client = self.client.read().await;

        let group_name = match request
            .params
            .as_ref()
            .and_then(|p| p.get("group"))
            .and_then(|g| g.as_str())
        {
            Some(name) => name.to_string(),
            None => {
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
        };

        let cursor = request
            .params
            .as_ref()
            .and_then(|p| p.get("cursor"))
            .and_then(|c| c.as_str())
            .map(String::from);

        match client.proxy_prompts_list(&group_name, cursor).await {
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
                    message: format!("Failed to list prompts: {}", e),
                    data: None,
                }),
            },
        }
    }

    async fn handle_prompts_get(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let client = self.client.read().await;

        let (group_name, prompt_name, arguments) = match request.params.as_ref() {
            Some(params) => {
                let group = params
                    .get("group")
                    .and_then(|g| g.as_str())
                    .map(String::from);
                let name = params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(String::from);
                let args = params.get("arguments").cloned();

                match (group, name) {
                    (Some(g), Some(n)) => (g, n, args),
                    _ => {
                        return JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Missing required parameters: group, name".to_string(),
                                data: None,
                            }),
                        };
                    }
                }
            }
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params object".to_string(),
                        data: None,
                    }),
                };
            }
        };

        match client
            .proxy_prompts_get(&group_name, prompt_name, arguments)
            .await
        {
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
                    message: format!("Failed to get prompt: {}", e),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::ModularMcpClient;

    fn create_test_server() -> ModularMcpServer {
        let client = ModularMcpClient::new();
        ModularMcpServer::new(
            Arc::new(tokio::sync::RwLock::new(client)),
            "test-server".to_string(),
            "1.0.0".to_string(),
        )
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "initialize");
        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), "2024-11-05");
        assert_eq!(
            result
                .get("serverInfo")
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap(),
            "test-server"
        );
    }

    #[tokio::test]
    async fn test_handle_list_tools_empty() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "tools/list");
        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(
            tools[0].get("name").unwrap().as_str().unwrap(),
            "get_dynamic_tools"
        );
        assert_eq!(
            tools[1].get("name").unwrap().as_str().unwrap(),
            "call_dynamic_tool"
        );
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "unknown/method");
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_call_tool_missing_params() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "tools/call")
            .with_params(json!({"name": "get_dynamic_tools", "arguments": {}}));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing required parameter"));
    }

    #[tokio::test]
    async fn test_handle_call_tool_unknown_tool() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "tools/call").with_params(json!({
            "name": "unknown-tool",
            "arguments": {}
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32601);
        assert!(error.message.contains("Unknown tool"));
    }

    #[tokio::test]
    async fn test_handle_get_dynamic_tools_nonexistent_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "tools/call").with_params(json!({
            "name": "get_dynamic_tools",
            "arguments": {
                "group": "nonexistent"
            }
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32603);
        assert!(error.message.contains("Failed to list tools"));
    }

    #[tokio::test]
    async fn test_handle_call_dynamic_tool_missing_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "tools/call").with_params(json!({
            "name": "call_dynamic_tool",
            "arguments": {
                "name": "some-tool"
            }
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
    }

    #[tokio::test]
    async fn test_handle_call_dynamic_tool_missing_name() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "tools/call").with_params(json!({
            "name": "call_dynamic_tool",
            "arguments": {
                "group": "some-group"
            }
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
    }

    #[tokio::test]
    async fn test_initialize_includes_resources_capability() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "initialize");
        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let capabilities = result.get("capabilities").unwrap();

        assert!(capabilities.get("resources").is_some());
        let resources = capabilities.get("resources").unwrap();
        assert_eq!(resources.get("subscribe").unwrap(), false);
        assert_eq!(resources.get("listChanged").unwrap(), false);
    }

    #[tokio::test]
    async fn test_handle_resources_list_missing_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "resources/list").with_params(json!({
            "cursor": None::<String>
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing required parameter: group"));
    }

    #[tokio::test]
    async fn test_handle_resources_list_nonexistent_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "resources/list").with_params(json!({
            "group": "nonexistent"
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32603);
        assert!(error.message.contains("Failed to list resources"));
    }

    #[tokio::test]
    async fn test_handle_resources_read_missing_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "resources/read").with_params(json!({
            "uri": "file:///test.txt"
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing required parameters"));
    }

    #[tokio::test]
    async fn test_handle_resources_read_missing_uri() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "resources/read").with_params(json!({
            "group": "some-group"
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing required parameters"));
    }

    #[tokio::test]
    async fn test_handle_resources_read_no_params() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "resources/read");
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("Missing params object"));
    }

    #[tokio::test]
    async fn test_handle_resources_read_nonexistent_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "resources/read").with_params(json!({
            "group": "nonexistent",
            "uri": "file:///test.txt"
        }));
        let response = server.handle_request(request).await;

        assert!(response.result.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, -32603);
        assert!(error.message.contains("Failed to read resource"));
    }

    #[tokio::test]
    async fn test_server_everything_configuration() {
        let config_json = r#"{
            "mcpServers": {
                "everything": {
                    "description": "Comprehensive MCP server with tools and resources",
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-everything"]
                }
            }
        }"#;

        let parsed: serde_json::Value =
            serde_json::from_str(config_json).expect("Config should parse");

        assert!(parsed.get("mcpServers").is_some());
        let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
        assert!(servers.contains_key("everything"));

        let everything = &servers["everything"];
        assert_eq!(
            everything.get("description").unwrap().as_str().unwrap(),
            "Comprehensive MCP server with tools and resources"
        );
    }

    #[tokio::test]
    async fn test_tools_list_structure_for_comprehensive_servers() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "tools/list");
        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert!(result.is_object());

        let has_tools = result.get("tools").is_some() || result.get("_meta").is_some();
        assert!(
            has_tools,
            "Response should have tools info or metadata for empty client"
        );
    }

    #[tokio::test]
    async fn test_resources_list_protocol_compliance() {
        let server = create_test_server();
        let request =
            JsonRpcRequest::new(1, "resources/list").with_params(json!({ "group": "test" }));
        let response = server.handle_request(request).await;

        assert!(response.jsonrpc == "2.0");
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert!(error.code <= -32600);
    }

    #[tokio::test]
    async fn test_resources_read_protocol_compliance() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "resources/read")
            .with_params(json!({ "group": "test", "uri": "file:///test.txt" }));
        let response = server.handle_request(request).await;

        assert!(response.jsonrpc == "2.0");
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert!(error.code <= -32600);
    }

    #[tokio::test]
    async fn test_initialize_includes_tools_and_resources_capabilities() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "initialize");
        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let capabilities = result.get("capabilities").unwrap();

        assert!(
            capabilities.get("tools").is_some(),
            "Should have tools capability"
        );
        assert!(
            capabilities.get("resources").is_some(),
            "Should have resources capability"
        );

        let resources_cap = capabilities.get("resources").unwrap();
        assert!(
            resources_cap.get("subscribe").is_some(),
            "Resources should declare subscribe capability"
        );
    }

    #[tokio::test]
    async fn test_handle_prompts_list_missing_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "prompts/list");
        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("group"));
    }

    #[tokio::test]
    async fn test_handle_prompts_list_nonexistent_group() {
        let server = create_test_server();
        let request =
            JsonRpcRequest::new(1, "prompts/list").with_params(json!({ "group": "nonexistent" }));
        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32603);
    }

    #[tokio::test]
    async fn test_handle_prompts_get_missing_group() {
        let server = create_test_server();
        let request =
            JsonRpcRequest::new(1, "prompts/get").with_params(json!({ "name": "test_prompt" }));
        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("group"));
    }

    #[tokio::test]
    async fn test_handle_prompts_get_missing_name() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "prompts/get").with_params(json!({ "group": "test" }));
        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32602);
        assert!(error.message.contains("name"));
    }

    #[tokio::test]
    async fn test_handle_prompts_get_nonexistent_group() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "prompts/get")
            .with_params(json!({ "group": "nonexistent", "name": "test_prompt" }));
        let response = server.handle_request(request).await;

        assert!(response.error.is_some());
        let error = response.error.unwrap();
        assert_eq!(error.code, -32603);
    }

    #[tokio::test]
    async fn test_prompts_list_protocol_compliance() {
        let server = create_test_server();
        let request =
            JsonRpcRequest::new(1, "prompts/list").with_params(json!({ "group": "test" }));
        let response = server.handle_request(request).await;

        assert!(response.jsonrpc == "2.0");
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert!(error.code <= -32600);
    }

    #[tokio::test]
    async fn test_prompts_get_protocol_compliance() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "prompts/get")
            .with_params(json!({ "group": "test", "name": "test_prompt" }));
        let response = server.handle_request(request).await;

        assert!(response.jsonrpc == "2.0");
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert!(error.code <= -32600);
    }

    #[tokio::test]
    async fn test_initialize_includes_prompts_capability() {
        let server = create_test_server();
        let request = JsonRpcRequest::new(1, "initialize");
        let response = server.handle_request(request).await;

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let capabilities = result.get("capabilities").unwrap();

        assert!(
            capabilities.get("prompts").is_some(),
            "Should have prompts capability"
        );

        let prompts_cap = capabilities.get("prompts").unwrap();
        assert!(
            prompts_cap.get("listChanged").is_some(),
            "Prompts should declare listChanged capability"
        );
    }
}
