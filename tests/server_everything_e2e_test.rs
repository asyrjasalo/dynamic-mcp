use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

struct DynamicMcpServer {
    process: Child,
    reader: BufReader<std::process::ChildStdout>,
}

impl DynamicMcpServer {
    fn start_with_everything_server() -> Self {
        let mut config_file = NamedTempFile::new().unwrap();
        let config = json!({
            "mcpServers": {
                "everything": {
                    "description": "Comprehensive MCP server for E2E testing",
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-everything"]
                }
            }
        });
        config_file
            .write_all(config.to_string().as_bytes())
            .unwrap();
        config_file.flush().unwrap();

        let mut process = Command::new("cargo")
            .args(["run", "--quiet", "--"])
            .arg(config_file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start dynamic-mcp");

        let stdout = process.stdout.take().expect("Failed to get stdout");
        let reader = BufReader::new(stdout);

        // Wait longer for everything-server to initialize
        // (npx may need to download the package on first run, which can take 15-20 seconds)
        thread::sleep(Duration::from_secs(15));

        Self { process, reader }
    }

    fn send_request(&mut self, request: Value) -> Value {
        let stdin = self.process.stdin.as_mut().expect("Failed to get stdin");
        let request_str = format!("{}\n", request);
        stdin
            .write_all(request_str.as_bytes())
            .expect("Failed to write request");
        stdin.flush().expect("Failed to flush stdin");

        let mut response_str = String::new();
        let reader = &mut self.reader;
        let bytes_read = reader
            .read_line(&mut response_str)
            .expect("Failed to read response");

        if bytes_read == 0 {
            panic!("Got EOF from server, no response data");
        }

        serde_json::from_str(&response_str).expect("Failed to parse response")
    }
}

impl Drop for DynamicMcpServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

#[test]
fn test_e2e_initialize() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });

    let response = server.send_request(request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["capabilities"]["tools"].is_object());
    assert!(response["result"]["capabilities"]["prompts"].is_object());
    assert!(response["result"]["capabilities"]["resources"].is_object());
}

#[test]
fn test_e2e_tools_list() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let tools_list_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let response = server.send_request(tools_list_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert!(response["result"]["tools"].is_array());
    let tools = response["result"]["tools"].as_array().unwrap();

    assert!(!tools.is_empty(), "Should have at least one tool");
    assert!(
        tools.iter().any(|t| t["name"] == "get_dynamic_tools"),
        "Should have get_dynamic_tools"
    );
    assert!(
        tools.iter().any(|t| t["name"] == "call_dynamic_tool"),
        "Should have call_dynamic_tool"
    );
}

#[test]
fn test_e2e_get_dynamic_tools_everything() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let tools_list_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let response = server.send_request(tools_list_request);

    assert!(response["result"]["tools"].is_array());

    let get_dynamic_tools = response["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "get_dynamic_tools")
        .expect("Should find get_dynamic_tools");

    assert!(get_dynamic_tools["inputSchema"]["properties"]["group"].is_object());
    assert!(
        get_dynamic_tools["inputSchema"]["properties"]["group"]["enum"].is_array(),
        "Should have enum field with available groups"
    );
}

#[test]
fn test_e2e_call_dynamic_tool_get_dynamic_tools() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let call_tool_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "call_dynamic_tool",
            "arguments": {
                "group": "everything",
                "name": "get_dynamic_tools",
                "args": {
                    "group": "everything"
                }
            }
        }
    });

    let response = server.send_request(call_tool_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    assert!(response["result"]["content"].is_array());

    let content = response["result"]["content"][0].clone();
    assert_eq!(content["type"], "text");
    assert!(content["text"].is_string());

    let tools_text = content["text"].as_str().unwrap();
    assert!(!tools_text.is_empty(), "Should have non-empty tool list");
}

#[test]
fn test_e2e_tools_echo_execution() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let call_tool_request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "call_dynamic_tool",
            "arguments": {
                "group": "everything",
                "name": "echo",
                "args": {
                    "message": "test_message_from_e2e"
                }
            }
        }
    });

    let response = server.send_request(call_tool_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 4);
    assert!(response["result"].is_object());
    assert!(response["result"]["content"].is_array());

    let content = response["result"]["content"][0].clone();
    assert_eq!(content["type"], "text");
    let result_text = content["text"].as_str().unwrap();
    assert!(result_text.contains("test_message_from_e2e"));
}

#[test]
fn test_e2e_prompts_list() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let prompts_list_request = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "prompts/list",
        "params": {
            "group": "everything"
        }
    });

    let response = server.send_request(prompts_list_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 5);
    assert!(response["result"]["prompts"].is_array());

    let prompts = response["result"]["prompts"].as_array().unwrap();
    assert!(
        !prompts.is_empty(),
        "Should have at least one prompt from everything-server"
    );

    for prompt in prompts {
        assert!(prompt["name"].is_string());
    }
}

#[test]
fn test_e2e_prompts_get_simple() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let prompts_get_request = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "prompts/get",
        "params": {
            "group": "everything",
            "name": "simple-prompt"
        }
    });

    let response = server.send_request(prompts_get_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 6);

    if response["result"].is_object() {
        assert!(response["result"]["messages"].is_array());
        let messages = response["result"]["messages"].as_array().unwrap();
        assert!(!messages.is_empty());

        for msg in messages {
            assert!(msg["role"].is_string());
            assert!(msg["content"].is_object());
        }
    }
}

#[test]
fn test_e2e_resources_list() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let resources_list_request = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "resources/list",
        "params": {
            "group": "everything"
        }
    });

    let response = server.send_request(resources_list_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 7);
    assert!(response["result"]["resources"].is_array());

    let resources = response["result"]["resources"].as_array().unwrap();
    assert!(
        !resources.is_empty(),
        "Should have at least one resource from everything-server"
    );

    for resource in resources {
        assert!(resource["uri"].is_string());
        assert!(resource["name"].is_string());
    }
}

#[test]
fn test_e2e_resources_read() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let resources_list_request = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "resources/list",
        "params": {
            "group": "everything"
        }
    });

    let list_response = server.send_request(resources_list_request);
    let resources = list_response["result"]["resources"].as_array().unwrap();

    if !resources.is_empty() {
        let first_resource = &resources[0];
        let uri = first_resource["uri"].as_str().unwrap();

        let resources_read_request = json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "resources/read",
            "params": {
                "group": "everything",
                "uri": uri
            }
        });

        let response = server.send_request(resources_read_request);

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 8);

        if response["result"].is_object() {
            assert!(response["result"]["contents"].is_array());
            let contents = response["result"]["contents"].as_array().unwrap();

            if !contents.is_empty() {
                let content = &contents[0];
                assert!(content["uri"].is_string());
                assert!(content["text"].is_string() || content["blob"].is_string());
            }
        }
    }
}

#[test]
fn test_e2e_resources_templates_list() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let templates_list_request = json!({
        "jsonrpc": "2.0",
        "id": 9,
        "method": "resources/templates/list",
        "params": {
            "group": "everything"
        }
    });

    let response = server.send_request(templates_list_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 9);

    if response["result"].is_object() && response["result"]["resourceTemplates"].is_array() {
        let templates = response["result"]["resourceTemplates"].as_array().unwrap();
        for template in templates {
            assert!(template["uriTemplate"].is_string());
            assert!(template["name"].is_string());
        }
    }
}

#[test]
fn test_e2e_error_handling_invalid_group() {
    let mut server = DynamicMcpServer::start_with_everything_server();

    let initialize_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let _ = server.send_request(initialize_request);

    let invalid_request = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/list",
        "params": {
            "group": "nonexistent"
        }
    });

    let response = server.send_request(invalid_request);

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 10);
    assert!(
        response["error"].is_object() || response["result"].is_object(),
        "Should either have error or handle gracefully"
    );
}
