use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

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
            },
            Http {
                description: String,
                url: String,
                headers: Option<HashMap<String, String>>,
                oauth_client_id: Option<String>,
                oauth_scopes: Option<Vec<String>>,
            },
            Sse {
                description: String,
                url: String,
                headers: Option<HashMap<String, String>>,
                oauth_client_id: Option<String>,
                oauth_scopes: Option<Vec<String>>,
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
            } => Ok(McpServerConfig::Stdio {
                description,
                command,
                args,
                env,
            }),
            McpServerConfigHelper::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
            } => Ok(McpServerConfig::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
            }),
            McpServerConfigHelper::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
            } => Ok(McpServerConfig::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
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

    #[allow(dead_code)]
    pub fn set_description(&mut self, new_description: String) {
        match self {
            McpServerConfig::Stdio { description, .. } => *description = new_description,
            McpServerConfig::Http { description, .. } => *description = new_description,
            McpServerConfig::Sse { description, .. } => *description = new_description,
        }
    }

    #[allow(dead_code)]
    pub fn requires_oauth(&self) -> bool {
        match self {
            McpServerConfig::Http {
                oauth_client_id, ..
            }
            | McpServerConfig::Sse {
                oauth_client_id, ..
            } => oauth_client_id.is_some(),
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn oauth_config(&self) -> Option<(String, String, Option<Vec<String>>)> {
        match self {
            McpServerConfig::Http {
                url,
                oauth_client_id: Some(client_id),
                oauth_scopes,
                ..
            }
            | McpServerConfig::Sse {
                url,
                oauth_client_id: Some(client_id),
                oauth_scopes,
                ..
            } => Some((url.clone(), client_id.clone(), oauth_scopes.clone())),
            _ => None,
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
                })
            } else {
                Ok(McpServerConfig::Http {
                    description,
                    url,
                    headers: self.headers,
                    oauth_client_id: None,
                    oauth_scopes: None,
                })
            }
        } else if let Some(command) = self.command {
            // Stdio server
            Ok(McpServerConfig::Stdio {
                description,
                command,
                args: self.args,
                env: self.env,
            })
        } else {
            Err("Server config must have either 'command' or 'url'".to_string())
        }
    }
}
