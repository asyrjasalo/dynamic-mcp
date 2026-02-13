use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Default tool call timeout in seconds
const DEFAULT_TOOL_TIMEOUT_SECS: u64 = 30;

/// Parse a duration string like "30s", "1min", "3000ms" into a Duration
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim().to_lowercase();

    if s.is_empty() {
        return Err("duration string is empty".to_string());
    }

    if let Ok(secs) = s.parse::<u64>() {
        return Ok(Duration::from_secs(secs));
    }

    if s.ends_with("ms") {
        let num_str = &s[..s.len() - 2];
        let millis: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid milliseconds: {}", num_str))?;
        return Ok(Duration::from_millis(millis));
    }

    if s.ends_with("s") {
        let num_str = &s[..s.len() - 1];
        let secs: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid seconds: {}", num_str))?;
        return Ok(Duration::from_secs(secs));
    }

    if s.ends_with("min") {
        let num_str = &s[..s.len() - 3];
        let mins: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid minutes: {}", num_str))?;
        return Ok(Duration::from_secs(mins * 60));
    }

    if s.ends_with("m") && !s.ends_with("ms") && !s.ends_with("min") {
        let num_str = &s[..s.len() - 1];
        let mins: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid minutes: {}", num_str))?;
        return Ok(Duration::from_secs(mins * 60));
    }

    if s.ends_with("h") {
        let num_str = &s[..s.len() - 1];
        let hours: u64 = num_str
            .parse()
            .map_err(|_| format!("invalid hours: {}", num_str))?;
        return Ok(Duration::from_secs(hours * 3600));
    }

    Err(format!(
        "invalid duration format '{}'. Use formats like: 30s, 1min, 500ms, 1h",
        s
    ))
}

/// Per-server timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Timeout {
    #[serde(default, deserialize_with = "deserialize_tools_timeout")]
    pub tools: Option<Duration>,
}

/// Custom deserializer for tools timeout that accepts Duration or string
fn deserialize_tools_timeout<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(n) => {
            let secs = n
                .as_u64()
                .ok_or_else(|| D::Error::custom("timeout must be a positive integer"))?;
            Ok(Some(Duration::from_secs(secs)))
        }
        serde_json::Value::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                parse_duration(&s).map(Some).map_err(D::Error::custom)
            }
        }
        _ => Err(D::Error::custom(
            "timeout must be a string (e.g., '30s', '1min', '500ms') or number (seconds)",
        )),
    }
}

impl Default for Timeout {
    fn default() -> Self {
        Self { tools: None }
    }
}

impl Timeout {
    /// Get the tool call timeout, returning the default if not configured
    pub fn tool_timeout(&self) -> Duration {
        self.tools
            .unwrap_or_else(|| Duration::from_secs(DEFAULT_TOOL_TIMEOUT_SECS))
    }

    /// Returns true if all timeouts are using defaults (None)
    pub fn is_default(&self) -> bool {
        self.tools.is_none()
    }
}

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
        #[serde(default, skip_serializing_if = "Timeout::is_default")]
        timeout: Timeout,
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
        #[serde(default, skip_serializing_if = "Timeout::is_default")]
        timeout: Timeout,
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
        #[serde(default, skip_serializing_if = "Timeout::is_default")]
        timeout: Timeout,
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
                #[serde(default)]
                timeout: Timeout,
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
                #[serde(default)]
                timeout: Timeout,
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
                #[serde(default)]
                timeout: Timeout,
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
                timeout,
            } => Ok(McpServerConfig::Stdio {
                description,
                command,
                args,
                env,
                features,
                enabled,
                timeout,
            }),
            McpServerConfigHelper::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
                timeout,
            } => Ok(McpServerConfig::Http {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
                timeout,
            }),
            McpServerConfigHelper::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
                timeout,
            } => Ok(McpServerConfig::Sse {
                description,
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                features,
                enabled,
                timeout,
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

    pub fn tool_timeout(&self) -> Duration {
        match self {
            McpServerConfig::Stdio { timeout, .. } => timeout.tool_timeout(),
            McpServerConfig::Http { timeout, .. } => timeout.tool_timeout(),
            McpServerConfig::Sse { timeout, .. } => timeout.tool_timeout(),
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
                    timeout: Timeout::default(),
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
                    timeout: Timeout::default(),
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
                timeout: Timeout::default(),
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
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

        assert!(!obj.contains_key("features"));
        assert!(!obj.contains_key("timeout"));
    }

    #[test]
    fn test_serialize_includes_disabled_features() {
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
            timeout: Timeout::default(),
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

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
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

        assert!(!obj.contains_key("enabled"));
        assert!(!obj.contains_key("timeout"));
    }

    #[test]
    fn test_serialize_includes_enabled_when_false() {
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: false,
            timeout: Timeout::default(),
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

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

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("100S").unwrap(), Duration::from_secs(100));
    }

    #[test]
    fn test_parse_duration_milliseconds() {
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(
            parse_duration("1000MS").unwrap(),
            Duration::from_millis(1000)
        );
    }

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("1min").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("5MIN").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2m").unwrap(), Duration::from_secs(120));
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("2H").unwrap(), Duration::from_secs(7200));
    }

    #[test]
    fn test_parse_duration_plain_number() {
        assert_eq!(parse_duration("30").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("60").unwrap(), Duration::from_secs(60));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("30x").is_err());
    }

    #[test]
    fn test_timeout_deserialize_string() {
        let timeout: Timeout = serde_json::from_value(json!({ "tools": "30s" })).unwrap();
        assert_eq!(timeout.tools, Some(Duration::from_secs(30)));

        let timeout: Timeout = serde_json::from_value(json!({ "tools": "1min" })).unwrap();
        assert_eq!(timeout.tools, Some(Duration::from_secs(60)));

        let timeout: Timeout = serde_json::from_value(json!({ "tools": "500ms" })).unwrap();
        assert_eq!(timeout.tools, Some(Duration::from_millis(500)));
    }

    #[test]
    fn test_timeout_deserialize_number() {
        let timeout: Timeout = serde_json::from_value(json!({ "tools": 60 })).unwrap();
        assert_eq!(timeout.tools, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_timeout_deserialize_null() {
        let timeout: Timeout = serde_json::from_value(json!({ "tools": null })).unwrap();
        assert!(timeout.tools.is_none());
    }

    #[test]
    fn test_timeout_default() {
        let timeout = Timeout::default();
        assert!(timeout.tools.is_none());
        assert_eq!(timeout.tool_timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_is_default() {
        let timeout = Timeout::default();
        assert!(timeout.is_default());

        let timeout: Timeout = serde_json::from_value(json!({ "tools": "30s" })).unwrap();
        assert!(!timeout.is_default());
    }

    #[test]
    fn test_server_config_with_timeout() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd",
            "timeout": {
                "tools": "60s"
            }
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.tool_timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_server_config_timeout_default() {
        let json = json!({
            "description": "Test server",
            "command": "test-cmd"
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.tool_timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_serialize_includes_custom_timeout() {
        let timeout: Timeout = serde_json::from_value(json!({ "tools": "60s" })).unwrap();
        let config = McpServerConfig::Stdio {
            description: "Test server".to_string(),
            command: "test-cmd".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: true,
            timeout,
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let obj = serialized.as_object().unwrap();

        assert!(obj.contains_key("timeout"));
    }
}
