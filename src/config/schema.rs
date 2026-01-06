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
    },
    Sse {
        description: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
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
            },
            Sse {
                description: String,
                url: String,
                headers: Option<HashMap<String, String>>,
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
            } => Ok(McpServerConfig::Http {
                description,
                url,
                headers,
            }),
            McpServerConfigHelper::Sse {
                description,
                url,
                headers,
            } => Ok(McpServerConfig::Sse {
                description,
                url,
                headers,
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

    pub fn set_description(&mut self, new_description: String) {
        match self {
            McpServerConfig::Stdio { description, .. } => *description = new_description,
            McpServerConfig::Http { description, .. } => *description = new_description,
            McpServerConfig::Sse { description, .. } => *description = new_description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMcpServerConfig {
    #[serde(default = "default_stdio_type")]
    pub r#type: String,
    #[serde(flatten)]
    pub config: serde_json::Value,
}

fn default_stdio_type() -> String {
    "stdio".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardServerConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, StandardMcpServerConfig>,
}
