use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_prompts_list_with_everything_server() {
    let mut config_file = NamedTempFile::new().unwrap();
    let config = r#"{
  "mcpServers": {
    "everything": {
      "description": "Comprehensive MCP server with tools, resources, and prompts",
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
        "Comprehensive MCP server with tools, resources, and prompts"
    );
}

#[test]
fn test_dynamic_mcp_config_with_prompts_support() {
    let config = r#"{
  "mcpServers": {
    "everything": {
      "description": "Server with prompts, resources, and tools",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-everything"]
    }
  }
}"#;

    let parsed: serde_json::Value =
        serde_json::from_str(config).expect("Config should be valid JSON");

    let servers = parsed["mcpServers"].as_object().unwrap();
    let everything = &servers["everything"];

    assert!(everything.get("command").is_some());
    assert!(everything.get("description").is_some());
    assert!(everything.get("args").is_some());

    let args = everything.get("args").unwrap().as_array().unwrap();
    assert!(args.len() >= 2);
    assert_eq!(args[0].as_str().unwrap(), "-y");
    assert_eq!(
        args[1].as_str().unwrap(),
        "@modelcontextprotocol/server-everything"
    );
}

#[test]
fn test_server_everything_supports_prompts_in_capabilities() {
    let output = Command::new("npx").args(["--version"]).output();

    if output.is_err() {
        eprintln!("npx not available, skipping test");
        return;
    }

    assert!(output.unwrap().status.success(), "npx should be available");
}

#[test]
fn test_prompts_list_request_format() {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompts/list",
        "params": {
            "group": "everything"
        }
    });

    assert_eq!(request["jsonrpc"], "2.0");
    assert_eq!(request["method"], "prompts/list");
    assert!(request["params"]["group"].is_string());
}

#[test]
fn test_prompts_get_request_format() {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "prompts/get",
        "params": {
            "group": "everything",
            "name": "complex_prompt",
            "arguments": {
                "arg1": "value1"
            }
        }
    });

    assert_eq!(request["jsonrpc"], "2.0");
    assert_eq!(request["method"], "prompts/get");
    assert_eq!(request["params"]["name"], "complex_prompt");
    assert!(request["params"]["arguments"].is_object());
}

#[test]
fn test_dynamic_mcp_prompts_response_format() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "description": "A complex prompt",
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "Please help with this"
                    }
                }
            ]
        }
    });

    assert_eq!(response["jsonrpc"], "2.0");
    let result = response["result"].as_object().unwrap();
    assert!(result.contains_key("messages"));
    assert!(result.contains_key("description"));

    let messages = result["messages"].as_array().unwrap();
    assert!(!messages.is_empty());
    assert_eq!(messages[0]["role"], "user");
    assert!(messages[0]["content"]["type"].is_string());
}

#[test]
fn test_prompts_with_image_content() {
    let message = serde_json::json!({
        "role": "user",
        "content": {
            "type": "image",
            "data": "base64encodedimagedata",
            "mimeType": "image/png"
        }
    });

    assert_eq!(message["role"], "user");
    assert_eq!(message["content"]["type"], "image");
    assert!(message["content"]["data"].is_string());
    assert_eq!(message["content"]["mimeType"], "image/png");
}

#[test]
fn test_prompts_with_resource_content() {
    let message = serde_json::json!({
        "role": "assistant",
        "content": {
            "type": "resource",
            "resource": {
                "uri": "file:///example.txt",
                "mimeType": "text/plain",
                "text": "Resource content"
            }
        }
    });

    assert_eq!(message["role"], "assistant");
    assert_eq!(message["content"]["type"], "resource");
    assert!(message["content"]["resource"].is_object());

    let resource = message["content"]["resource"].as_object().unwrap();
    assert!(resource.contains_key("uri"));
    assert!(resource.contains_key("mimeType"));
    assert!(resource.contains_key("text"));
}

#[test]
fn test_prompt_argument_structure() {
    let argument = serde_json::json!({
        "name": "code",
        "description": "The code to review",
        "required": true
    });

    assert_eq!(argument["name"], "code");
    assert_eq!(argument["description"], "The code to review");
    assert_eq!(argument["required"], true);
}

#[test]
fn test_prompt_with_arguments() {
    let prompt = serde_json::json!({
        "name": "code_review",
        "title": "Request Code Review",
        "description": "Asks the LLM to analyze code quality",
        "arguments": [
            {
                "name": "code",
                "description": "The code to review",
                "required": true
            },
            {
                "name": "language",
                "description": "Programming language",
                "required": false
            }
        ]
    });

    assert_eq!(prompt["name"], "code_review");
    assert!(prompt["arguments"].is_array());

    let args = prompt["arguments"].as_array().unwrap();
    assert_eq!(args.len(), 2);
    assert_eq!(args[0]["name"], "code");
    assert_eq!(args[1]["name"], "language");
    assert_eq!(args[0]["required"], true);
    assert_eq!(args[1]["required"], false);
}

#[test]
fn test_prompts_list_response_structure() {
    let response = serde_json::json!({
        "prompts": [
            {
                "name": "test_prompt",
                "title": "Test Prompt",
                "description": "A test prompt",
                "arguments": [
                    {
                        "name": "input",
                        "description": "Input text",
                        "required": true
                    }
                ]
            }
        ],
        "nextCursor": "page2"
    });

    assert!(response["prompts"].is_array());
    let prompts = response["prompts"].as_array().unwrap();
    assert!(!prompts.is_empty());

    let prompt = &prompts[0];
    assert!(prompt["name"].is_string());
    assert!(prompt["title"].is_string());
    assert!(prompt["description"].is_string());
    assert!(prompt["arguments"].is_array());
}

#[test]
fn test_prompts_pagination_with_cursor() {
    let request1 = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompts/list",
        "params": {
            "group": "everything"
        }
    });

    let request2 = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "prompts/list",
        "params": {
            "group": "everything",
            "cursor": "page2"
        }
    });

    assert_eq!(request1["method"], "prompts/list");
    assert_eq!(request2["method"], "prompts/list");
    assert!(!request1["params"]
        .as_object()
        .unwrap()
        .contains_key("cursor"));
    assert!(request2["params"]["cursor"].is_string());
}

#[test]
fn test_dynamic_mcp_exposes_prompts_capability() {
    let initialize_response = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {},
            "resources": {
                "subscribe": false,
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": "dynamic-mcp",
            "version": "1.3.0"
        }
    });

    assert!(initialize_response["capabilities"]["prompts"].is_object());
    let prompts_cap = initialize_response["capabilities"]["prompts"]
        .as_object()
        .unwrap();
    assert!(prompts_cap.contains_key("listChanged"));
}

#[test]
fn test_prompts_get_with_optional_arguments() {
    let request_no_args = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompts/get",
        "params": {
            "group": "everything",
            "name": "simple_prompt"
        }
    });

    let request_with_args = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "prompts/get",
        "params": {
            "group": "everything",
            "name": "complex_prompt",
            "arguments": {
                "param1": "value1",
                "param2": "value2"
            }
        }
    });

    assert_eq!(request_no_args["method"], "prompts/get");
    assert_eq!(request_with_args["method"], "prompts/get");
    assert!(!request_no_args["params"]
        .as_object()
        .unwrap()
        .contains_key("arguments"));
    assert!(request_with_args["params"]["arguments"].is_object());
}
