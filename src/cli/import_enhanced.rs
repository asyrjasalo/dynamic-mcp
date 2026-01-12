use crate::cli::config_parser::ConfigParser;
use crate::cli::tool_detector::Tool;
use crate::config::schema::Features;
use crate::config::{IntermediateServerConfig, McpServerConfig, ServerConfig};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use tokio::fs;

pub async fn run_import_from_tool(
    tool: Tool,
    is_global: bool,
    force: bool,
    output_path: &str,
) -> Result<()> {
    println!(
        "🔄 Starting import from {} to dynamic-mcp format",
        tool.name()
    );

    let input_path = determine_input_path(tool, is_global)?;

    println!("📖 Reading config from: {}", input_path.display());

    if !input_path.exists() {
        return Err(anyhow!(
            "Config file not found: {}\n\n\
            Expected location: {}\n\n\
            Suggestions:\n\
              - Verify {} is installed and configured\n\
              - {}",
            input_path.display(),
            input_path.display(),
            tool.name(),
            if is_global {
                "Or try project-level config by omitting --global flag"
            } else {
                "Or try global config with --global flag"
            }
        ));
    }

    check_output_file_exists(output_path, force).await?;

    let content = fs::read_to_string(&input_path)
        .await
        .with_context(|| format!("Failed to read input file: {}", input_path.display()))?;

    let parser = ConfigParser::new(tool);
    let intermediate_servers = parser
        .parse(&content)
        .with_context(|| format!("Failed to parse {} config", tool.name()))?;

    println!(
        "\n✅ Found {} MCP server(s) to import\n",
        intermediate_servers.len()
    );

    let mut imported_servers: HashMap<String, McpServerConfig> = HashMap::new();

    let mut server_entries: Vec<_> = intermediate_servers.into_iter().collect();
    server_entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, intermediate) in server_entries {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Server: {}", name);

        print_config_details(&intermediate);

        let description = prompt_for_description(&name)?;

        let features = prompt_for_features(&name)?;

        let mut imported = intermediate
            .to_mcp_config(description)
            .map_err(|e| anyhow!("Failed to convert server '{}': {}", name, e))?;

        // Update the features based on user selection
        imported = apply_features_to_config(imported, features);

        imported_servers.insert(name, imported);
    }

    let imported_config = ServerConfig {
        mcp_servers: imported_servers,
        schema: None,
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
    println!("  dmcp {}", output_path);

    Ok(())
}

fn determine_input_path(tool: Tool, is_global: bool) -> Result<PathBuf> {
    if tool == Tool::OpenCode {
        return determine_opencode_path(is_global);
    }

    if is_global {
        tool.global_config_path()
    } else {
        tool.project_config_path().ok_or_else(|| {
            anyhow!(
                "{} does not support project-level config.\n\n\
                Use --global flag to import from global config:\n\
                  dmcp import --global {}",
                tool.name(),
                tool.name()
            )
        })
    }
}

#[allow(clippy::if_same_then_else)]
fn determine_opencode_path(is_global: bool) -> Result<PathBuf> {
    if is_global {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory")?;

        let jsonc_path = PathBuf::from(&home).join(".config/opencode/opencode.jsonc");
        let json_path = PathBuf::from(&home).join(".config/opencode/opencode.json");

        if jsonc_path.exists() {
            Ok(jsonc_path)
        } else if json_path.exists() {
            Ok(json_path)
        } else {
            Ok(jsonc_path)
        }
    } else {
        let jsonc_path = PathBuf::from(".opencode/opencode.jsonc");
        let json_path = PathBuf::from(".opencode/opencode.json");

        if jsonc_path.exists() {
            Ok(jsonc_path)
        } else if json_path.exists() {
            Ok(json_path)
        } else {
            Ok(jsonc_path)
        }
    }
}

async fn check_output_file_exists(output_path: &str, force: bool) -> Result<()> {
    if PathBuf::from(output_path).exists() && !force {
        print!(
            "\n⚠️  Warning: Output file already exists: {}\n\
            \nOverwrite existing file? [y/N]: ",
            output_path
        );
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin()
            .read_line(&mut response)
            .context("Failed to read user response")?;

        let response = response.trim().to_lowercase();
        if response != "y" && response != "yes" {
            return Err(anyhow!(
                "Import cancelled.\n\n\
                Use --force flag to skip this prompt:\n\
                  dmcp import <tool-name> --force"
            ));
        }
    }
    Ok(())
}

fn print_config_details(config: &IntermediateServerConfig) {
    if let Some(url) = &config.url {
        println!("Type: {}", config.server_type.as_deref().unwrap_or("http"));
        println!("\nConfig details:");
        println!("  url: \"{}\"", url);
        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                println!("  headers.{}: \"{}\"", key, value);
            }
        }
    } else if let Some(command) = &config.command {
        println!("Type: stdio");
        println!("\nConfig details:");
        println!("  command: \"{}\"", command);
        if let Some(args) = &config.args {
            println!("  args: {:?}", args);
        }
        if let Some(env) = &config.env {
            for (key, value) in env {
                println!("  env.{}: \"{}\"", key, value);
            }
        }
    }
}

fn prompt_for_description(server_name: &str) -> Result<String> {
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
        return Err(anyhow!(
            "Description cannot be empty. Please provide a meaningful description for '{}'",
            server_name
        ));
    }

    Ok(description)
}

fn prompt_for_features(server_name: &str) -> Result<Features> {
    print!(
        "\n🔧 Keep all features (tools, resources, prompts) for '{}'? [Y/n]: ",
        server_name
    );
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin()
        .read_line(&mut response)
        .context("Failed to read feature selection from stdin")?;

    let response = response.trim().to_lowercase();

    // Default to yes if empty or y/yes
    if response.is_empty() || response == "y" || response == "yes" {
        return Ok(Features::default()); // All features enabled
    }

    // User wants to customize features
    println!("\n  Select features to enable (press Enter to accept default):");

    let tools = prompt_yes_no("  Enable tools?", true)?;
    let resources = prompt_yes_no("  Enable resources?", true)?;
    let prompts = prompt_yes_no("  Enable prompts?", true)?;

    Ok(Features {
        tools,
        resources,
        prompts,
    })
}

fn prompt_yes_no(prompt: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} [{}]: ", prompt, default_str);
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin()
        .read_line(&mut response)
        .context("Failed to read yes/no response from stdin")?;

    let response = response.trim().to_lowercase();

    if response.is_empty() {
        return Ok(default);
    }

    match response.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Ok(default), // Invalid input, use default
    }
}

fn apply_features_to_config(config: McpServerConfig, features: Features) -> McpServerConfig {
    match config {
        McpServerConfig::Stdio {
            description,
            command,
            args,
            env,
            enabled,
            ..
        } => McpServerConfig::Stdio {
            description,
            command,
            args,
            env,
            features,
            enabled,
        },
        McpServerConfig::Http {
            description,
            url,
            headers,
            oauth_client_id,
            oauth_scopes,
            enabled,
            ..
        } => McpServerConfig::Http {
            description,
            url,
            headers,
            oauth_client_id,
            oauth_scopes,
            features,
            enabled,
        },
        McpServerConfig::Sse {
            description,
            url,
            headers,
            oauth_client_id,
            oauth_scopes,
            enabled,
            ..
        } => McpServerConfig::Sse {
            description,
            url,
            headers,
            oauth_client_id,
            oauth_scopes,
            features,
            enabled,
        },
    }
}
