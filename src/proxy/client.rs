use crate::config::McpServerConfig;
use crate::proxy::transport::Transport;
use crate::proxy::types::{FailedGroupInfo, GroupInfo, JsonRpcRequest, ToolInfo};
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

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
        retry_count: u32,
        config: McpServerConfig,
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

        let config_to_use = config.clone();
        let transport = tokio::time::timeout(
            Duration::from_secs(5),
            Transport::new(&config_to_use, &group_name),
        )
        .await
        .with_context(|| format!("Transport creation timed out for group: {}", group_name))?
        .with_context(|| format!("Failed to create transport for group: {}", group_name))?;

        let init_request = JsonRpcRequest::new(1, "initialize").with_params(json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "dynamic-mcp-client",
                "version": env!("CARGO_PKG_VERSION")
            }
        }));

        let response = tokio::time::timeout(
            Duration::from_secs(5),
            transport.send_request(&init_request),
        )
        .await
        .with_context(|| format!("Initialize request timed out for: {}", group_name))?
        .with_context(|| format!("Failed to initialize connection to: {}", group_name))?;

        if let Some(error) = &response.error {
            anyhow::bail!(
                "Server {} rejected initialization: {}",
                group_name,
                error.message
            );
        }

        let server_version = response
            .result
            .as_ref()
            .and_then(|r| r.get("protocolVersion"))
            .and_then(|v| v.as_str())
            .unwrap_or("2025-06-18");

        if server_version != "2025-06-18" {
            let retry_request = JsonRpcRequest::new(2, "initialize").with_params(json!({
                "protocolVersion": server_version,
                "capabilities": {},
                "clientInfo": {
                    "name": "dynamic-mcp-client",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }));

            let retry_response = tokio::time::timeout(
                Duration::from_secs(5),
                transport.send_request(&retry_request),
            )
            .await
            .with_context(|| format!("Initialize retry timed out for: {}", group_name))?
            .with_context(|| {
                format!(
                    "Failed to initialize with server version: {}",
                    server_version
                )
            })?;

            if let Some(error) = &retry_response.error {
                anyhow::bail!(
                    "Server {} rejected version {}: {}",
                    group_name,
                    server_version,
                    error.message
                );
            }
        }

        transport.set_protocol_version(server_version.to_string());

        let session_id = uuid::Uuid::new_v4().to_string();
        transport.set_session_id(session_id);

        let list_tools_request = JsonRpcRequest::new(3, "tools/list");
        let tools_response = tokio::time::timeout(
            Duration::from_secs(5),
            transport.send_request(&list_tools_request),
        )
        .await
        .with_context(|| format!("List tools request timed out for: {}", group_name))?
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
        let retry_count =
            if let Some(GroupState::Failed { retry_count, .. }) = self.groups.get(&group_name) {
                retry_count + 1
            } else {
                0
            };

        self.groups.insert(
            group_name.clone(),
            GroupState::Failed {
                name: group_name,
                description: config.description().to_string(),
                error: error.to_string(),
                retry_count,
                config,
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
                    ..
                } => Some(FailedGroupInfo {
                    name: name.clone(),
                    description: description.clone(),
                    error: error.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    pub async fn retry_failed_connections(&mut self) -> Vec<String> {
        const MAX_RETRIES: u32 = 3;

        let failed_groups: Vec<_> = self
            .groups
            .iter()
            .filter_map(|(name, state)| {
                if let GroupState::Failed {
                    retry_count,
                    config,
                    ..
                } = state
                {
                    if *retry_count < MAX_RETRIES {
                        Some((name.clone(), config.clone(), *retry_count))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if failed_groups.is_empty() {
            return Vec::new();
        }

        let mut retry_handles = Vec::new();

        for (group_name, config, retry_count) in failed_groups {
            let backoff_secs = 2u64.pow(retry_count);
            tracing::info!(
                "Retrying connection to {} (attempt {}/{}), waiting {}s...",
                group_name,
                retry_count + 1,
                MAX_RETRIES,
                backoff_secs
            );

            let handle = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                (group_name, config, retry_count)
            });

            retry_handles.push(handle);
        }

        let mut successfully_retried = Vec::new();
        let mut failed_to_retry = Vec::new();

        for handle in retry_handles {
            if let Ok((group_name, config, _retry_count)) = handle.await {
                match self.connect(group_name.clone(), config.clone()).await {
                    Ok(_) => {
                        tracing::info!("✅ Successfully reconnected to MCP group: {}", group_name);
                        successfully_retried.push(group_name);
                    }
                    Err(e) => {
                        tracing::warn!("❌ Retry failed for {}: {:#}", group_name, e);
                        failed_to_retry.push((group_name, config, e));
                    }
                }
            }
        }

        for (group_name, config, error) in failed_to_retry {
            self.record_failed_connection(group_name, config, error);
        }

        successfully_retried
    }

    pub fn list_tools(&self, group_name: &str) -> Result<Vec<ToolInfo>> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { tools, .. } => Ok(tools.clone()),
            GroupState::Failed {
                error, retry_count, ..
            } => Err(anyhow::anyhow!(
                "Group failed to connect after {} attempts: {}",
                retry_count + 1,
                error
            )),
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

                let response =
                    tokio::time::timeout(Duration::from_secs(30), transport.send_request(&request))
                        .await
                        .with_context(|| format!("Tool call timed out: {}", tool_name))?
                        .with_context(|| format!("Tool call failed: {}", tool_name))?;

                if let Some(error) = response.error {
                    return Err(anyhow::anyhow!("Tool call failed: {}", error.message));
                }

                Ok(response.result.unwrap_or(json!({})))
            }
            GroupState::Failed {
                error, retry_count, ..
            } => Err(anyhow::anyhow!(
                "Group failed to connect after {} attempts: {}",
                retry_count + 1,
                error
            )),
        }
    }

    pub async fn proxy_resources_list(
        &self,
        group_name: &str,
        cursor: Option<String>,
    ) -> Result<serde_json::Value> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { transport, .. } => {
                let mut params = json!({});
                if let Some(cursor) = cursor {
                    params["cursor"] = json!(cursor);
                }

                let request =
                    JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "resources/list")
                        .with_params(params);

                let response =
                    tokio::time::timeout(Duration::from_secs(10), transport.send_request(&request))
                        .await
                        .with_context(|| "resources/list request timed out")?
                        .with_context(|| "Failed to list resources from upstream server")?;

                if let Some(error) = response.error {
                    return Err(anyhow::anyhow!("Upstream error: {}", error.message));
                }

                Ok(response.result.unwrap_or(json!({})))
            }
            GroupState::Failed {
                error, retry_count, ..
            } => Err(anyhow::anyhow!(
                "Group failed to connect after {} attempts: {}",
                retry_count + 1,
                error
            )),
        }
    }

    pub async fn proxy_resources_read(
        &self,
        group_name: &str,
        uri: String,
    ) -> Result<serde_json::Value> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { transport, .. } => {
                let request =
                    JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "resources/read")
                        .with_params(json!({ "uri": uri }));

                let response =
                    tokio::time::timeout(Duration::from_secs(10), transport.send_request(&request))
                        .await
                        .with_context(|| "resources/read request timed out")?
                        .with_context(|| "Failed to read resource from upstream server")?;

                if let Some(error) = response.error {
                    return Err(anyhow::anyhow!("Upstream error: {}", error.message));
                }

                Ok(response.result.unwrap_or(json!({})))
            }
            GroupState::Failed {
                error, retry_count, ..
            } => Err(anyhow::anyhow!(
                "Group failed to connect after {} attempts: {}",
                retry_count + 1,
                error
            )),
        }
    }

    pub async fn proxy_prompts_list(
        &self,
        group_name: &str,
        cursor: Option<String>,
    ) -> Result<serde_json::Value> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { transport, .. } => {
                let mut params = json!({});
                if let Some(cursor) = cursor {
                    params["cursor"] = json!(cursor);
                }

                let request = JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "prompts/list")
                    .with_params(params);

                let response =
                    tokio::time::timeout(Duration::from_secs(10), transport.send_request(&request))
                        .await
                        .with_context(|| "prompts/list request timed out")?
                        .with_context(|| "Failed to list prompts from upstream server")?;

                if let Some(error) = response.error {
                    return Err(anyhow::anyhow!("Upstream error: {}", error.message));
                }

                Ok(response.result.unwrap_or(json!({})))
            }
            GroupState::Failed {
                error, retry_count, ..
            } => Err(anyhow::anyhow!(
                "Group failed to connect after {} attempts: {}",
                retry_count + 1,
                error
            )),
        }
    }

    pub async fn proxy_prompts_get(
        &self,
        group_name: &str,
        prompt_name: String,
        arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { transport, .. } => {
                let mut params = json!({ "name": prompt_name });
                if let Some(args) = arguments {
                    params["arguments"] = args;
                }

                let request = JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "prompts/get")
                    .with_params(params);

                let response =
                    tokio::time::timeout(Duration::from_secs(10), transport.send_request(&request))
                        .await
                        .with_context(|| "prompts/get request timed out")?
                        .with_context(|| "Failed to get prompt from upstream server")?;

                if let Some(error) = response.error {
                    return Err(anyhow::anyhow!("Upstream error: {}", error.message));
                }

                Ok(response.result.unwrap_or(json!({})))
            }
            GroupState::Failed {
                error, retry_count, ..
            } => Err(anyhow::anyhow!(
                "Group failed to connect after {} attempts: {}",
                retry_count + 1,
                error
            )),
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
