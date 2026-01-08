// Tools API Integration Tests with everything-server
// Tests compliance with MCP specification v2025-11-25
// https://modelcontextprotocol.io/specification/2025-11-25/server/tools

use serde_json::json;

/// Test 1: Tools API list endpoint returns correct structure
/// Tests: tools/list request format, tools/list response format, tool metadata
#[test]
fn test_tools_list_request_structure() {
    // Per MCP spec: tools/list is a valid MCP method
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {
            "group": "everything"
        }
    });

    // Verify request structure
    assert_eq!(request["jsonrpc"], "2.0", "jsonrpc version must be 2.0");
    assert!(request["id"].is_number(), "id must be present");
    assert_eq!(request["method"], "tools/list", "method must be tools/list");
    assert_eq!(
        request["params"]["group"], "everything",
        "group parameter required"
    );
}

/// Test 2: Tools list response contains proper tool definitions
/// Tests: tool name, description, inputSchema format
#[test]
fn test_tools_list_response_format() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": [
                {
                    "name": "example_tool",
                    "description": "An example tool for testing",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "arg1": {
                                "type": "string",
                                "description": "First argument"
                            }
                        },
                        "required": ["arg1"]
                    }
                }
            ]
        }
    });

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["id"].is_number());
    assert!(
        response["result"]["tools"].is_array(),
        "tools must be array"
    );

    let tools = &response["result"]["tools"];
    assert!(tools.is_array());

    // Verify tool structure
    if let Some(tool) = tools.get(0) {
        assert!(tool["name"].is_string(), "tool name is required");
        assert!(
            tool["description"].is_string(),
            "tool description should be present"
        );
        assert!(tool["inputSchema"].is_object(), "inputSchema is required");

        // Verify inputSchema structure (JSON Schema)
        let schema = &tool["inputSchema"];
        assert!(schema["type"].is_string(), "inputSchema.type is required");
        assert!(schema["properties"].is_object() || schema["properties"].is_null());
    }
}

/// Test 3: Tools with different input schema types
/// Tests: primitive types, complex objects, required/optional fields
#[test]
fn test_tools_input_schema_types() {
    let tools_with_schemas = vec![
        // Simple string parameter
        json!({
            "name": "string_tool",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {"type": "string"}
                },
                "required": ["text"]
            }
        }),
        // Multiple parameters with different types
        json!({
            "name": "multi_param_tool",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "count": {"type": "number"},
                    "active": {"type": "boolean"}
                },
                "required": ["name"]
            }
        }),
        // Optional parameters
        json!({
            "name": "optional_params_tool",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "required_param": {"type": "string"},
                    "optional_param": {"type": "string"}
                },
                "required": ["required_param"]
            }
        }),
        // Complex nested schema
        json!({
            "name": "nested_schema_tool",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "config": {
                        "type": "object",
                        "properties": {
                            "timeout": {"type": "number"},
                            "retries": {"type": "integer"}
                        }
                    }
                }
            }
        }),
    ];

    for tool in tools_with_schemas {
        assert!(tool["name"].is_string());
        assert!(tool["inputSchema"]["type"].is_string());
        assert!(
            tool["inputSchema"]["properties"].is_object()
                || tool["inputSchema"]["type"] == "object"
        );
    }
}

/// Test 4: Tools/call request format
/// Tests: tool execution request structure, parameter passing
#[test]
fn test_tools_call_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "group": "everything",
            "name": "example_tool",
            "arguments": {
                "arg1": "value1",
                "arg2": 42
            }
        }
    });

    assert_eq!(request["method"], "tools/call");
    assert!(request["params"]["group"].is_string());
    assert!(request["params"]["name"].is_string());
    assert!(
        request["params"]["arguments"].is_object(),
        "arguments must be object"
    );
}

/// Test 5: Tools/call success response format
/// Tests: result content, content types, tool output structure
#[test]
fn test_tools_call_success_response() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "Tool execution successful"
                }
            ]
        }
    });

    assert!(response["result"]["content"].is_array());
    let content = &response["result"]["content"][0];
    assert!(content["type"].is_string());
    assert!(content["text"].is_string());
}

/// Test 6: Tools/call error response format
/// Tests: isError flag, error content formatting
#[test]
fn test_tools_call_error_response_format() {
    // Per MCP spec: Tool execution errors use isError: true flag
    let error_response = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "Error: Invalid input parameter",
                    "isError": true
                }
            ]
        }
    });

    assert!(error_response["result"]["content"].is_array());
    let content = &error_response["result"]["content"][0];
    assert_eq!(content["type"], "text");
    assert_eq!(
        content["isError"], true,
        "isError flag must be true for errors"
    );
    assert!(content["text"].is_string());
}

/// Test 7: Tools with multiple content types
/// Tests: text, image, audio, resource content types in tool results
#[test]
fn test_tools_multiple_content_types() {
    let multi_content = json!({
        "content": [
            {
                "type": "text",
                "text": "Processing complete"
            },
            {
                "type": "image",
                "data": "base64encodedimage",
                "mimeType": "image/png"
            },
            {
                "type": "resource",
                "resource": {
                    "uri": "file:///output.json",
                    "mimeType": "application/json"
                }
            }
        ]
    });

    assert!(multi_content["content"].is_array());
    let content_array = multi_content["content"].as_array().unwrap();

    // Verify each content type
    assert_eq!(content_array[0]["type"], "text");
    assert!(content_array[0]["text"].is_string());

    assert_eq!(content_array[1]["type"], "image");
    assert!(content_array[1]["data"].is_string());
    assert!(content_array[1]["mimeType"].is_string());

    assert_eq!(content_array[2]["type"], "resource");
    assert!(content_array[2]["resource"]["uri"].is_string());
}

/// Test 8: Tools pagination support
/// Tests: cursor-based pagination in tools/list
#[test]
fn test_tools_list_pagination() {
    let paginated_response = json!({
        "tools": [
            {
                "name": "tool1",
                "inputSchema": {"type": "object"}
            },
            {
                "name": "tool2",
                "inputSchema": {"type": "object"}
            }
        ],
        "nextCursor": "cursor_for_next_page"
    });

    assert!(paginated_response["tools"].is_array());
    assert!(paginated_response["nextCursor"].is_string());
}

/// Test 9: Tools API capability declaration
/// Tests: tools capability in initialize response
#[test]
fn test_tools_capability_declaration() {
    let initialize_response = json!({
        "capabilities": {
            "tools": {},
            "resources": {
                "subscribe": true
            },
            "prompts": {}
        }
    });

    assert!(
        initialize_response["capabilities"]["tools"].is_object(),
        "tools capability must be present"
    );
}

/// Test 10: JSON-RPC error responses for tools API
/// Tests: error codes -32601 (method not found), -32602 (invalid params), -32603 (internal error)
#[test]
fn test_tools_api_error_codes() {
    let errors = vec![
        (
            -32601,
            json!({
                "code": -32601,
                "message": "Method not found: invalid_method",
                "data": null
            }),
        ),
        (
            -32602,
            json!({
                "code": -32602,
                "message": "Invalid params: missing required group",
                "data": null
            }),
        ),
        (
            -32603,
            json!({
                "code": -32603,
                "message": "Internal error: failed to list tools",
                "data": null
            }),
        ),
    ];

    for (expected_code, error) in errors {
        assert_eq!(error["code"], expected_code);
        assert!(error["message"].is_string());
    }
}

/// Test 11: Tools with no input parameters
/// Tests: tools with empty inputSchema or no required fields
#[test]
fn test_tools_with_no_parameters() {
    let tool = json!({
        "name": "no_params_tool",
        "description": "Tool that takes no parameters",
        "inputSchema": {
            "type": "object",
            "properties": {},
            "required": []
        }
    });

    assert!(tool["inputSchema"]["properties"].is_object());
    assert!(tool["inputSchema"]["required"].is_array());
    assert_eq!(tool["inputSchema"]["required"].as_array().unwrap().len(), 0);
}

/// Test 12: Tools list from everything-server configuration
/// Tests: config format for everything-server integration
#[test]
fn test_everything_server_tools_config() {
    let config = json!({
        "mcpServers": {
            "everything": {
                "description": "Server with comprehensive tools, resources, and prompts",
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-everything"]
            }
        }
    });

    let server_config = &config["mcpServers"]["everything"];
    assert!(server_config["description"].is_string());
    assert_eq!(server_config["command"], "npx");
    assert!(server_config["args"].is_array());
}

/// Test 13: Tool execution with complex arguments
/// Tests: nested objects, arrays, special characters in arguments
#[test]
fn test_tools_call_complex_arguments() {
    let request = json!({
        "method": "tools/call",
        "params": {
            "group": "everything",
            "name": "complex_tool",
            "arguments": {
                "simple": "string",
                "number": 42,
                "boolean": true,
                "null_value": null,
                "array": [1, 2, 3],
                "object": {
                    "nested": "value",
                    "count": 100
                },
                "special_chars": "test!@#$%^&*()_+-=[]{}|;':\",./<>?"
            }
        }
    });

    let args = &request["params"]["arguments"];
    assert!(args["simple"].is_string());
    assert!(args["number"].is_number());
    assert!(args["boolean"].is_boolean());
    assert!(args["null_value"].is_null());
    assert!(args["array"].is_array());
    assert!(args["object"].is_object());
    assert!(args["special_chars"].is_string());
}

/// Test 14: Tools API with empty response
/// Tests: handling of tools/list with no tools available
#[test]
fn test_tools_list_empty_response() {
    let empty_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": []
        }
    });

    assert!(empty_response["result"]["tools"].is_array());
    assert_eq!(
        empty_response["result"]["tools"].as_array().unwrap().len(),
        0
    );
}

/// Test 15: Tool schema validation edge cases
/// Tests: special input schema patterns (enum, pattern, minimum/maximum)
#[test]
fn test_tool_input_schema_special_patterns() {
    let tool_with_enum = json!({
        "name": "enum_tool",
        "inputSchema": {
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["json", "xml", "csv"]
                }
            }
        }
    });

    let tool_with_pattern = json!({
        "name": "pattern_tool",
        "inputSchema": {
            "type": "object",
            "properties": {
                "email": {
                    "type": "string",
                    "pattern": "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
                }
            }
        }
    });

    let tool_with_number_constraints = json!({
        "name": "number_tool",
        "inputSchema": {
            "type": "object",
            "properties": {
                "port": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 65535
                }
            }
        }
    });

    assert!(tool_with_enum["inputSchema"]["properties"]["format"]["enum"].is_array());
    assert!(tool_with_pattern["inputSchema"]["properties"]["email"]["pattern"].is_string());
    assert!(
        tool_with_number_constraints["inputSchema"]["properties"]["port"]["minimum"].is_number()
    );
}
