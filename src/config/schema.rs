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
    },
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
            },
            Http {
                description: String,
                url: String,
                headers: Option<HashMap<String, String>>,
                oauth_client_id: Option<String>,
                oauth_scopes: Option<Vec<String>>,
                #[serde(default)]
                features: Features,
            },
            Sse {
                description: String,
                url: String,
                headers: Option<HashMap<String, String>>,
                oauth_client_id: Option<String>,
                oauth_scopes: Option<Vec<String>>,
                #[serde(default)]
                features: Features,
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
            } => Ok(McpServerConfig::Stdio {
                description,
                command,
                args,
                env,
                features,
            }),
            McpServerConfigHelper::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
            } => Ok(McpServerConfig::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
            }),
            McpServerConfigHelper::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
            } => Ok(McpServerConfig::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
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
}

impl IntermediateServerConfig {
    /// Convert to McpServerConfig with a description
    #[allow(clippy::wrong_self_convention)]
    pub fn to_mcp_config(self, description: String) -> Result<McpServerConfig, String> {
        // Determine server type
        if let Some(url) = self.url {
            // HTTP or SSE server
            let server_type = self.server_type.as_deref().unwrap_or("http").to_lowercase();

            if server_type == "sse" {
                Ok(McpServerConfig::Sse {
                    description,
                    url,
                    headers: self.headers,
                    oauth_client_id: None,
                    oauth_scopes: None,
                    features: Features::default(),
                })
            } else {
                Ok(McpServerConfig::Http {
                    description,
                    url,
                    headers: self.headers,
                    oauth_client_id: None,
                    oauth_scopes: None,
                    features: Features::default(),
                })
            }
        } else if let Some(command) = self.command {
            // Stdio server
            Ok(McpServerConfig::Stdio {
                description,
                command,
                args: self.args,
                env: self.env,
                features: Features::default(),
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
}
