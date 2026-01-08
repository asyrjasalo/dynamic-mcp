use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Cursor,
    OpenCode,
    ClaudeDesktop,
    ClaudeCodeCli,
    VSCode,
    Antigravity,
    Gemini,
    Codex,
    Cline,
    KiloCode,
}

impl Tool {
    pub fn from_name(name: &str) -> Result<Self> {
        let normalized = name.to_lowercase().replace('_', "-");

        match normalized.as_str() {
            "cursor" => Ok(Tool::Cursor),
            "opencode" | "open-code" => Ok(Tool::OpenCode),
            "claude-desktop" => Ok(Tool::ClaudeDesktop),
            "claude" | "claude-code" | "claude-cli" => Ok(Tool::ClaudeCodeCli),
            "vscode" | "vs-code" | "visualstudiocode" => Ok(Tool::VSCode),
            "antigravity" => Ok(Tool::Antigravity),
            "gemini" | "gemini-cli" => Ok(Tool::Gemini),
            "codex" | "codex-cli" => Ok(Tool::Codex),
            "cline" => Ok(Tool::Cline),
            "kilocode" | "kilo-code" => Ok(Tool::KiloCode),
            _ => Err(anyhow!(
                "Unknown tool name '{}'\n\n\
                Supported tools:\n\
                  - cursor\n\
                  - opencode\n\
                  - claude-desktop\n\
                  - claude (Claude Code CLI)\n\
                  - vscode (or: vs-code)\n\
                  - antigravity\n\
                  - gemini\n\
                  - codex\n\
                  - cline\n\
                  - kilocode\n\n\
                Usage: dmcp import <tool-name>",
                name
            )),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tool::Cursor => "cursor",
            Tool::OpenCode => "opencode",
            Tool::ClaudeDesktop => "claude-desktop",
            Tool::ClaudeCodeCli => "claude",
            Tool::VSCode => "vscode",
            Tool::Antigravity => "antigravity",
            Tool::Gemini => "gemini",
            Tool::Codex => "codex",
            Tool::Cline => "cline",
            Tool::KiloCode => "kilocode",
        }
    }

    pub fn project_config_path(&self) -> Option<PathBuf> {
        match self {
            Tool::Cursor => Some(PathBuf::from(".cursor/mcp.json")),
            Tool::OpenCode => Some(PathBuf::from(".opencode/mcp.json")),
            Tool::ClaudeCodeCli => Some(PathBuf::from(".mcp.json")),
            Tool::Gemini => Some(PathBuf::from(".gemini/settings.json")),
            Tool::VSCode => Some(PathBuf::from(".vscode/mcp.json")),
            Tool::Cline => Some(PathBuf::from(".cline/mcp.json")),
            Tool::KiloCode => Some(PathBuf::from(".kilocode/mcp.json")),
            Tool::ClaudeDesktop | Tool::Antigravity | Tool::Codex => None,
        }
    }

    pub fn global_config_path(&self) -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Could not determine home directory (HOME or USERPROFILE not set)")?;

        let path = match self {
            Tool::Cursor => PathBuf::from(home).join(".cursor/mcp.json"),
            Tool::OpenCode => PathBuf::from(home).join(".config/opencode/opencode.jsonc"),
            Tool::ClaudeCodeCli => PathBuf::from(home).join(".claude/mcp.json"),
            Tool::ClaudeDesktop => {
                if cfg!(target_os = "macos") {
                    PathBuf::from(home)
                        .join("Library/Application Support/Claude/claude_desktop_config.json")
                } else if cfg!(target_os = "windows") {
                    PathBuf::from(home).join("AppData/Roaming/Claude/claude_desktop_config.json")
                } else {
                    PathBuf::from(home).join(".config/Claude/claude_desktop_config.json")
                }
            }
            Tool::VSCode => {
                if cfg!(target_os = "macos") {
                    PathBuf::from(home).join("Library/Application Support/Code/User/mcp.json")
                } else if cfg!(target_os = "windows") {
                    PathBuf::from(home).join("AppData/Roaming/Code/User/mcp.json")
                } else {
                    PathBuf::from(home).join(".config/Code/User/mcp.json")
                }
            }
            Tool::Antigravity => {
                return Err(anyhow!(
                    "Antigravity uses UI-managed config.\n\
                    Please manually locate mcp_config.json and specify the path:\n\
                      dmcp import mcp_config.json"
                ))
            }
            Tool::Gemini => PathBuf::from(home).join(".gemini/settings.json"),
            Tool::Codex => PathBuf::from(home).join(".codex/config.toml"),
            Tool::Cline => {
                return Err(anyhow!(
                    "Cline stores global config in VS Code extension settings.\n\
                    Please use project-level config instead: .cline/mcp.json"
                ))
            }
            Tool::KiloCode => {
                return Err(anyhow!(
                    "KiloCode stores global config in VS Code settings (mcp_settings.json).\n\
                    Please use project-level config instead: .kilocode/mcp.json"
                ))
            }
        };

        Ok(path)
    }

    pub fn env_var_pattern(&self) -> EnvVarPattern {
        match self {
            Tool::Cursor => EnvVarPattern::EnvColon,
            Tool::ClaudeDesktop | Tool::ClaudeCodeCli => EnvVarPattern::CurlyBraces,
            Tool::VSCode | Tool::Cline => {
                EnvVarPattern::Multiple(vec![EnvVarPattern::EnvColon, EnvVarPattern::InputPrompt])
            }
            Tool::Codex => EnvVarPattern::CurlyBraces,
            Tool::OpenCode | Tool::Antigravity | Tool::Gemini | Tool::KiloCode => {
                EnvVarPattern::SystemEnv
            }
        }
    }

    pub fn config_format(&self) -> ConfigFormat {
        match self {
            Tool::Codex => ConfigFormat::Toml,
            Tool::OpenCode => ConfigFormat::JsonOrJsonc,
            _ => ConfigFormat::Json,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvVarPattern {
    EnvColon,
    CurlyBraces,
    SystemEnv,
    InputPrompt,
    Multiple(Vec<EnvVarPattern>),
}

impl EnvVarPattern {
    pub fn normalize(&self, value: &str) -> String {
        match self {
            EnvVarPattern::EnvColon => value.replace("${env:", "${").replace("}}", "}"),
            EnvVarPattern::CurlyBraces => value.to_string(),
            EnvVarPattern::SystemEnv => value.to_string(),
            EnvVarPattern::InputPrompt => value.to_string(),
            EnvVarPattern::Multiple(patterns) => {
                let mut result = value.to_string();
                for pattern in patterns {
                    result = pattern.normalize(&result);
                }
                result
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    #[allow(dead_code)]
    Jsonc,
    JsonOrJsonc,
    Toml,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_from_name() {
        assert_eq!(Tool::from_name("cursor").unwrap(), Tool::Cursor);
        assert_eq!(Tool::from_name("CURSOR").unwrap(), Tool::Cursor);
        assert_eq!(
            Tool::from_name("claude-desktop").unwrap(),
            Tool::ClaudeDesktop
        );
        assert_eq!(Tool::from_name("claude").unwrap(), Tool::ClaudeCodeCli);
        assert_eq!(Tool::from_name("claude-code").unwrap(), Tool::ClaudeCodeCli);
        assert_eq!(Tool::from_name("vscode").unwrap(), Tool::VSCode);
        assert_eq!(Tool::from_name("vs-code").unwrap(), Tool::VSCode);
        assert_eq!(Tool::from_name("cline").unwrap(), Tool::Cline);
        assert_eq!(Tool::from_name("kilocode").unwrap(), Tool::KiloCode);
        assert_eq!(Tool::from_name("kilo-code").unwrap(), Tool::KiloCode);
    }

    #[test]
    fn test_unknown_tool() {
        let result = Tool::from_name("unknown");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown tool name"));
        assert!(err.contains("Supported tools:"));
    }

    #[test]
    fn test_project_config_paths() {
        assert_eq!(
            Tool::Cursor.project_config_path(),
            Some(PathBuf::from(".cursor/mcp.json"))
        );
        assert_eq!(
            Tool::OpenCode.project_config_path(),
            Some(PathBuf::from(".opencode/mcp.json"))
        );
        assert_eq!(
            Tool::VSCode.project_config_path(),
            Some(PathBuf::from(".vscode/mcp.json"))
        );
        assert_eq!(Tool::ClaudeDesktop.project_config_path(), None);
    }

    #[test]
    fn test_global_config_path_cursor() {
        let path = Tool::Cursor.global_config_path().unwrap();
        assert!(path.to_string_lossy().contains(".cursor/mcp.json"));
    }

    #[test]
    fn test_env_var_pattern_normalize() {
        let env_colon = EnvVarPattern::EnvColon;
        assert_eq!(env_colon.normalize("${env:VAR}"), "${VAR}");

        let curly_braces = EnvVarPattern::CurlyBraces;
        assert_eq!(curly_braces.normalize("${VAR}"), "${VAR}");

        let system_env = EnvVarPattern::SystemEnv;
        assert_eq!(system_env.normalize("VAR"), "VAR");
    }

    #[test]
    fn test_config_format() {
        assert_eq!(Tool::Cursor.config_format(), ConfigFormat::Json);
        assert_eq!(Tool::OpenCode.config_format(), ConfigFormat::JsonOrJsonc);
        assert_eq!(Tool::Codex.config_format(), ConfigFormat::Toml);
        assert_eq!(Tool::ClaudeCodeCli.config_format(), ConfigFormat::Json);
    }
}
