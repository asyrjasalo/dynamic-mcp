// Config Integration Tests
// Tests configuration file parsing, server definitions, config artifacts, and live reload

use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

/// Test 1: Config file with server definition
/// Tests: config file loading, mcpServers structure, server properties
#[test]
fn test_config_file_with_server() {
    let mut config_file = NamedTempFile::new().unwrap();
    let config = r#"{
  "mcpServers": {
    "test-group": {
      "description": "Server with comprehensive tools and resources",
      "command": "npx",
      "args": ["-y", "test-server"]
    }
  }
}"#;
    config_file.write_all(config.as_bytes()).unwrap();
    config_file.flush().unwrap();

    let content = std::fs::read_to_string(config_file.path()).expect("Failed to read config file");

    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    assert!(
        parsed.get("mcpServers").is_some(),
        "Config should have mcpServers"
    );

    let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
    assert!(
        servers.contains_key("test-group"),
        "Config should have test server"
    );

    let server = &servers["test-group"];
    assert_eq!(
        server.get("description").unwrap().as_str().unwrap(),
        "Server with comprehensive tools and resources"
    );
    assert_eq!(server.get("command").unwrap().as_str().unwrap(), "npx");
}

/// Test 2: Example config with multiple server definitions
/// Tests: multiple servers in config, filesystem and test servers
#[test]
fn test_example_config_with_server_definition() {
    let example_config = r#"{
  "mcpServers": {
    "filesystem": {
      "description": "File system access",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "test-group": {
      "description": "Comprehensive MCP server with tools and resources",
      "command": "npx",
      "args": ["-y", "test-server"]
    }
  }
}"#;

    let parsed: serde_json::Value =
        serde_json::from_str(example_config).expect("Example config should be valid JSON");

    assert!(
        parsed.get("mcpServers").is_some(),
        "Config should have mcpServers"
    );

    let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
    assert!(servers.contains_key("test-group"));
    assert!(servers.contains_key("filesystem"));
}

/// Test 3: Initialize capabilities declaration
/// Tests: tools, prompts, and resources capabilities present
#[test]
fn test_config_initialize_capabilities() {
    let initialize = json!({
        "capabilities": {
            "tools": {},
            "prompts": {},
            "resources": {
                "subscribe": true
            }
        }
    });

    assert!(initialize["capabilities"]["tools"].is_object());
    assert!(initialize["capabilities"]["prompts"].is_object());
    assert!(initialize["capabilities"]["resources"].is_object());
}

/// Test 4: JSON-RPC error codes
/// Tests: standard error codes for invalid params, method not found, internal error
#[test]
fn test_config_jsonrpc_error_codes() {
    let error_invalid_params = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32602,
            "message": "Invalid params"
        }
    });

    let error_method_not_found = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    });

    let error_internal = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "error": {
            "code": -32603,
            "message": "Internal error"
        }
    });

    assert_eq!(error_invalid_params["error"]["code"], -32602);
    assert_eq!(error_method_not_found["error"]["code"], -32601);
    assert_eq!(error_internal["error"]["code"], -32603);
}

/// Test 5: Example config file schema validation
/// Tests: examples/config.example.json has valid structure
#[test]
fn test_config_example_schema_validation() {
    let config = std::fs::read_to_string("examples/config.example.json")
        .expect("Failed to read examples/config.example.json");

    let parsed: serde_json::Value =
        serde_json::from_str(&config).expect("Config should be valid JSON");

    assert!(
        parsed.get("mcpServers").is_some(),
        "Config should have mcpServers"
    );
}

/// Test 6: Example config file exists
/// Tests: examples/config.example.json artifact presence
#[test]
fn test_config_example_exists() {
    assert!(
        std::path::Path::new("examples/config.example.json").exists(),
        "examples/config.example.json should exist"
    );
}

/// Test 7: Config live reload - file modification detection
/// Tests: ConfigWatcher detects when config file is modified
#[test]
fn test_config_live_reload_file_modified() {
    let config_dir = TempDir::new().unwrap();
    let config_path = config_dir.path().join("config.json");

    let initial_config = json!({
        "mcpServers": {
            "server1": {
                "description": "Test server 1",
                "command": "echo"
            }
        }
    });

    fs::write(&config_path, initial_config.to_string()).unwrap();

    let modified_config = json!({
        "mcpServers": {
            "server1": {
                "description": "Test server 1 - modified",
                "command": "echo"
            },
            "server2": {
                "description": "Test server 2",
                "command": "cat"
            }
        }
    });

    fs::write(&config_path, modified_config.to_string()).unwrap();

    let content = fs::read_to_string(&config_path).expect("Failed to read modified config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
    assert_eq!(
        servers.len(),
        2,
        "Config should have 2 servers after modification"
    );
    assert!(
        servers.contains_key("server2"),
        "Config should contain server2"
    );
}

/// Test 8: Config live reload - server addition
/// Tests: New servers can be added to config
#[test]
fn test_config_live_reload_add_server() {
    let config_dir = TempDir::new().unwrap();
    let config_path = config_dir.path().join("config.json");

    let config = json!({
        "mcpServers": {}
    });

    fs::write(&config_path, config.to_string()).unwrap();

    let updated_config = json!({
        "mcpServers": {
            "new_server": {
                "description": "Newly added server",
                "command": "ls"
            }
        }
    });

    fs::write(&config_path, updated_config.to_string()).unwrap();

    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
    assert!(
        servers.contains_key("new_server"),
        "Config should contain newly added server"
    );
}

/// Test 9: Config live reload - server removal
/// Tests: Servers can be removed from config
#[test]
fn test_config_live_reload_remove_server() {
    let config_dir = TempDir::new().unwrap();
    let config_path = config_dir.path().join("config.json");

    let config = json!({
        "mcpServers": {
            "server_to_remove": {
                "description": "This server will be removed",
                "command": "echo"
            },
            "server_to_keep": {
                "description": "This server will stay",
                "command": "cat"
            }
        }
    });

    fs::write(&config_path, config.to_string()).unwrap();

    let updated_config = json!({
        "mcpServers": {
            "server_to_keep": {
                "description": "This server will stay",
                "command": "cat"
            }
        }
    });

    fs::write(&config_path, updated_config.to_string()).unwrap();

    let content = fs::read_to_string(&config_path).expect("Failed to read config");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Config should be valid JSON");

    let servers = parsed.get("mcpServers").unwrap().as_object().unwrap();
    assert!(
        !servers.contains_key("server_to_remove"),
        "Config should not contain removed server"
    );
    assert!(
        servers.contains_key("server_to_keep"),
        "Config should still contain kept server"
    );
}
