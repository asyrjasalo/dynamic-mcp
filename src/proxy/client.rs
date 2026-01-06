use crate::config::McpServerConfig;
use crate::proxy::transport::Transport;
use crate::proxy::types::{FailedGroupInfo, GroupInfo, JsonRpcRequest, ToolInfo};
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;

pub enum GroupState {
    Connected {
        name: String,
        description: String,
        tools: Vec<ToolInfo>,
        transport: Transport,
    },
    Failed {
        name: String,
        description: String,
        error: String,
    },
}

pub struct ModularMcpClient {
    groups: HashMap<String, GroupState>,
}

impl ModularMcpClient {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub async fn connect(&mut self, group_name: String, config: McpServerConfig) -> Result<()> {
        if self.groups.contains_key(&group_name) {
            return Ok(());
        }

        let description = config.description().to_string();

        // Try to create transport
        let mut config_to_use = config.clone();
        let mut transport = Transport::new(&config_to_use, &group_name)
            .await
            .with_context(|| format!("Failed to create transport for group: {}", group_name))?;

        let init_request = JsonRpcRequest::new(1, "initialize").with_params(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "dynamic-mcp-client",
                "version": env!("CARGO_PKG_VERSION")
            }
        }));

        // Streamable HTTP transport handles both JSON and SSE responses automatically
        transport
            .send_request(&init_request)
            .await
            .with_context(|| format!("Failed to initialize connection to: {}", group_name))?;

        let list_tools_request = JsonRpcRequest::new(2, "tools/list");
        let tools_response = transport
            .send_request(&list_tools_request)
            .await
            .with_context(|| format!("Failed to list tools from: {}", group_name))?;

        let tools = if let Some(result) = tools_response.result {
            if let Some(tools_array) = result.get("tools").and_then(|v| v.as_array()) {
                tools_array
                    .iter()
                    .filter_map(|tool| {
                        Some(ToolInfo {
                            name: tool.get("name")?.as_str()?.to_string(),
                            description: tool
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            input_schema: tool.get("inputSchema").cloned().unwrap_or(json!({})),
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        self.groups.insert(
            group_name.clone(),
            GroupState::Connected {
                name: group_name,
                description,
                tools,
                transport,
            },
        );

        Ok(())
    }

    pub fn record_failed_connection(
        &mut self,
        group_name: String,
        config: McpServerConfig,
        error: anyhow::Error,
    ) {
        self.groups.insert(
            group_name.clone(),
            GroupState::Failed {
                name: group_name,
                description: config.description().to_string(),
                error: error.to_string(),
            },
        );
    }

    pub fn list_groups(&self) -> Vec<GroupInfo> {
        self.groups
            .values()
            .filter_map(|state| match state {
                GroupState::Connected {
                    name, description, ..
                } => Some(GroupInfo {
                    name: name.clone(),
                    description: description.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    pub fn list_failed_groups(&self) -> Vec<FailedGroupInfo> {
        self.groups
            .values()
            .filter_map(|state| match state {
                GroupState::Failed {
                    name,
                    description,
                    error,
                } => Some(FailedGroupInfo {
                    name: name.clone(),
                    description: description.clone(),
                    error: error.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    pub fn list_tools(&self, group_name: &str) -> Result<Vec<ToolInfo>> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { tools, .. } => Ok(tools.clone()),
            GroupState::Failed { error, .. } => {
                Err(anyhow::anyhow!("Group failed to connect: {}", error))
            }
        }
    }

    pub async fn call_tool(
        &self,
        group_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { transport, .. } => {
                let request = JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "tools/call")
                    .with_params(json!({
                        "name": tool_name,
                        "arguments": arguments
                    }));

                let response = transport.send_request(&request).await?;

                if let Some(error) = response.error {
                    return Err(anyhow::anyhow!("Tool call failed: {}", error.message));
                }

                Ok(response.result.unwrap_or(json!({})))
            }
            GroupState::Failed { error, .. } => {
                Err(anyhow::anyhow!("Group failed to connect: {}", error))
            }
        }
    }

    pub async fn disconnect_all(&mut self) -> Result<()> {
        tracing::info!("Disconnecting {} groups", self.groups.len());
        for (name, state) in self.groups.drain() {
            if let GroupState::Connected { mut transport, .. } = state {
                tracing::info!("Closing transport for group: {}", name);
                let _ = transport.close().await;
            }
        }
        Ok(())
    }
}
