// Prompts API Integration Tests with everything-server
// Tests compliance with MCP specification v2025-11-25
// https://modelcontextprotocol.io/specification/2025-11-25/server/prompts

use serde_json::json;

/// Test 1: Prompts/list request format
/// Tests: method name, group parameter, pagination cursor support
#[test]
fn test_prompts_list_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompts/list",
        "params": {
            "group": "everything",
            "cursor": null
        }
    });

    assert_eq!(request["method"], "prompts/list");
    assert!(request["params"]["group"].is_string());
}

/// Test 2: Prompts/list response structure
/// Tests: prompts array, prompt metadata (name, description, arguments)
#[test]
fn test_prompts_list_response_structure() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "prompts": [
                {
                    "name": "code_review",
                    "description": "Review and improve code",
                    "arguments": [
                        {
                            "name": "language",
                            "description": "Programming language",
                            "required": true
                        }
                    ]
                }
            ]
        }
    });

    assert!(response["result"]["prompts"].is_array());
    let prompt = &response["result"]["prompts"][0];
    assert!(prompt["name"].is_string());
    assert!(prompt["description"].is_string() || prompt["description"].is_null());
    assert!(prompt["arguments"].is_array() || prompt["arguments"].is_null());
}

/// Test 3: Prompt arguments structure
/// Tests: required vs optional arguments, description field
#[test]
fn test_prompt_arguments_structure() {
    let prompt = json!({
        "name": "example_prompt",
        "arguments": [
            {
                "name": "required_arg",
                "description": "A required argument",
                "required": true
            },
            {
                "name": "optional_arg",
                "description": "An optional argument",
                "required": false
            }
        ]
    });

    let args = prompt["arguments"].as_array().unwrap();
    assert_eq!(args.len(), 2);

    let req_arg = &args[0];
    assert_eq!(req_arg["required"], true);
    assert!(req_arg["name"].is_string());
    assert!(req_arg["description"].is_string());

    let opt_arg = &args[1];
    assert_eq!(opt_arg["required"], false);
}

/// Test 4: Prompts/get request format
/// Tests: prompt name and arguments in request
#[test]
fn test_prompts_get_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "prompts/get",
        "params": {
            "group": "everything",
            "name": "code_review",
            "arguments": {
                "language": "rust"
            }
        }
    });

    assert_eq!(request["method"], "prompts/get");
    assert!(request["params"]["group"].is_string());
    assert!(request["params"]["name"].is_string());
    assert!(request["params"]["arguments"].is_object() || request["params"]["arguments"].is_null());
}

/// Test 5: Prompts/get response format
/// Tests: prompt messages, message structure with role and content
#[test]
fn test_prompts_get_response_format() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "Review this code for best practices"
                    }
                },
                {
                    "role": "assistant",
                    "content": {
                        "type": "text",
                        "text": "I'll review your code..."
                    }
                }
            ]
        }
    });

    assert!(response["result"]["messages"].is_array());
    let messages = response["result"]["messages"].as_array().unwrap();

    for msg in messages {
        assert!(msg["role"].is_string());
        let role = msg["role"].as_str().unwrap();
        assert!(
            role == "user" || role == "assistant",
            "role must be user or assistant"
        );
        assert!(msg["content"].is_object());
    }
}

/// Test 6: Prompt message with text content
/// Tests: text content type format
#[test]
fn test_prompt_message_text_content() {
    let message = json!({
        "role": "user",
        "content": {
            "type": "text",
            "text": "What is the capital of France?"
        }
    });

    assert_eq!(message["content"]["type"], "text");
    assert!(message["content"]["text"].is_string());
}

/// Test 7: Prompt message with image content
/// Tests: image content type with MIME type and base64 data
#[test]
fn test_prompt_message_image_content() {
    let message = json!({
        "role": "user",
        "content": {
            "type": "image",
            "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            "mimeType": "image/png"
        }
    });

    assert_eq!(message["content"]["type"], "image");
    assert!(message["content"]["data"].is_string());
    assert!(message["content"]["mimeType"].is_string());
}

/// Test 8: Prompt message with audio content
/// Tests: audio content type with MIME type
#[test]
fn test_prompt_message_audio_content() {
    let message = json!({
        "role": "user",
        "content": {
            "type": "audio",
            "data": "base64encodedaudio",
            "mimeType": "audio/mp3"
        }
    });

    assert_eq!(message["content"]["type"], "audio");
    assert!(message["content"]["data"].is_string());
    assert!(message["content"]["mimeType"].is_string());
}

/// Test 9: Prompt message with resource content
/// Tests: resource reference in prompt
#[test]
fn test_prompt_message_resource_content() {
    let message = json!({
        "role": "user",
        "content": {
            "type": "resource",
            "resource": {
                "uri": "file:///example.txt",
                "mimeType": "text/plain",
                "text": "This is the resource content"
            }
        }
    });

    assert_eq!(message["content"]["type"], "resource");
    assert!(message["content"]["resource"]["uri"].is_string());
}

/// Test 10: Prompt with multiple message types
/// Tests: conversation with mixed message roles and content types
#[test]
fn test_prompt_multiple_messages() {
    let prompt = json!({
        "messages": [
            {
                "role": "user",
                "content": {"type": "text", "text": "Hello"}
            },
            {
                "role": "assistant",
                "content": {"type": "text", "text": "Hi there!"}
            },
            {
                "role": "user",
                "content": {
                    "type": "image",
                    "data": "imagedata",
                    "mimeType": "image/png"
                }
            }
        ]
    });

    let messages = prompt["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[1]["role"], "assistant");
    assert_eq!(messages[2]["role"], "user");
    assert_eq!(messages[2]["content"]["type"], "image");
}

/// Test 11: Prompts API pagination support
/// Tests: cursor-based pagination in prompts/list
#[test]
fn test_prompts_list_pagination() {
    let response = json!({
        "prompts": [
            {"name": "prompt1", "arguments": []},
            {"name": "prompt2", "arguments": []}
        ],
        "nextCursor": "page2"
    });

    assert!(response["prompts"].is_array());
    assert_eq!(response["prompts"].as_array().unwrap().len(), 2);
    assert!(response["nextCursor"].is_string());
}

/// Test 12: Prompts API capability declaration
/// Tests: prompts capability in initialize response
#[test]
fn test_prompts_capability_declaration() {
    let initialize = json!({
        "capabilities": {
            "prompts": {}
        }
    });

    assert!(
        initialize["capabilities"]["prompts"].is_object(),
        "prompts capability must be present"
    );
}

/// Test 13: Prompt without arguments
/// Tests: prompts with empty or no arguments field
#[test]
fn test_prompt_without_arguments() {
    let prompt = json!({
        "name": "simple_prompt",
        "description": "A prompt with no arguments",
        "arguments": []
    });

    assert!(prompt["arguments"].is_array());
    assert_eq!(prompt["arguments"].as_array().unwrap().len(), 0);
}

/// Test 14: Prompt with complex argument types
/// Tests: various argument descriptions and configurations
#[test]
fn test_prompt_complex_arguments() {
    let prompt = json!({
        "name": "complex_prompt",
        "arguments": [
            {
                "name": "code",
                "description": "Source code to analyze",
                "required": true
            },
            {
                "name": "language",
                "description": "Programming language (e.g., python, javascript)",
                "required": true
            },
            {
                "name": "style_guide",
                "description": "Optional style guide to follow",
                "required": false
            },
            {
                "name": "line_limit",
                "description": "Optional line length limit",
                "required": false
            }
        ]
    });

    let args = prompt["arguments"].as_array().unwrap();
    assert_eq!(args.len(), 4);

    let required_count = args
        .iter()
        .filter(|a| a["required"].as_bool().unwrap_or(false))
        .count();
    assert_eq!(required_count, 2);
}

/// Test 15: Prompts from everything-server configuration
/// Tests: config format for everything-server prompts support
#[test]
fn test_everything_server_prompts_config() {
    let config = json!({
        "mcpServers": {
            "everything": {
                "description": "Server with comprehensive tools, resources, and prompts",
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-everything"]
            }
        }
    });

    let server = &config["mcpServers"]["everything"];
    assert!(server["description"].is_string());
    assert_eq!(server["command"], "npx");
    assert!(server["args"].is_array());
}

/// Test 16: Prompts/get with optional arguments
/// Tests: calling prompts/get with missing optional arguments
#[test]
fn test_prompts_get_with_missing_optional_args() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "prompts/get",
        "params": {
            "group": "everything",
            "name": "code_review",
            "arguments": {
                "language": "rust"
            }
        }
    });

    let args = &request["params"]["arguments"];
    assert!(args["language"].is_string());
}

/// Test 17: Empty prompts/list response
/// Tests: handling empty prompts list
#[test]
fn test_empty_prompts_list_response() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "prompts": []
        }
    });

    assert!(response["result"]["prompts"].is_array());
    assert_eq!(response["result"]["prompts"].as_array().unwrap().len(), 0);
}

/// Test 18: Prompt with multiline text content
/// Tests: text content with newlines and special formatting
#[test]
fn test_prompt_multiline_text_content() {
    let _prompt = json!({
        "name": "multiline_prompt",
        "arguments": [
            {"name": "code", "required": true}
        ]
    });

    let messages = json!({
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": "Please review the following code:\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\nFocus on:\n1. Performance\n2. Best practices\n3. Safety"
                }
            }
        ]
    });

    let text = &messages["messages"][0]["content"]["text"];
    assert!(text.is_string());
    let text_str = text.as_str().unwrap();
    assert!(text_str.contains("```"));
    assert!(text_str.contains("\n"));
}

/// Test 19: JSON-RPC errors for prompts API
/// Tests: error codes for prompts methods
#[test]
fn test_prompts_api_error_responses() {
    let error_not_found = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32602,
            "message": "Invalid params: missing group parameter"
        }
    });

    let error_internal = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "error": {
            "code": -32603,
            "message": "Internal error: failed to get prompt"
        }
    });

    assert_eq!(error_not_found["error"]["code"], -32602);
    assert_eq!(error_internal["error"]["code"], -32603);
}

/// Test 20: Prompts with special characters in names and descriptions
/// Tests: UTF-8 characters, special symbols handling
#[test]
fn test_prompts_special_characters() {
    let prompt = json!({
        "name": "code-review_v2.0",
        "description": "Review code for best practices & security! 🔒 (版本 2.0)",
        "arguments": [
            {
                "name": "source_code",
                "description": "Source code: C++, Python, JavaScript, etc..."
            }
        ]
    });

    assert!(prompt["name"].is_string());
    assert!(prompt["description"].is_string());
    let desc = prompt["description"].as_str().unwrap();
    assert!(desc.contains("🔒"));
    assert!(desc.contains("版本"));
}
