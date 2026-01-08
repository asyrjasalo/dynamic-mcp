use serde_json::json;

#[test]
fn test_resource_size_field_present() {
    let resource_json = json!({
        "uri": "file:///large.bin",
        "name": "large.bin",
        "description": "A large binary file",
        "mimeType": "application/octet-stream",
        "size": 5242880
    });

    assert_eq!(resource_json["size"], 5242880);
    assert_eq!(resource_json["name"], "large.bin");
}

#[test]
fn test_resource_without_size_field() {
    let resource_json = json!({
        "uri": "file:///test.txt",
        "name": "test.txt",
        "mimeType": "text/plain"
    });

    assert!(resource_json["size"].is_null());
    assert_eq!(resource_json["name"], "test.txt");
}

#[test]
fn test_resource_template_structure() {
    let template_json = json!({
        "uriTemplate": "file:///{path}",
        "name": "Project Files",
        "description": "Access files in the project directory",
        "mimeType": "application/octet-stream"
    });

    assert_eq!(template_json["uriTemplate"], "file:///{path}");
    assert_eq!(template_json["name"], "Project Files");
    assert_eq!(
        template_json["description"],
        "Access files in the project directory"
    );
    assert_eq!(template_json["mimeType"], "application/octet-stream");
}

#[test]
fn test_resource_template_minimal() {
    let template_json = json!({
        "uriTemplate": "git:///{repo}/blob/{ref}/{path}",
        "name": "Git Files"
    });

    assert_eq!(
        template_json["uriTemplate"],
        "git:///{repo}/blob/{ref}/{path}"
    );
    assert_eq!(template_json["name"], "Git Files");
    assert!(template_json["description"].is_null());
}

#[test]
fn test_resources_list_response_format_with_size() {
    let list_response = json!({
        "resources": [
            {
                "uri": "file:///project/src/main.rs",
                "name": "main.rs",
                "description": "Entry point",
                "mimeType": "text/x-rust",
                "size": 2048,
                "annotations": {
                    "audience": ["user", "assistant"],
                    "priority": 0.8
                }
            },
            {
                "uri": "file:///project/data.bin",
                "name": "data.bin",
                "size": 1048576,
                "mimeType": "application/octet-stream"
            }
        ],
        "nextCursor": "next-page"
    });

    assert!(list_response["resources"].is_array());
    assert_eq!(list_response["resources"][0]["size"], 2048);
    assert_eq!(list_response["resources"][1]["size"], 1048576);
    assert_eq!(list_response["nextCursor"], "next-page");
}

#[test]
fn test_resources_templates_list_response_format() {
    let templates_response = json!({
        "resourceTemplates": [
            {
                "uriTemplate": "file:///{path}",
                "name": "Local Files",
                "description": "Access files in local filesystem",
                "mimeType": "application/octet-stream"
            },
            {
                "uriTemplate": "git:///{repo}/blob/{ref}/{path}",
                "name": "Git Repository Files",
                "description": "Access files in git repositories"
            }
        ]
    });

    assert!(templates_response["resourceTemplates"].is_array());
    assert_eq!(
        templates_response["resourceTemplates"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        templates_response["resourceTemplates"][0]["uriTemplate"],
        "file:///{path}"
    );
    assert_eq!(
        templates_response["resourceTemplates"][1]["name"],
        "Git Repository Files"
    );
}

#[test]
fn test_resource_with_icons_and_size() {
    let resource_json = json!({
        "uri": "file:///image.png",
        "name": "image.png",
        "mimeType": "image/png",
        "size": 512000,
        "icons": [
            {
                "src": "https://example.com/icon-48.png",
                "mimeType": "image/png",
                "sizes": ["48x48"]
            }
        ]
    });

    assert_eq!(resource_json["size"], 512000);
    assert!(resource_json["icons"].is_array());
    assert_eq!(
        resource_json["icons"][0]["src"],
        "https://example.com/icon-48.png"
    );
}

#[test]
fn test_resource_read_response_with_annotations() {
    let read_response = json!({
        "contents": [
            {
                "uri": "file:///source.rs",
                "mimeType": "text/x-rust",
                "text": "fn main() { println!(\"Hello\"); }",
                "annotations": {
                    "lastModified": "2025-01-08T20:00:00Z",
                    "priority": 0.9
                }
            }
        ]
    });

    assert!(read_response["contents"].is_array());
    assert_eq!(read_response["contents"][0]["uri"], "file:///source.rs");
    assert_eq!(
        read_response["contents"][0]["annotations"]["lastModified"],
        "2025-01-08T20:00:00Z"
    );
}

#[test]
fn test_different_resource_uri_schemes() {
    let resources = vec![
        json!({"uri": "file:///local/path", "name": "Local File", "size": 1024}),
        json!({"uri": "https://example.com/resource", "name": "Web Resource", "size": 2048}),
        json!({"uri": "git://example.com/repo/blob/main/file.txt", "name": "Git File", "size": 512}),
    ];

    for (idx, resource) in resources.iter().enumerate() {
        assert!(resource["uri"].is_string());
        assert!(resource["size"].is_number());
        match idx {
            0 => assert!(resource["uri"].as_str().unwrap().starts_with("file://")),
            1 => assert!(resource["uri"].as_str().unwrap().starts_with("https://")),
            2 => assert!(resource["uri"].as_str().unwrap().starts_with("git://")),
            _ => unreachable!(),
        }
    }
}
