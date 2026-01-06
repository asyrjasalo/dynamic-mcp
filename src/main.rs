mod auth;
mod cli;
mod config;
mod proxy;
mod server;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};
use proxy::ModularMcpClient;
use server::ModularMcpServer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;
use watcher::ConfigWatcher;

#[derive(Parser)]
#[command(name = "dynamic-mcp")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Dynamic MCP Proxy Server - Reduce context overhead with on-demand tool loading")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Configuration file path (when running as server without subcommand)
    config_path: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate standard MCP config to dynamic-mcp format
    Migrate {
        /// Path to standard MCP config file
        mcp_config_path: String,

        /// Output path for dynamic-mcp.json
        #[arg(short, long, default_value = "dynamic-mcp.json")]
        output: String,
    },
}

fn get_config_path(cli_arg: Option<String>) -> Option<(String, &'static str)> {
    if let Some(path) = cli_arg {
        Some((path, "command line argument"))
    } else if let Ok(path) = std::env::var("DYNAMIC_MCP_CONFIG") {
        if path.is_empty() {
            None
        } else {
            Some((path, "DYNAMIC_MCP_CONFIG environment variable"))
        }
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Migrate {
            mcp_config_path,
            output,
        }) => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::new("info"))
                .init();
            cli::migrate::run_migration(&mcp_config_path, &output).await
        }
        None => {
            tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off")),
                )
                .init();

            let (config_path, config_source) =
                get_config_path(cli.config_path).unwrap_or_else(|| {
                    eprintln!("Error: No configuration file specified");
                    eprintln!();
                    eprintln!("Usage: dynamic-mcp <config-file>");
                    eprintln!("   or: DYNAMIC_MCP_CONFIG=<config-file> dynamic-mcp");
                    eprintln!();
                    eprintln!("Example: dynamic-mcp config.example.json");
                    eprintln!("     or: DYNAMIC_MCP_CONFIG=config.example.json dynamic-mcp");
                    std::process::exit(1);
                });

            run_server(config_path, config_source).await
        }
    }
}

async fn run_server(config_path: String, config_source: &str) -> Result<()> {
    tracing::info!(
        "Starting dynamic-mcp server with config: {} (from {})",
        &config_path,
        config_source
    );

    let config_path_buf = std::path::Path::new(&config_path).canonicalize()?;
    let (config_watcher, mut reload_rx) = ConfigWatcher::new(&config_path_buf)?;

    let client = Arc::new(RwLock::new(ModularMcpClient::new()));

    // Initial load
    {
        let config = config::load_config(&config_path).await?;
        let mut client_lock = client.write().await;

        for (group_name, server_config) in config.mcp_servers {
            match client_lock
                .connect(group_name.clone(), server_config.clone())
                .await
            {
                Ok(_) => {
                    tracing::info!("✅ Successfully connected to MCP group: {}", group_name);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to connect to {}: {:#}", group_name, e);
                    client_lock.record_failed_connection(group_name, server_config, e);
                }
            }
        }

        let groups = client_lock.list_groups();
        let failed = client_lock.list_failed_groups();

        if failed.is_empty() {
            tracing::info!(
                "Successfully connected {} MCP groups. All groups are valid.",
                groups.len()
            );
        } else {
            tracing::warn!(
                "Some MCP groups failed to connect. success_groups=[{}], failed_groups=[{}]",
                groups
                    .iter()
                    .map(|g| &g.name)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                failed
                    .iter()
                    .map(|g| &g.name)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            tracing::info!("Attempting to retry failed connections...");
            let retried = client_lock.retry_failed_connections().await;
            if !retried.is_empty() {
                tracing::info!("Successfully reconnected to: {}", retried.join(", "));
            }
        }
    }

    // Spawn periodic retry handler for failed connections
    let client_retry = client.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        interval.tick().await;

        loop {
            interval.tick().await;
            let mut client_lock = client_retry.write().await;
            let failed = client_lock.list_failed_groups();

            if !failed.is_empty() {
                tracing::debug!("Periodic retry check: {} failed groups", failed.len());
                let retried = client_lock.retry_failed_connections().await;
                if !retried.is_empty() {
                    tracing::info!("✅ Periodic retry reconnected: {}", retried.join(", "));
                }
            }
        }
    });

    // Spawn config reload handler
    let client_clone = client.clone();
    let config_path_clone = config_path.clone();
    tokio::spawn(async move {
        while reload_rx.recv().await.is_some() {
            tracing::info!("Config file changed, reloading...");

            match config::load_config(&config_path_clone).await {
                Ok(new_config) => {
                    let mut client_lock = client_clone.write().await;

                    // Disconnect all existing connections
                    if let Err(e) = client_lock.disconnect_all().await {
                        tracing::error!("Failed to disconnect all groups: {}", e);
                    }

                    // Reconnect with new config
                    for (group_name, server_config) in new_config.mcp_servers {
                        match client_lock
                            .connect(group_name.clone(), server_config.clone())
                            .await
                        {
                            Ok(_) => {
                                tracing::info!(
                                    "✅ Successfully reconnected to MCP group: {}",
                                    group_name
                                );
                            }
                            Err(e) => {
                                tracing::error!("❌ Failed to reconnect to {}: {}", group_name, e);
                                client_lock.record_failed_connection(group_name, server_config, e);
                            }
                        }
                    }

                    let groups = client_lock.list_groups();
                    let failed = client_lock.list_failed_groups();

                    if failed.is_empty() {
                        tracing::info!(
                            "✅ Config reload complete: {} groups connected",
                            groups.len()
                        );
                    } else {
                        tracing::warn!(
                            "⚠️ Config reload complete with errors. success_groups=[{}], failed_groups=[{}]",
                            groups.iter().map(|g| &g.name).cloned().collect::<Vec<_>>().join(", "),
                            failed.iter().map(|g| &g.name).cloned().collect::<Vec<_>>().join(", ")
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to reload config: {}", e);
                }
            }
        }
    });

    let server = ModularMcpServer::new(
        client.clone(),
        env!("CARGO_PKG_NAME").to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );

    tracing::info!("MCP server initialized, starting stdio listener...");

    // Keep watcher alive
    std::mem::forget(config_watcher);

    // Set up signal handler for graceful shutdown
    let client_for_shutdown = client.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("Received shutdown signal, disconnecting all servers...");
        let mut client_lock = client_for_shutdown.write().await;
        let _ = client_lock.disconnect_all().await;
        std::process::exit(0);
    });

    let result = server.run_stdio().await;

    // Cleanup on normal exit (stdin closed)
    {
        let mut client_lock = client.write().await;
        let _ = client_lock.disconnect_all().await;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_cli_arg_takes_precedence() {
        let cli_path = Some("cli-config.json".to_string());
        env::set_var("DYNAMIC_MCP_CONFIG", "env-config.json");

        let result = get_config_path(cli_path);
        assert!(result.is_some());

        let (path, source) = result.unwrap();
        assert_eq!(path, "cli-config.json");
        assert_eq!(source, "command line argument");

        env::remove_var("DYNAMIC_MCP_CONFIG");
    }

    #[test]
    #[serial]
    fn test_env_var_used_when_no_cli() {
        env::set_var("DYNAMIC_MCP_CONFIG", "env-config.json");

        let result = get_config_path(None);
        assert!(result.is_some());

        let (path, source) = result.unwrap();
        assert_eq!(path, "env-config.json");
        assert_eq!(source, "DYNAMIC_MCP_CONFIG environment variable");

        env::remove_var("DYNAMIC_MCP_CONFIG");
    }

    #[test]
    #[serial]
    fn test_no_config_returns_none() {
        env::remove_var("DYNAMIC_MCP_CONFIG");

        let result = get_config_path(None);
        assert!(result.is_none());
    }

    #[test]
    #[serial]
    fn test_empty_env_var_is_invalid() {
        env::set_var("DYNAMIC_MCP_CONFIG", "");

        let result = get_config_path(None);
        assert!(result.is_none());

        env::remove_var("DYNAMIC_MCP_CONFIG");
    }
}
