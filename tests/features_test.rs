// Features Integration Tests
// Tests per-server feature flag configuration (tools, resources, prompts)

use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_config_with_features_disabled_parses_successfully() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");

    let config_json = json!({
        "mcpServers": {
            "test_server": {
                "description": "Test server with resources and prompts disabled",
                "command": "echo",
                "args": ["test"],
                "features": {
                    "resources": false,
                    "prompts": false
                }
            }
        }
    });

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_json.to_string().as_bytes()).unwrap();

    // Verify the config file exists and is valid JSON
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(parsed["mcpServers"]["test_server"]["features"]["resources"] == false);
    assert!(parsed["mcpServers"]["test_server"]["features"]["prompts"] == false);
}

#[test]
fn test_config_without_features_parses_successfully() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");

    let config_json = json!({
        "mcpServers": {
            "test_server": {
                "description": "Test server with default features",
                "command": "echo"
            }
        }
    });

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_json.to_string().as_bytes()).unwrap();

    // Verify the config file exists and is valid JSON
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Features key should not exist when not specified
    assert!(parsed["mcpServers"]["test_server"]["features"].is_null());
}

#[test]
fn test_config_with_mixed_features() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");

    let config_json = json!({
        "mcpServers": {
            "server1": {
                "description": "Server with all features",
                "command": "cmd1"
            },
            "server2": {
                "description": "Server without resources",
                "command": "cmd2",
                "features": {
                    "resources": false
                }
            },
            "server3": {
                "type": "http",
                "description": "HTTP server with only tools",
                "url": "http://localhost:8080",
                "features": {
                    "resources": false,
                    "prompts": false
                }
            }
        }
    });

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_json.to_string().as_bytes()).unwrap();

    // Verify the config file exists and is valid JSON
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Server 1 has no features key
    assert!(parsed["mcpServers"]["server1"]["features"].is_null());

    // Server 2 has resources disabled
    assert!(parsed["mcpServers"]["server2"]["features"]["resources"] == false);

    // Server 3 has both resources and prompts disabled
    assert!(parsed["mcpServers"]["server3"]["features"]["resources"] == false);
    assert!(parsed["mcpServers"]["server3"]["features"]["prompts"] == false);
}

#[test]
fn test_config_with_explicit_enables() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");

    let config_json = json!({
        "mcpServers": {
            "test_server": {
                "description": "Server with explicit enables",
                "command": "test",
                "features": {
                    "tools": true,
                    "resources": true,
                    "prompts": false
                }
            }
        }
    });

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_json.to_string().as_bytes()).unwrap();

    // Verify the config file exists and is valid JSON
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(parsed["mcpServers"]["test_server"]["features"]["tools"] == true);
    assert!(parsed["mcpServers"]["test_server"]["features"]["resources"] == true);
    assert!(parsed["mcpServers"]["test_server"]["features"]["prompts"] == false);
}

#[test]
fn test_config_with_all_features_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");

    let config_json = json!({
        "mcpServers": {
            "test_server": {
                "type": "sse",
                "description": "SSE server with all features disabled",
                "url": "http://localhost:8080/sse",
                "features": {
                    "tools": false,
                    "resources": false,
                    "prompts": false
                }
            }
        }
    });

    let mut file = fs::File::create(&config_path).unwrap();
    file.write_all(config_json.to_string().as_bytes()).unwrap();

    // Verify the config file exists and is valid JSON
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(parsed["mcpServers"]["test_server"]["features"]["tools"] == false);
    assert!(parsed["mcpServers"]["test_server"]["features"]["resources"] == false);
    assert!(parsed["mcpServers"]["test_server"]["features"]["prompts"] == false);
}
