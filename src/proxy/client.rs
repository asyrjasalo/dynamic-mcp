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
        config: McpServerConfig,
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

/// Derive capability tags from a group's server config. Used to enrich
/// `GroupInfo` and per-tool `x-capabilities` metadata so hosts can learn a
/// group's transport kind and whether it requires OAuth without probing.
fn derive_capability_tags(config: &McpServerConfig) -> Vec<String> {
    let mut tags = match config {
        McpServerConfig::Stdio { .. } => vec!["stdio".to_string()],
        McpServerConfig::Http { .. } => vec!["http".to_string()],
        McpServerConfig::Sse { .. } => vec!["sse".to_string()],
    };
    if config.needs_oauth() {
        tags.push("oauth".to_string());
    }
    tags
}

impl ModularMcpClient {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub async fn connect(&mut self, group_name: String, config: McpServerConfig) -> Result<()> {
        // 已处于 Connected 状态则跳过（避免并发重复 connect 同 group）。
        // Failed 或缺失：移除旧状态后真正重连（原实现 contains_key 短路会使
        // retry/自愈永远不重连，是 cbm 类后端 flapping 的根因之一）。
        if let Some(GroupState::Connected { .. }) = self.groups.get(&group_name) {
            return Ok(());
        }
        self.groups.remove(&group_name);

        let description = config.description().to_string();

        let config_to_use = config.clone();
        let needs_oauth = config.needs_oauth();
        let transport_timeout = if needs_oauth {
            Duration::from_secs(300) // OAuth requires user interaction (v1.8.2: 120s → 300s)
        } else {
            config.initialize_timeout()
        };
        let transport = tokio::time::timeout(
            transport_timeout,
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
            config.initialize_timeout(),
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
                config.initialize_timeout(),
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

        // If the server didn't assign a session ID in the initialize response,
        // generate a client-side one as fallback
        if !transport.has_session_id() {
            let session_id = uuid::Uuid::new_v4().to_string();
            transport.set_session_id(session_id);
        }

        // Only list tools if tools feature is enabled
        let tools = if config.features().tools {
            let list_tools_request = JsonRpcRequest::new(3, "tools/list");
            let tools_response = tokio::time::timeout(
                config.initialize_timeout(),
                transport.send_request(&list_tools_request),
            )
            .await
            .with_context(|| format!("List tools request timed out for: {}", group_name))?
            .with_context(|| format!("Failed to list tools from: {}", group_name))?;

            if let Some(result) = tools_response.result {
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
                config,
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
                    name,
                    description,
                    tools,
                    config,
                    ..
                } => Some(GroupInfo {
                    name: name.clone(),
                    description: description.clone(),
                    tool_count: tools.len(),
                    capability_tags: derive_capability_tags(config),
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
        // Path A: 放宽周期重连上限（3→10），给慢冷启动后端（如 cbm）足够多的
        // 自愈机会，避免过早放弃进入永久 Failed。
        const MAX_RETRIES: u32 = 10;

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
            // Path A: 有界指数退避，避免 2^n 在 retry_count 较大时溢出/等待过久。
            // 退避 = min(30, 2 + retry_count*5)：第0次2s、第1次7s、第2次12s…封顶30s。
            let backoff_secs = std::cmp::min(30u64, 2 + retry_count as u64 * 5);
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

    pub async fn list_tools(&mut self, group_name: &str) -> Result<Vec<ToolInfo>> {
        // Path A: Failed 状态下自愈重连（取回 config 后交给 connect 真正重连）。
        let failed_cfg: Option<McpServerConfig> = match self.groups.get(group_name) {
            Some(GroupState::Failed { config, .. }) => Some(config.clone()),
            _ => None,
        };
        if let Some(cfg) = failed_cfg {
            if let Err(e) = self.connect(group_name.to_string(), cfg).await {
                return Err(anyhow::anyhow!(
                    "Group '{}' failed to reconnect: {}",
                    group_name,
                    e
                ));
            }
        }

        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected { tools, .. } => Ok(tools.clone()),
            GroupState::Failed {
                error, retry_count, ..
            } => Err(anyhow::anyhow!(
                "Group '{}' failed to connect after {} attempts: {}",
                group_name,
                retry_count + 1,
                error
            )),
        }
    }

    pub async fn call_tool(
        &mut self,
        group_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Path A: Failed 状态下自愈重连，使首个到达的 request 能触发重连而非立即报错。
        let failed_cfg: Option<McpServerConfig> = match self.groups.get(group_name) {
            Some(GroupState::Failed { config, .. }) => Some(config.clone()),
            _ => None,
        };
        if let Some(cfg) = failed_cfg {
            if let Err(e) = self.connect(group_name.to_string(), cfg).await {
                return Err(anyhow::anyhow!(
                    "Group '{}' failed to reconnect: {}",
                    group_name,
                    e
                ));
            }
        }

        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected {
                transport, config, ..
            } => {
                let request = JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "tools/call")
                    .with_params(json!({
                        "name": tool_name,
                        "arguments": arguments
                    }));

                let response =
                    tokio::time::timeout(config.tool_timeout(), transport.send_request(&request))
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
            GroupState::Connected {
                transport, config, ..
            } => {
                // Check if resources feature is enabled
                if !config.features().resources {
                    return Err(anyhow::anyhow!(
                        "Resources feature is disabled for group: {}",
                        group_name
                    ));
                }

                let mut params = json!({});
                if let Some(cursor) = cursor {
                    params["cursor"] = json!(cursor);
                }

                let request =
                    JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "resources/list")
                        .with_params(params);

                let response = tokio::time::timeout(
                    config.resource_timeout(),
                    transport.send_request(&request),
                )
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
            GroupState::Connected {
                transport, config, ..
            } => {
                // Check if resources feature is enabled
                if !config.features().resources {
                    return Err(anyhow::anyhow!(
                        "Resources feature is disabled for group: {}",
                        group_name
                    ));
                }

                let request =
                    JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "resources/read")
                        .with_params(json!({ "uri": uri }));

                let response = tokio::time::timeout(
                    config.resource_timeout(),
                    transport.send_request(&request),
                )
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

    pub async fn proxy_resources_templates_list(
        &self,
        group_name: &str,
    ) -> Result<serde_json::Value> {
        let group = self.groups.get(group_name).context("Group not found")?;

        match group {
            GroupState::Connected {
                transport, config, ..
            } => {
                // Check if resources feature is enabled
                if !config.features().resources {
                    return Err(anyhow::anyhow!(
                        "Resources feature is disabled for group: {}",
                        group_name
                    ));
                }

                let request = JsonRpcRequest::new(
                    uuid::Uuid::new_v4().to_string(),
                    "resources/templates/list",
                );

                let response = tokio::time::timeout(
                    config.resource_timeout(),
                    transport.send_request(&request),
                )
                .await
                .with_context(|| "resources/templates/list request timed out")?
                .with_context(|| "Failed to list resource templates from upstream server")?;

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
            GroupState::Connected {
                transport, config, ..
            } => {
                // Check if prompts feature is enabled
                if !config.features().prompts {
                    return Err(anyhow::anyhow!(
                        "Prompts feature is disabled for group: {}",
                        group_name
                    ));
                }

                let mut params = json!({});
                if let Some(cursor) = cursor {
                    params["cursor"] = json!(cursor);
                }

                let request = JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "prompts/list")
                    .with_params(params);

                let response =
                    tokio::time::timeout(config.prompt_timeout(), transport.send_request(&request))
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
            GroupState::Connected {
                transport, config, ..
            } => {
                // Check if prompts feature is enabled
                if !config.features().prompts {
                    return Err(anyhow::anyhow!(
                        "Prompts feature is disabled for group: {}",
                        group_name
                    ));
                }

                let mut params = json!({ "name": prompt_name });
                if let Some(args) = arguments {
                    params["arguments"] = args;
                }

                let request = JsonRpcRequest::new(uuid::Uuid::new_v4().to_string(), "prompts/get")
                    .with_params(params);

                let response =
                    tokio::time::timeout(config.prompt_timeout(), transport.send_request(&request))
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
