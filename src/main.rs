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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    let cli = Cli::parse();
    
    if let Some(config_path) = cli.config_path {
        tracing::info!("Starting modular-mcp server with config: {}", config_path);
        
        let _config = config::load_config(&config_path).await?;
        
        let client = ModularMcpClient::new();
        
        let server = ModularMcpServer::new(
            client,
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );
        
        tracing::info!("MCP server initialized, starting stdio listener...");
        server.run_stdio().await?;
    } else {
        eprintln!("Usage: modular-mcp <config-file>");
        eprintln!("Example: modular-mcp config.example.json");
        std::process::exit(1);
    }
    
    Ok(())
}

