use crate::cli::import_enhanced;
use crate::cli::tool_detector::Tool;
use anyhow::Result;

pub async fn run_import_from_tool(
    tool_name: &str,
    is_global: bool,
    force: bool,
    output_path: &str,
) -> Result<()> {
    let tool = Tool::from_name(tool_name)?;
    import_enhanced::run_import_from_tool(tool, is_global, force, output_path).await
}

#[allow(dead_code)]
pub async fn run_import_legacy(input_path: &str, output_path: &str) -> Result<()> {
    use crate::config::{McpServerConfig, ServerConfig, StandardServerConfig};
    use anyhow::Context;
    use std::collections::HashMap;
    use tokio::fs;
    println!("🔄 Starting import from standard MCP config to dynamic-mcp format");
    println!("📖 Reading config from: {}", input_path);

    let content = fs::read_to_string(input_path)
        .await
        .with_context(|| format!("Failed to read input file: {}", input_path))?;

    let standard_config: StandardServerConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse standard MCP config: {}", input_path))?;

    println!(
        "\n✅ Found {} MCP server(s) to import\n",
        standard_config.mcp_servers.len()
    );

    let mut imported_servers: HashMap<String, McpServerConfig> = HashMap::new();

    for (name, standard_server) in standard_config.mcp_servers {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Server: {}", name);
        println!("Type: {}", standard_server.r#type);

        let mut config_value = standard_server.config.clone();
        if let Some(obj) = config_value.as_object_mut() {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String(standard_server.r#type.clone()),
            );
        }

        let description = prompt_for_description(&name, &config_value)?;

        if let Some(obj) = config_value.as_object_mut() {
            obj.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }

        let imported_server: McpServerConfig =
            serde_json::from_value(config_value).with_context(|| {
                format!("Failed to convert server '{}' to dynamic-mcp format", name)
            })?;

        imported_servers.insert(name, imported_server);
    }

    let imported_config = ServerConfig {
        mcp_servers: imported_servers,
    };

    let output_json = serde_json::to_string_pretty(&imported_config)
        .context("Failed to serialize imported config")?;

    fs::write(output_path, output_json)
        .await
        .with_context(|| format!("Failed to write output file: {}", output_path))?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Import complete!");
    println!("📝 Output saved to: {}", output_path);
    println!("\nYou can now use this config with:");
    println!("  dynamic-mcp {}", output_path);

    Ok(())
}

fn prompt_for_description(server_name: &str, config: &serde_json::Value) -> Result<String> {
    use anyhow::Context;
    use std::io::{self, Write};
    println!("\nConfig details:");
    if let Some(obj) = config.as_object() {
        for (key, value) in obj {
            if key != "type" {
                println!("  {}: {}", key, value);
            }
        }
    }

    print!(
        "\n💬 Enter description for '{}' (what this server does): ",
        server_name
    );
    io::stdout().flush()?;

    let mut description = String::new();
    io::stdin()
        .read_line(&mut description)
        .context("Failed to read description from stdin")?;

    let description = description.trim().to_string();

    if description.is_empty() {
        anyhow::bail!(
            "Description cannot be empty. Please provide a meaningful description for '{}'",
            server_name
        );
    }

    Ok(description)
}
