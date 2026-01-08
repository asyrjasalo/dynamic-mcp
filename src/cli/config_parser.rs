use crate::cli::tool_detector::{ConfigFormat, Tool};
use crate::config::schema::IntermediateServerConfig;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;

pub struct ConfigParser {
    tool: Tool,
}

impl ConfigParser {
    pub fn new(tool: Tool) -> Self {
        Self { tool }
    }

    pub fn parse(&self, content: &str) -> Result<HashMap<String, IntermediateServerConfig>> {
        match self.tool.config_format() {
            ConfigFormat::Json => self.parse_json(content),
            ConfigFormat::Jsonc => self.parse_jsonc(content),
            ConfigFormat::JsonOrJsonc => self
                .parse_jsonc(content)
                .or_else(|_| self.parse_json(content)),
            ConfigFormat::Toml => self.parse_toml(content),
        }
    }

    fn parse_json(&self, content: &str) -> Result<HashMap<String, IntermediateServerConfig>> {
        let value: serde_json::Value =
            serde_json::from_str(content).context("Failed to parse JSON config")?;

        self.extract_servers(&value)
    }

    fn parse_jsonc(&self, content: &str) -> Result<HashMap<String, IntermediateServerConfig>> {
        let content_without_comments = Self::strip_line_comments(content);
        let stripped = json_comments::StripComments::new(content_without_comments.as_bytes());
        let value: serde_json::Value = serde_json::from_reader(stripped)
            .context("Failed to parse JSONC config (JSON with comments)")?;

        self.extract_servers(&value)
    }

    fn strip_line_comments(content: &str) -> String {
        content
            .lines()
            .map(|line| {
                if let Some(pos) = line.find("//") {
                    let before_comment = &line[..pos];
                    let in_string = before_comment.matches('"').count() % 2 != 0;
                    if in_string {
                        line.to_string()
                    } else {
                        before_comment.to_string()
                    }
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn parse_toml(&self, content: &str) -> Result<HashMap<String, IntermediateServerConfig>> {
        let value: toml::Value = toml::from_str(content).context("Failed to parse TOML config")?;

        let mcp_table = value
            .get("mcp")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow!("TOML config missing 'mcp' table"))?;

        let mut servers = HashMap::new();

        for (name, server_value) in mcp_table {
            let server_table = server_value
                .as_table()
                .ok_or_else(|| anyhow!("Server '{}' is not a table", name))?;

            let command = server_table
                .get("command")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let args = server_table
                .get("args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                });

            let env = server_table
                .get("env")
                .and_then(|v| v.as_table())
                .map(|table| {
                    table
                        .iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                });

            let url = server_table
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let server_type = server_table
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let intermediate = IntermediateServerConfig {
                command,
                args,
                env: env.map(|e| self.normalize_env_vars(e)),
                url,
                headers: None,
                server_type,
            };

            servers.insert(name.clone(), intermediate);
        }

        Ok(servers)
    }

    fn extract_servers(
        &self,
        value: &serde_json::Value,
    ) -> Result<HashMap<String, IntermediateServerConfig>> {
        let servers_key = match self.tool {
            Tool::OpenCode => "mcp",
            Tool::VSCode => "servers",
            _ => "mcpServers",
        };

        let servers_obj = value
            .get(servers_key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                anyhow!(
                    "Config missing '{}' object. Expected format:\n{{\n  \"{}\": {{\n    \"server-name\": {{ ... }}\n  }}\n}}",
                    servers_key,
                    servers_key
                )
            })?;

        let mut result = HashMap::new();

        for (name, server_value) in servers_obj {
            let server_obj = server_value.as_object().ok_or_else(|| {
                anyhow!(
                    "Server '{}' is not an object. Each server must be a JSON object.",
                    name
                )
            })?;

            let command = match self.tool {
                Tool::OpenCode => server_obj
                    .get("command")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                _ => server_obj
                    .get("command")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };

            let args = match self.tool {
                Tool::OpenCode => server_obj
                    .get("command")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .skip(1)
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
                _ => server_obj
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    }),
            };

            let env = server_obj
                .get("env")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                });

            let url = server_obj
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let headers = server_obj
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                });

            let server_type = server_obj
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let intermediate = IntermediateServerConfig {
                command,
                args,
                env: env.map(|e| self.normalize_env_vars(e)),
                url,
                headers: headers.map(|h| self.normalize_env_vars(h)),
                server_type,
            };

            result.insert(name.clone(), intermediate);
        }

        Ok(result)
    }

    fn normalize_env_vars(&self, map: HashMap<String, String>) -> HashMap<String, String> {
        let pattern = self.tool.env_var_pattern();
        map.into_iter()
            .map(|(k, v)| (k, pattern.normalize(&v)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cursor_json() {
        let config = r#"{
            "mcpServers": {
                "test": {
                    "command": "npx",
                    "args": ["-y", "package"],
                    "env": {
                        "TOKEN": "${env:GITHUB_TOKEN}"
                    }
                }
            }
        }"#;

        let parser = ConfigParser::new(Tool::Cursor);
        let result = parser.parse(config).unwrap();

        assert_eq!(result.len(), 1);
        let server = result.get("test").unwrap();
        assert_eq!(server.command, Some("npx".to_string()));
        assert_eq!(
            server.args,
            Some(vec!["-y".to_string(), "package".to_string()])
        );

        let env = server.env.as_ref().unwrap();
        assert_eq!(env.get("TOKEN").unwrap(), "${GITHUB_TOKEN}");
    }

    #[test]
    fn test_parse_opencode_jsonc() {
        let config = r#"{
            // Comment
            "mcp": {
                "test": {
                    "command": ["npx", "-y", "package"],
                    "enabled": true
                }
            }
        }"#;

        let parser = ConfigParser::new(Tool::OpenCode);
        let result = parser.parse(config).unwrap();

        assert_eq!(result.len(), 1);
        let server = result.get("test").unwrap();
        assert_eq!(server.command, Some("npx".to_string()));
        assert_eq!(
            server.args,
            Some(vec!["-y".to_string(), "package".to_string()])
        );
    }

    #[test]
    fn test_parse_claude_desktop_json() {
        let config = r#"{
            "mcpServers": {
                "test": {
                    "command": "docker",
                    "args": ["run", "-i", "image"],
                    "env": {
                        "TOKEN": "${GITHUB_TOKEN}"
                    }
                }
            }
        }"#;

        let parser = ConfigParser::new(Tool::ClaudeDesktop);
        let result = parser.parse(config).unwrap();

        let server = result.get("test").unwrap();
        let env = server.env.as_ref().unwrap();
        assert_eq!(env.get("TOKEN").unwrap(), "${GITHUB_TOKEN}");
    }

    #[test]
    fn test_parse_vscode_json_with_url() {
        let config = r#"{
            "servers": {
                "api": {
                    "type": "http",
                    "url": "https://api.example.com",
                    "headers": {
                        "API_Key": "${env:API_KEY}"
                    }
                }
            }
        }"#;

        let parser = ConfigParser::new(Tool::VSCode);
        let result = parser.parse(config).unwrap();

        let server = result.get("api").unwrap();
        assert_eq!(server.url, Some("https://api.example.com".to_string()));
        assert_eq!(server.server_type, Some("http".to_string()));

        let headers = server.headers.as_ref().unwrap();
        assert_eq!(headers.get("API_Key").unwrap(), "${API_KEY}");
    }

    #[test]
    fn test_parse_codex_toml() {
        let config = r#"
[mcp.test]
command = "npx"
args = ["-y", "package"]

[mcp.test.env]
TOKEN = "${GITHUB_TOKEN}"
        "#;

        let parser = ConfigParser::new(Tool::Codex);
        let result = parser.parse(config).unwrap();

        assert_eq!(result.len(), 1);
        let server = result.get("test").unwrap();
        assert_eq!(server.command, Some("npx".to_string()));
        assert_eq!(
            server.args,
            Some(vec!["-y".to_string(), "package".to_string()])
        );

        let env = server.env.as_ref().unwrap();
        assert_eq!(env.get("TOKEN").unwrap(), "${GITHUB_TOKEN}");
    }

    #[test]
    fn test_parse_missing_mcpservers() {
        let config = r#"{"other": {}}"#;
        let parser = ConfigParser::new(Tool::Cursor);
        let result = parser.parse(config);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing 'mcpServers'"));
    }

    #[test]
    fn test_parse_invalid_server_format() {
        let config = r#"{"mcpServers": {"test": "not-an-object"}}"#;
        let parser = ConfigParser::new(Tool::Cursor);
        let result = parser.parse(config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not an object"));
    }
}
