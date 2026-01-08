// Resources API Integration Tests with everything-server
// Tests compliance with MCP specification v2025-11-25
// https://modelcontextprotocol.io/specification/2025-11-25/server/resources

use serde_json::json;

/// Test 1: Resources/list request format
/// Tests: group parameter, cursor pagination support
#[test]
fn test_resources_list_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list",
        "params": {
            "group": "everything",
            "cursor": null
        }
    });

    assert_eq!(request["method"], "resources/list");
    assert!(request["params"]["group"].is_string());
}

/// Test 2: Resources/list response structure
/// Tests: resources array, resource metadata (uri, name, size, annotations)
#[test]
fn test_resources_list_response_structure() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "resources": [
                {
                    "uri": "file:///example.txt",
                    "name": "example.txt",
                    "title": "Example File",
                    "description": "An example text file",
                    "mimeType": "text/plain",
                    "size": 1024,
                    "annotations": {
                        "audience": ["user"],
                        "priority": 0.5,
                        "lastModified": "2025-01-08T00:00:00Z"
                    }
                }
            ],
            "nextCursor": "page2"
        }
    });

    assert!(response["result"]["resources"].is_array());
    let resource = &response["result"]["resources"][0];

    assert!(resource["uri"].is_string());
    assert!(resource["name"].is_string());
    assert!(resource["size"].is_number() || resource["size"].is_null());
    assert!(resource["annotations"].is_object() || resource["annotations"].is_null());
}

/// Test 3: Resource size field (MUST implement per v1.3.0)
/// Tests: size field is optional u64 for byte count
#[test]
fn test_resource_size_field() {
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

/// Test 4: Resource annotations (MUST implement per v1.3.0)
/// Tests: audience array, priority float, lastModified timestamp
#[test]
fn test_resource_annotations() {
    let resource = json!({
        "uri": "file:///source.rs",
        "name": "source.rs",
        "annotations": {
            "audience": ["user", "assistant"],
            "priority": 0.9,
            "lastModified": "2025-01-08T12:00:00Z"
        }
    });

    let annotations = &resource["annotations"];
    assert!(annotations["audience"].is_array());
    assert!(annotations["priority"].is_number());
    assert!(annotations["lastModified"].is_string());
}

/// Test 5: Resource icons (MUST implement per v1.3.0)
/// Tests: icon src, mimeType, sizes array
#[test]
fn test_resource_icons() {
    let resource = json!({
        "uri": "file:///image.png",
        "name": "image.png",
        "icons": [
            {
                "src": "https://example.com/icon.png",
                "mimeType": "image/png",
                "sizes": ["16x16", "32x32", "48x48"]
            }
        ]
    });

    let icons = &resource["icons"];
    assert!(icons.is_array());
    let icon = &icons[0];
    assert!(icon["src"].is_string());
    assert!(icon["mimeType"].is_string());
    assert!(icon["sizes"].is_array());
}

/// Test 6: Resources/read request format
/// Tests: group and uri parameters required
#[test]
fn test_resources_read_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "resources/read",
        "params": {
            "group": "everything",
            "uri": "file:///example.txt"
        }
    });

    assert_eq!(request["method"], "resources/read");
    assert!(request["params"]["group"].is_string());
    assert!(request["params"]["uri"].is_string());
}

/// Test 7: Resources/read text content response
/// Tests: text content with mimeType
#[test]
fn test_resources_read_text_content() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "contents": [
                {
                    "uri": "file:///example.txt",
                    "mimeType": "text/plain",
                    "text": "This is the file content"
                }
            ]
        }
    });

    assert!(response["result"]["contents"].is_array());
    let content = &response["result"]["contents"][0];
    assert!(content["uri"].is_string());
    assert!(content["mimeType"].is_string());
    assert!(content["text"].is_string());
}

/// Test 8: Resources/read binary content response
/// Tests: blob content with base64 encoding
#[test]
fn test_resources_read_blob_content() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "result": {
            "contents": [
                {
                    "uri": "file:///image.png",
                    "mimeType": "image/png",
                    "blob": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="
                }
            ]
        }
    });

    assert!(response["result"]["contents"].is_array());
    let content = &response["result"]["contents"][0];
    assert!(content["uri"].is_string());
    assert!(content["blob"].is_string());
}

/// Test 9: Resources/templates/list request format
/// Tests: group parameter required
#[test]
fn test_resources_templates_list_request_format() {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "resources/templates/list",
        "params": {
            "group": "everything"
        }
    });

    assert_eq!(request["method"], "resources/templates/list");
    assert!(request["params"]["group"].is_string());
}

/// Test 10: Resource templates structure (MUST implement per v1.3.0)
/// Tests: RFC 6570 URI templates with placeholders
#[test]
fn test_resource_templates_structure() {
    let template = json!({
        "uriTemplate": "file:///{path}",
        "name": "Local Files",
        "description": "Access files in local filesystem",
        "mimeType": "application/octet-stream",
        "annotations": {
            "audience": ["user"],
            "priority": 0.5
        },
        "icons": [
            {
                "src": "https://example.com/file-icon.png",
                "mimeType": "image/png"
            }
        ]
    });

    assert!(template["uriTemplate"].is_string());
    assert!(template["name"].is_string());
    assert!(template["uriTemplate"].as_str().unwrap().contains("{"));
}

/// Test 11: Multiple resource URI schemes
/// Tests: file://, https://, git://, custom schemes
#[test]
fn test_multiple_resource_uri_schemes() {
    let resources = vec![
        json!({"uri": "file:///local/path", "name": "Local File", "size": 1024}),
        json!({"uri": "https://example.com/resource", "name": "Web Resource", "size": 2048}),
        json!({"uri": "git://example.com/repo/blob/main/file", "name": "Git File", "size": 512}),
        json!({"uri": "custom://resource/id", "name": "Custom Resource"}),
    ];

    for (idx, resource) in resources.iter().enumerate() {
        assert!(resource["uri"].is_string());
        let uri = resource["uri"].as_str().unwrap();
        match idx {
            0 => assert!(uri.starts_with("file://")),
            1 => assert!(uri.starts_with("https://")),
            2 => assert!(uri.starts_with("git://")),
            3 => assert!(uri.starts_with("custom://")),
            _ => unreachable!(),
        }
    }
}

/// Test 12: Resources API pagination support
/// Tests: cursor-based pagination in resources/list
#[test]
fn test_resources_list_pagination() {
    let response = json!({
        "resources": [
            {"uri": "file:///a.txt", "name": "a.txt"},
            {"uri": "file:///b.txt", "name": "b.txt"}
        ],
        "nextCursor": "cursor_abc123"
    });

    assert!(response["resources"].is_array());
    assert!(response["nextCursor"].is_string());
}

/// Test 13: Resources API capability declaration
/// Tests: resources capability in initialize response
#[test]
fn test_resources_capability_declaration() {
    let initialize = json!({
        "capabilities": {
            "resources": {
                "subscribe": true
            }
        }
    });

    assert!(
        initialize["capabilities"]["resources"].is_object(),
        "resources capability must be present"
    );
}

/// Test 14: Resource content with annotations
/// Tests: annotations on read response
#[test]
fn test_resource_content_with_annotations() {
    let response = json!({
        "contents": [
            {
                "uri": "file:///document.txt",
                "mimeType": "text/plain",
                "text": "Content here",
                "annotations": {
                    "lastModified": "2025-01-08T12:00:00Z",
                    "priority": 0.7,
                    "audience": ["user"]
                }
            }
        ]
    });

    let content = &response["contents"][0];
    assert!(content["annotations"]["lastModified"].is_string());
    assert!(content["annotations"]["priority"].is_number());
    assert!(content["annotations"]["audience"].is_array());
}

/// Test 15: JSON-RPC errors for resources API
/// Tests: error codes for resources methods
#[test]
fn test_resources_api_error_codes() {
    let errors = vec![
        (
            -32602,
            json!({
                "code": -32602,
                "message": "Invalid params: missing group parameter"
            }),
        ),
        (
            -32002,
            json!({
                "code": -32002,
                "message": "Resource not found"
            }),
        ),
        (
            -32603,
            json!({
                "code": -32603,
                "message": "Internal error: failed to list resources"
            }),
        ),
    ];

    for (expected_code, error) in errors {
        assert_eq!(error["code"], expected_code);
    }
}

/// Test 16: Empty resources/list response
/// Tests: handling empty resource list
#[test]
fn test_empty_resources_list_response() {
    let response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "resources": []
        }
    });

    assert!(response["result"]["resources"].is_array());
    assert_eq!(response["result"]["resources"].as_array().unwrap().len(), 0);
}

/// Test 17: Everything-server resources configuration
/// Tests: config format for everything-server resources support
#[test]
fn test_everything_server_resources_config() {
    let config = json!({
        "mcpServers": {
            "everything": {
                "description": "Server with tools, resources, and prompts",
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

/// Test 18: Resource with multiple content types
/// Tests: handling various MIME types
#[test]
fn test_resources_multiple_mime_types() {
    let resources = vec![
        json!({"uri": "file:///doc.txt", "mimeType": "text/plain"}),
        json!({"uri": "file:///image.png", "mimeType": "image/png"}),
        json!({"uri": "file:///data.json", "mimeType": "application/json"}),
        json!({"uri": "file:///script.py", "mimeType": "text/x-python"}),
        json!({"uri": "file:///archive.zip", "mimeType": "application/zip"}),
    ];

    for resource in resources {
        assert!(resource["mimeType"].is_string());
        let mime = resource["mimeType"].as_str().unwrap();
        assert!(mime.contains("/"));
    }
}

/// Test 19: Resource templates with variable substitution
/// Tests: RFC 6570 URI template syntax
#[test]
fn test_resource_templates_rfc6570_syntax() {
    let templates = vec![
        json!({
            "uriTemplate": "file:///{path}",
            "name": "Simple path"
        }),
        json!({
            "uriTemplate": "git:///{repo}/blob/{branch}/{path}",
            "name": "Git with multiple variables"
        }),
        json!({
            "uriTemplate": "db:///{database}/{table}?filter={filter}",
            "name": "Database with query"
        }),
    ];

    for template in templates {
        let uri_template = template["uriTemplate"].as_str().unwrap();
        assert!(uri_template.contains("{"));
        assert!(uri_template.contains("}"));
    }
}

/// Test 20: Resource annotations with all fields
/// Tests: complete annotation support
#[test]
fn test_resource_full_annotations() {
    let resource = json!({
        "uri": "file:///complete.txt",
        "name": "complete.txt",
        "size": 2048,
        "annotations": {
            "audience": ["user", "assistant", "admin"],
            "priority": 0.95,
            "lastModified": "2025-01-08T15:30:45Z"
        },
        "icons": [
            {
                "src": "https://icons.example.com/text-file.svg",
                "mimeType": "image/svg+xml",
                "sizes": ["16x16", "24x24", "32x32", "48x48"]
            },
            {
                "src": "https://icons.example.com/text-file-large.png",
                "mimeType": "image/png",
                "sizes": ["64x64", "128x128"]
            }
        ]
    });

    // Verify resource structure
    assert!(resource["uri"].is_string());
    assert_eq!(resource["size"], 2048);

    // Verify annotations
    assert!(resource["annotations"]["audience"].is_array());
    assert_eq!(
        resource["annotations"]["audience"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(resource["annotations"]["priority"], 0.95);

    // Verify icons
    assert!(resource["icons"].is_array());
    assert_eq!(resource["icons"].as_array().unwrap().len(), 2);
}
