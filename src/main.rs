mod config;
mod proxy;
mod server;
mod cli;

use clap::Parser;
use tracing_subscriber::EnvFilter;
use anyhow::Result;
use proxy::ModularMcpClient;
use server::ModularMcpServer;

#[derive(Parser)]
#[command(name = "modular-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Modular MCP Proxy Server - Reduce context overhead with on-demand tool loading")]
struct Cli {
    config_path: Option<String>,
}

fn get_config_path(cli_arg: Option<String>) -> Option<(String, &'static str)> {
    if let Some(path) = cli_arg {
        Some((path, "command line argument"))
    } else if let Ok(path) = std::env::var("GATEWAY_MCP_CONFIG") {
        Some((path, "GATEWAY_MCP_CONFIG environment variable"))
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    let cli = Cli::parse();
    
    let (config_path, config_source) = get_config_path(cli.config_path)
        .unwrap_or_else(|| {
            eprintln!("Error: No configuration file specified");
            eprintln!();
            eprintln!("Usage: modular-mcp <config-file>");
            eprintln!("   or: GATEWAY_MCP_CONFIG=<config-file> modular-mcp");
            eprintln!();
            eprintln!("Example: modular-mcp config.example.json");
            eprintln!("     or: GATEWAY_MCP_CONFIG=config.example.json modular-mcp");
            std::process::exit(1);
        });
    
    tracing::info!("Starting modular-mcp server with config: {} (from {})", config_path, config_source);
    
    let config = config::load_config(&config_path).await?;
    
    let mut client = ModularMcpClient::new();
    
    for (group_name, server_config) in config.mcp_servers {
        match client.connect(group_name.clone(), server_config.clone()).await {
            Ok(_) => {
                tracing::info!("✅ Successfully connected to MCP group: {}", group_name);
            }
            Err(e) => {
                tracing::error!("❌ Failed to connect to {}: {}", group_name, e);
                client.record_failed_connection(group_name, server_config, e);
            }
        }
    }
    
    let groups = client.list_groups();
    let failed = client.list_failed_groups();
    
    if failed.is_empty() {
        tracing::info!("Successfully connected {} MCP groups. All groups are valid.", groups.len());
    } else {
        tracing::warn!(
            "Some MCP groups failed to connect. success_groups=[{}], failed_groups=[{}]",
            groups.iter().map(|g| &g.name).cloned().collect::<Vec<_>>().join(", "),
            failed.iter().map(|g| &g.name).cloned().collect::<Vec<_>>().join(", ")
        );
    }
    
    let server = ModularMcpServer::new(
        client,
        env!("CARGO_PKG_NAME").to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    
    tracing::info!("MCP server initialized, starting stdio listener...");
    server.run_stdio().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_cli_arg_takes_precedence() {
        let cli_path = Some("cli-config.json".to_string());
        env::set_var("GATEWAY_MCP_CONFIG", "env-config.json");
        
        let result = get_config_path(cli_path);
        assert!(result.is_some());
        
        let (path, source) = result.unwrap();
        assert_eq!(path, "cli-config.json");
        assert_eq!(source, "command line argument");
        
        env::remove_var("GATEWAY_MCP_CONFIG");
    }

    #[test]
    fn test_env_var_used_when_no_cli() {
        env::set_var("GATEWAY_MCP_CONFIG", "env-config.json");
        
        let result = get_config_path(None);
        assert!(result.is_some());
        
        let (path, source) = result.unwrap();
        assert_eq!(path, "env-config.json");
        assert_eq!(source, "GATEWAY_MCP_CONFIG environment variable");
        
        env::remove_var("GATEWAY_MCP_CONFIG");
    }

    #[test]
    fn test_no_config_returns_none() {
        env::remove_var("GATEWAY_MCP_CONFIG");
        
        let result = get_config_path(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_env_var_is_invalid() {
        env::set_var("GATEWAY_MCP_CONFIG", "");
        
        let result = get_config_path(None);
        assert!(result.is_some());
        
        let (path, _) = result.unwrap();
        assert_eq!(path, "");
        
        env::remove_var("GATEWAY_MCP_CONFIG");
    }
}

