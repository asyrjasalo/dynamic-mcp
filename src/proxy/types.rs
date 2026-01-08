use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedGroupInfo {
    pub name: String,
    pub description: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(default = "default_null_id", skip_serializing_if = "is_null")]
    pub id: serde_json::Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

fn default_null_id() -> serde_json::Value {
    serde_json::Value::Null
}

fn is_null(value: &serde_json::Value) -> bool {
    matches!(value, serde_json::Value::Null)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Batch(Vec<JsonRpcRequest>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcNotification {
    pub fn resources_list_changed() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "notifications/resources/list_changed".to_string(),
            params: None,
        }
    }

    pub fn prompts_list_changed() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "notifications/prompts/list_changed".to_string(),
            params: None,
        }
    }

    pub fn resources_updated(uri: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: "notifications/resources/updated".to_string(),
            params: Some(serde_json::json!({ "uri": uri })),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressToken {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ResourceAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ResourceIcon {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<ResourceIcon>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct StreamingBinaryContent {
    pub uri: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "byteLength")]
    pub byte_length: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "chunkSize")]
    pub chunk_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ResourceTemplate {
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<ResourceIcon>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum PromptContentType {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "audio")]
    Audio {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PromptMessage {
    pub role: String,
    pub content: PromptContentType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Prompt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<ResourceIcon>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PromptContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

impl JsonRpcRequest {
    pub fn new(id: impl Into<serde_json::Value>, method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params: None,
        }
    }

    pub fn with_params(mut self, params: serde_json::Value) -> Self {
        self.params = Some(params);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "test.txt".to_string(),
            title: Some("Test File".to_string()),
            description: None,
            mime_type: Some("text/plain".to_string()),
            size: Some(1024),
            icons: None,
            annotations: None,
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["uri"], "file:///test.txt");
        assert_eq!(json["name"], "test.txt");
        assert_eq!(json["title"], "Test File");
        assert_eq!(json["mimeType"], "text/plain");
        assert_eq!(json["size"], 1024);
    }

    #[test]
    fn test_resource_with_size() {
        let resource = Resource {
            uri: "file:///large.bin".to_string(),
            name: "large.bin".to_string(),
            title: None,
            description: Some("A large binary file".to_string()),
            mime_type: Some("application/octet-stream".to_string()),
            size: Some(5_242_880), // 5 MB
            icons: None,
            annotations: None,
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["size"], 5_242_880);
    }

    #[test]
    fn test_resource_optional_fields_omitted() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "test.txt".to_string(),
            title: None,
            description: None,
            mime_type: None,
            size: None,
            icons: None,
            annotations: None,
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert!(json["title"].is_null());
        assert!(json["description"].is_null());
        assert!(json["mimeType"].is_null());
        assert!(json["size"].is_null());
    }

    #[test]
    fn test_resource_content_with_text() {
        let content = ResourceContent {
            uri: "file:///test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("Hello, world!".to_string()),
            blob: None,
            annotations: None,
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["uri"], "file:///test.txt");
        assert_eq!(json["text"], "Hello, world!");
        assert!(json["blob"].is_null());
    }

    #[test]
    fn test_resource_content_with_blob() {
        let content = ResourceContent {
            uri: "file:///test.png".to_string(),
            mime_type: Some("image/png".to_string()),
            text: None,
            blob: Some("base64encodeddata".to_string()),
            annotations: None,
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["blob"], "base64encodeddata");
        assert!(json["text"].is_null());
    }

    #[test]
    fn test_resource_annotations() {
        let annotations = ResourceAnnotations {
            audience: Some(vec!["user".to_string(), "assistant".to_string()]),
            priority: Some(0.8),
            last_modified: Some("2025-01-12T15:00:58Z".to_string()),
        };

        let json = serde_json::to_value(&annotations).unwrap();
        assert_eq!(json["priority"], 0.8);
        assert_eq!(json["lastModified"], "2025-01-12T15:00:58Z");
    }

    #[test]
    fn test_resource_icon() {
        let icon = ResourceIcon {
            src: "https://example.com/icon.png".to_string(),
            mime_type: Some("image/png".to_string()),
            sizes: Some(vec!["48x48".to_string(), "64x64".to_string()]),
        };

        let json = serde_json::to_value(&icon).unwrap();
        assert_eq!(json["src"], "https://example.com/icon.png");
        assert_eq!(json["sizes"][0], "48x48");
    }

    #[test]
    fn test_resource_template_serialization() {
        let template = ResourceTemplate {
            uri_template: "file:///{path}".to_string(),
            name: "Project Files".to_string(),
            description: Some("Access files in the project directory".to_string()),
            mime_type: Some("application/octet-stream".to_string()),
            annotations: Some(ResourceAnnotations {
                audience: Some(vec!["user".to_string()]),
                priority: Some(0.8),
                last_modified: None,
            }),
            icons: None,
        };

        let json = serde_json::to_value(&template).unwrap();
        assert_eq!(json["uriTemplate"], "file:///{path}");
        assert_eq!(json["name"], "Project Files");
        assert_eq!(json["description"], "Access files in the project directory");
        assert_eq!(json["mimeType"], "application/octet-stream");
        assert_eq!(json["annotations"]["priority"], 0.8);
    }

    #[test]
    fn test_resource_template_minimal() {
        let template = ResourceTemplate {
            uri_template: "git:///{repo}/blob/{ref}/{path}".to_string(),
            name: "Git Files".to_string(),
            description: None,
            mime_type: None,
            annotations: None,
            icons: None,
        };

        let json = serde_json::to_value(&template).unwrap();
        assert_eq!(json["uriTemplate"], "git:///{repo}/blob/{ref}/{path}");
        assert_eq!(json["name"], "Git Files");
        assert!(json["description"].is_null());
        assert!(json["mimeType"].is_null());
        assert!(json["annotations"].is_null());
    }

    #[test]
    fn test_prompt_argument_serialization() {
        let arg = PromptArgument {
            name: "code".to_string(),
            description: Some("The code to review".to_string()),
            required: true,
        };

        let json = serde_json::to_value(&arg).unwrap();
        assert_eq!(json["name"], "code");
        assert_eq!(json["description"], "The code to review");
        assert_eq!(json["required"], true);
    }

    #[test]
    fn test_prompt_argument_optional_description() {
        let arg = PromptArgument {
            name: "code".to_string(),
            description: None,
            required: false,
        };

        let json = serde_json::to_value(&arg).unwrap();
        assert_eq!(json["name"], "code");
        assert!(json["description"].is_null());
        assert_eq!(json["required"], false);
    }

    #[test]
    fn test_prompt_content_type_text() {
        let content = PromptContentType::Text {
            text: "Hello, world!".to_string(),
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello, world!");
    }

    #[test]
    fn test_prompt_content_type_image() {
        let content = PromptContentType::Image {
            data: "base64imagedata".to_string(),
            mime_type: "image/png".to_string(),
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["mimeType"], "image/png");
        assert_eq!(json["data"], "base64imagedata");
    }

    #[test]
    fn test_prompt_content_type_audio() {
        let content = PromptContentType::Audio {
            data: "base64audiodata".to_string(),
            mime_type: "audio/wav".to_string(),
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "audio");
        assert_eq!(json["mimeType"], "audio/wav");
    }

    #[test]
    fn test_prompt_message() {
        let message = PromptMessage {
            role: "user".to_string(),
            content: PromptContentType::Text {
                text: "Please review this code".to_string(),
            },
            annotations: None,
        };

        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"]["type"], "text");
        assert_eq!(json["content"]["text"], "Please review this code");
    }

    #[test]
    fn test_prompt_basic() {
        let prompt = Prompt {
            name: "code_review".to_string(),
            title: Some("Request Code Review".to_string()),
            description: Some("Asks the LLM to analyze code quality".to_string()),
            arguments: Some(vec![PromptArgument {
                name: "code".to_string(),
                description: Some("The code to review".to_string()),
                required: true,
            }]),
            icons: None,
        };

        let json = serde_json::to_value(&prompt).unwrap();
        assert_eq!(json["name"], "code_review");
        assert_eq!(json["title"], "Request Code Review");
        assert_eq!(json["arguments"][0]["name"], "code");
        assert_eq!(json["arguments"][0]["required"], true);
    }

    #[test]
    fn test_prompt_content() {
        let content = PromptContent {
            description: Some("Code review prompt".to_string()),
            messages: vec![PromptMessage {
                role: "user".to_string(),
                content: PromptContentType::Text {
                    text: "Please review this Python code".to_string(),
                },
                annotations: None,
            }],
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["description"], "Code review prompt");
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"]["type"], "text");
    }
}
