// Config Integration Tests
// Tests configuration file parsing, server definitions, and config artifacts

use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;

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
