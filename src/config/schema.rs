use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Per-server feature flags (opt-out design: all features enabled by default)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpServerConfig {
    #[serde(rename = "stdio")]
    Stdio {
        description: String,
        command: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        env: Option<HashMap<String, String>>,
        #[serde(default)]
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
        #[serde(default)]
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
        #[serde(default)]
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
        #[serde(tag = "type", rename_all = "lowercase")]
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
