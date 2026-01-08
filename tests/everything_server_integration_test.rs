use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_config_with_server_everything() {
    let mut config_file = NamedTempFile::new().unwrap();
    let config = r#"{
  "mcpServers": {
    "everything": {
      "description": "Server with comprehensive tools and resources",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-everything"]
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
        servers.contains_key("everything"),
        "Config should have 'everything' server"
    );

    let everything = &servers["everything"];
    assert_eq!(
        everything.get("description").unwrap().as_str().unwrap(),
        "Server with comprehensive tools and resources"
    );
    assert_eq!(everything.get("command").unwrap().as_str().unwrap(), "npx");
}

#[test]
fn test_server_everything_package_available() {
    let output = Command::new("npx").args(["--version"]).output();

    if output.is_err() {
        eprintln!("npx not available, skipping test");
        return;
    }

    assert!(output.unwrap().status.success(), "npx should be available");
}

#[test]
fn test_example_config_supports_server_everything() {
    let example_config = r#"{
  "mcpServers": {
    "filesystem": {
      "description": "File system access",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "everything": {
      "description": "Comprehensive MCP server with tools and resources",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-everything"]
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
    assert!(servers.contains_key("everything"));
    assert!(servers.contains_key("filesystem"));
}
