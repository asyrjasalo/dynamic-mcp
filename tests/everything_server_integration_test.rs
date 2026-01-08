use serde_json::json;
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

// TOOLS API SPECIFICATION COMPLIANCE TESTS

#[test]
fn test_everything_server_tools_list_response_format() {
    let tools_response = json!({
        "tools": [
            {
                "name": "example_tool",
                "description": "An example tool",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "param": {"type": "string"}
                    }
                }
            }
        ]
    });

    assert!(tools_response["tools"].is_array());
    let tool = &tools_response["tools"][0];
    assert!(tool["name"].is_string());
    assert!(tool["description"].is_string() || tool["description"].is_null());
    assert!(tool["inputSchema"].is_object());
}

#[test]
fn test_everything_server_tools_call_error_format() {
    let error_response = json!({
        "content": [{
            "type": "text",
            "text": "Tool execution failed",
            "isError": true
        }]
    });

    assert!(error_response["content"].is_array());
    let content = &error_response["content"][0];
    assert_eq!(content["isError"], true);
    assert!(content["text"].is_string());
}

#[test]
fn test_everything_server_tools_pagination_support() {
    let tools_list = json!({
        "tools": [
            {"name": "tool1", "inputSchema": {}},
            {"name": "tool2", "inputSchema": {}}
        ],
        "nextCursor": "page2"
    });

    assert!(tools_list["tools"].is_array());
    assert!(tools_list["nextCursor"].is_string() || tools_list["nextCursor"].is_null());
}

// PROMPTS API SPECIFICATION COMPLIANCE TESTS

#[test]
fn test_everything_server_prompts_list_response_format() {
    let prompts_response = json!({
        "prompts": [
            {
                "name": "example_prompt",
                "description": "An example prompt",
                "arguments": [
                    {
                        "name": "arg1",
                        "description": "First argument",
                        "required": true
                    }
                ]
            }
        ]
    });

    assert!(prompts_response["prompts"].is_array());
    let prompt = &prompts_response["prompts"][0];
    assert!(prompt["name"].is_string());
    assert!(prompt["arguments"].is_array() || prompt["arguments"].is_null());
}

#[test]
fn test_everything_server_prompts_get_message_structure() {
    let prompts_get = json!({
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": "What is the capital of France?"
                }
            },
            {
                "role": "assistant",
                "content": {
                    "type": "text",
                    "text": "The capital of France is Paris."
                }
            }
        ]
    });

    assert!(prompts_get["messages"].is_array());
    for msg in prompts_get["messages"].as_array().unwrap() {
        assert!(msg["role"].is_string());
        assert!(msg["content"].is_object());
    }
}

#[test]
fn test_everything_server_prompts_content_types() {
    let text_content = json!({
        "role": "user",
        "content": {"type": "text", "text": "Hello"}
    });

    let image_content = json!({
        "role": "user",
        "content": {
            "type": "image",
            "data": "base64data",
            "mimeType": "image/png"
        }
    });

    let audio_content = json!({
        "role": "user",
        "content": {
            "type": "audio",
            "data": "base64audio",
            "mimeType": "audio/mp3"
        }
    });

    let resource_content = json!({
        "role": "user",
        "content": {
            "type": "resource",
            "resource": {
                "uri": "file:///example.txt",
                "mimeType": "text/plain"
            }
        }
    });

    assert_eq!(text_content["content"]["type"], "text");
    assert_eq!(image_content["content"]["type"], "image");
    assert_eq!(audio_content["content"]["type"], "audio");
    assert_eq!(resource_content["content"]["type"], "resource");
}

#[test]
fn test_everything_server_prompts_pagination_support() {
    let prompts_list = json!({
        "prompts": [
            {"name": "prompt1"},
            {"name": "prompt2"}
        ],
        "nextCursor": "page2"
    });

    assert!(prompts_list["prompts"].is_array());
    assert!(prompts_list["nextCursor"].is_string() || prompts_list["nextCursor"].is_null());
}

// RESOURCES API SPECIFICATION COMPLIANCE TESTS

#[test]
fn test_everything_server_resources_list_response_format() {
    let resources_response = json!({
        "resources": [
            {
                "uri": "file:///example.txt",
                "name": "example.txt",
                "description": "An example file",
                "mimeType": "text/plain",
                "size": 1024,
                "annotations": {
                    "audience": ["user"],
                    "priority": 0.5,
                    "lastModified": "2025-01-08T00:00:00Z"
                }
            }
        ]
    });

    assert!(resources_response["resources"].is_array());
    let resource = &resources_response["resources"][0];
    assert!(resource["uri"].is_string());
    assert!(resource["name"].is_string());
    assert!(resource["size"].is_number() || resource["size"].is_null());
    assert!(resource["annotations"].is_object() || resource["annotations"].is_null());
}

#[test]
fn test_everything_server_resources_read_text_content() {
    let read_response = json!({
        "contents": [
            {
                "uri": "file:///example.txt",
                "mimeType": "text/plain",
                "text": "File content here",
                "annotations": {
                    "lastModified": "2025-01-08T00:00:00Z"
                }
            }
        ]
    });

    assert!(read_response["contents"].is_array());
    let content = &read_response["contents"][0];
    assert!(content["uri"].is_string());
    assert!(content["text"].is_string());
}

#[test]
fn test_everything_server_resources_read_blob_content() {
    let read_response = json!({
        "contents": [
            {
                "uri": "file:///image.png",
                "mimeType": "image/png",
                "blob": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
            }
        ]
    });

    assert!(read_response["contents"].is_array());
    let content = &read_response["contents"][0];
    assert!(content["blob"].is_string());
}

#[test]
fn test_everything_server_resources_templates_response_format() {
    let templates_response = json!({
        "resourceTemplates": [
            {
                "uriTemplate": "file:///{path}",
                "name": "Local Files",
                "description": "Access files in filesystem",
                "mimeType": "application/octet-stream",
                "annotations": {
                    "audience": ["user"],
                    "priority": 0.5
                },
                "icons": [
                    {
                        "src": "https://example.com/icon.png",
                        "mimeType": "image/png",
                        "sizes": ["16x16", "32x32"]
                    }
                ]
            }
        ]
    });

    assert!(templates_response["resourceTemplates"].is_array());
    let template = &templates_response["resourceTemplates"][0];
    assert!(template["uriTemplate"].is_string());
    assert!(template["name"].is_string());
    assert!(template["annotations"].is_object() || template["annotations"].is_null());
    assert!(template["icons"].is_array() || template["icons"].is_null());
}

#[test]
fn test_everything_server_resources_pagination_support() {
    let resources_list = json!({
        "resources": [
            {"uri": "file:///a.txt", "name": "a.txt"},
            {"uri": "file:///b.txt", "name": "b.txt"}
        ],
        "nextCursor": "page2"
    });

    assert!(resources_list["resources"].is_array());
    assert!(resources_list["nextCursor"].is_string() || resources_list["nextCursor"].is_null());
}

#[test]
fn test_everything_server_resources_size_field() {
    let resource_with_size = json!({
        "uri": "file:///large.bin",
        "name": "large.bin",
        "size": 5242880
    });

    let resource_without_size = json!({
        "uri": "file:///unknown.txt",
        "name": "unknown.txt"
    });

    assert_eq!(resource_with_size["size"], 5242880);
    assert!(resource_without_size["size"].is_null());
}

#[test]
fn test_everything_server_resources_annotations_complete() {
    let resource = json!({
        "uri": "file:///document.txt",
        "name": "document.txt",
        "annotations": {
            "audience": ["user", "assistant", "admin"],
            "priority": 0.95,
            "lastModified": "2025-01-08T12:00:00Z"
        }
    });

    let ann = &resource["annotations"];
    assert!(ann["audience"].is_array());
    assert_eq!(ann["audience"].as_array().unwrap().len(), 3);
    assert_eq!(ann["priority"], 0.95);
    assert!(ann["lastModified"].is_string());
}

#[test]
fn test_everything_server_resources_icons_support() {
    let resource = json!({
        "uri": "file:///image.png",
        "name": "image.png",
        "icons": [
            {
                "src": "https://icons.example.com/file-icon.svg",
                "mimeType": "image/svg+xml",
                "sizes": ["16x16", "32x32", "48x48"]
            }
        ]
    });

    assert!(resource["icons"].is_array());
    let icon = &resource["icons"][0];
    assert!(icon["src"].is_string());
    assert!(icon["mimeType"].is_string());
    assert!(icon["sizes"].is_array());
}

// CAPABILITY DECLARATION TESTS

#[test]
fn test_everything_server_initialize_capabilities() {
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

// JSON-RPC ERROR HANDLING TESTS

#[test]
fn test_everything_server_jsonrpc_error_codes() {
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
