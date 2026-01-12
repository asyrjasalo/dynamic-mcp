use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Per-server feature flags (opt-out design: all features enabled by default)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Features {
    #[serde(default = "default_true")]
    pub tools: bool,
    #[serde(default = "default_true")]
    pub resources: bool,
    #[serde(default = "default_true")]
    pub prompts: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Features {
    fn default() -> Self {
        Self {
            tools: true,
            resources: true,
            prompts: true,
        }
    }
}

impl Features {
    /// Returns true if all features are enabled (default state)
    pub fn is_default(&self) -> bool {
        self.tools && self.resources && self.prompts
    }
}

fn default_true_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio {
        description: String,
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<HashMap<String, String>>,
        #[serde(default, skip_serializing_if = "Features::is_default")]
        features: Features,
        #[serde(default = "default_true_enabled", skip_serializing_if = "is_true")]
        enabled: bool,
    },
    Http {
        description: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        oauth_client_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        oauth_scopes: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Features::is_default")]
        features: Features,
        #[serde(default = "default_true_enabled", skip_serializing_if = "is_true")]
        enabled: bool,
    },
    Sse {
        description: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        oauth_client_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        oauth_scopes: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Features::is_default")]
        features: Features,
        #[serde(default = "default_true_enabled", skip_serializing_if = "is_true")]
        enabled: bool,
    },
}

fn is_true(value: &bool) -> bool {
    *value
}

impl<'de> Deserialize<'de> for McpServerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut value = serde_json::Value::deserialize(deserializer)?;

        if let Some(obj) = value.as_object_mut() {
            if !obj.contains_key("type") {
                if obj.contains_key("url") {
                    // Default to "http" when url is present but type is not specified
                    // The transport layer will auto-detect SSE responses per MCP spec
                    obj.insert(
                        "type".to_string(),
                        serde_json::Value::String("http".to_string()),
                    );
                } else {
                    obj.insert(
                        "type".to_string(),
                        serde_json::Value::String("stdio".to_string()),
                    );
                }
            }
        }

        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
        enum McpServerConfigHelper {
            #[serde(rename = "stdio")]
            Stdio {
                description: String,
                command: String,
                args: Option<Vec<String>>,
                env: Option<HashMap<String, String>>,
                #[serde(default)]
                features: Features,
                #[serde(default = "default_true_enabled")]
                enabled: bool,
            },
            Http {
                description: String,
                url: String,
                headers: Option<HashMap<String, String>>,
                oauth_client_id: Option<String>,
                oauth_scopes: Option<Vec<String>>,
                #[serde(default)]
                features: Features,
                #[serde(default = "default_true_enabled")]
                enabled: bool,
            },
            Sse {
                description: String,
                url: String,
                headers: Option<HashMap<String, String>>,
                oauth_client_id: Option<String>,
                oauth_scopes: Option<Vec<String>>,
                #[serde(default)]
                features: Features,
                #[serde(default = "default_true_enabled")]
                enabled: bool,
            },
        }

        match serde_json::from_value::<McpServerConfigHelper>(value)
            .map_err(serde::de::Error::custom)?
        {
            McpServerConfigHelper::Stdio {
                description,
                command,
                args,
                env,
                features,
                enabled,
            } => Ok(McpServerConfig::Stdio {
                description,
                command,
                args,
                env,
                features,
                enabled,
            }),
            McpServerConfigHelper::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
            } => Ok(McpServerConfig::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
            }),
            McpServerConfigHelper::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
            } => Ok(McpServerConfig::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
            }),
        }
    }
}

impl McpServerConfig {
    pub fn description(&self) -> &str {
        match self {
            McpServerConfig::Stdio { description, .. } => description,
            McpServerConfig::Http { description, .. } => description,
            McpServerConfig::Sse { description, .. } => description,
        }
    }

    pub fn features(&self) -> &Features {
        match self {
            McpServerConfig::Stdio { features, .. } => features,
            McpServerConfig::Http { features, .. } => features,
            McpServerConfig::Sse { features, .. } => features,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            McpServerConfig::Stdio { enabled, .. } => *enabled,
            McpServerConfig::Http { enabled, .. } => *enabled,
            McpServerConfig::Sse { enabled, .. } => *enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "$schema")]
    pub schema: Option<String>,
}

/// Intermediate representation for migration from various tools
/// Normalized format that can be converted to McpServerConfig
#[derive(Debug, Clone)]
pub struct IntermediateServerConfig {
    /// Command executable (for stdio servers)
    pub command: Option<String>,
    /// Command arguments
    pub args: Option<Vec<String>>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Server URL (for http/sse servers)
    pub url: Option<String>,
    /// HTTP headers (for http/sse servers)
    pub headers: Option<HashMap<String, String>>,
    /// Server type hint
    pub server_type: Option<String>,
    /// Whether the server is enabled (defaults to true if not specified)
    pub enabled: Option<bool>,
}

impl IntermediateServerConfig {
    /// Convert to McpServerConfig with a description
    #[allow(clippy::wrong_self_convention)]
    pub fn to_mcp_config(self, description: String) -> Result<McpServerConfig, String> {
        let enabled = self.enabled.unwrap_or(true);

        if let Some(url) = self.url {
            let server_type = self.server_type.as_deref().unwrap_or("http").to_lowercase();

            if server_type == "sse" {
                Ok(McpServerConfig::Sse {
                    description,
                    url,
                    headers: self.headers,
                    oauth_client_id: None,
                    oauth_scopes: None,
                    features: Features::default(),
                    enabled,
                })
            } else {
                Ok(McpServerConfig::Http {
                    description,
                    url,
                    headers: self.headers,
                    oauth_client_id: None,
                    oauth_scopes: None,
                    features: Features::default(),
                    enabled,
                })
            }
        } else if let Some(command) = self.command {
            Ok(McpServerConfig::Stdio {
                description,
                command,
                args: self.args,
                env: self.env,
                features: Features::default(),
                enabled,
            })
        } else {
            Err("Server config must have either 'command' or 'url'".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_features_default_all_enabled() {
        let features = Features::default();
        assert!(features.tools);
        assert!(features.resources);
        assert!(features.prompts);
    }

    #[test]
    fn test_features_deserialize_empty_object() {
        let json = json!({});
        let features: Features = serde_json::from_value(json).unwrap();
        assert!(features.tools);
        assert!(features.resources);
        assert!(features.prompts);
    }

    #[test]
    fn test_features_deserialize_disable_resources() {
        let json = json!({
            "resources": false
        });
        let features: Features = serde_json::from_value(json).unwrap();
        assert!(features.tools);
        assert!(!features.resources);
        assert!(features.prompts);
    }

    #[test]
    fn test_features_deserialize_disable_prompts() {
        let json = json!({
            "prompts": false
        });
        let features: Features = serde_json::from_value(json).unwrap();
        assert!(features.tools);
        assert!(features.resources);
        assert!(!features.prompts);
    }

    #[test]
    fn test_features_deserialize_disable_all() {
        let json = json!({
            "tools": false,
            "resources": false,
            "prompts": false
        });
        let features: Features = serde_json::from_value(json).unwrap();
        assert!(!features.tools);
        assert!(!features.resources);
        assert!(!features.prompts);
    }

    #[test]
    fn test_features_deserialize_explicit_enable() {
        let json = json!({
            "tools": true,
            "resources": false,
            "prompts": true
        });
        let features: Features = serde_json::from_value(json).unwrap();
        assert!(features.tools);
        assert!(!features.resources);
        assert!(features.prompts);
    }

    #[test]
    fn test_server_config_with_features() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd",
            "features": {
                "resources": false,
                "prompts": false
            }
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Stdio { features, .. } => {
                assert!(features.tools);
                assert!(!features.resources);
                assert!(!features.prompts);
            }
            _ => panic!("Expected Stdio config"),
        }
    }

    #[test]
    fn test_server_config_without_features() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd"
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Stdio { features, .. } => {
                assert!(features.tools);
                assert!(features.resources);
                assert!(features.prompts);
            }
            _ => panic!("Expected Stdio config"),
        }
    }

    #[test]
    fn test_http_server_config_with_features() {
        let json = json!({
            "type": "http",
            "description": "Test HTTP server",
            "url": "http://localhost:8080",
            "features": {
                "tools": false
            }
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Http { features, .. } => {
                assert!(!features.tools);
                assert!(features.resources);
                assert!(features.prompts);
            }
            _ => panic!("Expected Http config"),
        }
    }

    #[test]
    fn test_serialize_omits_default_features() {
        // Test that features field is omitted when all features are enabled (default)
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: true,
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

        // features should NOT be present when all are enabled (default)
        assert!(!obj.contains_key("features"));
    }

    #[test]
    fn test_serialize_includes_disabled_features() {
        // Test that features field IS included when some features are disabled
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features {
                tools: true,
                resources: false,
                prompts: true,
            },
            enabled: true,
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

        // features SHOULD be present when some are disabled
        assert!(obj.contains_key("features"));

        let features = obj.get("features").unwrap().as_object().unwrap();
        assert!(features.get("tools").unwrap().as_bool().unwrap());
        assert!(!features.get("resources").unwrap().as_bool().unwrap());
        assert!(features.get("prompts").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_stdio_server_rejects_unknown_field() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd",
            "unknown_field": "should fail"
        });
        let result: Result<McpServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn test_stdio_server_rejects_multiple_unknown_fields() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd",
            "typo_field": "error",
            "invalid_field": "also error"
        });
        let result: Result<McpServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_server_rejects_unknown_field() {
        let json = json!({
            "type": "http",
            "description": "Test HTTP server",
            "url": "http://localhost:8080",
            "unknown_field": "should fail"
        });
        let result: Result<McpServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn test_sse_server_rejects_unknown_field() {
        let json = json!({
            "type": "sse",
            "description": "Test SSE server",
            "url": "http://localhost:8080",
            "typo_field": "should fail"
        });
        let result: Result<McpServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn test_features_rejects_unknown_field() {
        let json = json!({
            "tools": true,
            "unknown_feature": true
        });
        let result: Result<Features, _> = serde_json::from_value(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn test_server_config_rejects_unknown_top_level_field() {
        let json = json!({
            "mcpServers": {
                "test": {
                    "description": "Test server",
                    "command": "test-cmd"
                }
            },
            "unknown_top_level": "should fail"
        });
        let result: Result<ServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn test_stdio_server_accepts_all_valid_fields() {
        let json = json!({
            "type": "stdio",
            "description": "Valid server",
            "command": "test-cmd",
            "args": ["arg1", "arg2"],
            "env": {
                "KEY": "value"
            },
            "features": {
                "tools": true,
                "resources": false,
                "prompts": true
            }
        });
        let result: Result<McpServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_http_server_accepts_all_valid_fields() {
        let json = json!({
            "type": "http",
            "description": "Valid HTTP server",
            "url": "http://localhost:8080",
            "headers": {
                "Authorization": "Bearer token"
            },
            "oauth_client_id": "client-id",
            "oauth_scopes": ["read", "write"],
            "features": {
                "tools": true,
                "resources": true,
                "prompts": false
            }
        });
        let result: Result<McpServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sse_server_accepts_all_valid_fields() {
        let json = json!({
            "type": "sse",
            "description": "Valid SSE server",
            "url": "http://localhost:8080/sse",
            "headers": {
                "Authorization": "Bearer token"
            },
            "oauth_client_id": "client-id",
            "oauth_scopes": ["read"],
            "features": {
                "tools": false,
                "resources": true,
                "prompts": true
            }
        });
        let result: Result<McpServerConfig, _> = serde_json::from_value(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_server_enabled_default_true() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd"
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert!(config.is_enabled());
    }

    #[test]
    fn test_server_enabled_explicit_true() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd",
            "enabled": true
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert!(config.is_enabled());
    }

    #[test]
    fn test_server_enabled_explicit_false() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd",
            "enabled": false
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_http_server_enabled_false() {
        let json = json!({
            "type": "http",
            "description": "HTTP test server",
            "url": "http://localhost:8080",
            "enabled": false
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_sse_server_enabled_false() {
        let json = json!({
            "type": "sse",
            "description": "SSE test server",
            "url": "http://localhost:8080/sse",
            "enabled": false
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_serialize_omits_enabled_when_true() {
        // Test that enabled field is omitted when set to true (default)
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: true,
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

        // enabled should NOT be present when true (default)
        assert!(!obj.contains_key("enabled"));
    }

    #[test]
    fn test_serialize_includes_enabled_when_false() {
        // Test that enabled field IS included when set to false
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: false,
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

        // enabled SHOULD be present when false
        assert!(obj.contains_key("enabled"));
        assert!(!obj.get("enabled").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_http_server_without_type_field() {
        let json = json!({
            "description": "HTTP server without explicit type",
            "url": "http://localhost:8080"
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Http { url, .. } => {
                assert_eq!(url, "http://localhost:8080");
            }
            _ => panic!("Expected Http config when url is present without type field"),
        }
    }

    #[test]
    fn test_sse_server_with_explicit_type() {
        let json = json!({
            "type": "sse",
            "description": "SSE server with explicit type",
            "url": "http://localhost:8080/sse"
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Sse { url, .. } => {
                assert_eq!(url, "http://localhost:8080/sse");
            }
            _ => panic!("Expected Sse config when type is explicitly sse"),
        }
    }

    #[test]
    fn test_http_server_with_explicit_type() {
        let json = json!({
            "type": "http",
            "description": "HTTP server with explicit type",
            "url": "http://localhost:8080"
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Http { url, .. } => {
                assert_eq!(url, "http://localhost:8080");
            }
            _ => panic!("Expected Http config when type is explicitly http"),
        }
    }

    #[test]
    fn test_stdio_server_without_type_field() {
        let json = json!({
            "description": "Stdio server without explicit type",
            "command": "npx"
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Stdio { command, .. } => {
                assert_eq!(command, "npx");
            }
            _ => panic!("Expected Stdio config when command is present without type field"),
        }
    }

    #[test]
    fn test_url_based_config_with_headers_and_no_type() {
        let json = json!({
            "description": "URL-based server with headers but no type",
            "url": "https://api.example.com/mcp",
            "headers": {
                "Authorization": "Bearer token"
            }
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        match config {
            McpServerConfig::Http { url, headers, .. } => {
                assert_eq!(url, "https://api.example.com/mcp");
                assert!(headers.is_some());
                assert_eq!(
                    headers.unwrap().get("Authorization"),
                    Some(&"Bearer token".to_string())
                );
            }
            _ => panic!("Expected Http config when url is present with headers but no type"),
        }
    }

    #[test]
    fn test_intermediate_config_preserves_enabled_true() {
        let intermediate = IntermediateServerConfig {
            command: Some("test-cmd".to_string()),
            args: None,
            env: None,
            url: None,
            headers: None,
            server_type: None,
            enabled: Some(true),
        };

        let config = intermediate
            .to_mcp_config("Test server".to_string())
            .unwrap();
        assert!(config.is_enabled());
    }

    #[test]
    fn test_intermediate_config_preserves_enabled_false() {
        let intermediate = IntermediateServerConfig {
            command: Some("test-cmd".to_string()),
            args: None,
            env: None,
            url: None,
            headers: None,
            server_type: None,
            enabled: Some(false),
        };

        let config = intermediate
            .to_mcp_config("Test server".to_string())
            .unwrap();
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_intermediate_config_defaults_enabled_to_true() {
        let intermediate = IntermediateServerConfig {
            command: Some("test-cmd".to_string()),
            args: None,
            env: None,
            url: None,
            headers: None,
            server_type: None,
            enabled: None,
        };

        let config = intermediate
            .to_mcp_config("Test server".to_string())
            .unwrap();
        assert!(config.is_enabled());
    }

    #[test]
    fn test_intermediate_http_preserves_enabled_false() {
        let intermediate = IntermediateServerConfig {
            command: None,
            args: None,
            env: None,
            url: Some("http://localhost:8080".to_string()),
            headers: None,
            server_type: None,
            enabled: Some(false),
        };

        let config = intermediate
            .to_mcp_config("HTTP test server".to_string())
            .unwrap();
        assert!(!config.is_enabled());
    }
}
