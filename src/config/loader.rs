use crate::config::env_sub::substitute_in_config;
use crate::config::schema::{McpServerConfig, ServerConfig};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

pub async fn load_config(path: &str) -> Result<ServerConfig> {
    let absolute_path = Path::new(path)
        .canonicalize()
        .with_context(|| format!("Failed to resolve config path: {}", path))?;

    let content = fs::read_to_string(&absolute_path)
        .await
        .with_context(|| format!("Failed to read config file: {:?}", absolute_path))?;

    let mut config: ServerConfig = serde_json::from_str(&content).map_err(|e| {
        let error_msg = e.to_string();
        if error_msg.contains("missing field") && error_msg.contains("description") {
            anyhow::anyhow!(
                "❌ Configuration Error: Server missing 'description' field\n\n\
                     All MCP servers in your config must have a 'description' field.\n\
                     The 'description' explains what the server does to the LLM.\n\n\
                     Example:\n  \
                     {{\n    \
                     \"description\": \"File system access for reading and writing files\",\n    \
                     \"command\": \"npx\",\n    \
                     \"args\": [\"@modelcontextprotocol/server-filesystem\"]\n  \
                     }}\n\n\
                     Error details: {}",
                error_msg
            )
        } else if error_msg.contains("missing field") {
            let field = if let Some(start) = error_msg.find("`") {
                if let Some(end) = error_msg[start + 1..].find("`") {
                    error_msg[start + 1..start + 1 + end].to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };
            anyhow::anyhow!(
                "❌ Configuration Error: Missing required field '{}'.\n\n\
                     Check your config file for incomplete server definitions.\n\n\
                     Error details: {}",
                field,
                error_msg
            )
        } else {
            anyhow::anyhow!(
                "❌ Configuration Error: Invalid config format\n\n\
                     Error details: {}",
                error_msg
            )
        }
    })?;

    // Merge the document-level `env` (a single shared environment, most
    // importantly a shared `PATH`) into every stdio server's own `env` so users
    // do not have to repeat it per server. The final OS/system PATH merge and
    // de-duplication happens later in `StdioTransport::new` via
    // `config::path_resolver`.
    if let Some(root_env) = config.env.clone() {
        for server in config.mcp_servers.values_mut() {
            merge_root_env_into_server(server, &root_env);
        }
    }

    config.mcp_servers = config
        .mcp_servers
        .into_iter()
        .map(|(name, server_config)| (name, substitute_in_config(server_config)))
        .collect();

    tracing::info!("✅ MCP server config loaded successfully");

    Ok(config)
}

/// Merge the global `env` block into a single server's `env`.
///
/// - `PATH` is concatenated (global first, then per-server) so both sources
///   contribute; de-duplication against the inherited/system PATH happens later
///   in `config::path_resolver::build_merged_path`.
/// - All other variables: the server's value wins; absent ones fall back to the
///   global value.
fn merge_root_env_into_server(server: &mut McpServerConfig, root_env: &HashMap<String, String>) {
    if let McpServerConfig::Stdio { env, .. } = server {
        let mut merged: HashMap<String, String> = root_env.clone();
        if let Some(server_env) = env {
            let root_path = merged.remove("PATH").unwrap_or_default();
            let server_path = server_env.get("PATH").cloned().unwrap_or_default();
            let combined_path = combine_path_values(&root_path, &server_path);
            for (k, v) in server_env.iter() {
                merged.insert(k.clone(), v.clone());
            }
            if !combined_path.is_empty() {
                merged.insert("PATH".to_string(), combined_path);
            }
        }
        *env = Some(merged);
    }
}

#[cfg(windows)]
const PATH_SEPARATOR: char = ';';
#[cfg(not(windows))]
const PATH_SEPARATOR: char = ':';

/// Join two PATH strings, skipping empty parts, using the platform separator.
fn combine_path_values(a: &str, b: &str) -> String {
    let a = a.trim();
    let b = b.trim();
    match (a, b) {
        ("", "") => String::new(),
        (_, "") => a.to_string(),
        ("", _) => b.to_string(),
        _ => format!(
            "{}{}{}",
            a.trim_end_matches(PATH_SEPARATOR),
            PATH_SEPARATOR,
            b.trim_start_matches(PATH_SEPARATOR)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_load_valid_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "description": "Test server",
                    "command": "node",
                    "args": ["server.js"]
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.mcp_servers.contains_key("test"));
    }

    #[tokio::test]
    async fn test_load_config_with_env_vars() {
        std::env::set_var("TEST_CONFIG_VAR", "/test/path");

        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "description": "Test server",
                    "command": "node",
                    "args": ["${TEST_CONFIG_VAR}/server.js"]
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        if let crate::config::McpServerConfig::Stdio { args, .. } =
            config.mcp_servers.get("test").unwrap()
        {
            assert_eq!(args.as_ref().unwrap()[0], "/test/path/server.js");
        } else {
            panic!("Expected Stdio config");
        }

        std::env::remove_var("TEST_CONFIG_VAR");
    }

    #[tokio::test]
    async fn test_load_nonexistent_file() {
        let result = load_config("/nonexistent/path/config.json").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to resolve"));
    }

    #[tokio::test]
    async fn test_load_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"{ invalid json }").unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Configuration Error") || error_msg.contains("Invalid"));
    }

    #[tokio::test]
    async fn test_load_config_missing_required_fields() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "command": "node"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_http_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "http_server": {
                    "type": "http",
                    "description": "HTTP test server",
                    "url": "https://api.example.com",
                    "headers": {
                        "Authorization": "Bearer token"
                    }
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(config.mcp_servers.len(), 1);
        if let crate::config::McpServerConfig::Http { url, headers, .. } =
            config.mcp_servers.get("http_server").unwrap()
        {
            assert_eq!(url, "https://api.example.com");
            assert!(headers.is_some());
        } else {
            panic!("Expected Http config");
        }
    }

    #[tokio::test]
    async fn test_load_sse_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "sse_server": {
                    "type": "sse",
                    "description": "SSE test server",
                    "url": "https://api.example.com/sse"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(config.mcp_servers.len(), 1);
        if let crate::config::McpServerConfig::Sse { url, .. } =
            config.mcp_servers.get("sse_server").unwrap()
        {
            assert_eq!(url, "https://api.example.com/sse");
        } else {
            panic!("Expected Sse config");
        }
    }

    #[tokio::test]
    async fn test_load_multiple_servers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "stdio_server": {
                    "type": "stdio",
                    "description": "Stdio server",
                    "command": "node"
                },
                "http_server": {
                    "type": "http",
                    "description": "HTTP server",
                    "url": "https://api.example.com"
                },
                "sse_server": {
                    "type": "sse",
                    "description": "SSE server",
                    "url": "https://api.example.com/sse"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(config.mcp_servers.len(), 3);
        assert!(config.mcp_servers.contains_key("stdio_server"));
        assert!(config.mcp_servers.contains_key("http_server"));
        assert!(config.mcp_servers.contains_key("sse_server"));
    }

    #[tokio::test]
    async fn test_load_server_missing_description() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "command": "node"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_multiple_servers_one_missing_description() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "server1": {
                    "type": "stdio",
                    "description": "Server 1",
                    "command": "node"
                },
                "server2": {
                    "type": "stdio",
                    "command": "node"
                },
                "server3": {
                    "type": "http",
                    "description": "Server 3",
                    "url": "https://api.example.com"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_multiple_servers_multiple_missing_descriptions() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "server1": {
                    "type": "stdio",
                    "command": "node"
                },
                "server2": {
                    "type": "http",
                    "url": "https://api.example.com"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_config_rejects_unknown_field_in_server() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "description": "Test server",
                    "command": "node",
                    "unknown_field": "invalid"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("unknown field") || error_msg.contains("Configuration Error"));
    }

    #[tokio::test]
    async fn test_load_config_rejects_unknown_top_level_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "description": "Test server",
                    "command": "node"
                }
            },
            "unknown_top_level": "invalid"
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("unknown field") || error_msg.contains("Configuration Error"));
    }

    #[tokio::test]
    async fn test_load_config_rejects_unknown_field_in_features() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "description": "Test server",
                    "command": "node",
                    "features": {
                        "tools": true,
                        "invalid_feature": true
                    }
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_http_config_rejects_unknown_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "http_server": {
                    "type": "http",
                    "description": "HTTP test server",
                    "url": "https://api.example.com",
                    "typo_field": "error"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_sse_config_rejects_unknown_field() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "sse_server": {
                    "type": "sse",
                    "description": "SSE test server",
                    "url": "https://api.example.com/sse",
                    "unknown_field": "error"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_config_with_optional_fields_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "mcpServers": {
                "full_featured": {
                    "type": "http",
                    "description": "Full featured HTTP server",
                    "url": "https://api.example.com",
                    "headers": {
                        "Authorization": "Bearer token",
                        "Content-Type": "application/json"
                    },
                    "oauth_client_id": "client-123",
                    "oauth_scopes": ["read", "write", "admin"],
                    "features": {
                        "tools": true,
                        "resources": false,
                        "prompts": true
                    }
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = load_config(temp_file.path().to_str().unwrap()).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.mcp_servers.contains_key("full_featured"));
    }

    #[tokio::test]
    async fn test_load_config_merges_root_env_into_servers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "env": {
                "GLOBAL_VAR": "from-root",
                "PATH": "/global/bin"
            },
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "description": "Test server",
                    "command": "node",
                    "env": {
                        "LOCAL_VAR": "from-server",
                        "PATH": "/server/bin"
                    }
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        if let crate::config::McpServerConfig::Stdio { env, .. } =
            config.mcp_servers.get("test").unwrap()
        {
            let env = env.as_ref().expect("merged env should be present");
            // Server override wins for non-PATH vars.
            assert_eq!(env.get("LOCAL_VAR"), Some(&"from-server".to_string()));
            // Global default applies when the server does not set it.
            assert_eq!(env.get("GLOBAL_VAR"), Some(&"from-root".to_string()));
            // PATH is concatenated (root first, then server).
            let sep = if cfg!(windows) { ';' } else { ':' };
            let path = env.get("PATH").expect("PATH should be present");
            let parts: Vec<&str> = path.split(sep).collect();
            let root_idx = parts.iter().position(|p| *p == "/global/bin");
            let server_idx = parts.iter().position(|p| *p == "/server/bin");
            assert!(root_idx.is_some() && server_idx.is_some(), "PATH not merged: {path}");
            assert!(root_idx.unwrap() < server_idx.unwrap(), "root PATH should precede server PATH: {path}");
        } else {
            panic!("Expected Stdio config");
        }
    }

    #[tokio::test]
    async fn test_load_config_root_env_applies_when_server_has_no_env() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_json = r#"{
            "env": { "GLOBAL_VAR": "from-root" },
            "mcpServers": {
                "test": {
                    "type": "stdio",
                    "description": "Test server",
                    "command": "node"
                }
            }
        }"#;
        temp_file.write_all(config_json.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        if let crate::config::McpServerConfig::Stdio { env, .. } =
            config.mcp_servers.get("test").unwrap()
        {
            let env = env.as_ref().expect("merged env should be present");
            assert_eq!(env.get("GLOBAL_VAR"), Some(&"from-root".to_string()));
        } else {
            panic!("Expected Stdio config");
        }
    }
}
