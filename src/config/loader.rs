use crate::config::env_sub::substitute_in_config;
use crate::config::schema::ServerConfig;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

pub async fn load_config(path: &str) -> Result<ServerConfig> {
    let absolute_path = Path::new(path)
        .canonicalize()
        .with_context(|| format!("Failed to resolve config path: {}", path))?;

    let content = fs::read_to_string(&absolute_path)
        .await
        .with_context(|| format!("Failed to read config file: {:?}", absolute_path))?;

    let mut config: ServerConfig = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON in config file: {:?}", absolute_path))?;

    config.mcp_servers = config
        .mcp_servers
        .into_iter()
        .map(|(name, server_config)| (name, substitute_in_config(server_config)))
        .collect();

    tracing::info!("✅ MCP server config loaded successfully");

    Ok(config)
}
