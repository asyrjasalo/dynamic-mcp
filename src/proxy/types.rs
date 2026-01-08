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
            icons: None,
            annotations: None,
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["uri"], "file:///test.txt");
        assert_eq!(json["name"], "test.txt");
        assert_eq!(json["title"], "Test File");
        assert_eq!(json["mimeType"], "text/plain");
    }

    #[test]
    fn test_resource_optional_fields_omitted() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "test.txt".to_string(),
            title: None,
            description: None,
            mime_type: None,
            icons: None,
            annotations: None,
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert!(json["title"].is_null());
        assert!(json["description"].is_null());
        assert!(json["mimeType"].is_null());
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
}
