use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Once;
use tempfile::TempDir;

static BUILD_BINARY: Once = Once::new();

fn ensure_binary_built() {
    BUILD_BINARY.call_once(|| {
        let output = Command::new("cargo")
            .args(["build", "--quiet"])
            .output()
            .expect("Failed to build binary");

        assert!(output.status.success(), "Binary build failed");
    });
}

fn get_binary_path() -> PathBuf {
    let binary_name = if cfg!(windows) { "dmcp.exe" } else { "dmcp" };

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .join("target")
        .join("debug")
        .join(binary_name)
}

struct TestProject {
    dir: TempDir,
}

impl TestProject {
    fn new(_tool: &str, config_dir: &str, config_file: &str, content: &str) -> Self {
        let dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir_path = dir.path().join(config_dir);
        fs::create_dir_all(&config_dir_path).expect("Failed to create config dir");

        let tool_config_path = config_dir_path.join(config_file);
        fs::write(&tool_config_path, content).expect("Failed to write config");

        Self { dir }
    }

    fn path(&self) -> &std::path::Path {
        self.dir.path()
    }

    fn output_path(&self) -> PathBuf {
        self.dir.path().join("dynamic-mcp.json")
    }
}

fn run_import_with_input(
    tool_name: &str,
    working_dir: &std::path::Path,
    output_path: &str,
    force: bool,
    global: bool,
    input_lines: Vec<&str>,
) -> std::process::Output {
    ensure_binary_built();

    let binary_path = get_binary_path();
    let mut cmd = Command::new(binary_path);
    let mut args = vec!["import", tool_name];

    if global {
        args.push("--global");
    }

    if force {
        args.push("--force");
    }

    args.push("-o");
    args.push(output_path);

    cmd.args(&args)
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn import command");

    if let Some(mut stdin) = child.stdin.take() {
        for line in input_lines {
            writeln!(stdin, "{}", line).expect("Failed to write to stdin");
        }
    }

    child
        .wait_with_output()
        .expect("Failed to wait for command")
}

#[test]
fn test_import_cursor_project_success() {
    let config_content = r#"{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "git": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-git"]
    }
  }
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec![
            "File operations on /tmp", // Description for filesystem
            "",                        // Keep all features (default Y)
            "Git operations",          // Description for git
            "",                        // Keep all features (default Y)
        ],
    );

    assert!(
        output.status.success(),
        "Import should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output file should be created");

    let output_content = fs::read_to_string(&output_path).expect("Failed to read output file");
    let parsed: serde_json::Value =
        serde_json::from_str(&output_content).expect("Output should be valid JSON");

    assert!(parsed.get("mcpServers").is_some());
    let servers = parsed["mcpServers"].as_object().unwrap();
    assert_eq!(servers.len(), 2);

    let filesystem = &servers["filesystem"];
    assert_eq!(
        filesystem["description"].as_str().unwrap(),
        "File operations on /tmp"
    );
    assert_eq!(filesystem["command"].as_str().unwrap(), "npx");
    let args = filesystem["args"].as_array().unwrap();
    assert_eq!(args.len(), 3);

    let git = &servers["git"];
    assert_eq!(git["description"].as_str().unwrap(), "Git operations");
}

#[test]
fn test_import_opencode_jsonc_success() {
    let config_content = r#"{
  // OpenCode MCP configuration with comments
  "mcp": {
    "web-search": {
      "command": ["node", "server.js"],
      "env": {
        "API_KEY": "${SEARCH_API_KEY}"
      },
      "enabled": true,
      "type": "local"
    }
  }
}"#;

    let project = TestProject::new("opencode", ".opencode", "opencode.jsonc", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "opencode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Web search functionality", ""],
    );

    assert!(
        output.status.success(),
        "Import should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists());

    let output_content = fs::read_to_string(&output_path).expect("Failed to read output");
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();

    let servers = parsed["mcpServers"].as_object().unwrap();
    let web_search = &servers["web-search"];
    assert_eq!(
        web_search["description"].as_str().unwrap(),
        "Web search functionality"
    );

    let env = web_search["env"].as_object().unwrap();
    assert_eq!(env["API_KEY"].as_str().unwrap(), "${SEARCH_API_KEY}");
}

#[test]
fn test_import_vscode_env_var_normalization() {
    let config_content = r#"{
  "servers": {
    "github": {
      "command": "node",
      "args": ["github-server.js"],
      "env": {
        "GITHUB_TOKEN": "${env:GITHUB_TOKEN}"
      }
    }
  }
}"#;

    let project = TestProject::new("vscode", ".vscode", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "vscode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["GitHub API access", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();

    let github = &parsed["mcpServers"]["github"];
    let env = github["env"].as_object().unwrap();

    assert_eq!(env["GITHUB_TOKEN"].as_str().unwrap(), "${GITHUB_TOKEN}");
}

#[test]
fn test_import_claude_project_success() {
    let config_content = r#"{
  "mcpServers": {
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://localhost/mydb"]
    }
  }
}"#;

    let project = TestProject::new("claude", ".", ".mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "claude",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["PostgreSQL database access", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();

    let postgres = &parsed["mcpServers"]["postgres"];
    assert_eq!(
        postgres["description"].as_str().unwrap(),
        "PostgreSQL database access"
    );
}

#[test]
fn test_import_cline_success() {
    let config_content = r#"{
  "mcpServers": {
    "brave-search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "${env:BRAVE_API_KEY}"
      }
    }
  }
}"#;

    let project = TestProject::new("cline", ".cline", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cline",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Brave search integration", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();

    let brave = &parsed["mcpServers"]["brave-search"];

    let env = brave["env"].as_object().unwrap();
    assert_eq!(env["BRAVE_API_KEY"].as_str().unwrap(), "${BRAVE_API_KEY}");
}

#[test]
fn test_import_force_flag_skips_overwrite_prompt() {
    let config_content = r#"{
  "mcpServers": {
    "test": {
      "command": "node",
      "args": ["test.js"]
    }
  }
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", config_content);
    let output_path = project.output_path();

    fs::write(&output_path, "existing content").expect("Failed to create existing file");

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server", ""],
    );

    assert!(
        output.status.success(),
        "Should succeed with --force flag. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("mcpServers"));
    assert!(!content.contains("existing content"));
}

#[test]
fn test_import_missing_config_file_error() {
    let project = TempDir::new().expect("Failed to create temp dir");

    let output = run_import_with_input(
        "cursor",
        project.path(),
        project.path().join("output.json").to_str().unwrap(),
        true,
        false,
        vec![],
    );

    assert!(
        !output.status.success(),
        "Should fail when config file missing"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Config file not found") || stderr.contains("not found"));
}

#[test]
fn test_import_empty_description_error() {
    let config_content = r#"{
  "mcpServers": {
    "test": {
      "command": "node",
      "args": ["test.js"]
    }
  }
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["", ""],
    );

    assert!(
        !output.status.success(),
        "Should fail with empty description"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Description cannot be empty") || stderr.contains("empty"));
}

#[test]
fn test_import_invalid_json_error() {
    let invalid_config = r#"{
  "mcpServers": {
    "test": {
      "command": "node"
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", invalid_config);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec![],
    );

    assert!(!output.status.success(), "Should fail with invalid JSON");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to parse") || stderr.contains("parse") || stderr.contains("JSON")
    );
}

#[test]
fn test_import_multiple_servers_interactive() {
    let config_content = r#"{
  "mcpServers": {
    "server1": {
      "command": "node",
      "args": ["s1.js"]
    },
    "server2": {
      "command": "node",
      "args": ["s2.js"]
    },
    "server3": {
      "command": "node",
      "args": ["s3.js"]
    }
  }
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec![
            "Description 1",
            "", // server1: description + keep all features
            "Description 2",
            "", // server2: description + keep all features
            "Description 3",
            "", // server3: description + keep all features
        ],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists());

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();

    let servers = parsed["mcpServers"].as_object().unwrap();
    assert_eq!(servers.len(), 3);

    assert_eq!(
        servers["server1"]["description"].as_str().unwrap(),
        "Description 1"
    );
    assert_eq!(
        servers["server2"]["description"].as_str().unwrap(),
        "Description 2"
    );
    assert_eq!(
        servers["server3"]["description"].as_str().unwrap(),
        "Description 3"
    );
}

#[test]
fn test_import_cursor_env_var_conversion() {
    let config_content = r#"{
  "mcpServers": {
    "test": {
      "command": "npx",
      "args": ["server"],
      "env": {
        "API_KEY": "${env:API_KEY}",
        "CONFIG_PATH": "${env:CONFIG_PATH}",
        "HOME_DIR": "${env:HOME}"
      }
    }
  }
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let test = &parsed["mcpServers"]["test"];

    // Verify ${env:VAR} converted to ${VAR} in env
    let env = test["env"].as_object().unwrap();
    assert_eq!(env["API_KEY"].as_str().unwrap(), "${API_KEY}");
    assert_eq!(env["CONFIG_PATH"].as_str().unwrap(), "${CONFIG_PATH}");
    assert_eq!(env["HOME_DIR"].as_str().unwrap(), "${HOME}");
}

#[test]
fn test_import_vscode_env_var_conversion_in_env() {
    let config_content = r#"{
  "servers": {
    "test": {
      "command": "node",
      "args": ["server.js"],
      "env": {
        "PORT": "${env:SERVER_PORT}",
        "WORKSPACE": "${env:WORKSPACE_ROOT}",
        "HOME": "${env:HOME}"
      }
    }
  }
}"#;

    let project = TestProject::new("vscode", ".vscode", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "vscode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let test = &parsed["mcpServers"]["test"];

    // Verify ${env:VAR} converted to ${VAR} in env
    let env = test["env"].as_object().unwrap();
    assert_eq!(env["PORT"].as_str().unwrap(), "${SERVER_PORT}");
    assert_eq!(env["WORKSPACE"].as_str().unwrap(), "${WORKSPACE_ROOT}");
    assert_eq!(env["HOME"].as_str().unwrap(), "${HOME}");
}

#[test]
fn test_import_vscode_env_var_conversion_in_headers() {
    let config_content = r#"{
  "servers": {
    "api": {
      "type": "http",
      "url": "https://api.example.com",
      "headers": {
        "Authorization": "${env:API_TOKEN}",
        "X-Custom-Header": "${env:CUSTOM_VALUE}"
      }
    }
  }
}"#;

    let project = TestProject::new("vscode", ".vscode", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "vscode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["API server", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let api = &parsed["mcpServers"]["api"];

    let headers = api["headers"].as_object().unwrap();
    assert_eq!(headers["Authorization"].as_str().unwrap(), "${API_TOKEN}");
    assert_eq!(
        headers["X-Custom-Header"].as_str().unwrap(),
        "${CUSTOM_VALUE}"
    );
}

#[test]
fn test_import_codex_env_var_passthrough() {
    let config_content = r#"
[mcp.test]
command = "node"
args = ["server.js"]

[mcp.test.env]
API_KEY = "${API_KEY}"
CONFIG_PATH = "${CONFIG_PATH}"
WORKSPACE = "${WORKSPACE_ROOT}"
"#;

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap();

    let dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let codex_dir = dir.path().join(".codex");
    std::fs::create_dir_all(&codex_dir).expect("Failed to create .codex dir");
    std::fs::write(codex_dir.join("config.toml"), config_content).expect("Failed to write config");

    std::env::set_var("HOME", dir.path());

    let output_path = dir.path().join("dynamic-mcp.json");

    let output = run_import_with_input(
        "codex",
        dir.path(),
        output_path.to_str().unwrap(),
        true,
        true,
        vec!["Test server", ""],
    );

    std::env::set_var("HOME", home);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let test = &parsed["mcpServers"]["test"];

    // Verify ${VAR} stays as ${VAR} in env
    let env = test["env"].as_object().unwrap();
    assert_eq!(env["API_KEY"].as_str().unwrap(), "${API_KEY}");
    assert_eq!(env["CONFIG_PATH"].as_str().unwrap(), "${CONFIG_PATH}");
    assert_eq!(env["WORKSPACE"].as_str().unwrap(), "${WORKSPACE_ROOT}");
}

#[test]
fn test_import_claude_env_var_passthrough() {
    let config_content = r#"{
  "mcpServers": {
    "test": {
      "command": "node",
      "args": ["server.js"],
      "env": {
        "DATABASE_URL": "${DATABASE_URL}",
        "API_KEY": "${API_KEY}",
        "WORKSPACE": "${WORKSPACE_ROOT}"
      }
    }
  }
}"#;

    let project = TestProject::new("claude", ".", ".mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "claude",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let test = &parsed["mcpServers"]["test"];

    // Verify ${VAR} stays as ${VAR} in env
    let env = test["env"].as_object().unwrap();
    assert_eq!(env["DATABASE_URL"].as_str().unwrap(), "${DATABASE_URL}");
    assert_eq!(env["API_KEY"].as_str().unwrap(), "${API_KEY}");
    assert_eq!(env["WORKSPACE"].as_str().unwrap(), "${WORKSPACE_ROOT}");
}

#[test]
fn test_import_opencode_env_var_passthrough() {
    let config_content = r#"{
  "mcp": {
    "test": {
      "command": ["node", "server.js"],
      "env": {
        "API_KEY": "${API_KEY}",
        "PORT": "${PORT}",
        "WORKSPACE": "${WORKSPACE_ROOT}"
      },
      "enabled": true
    }
  }
}"#;

    let project = TestProject::new("opencode", ".opencode", "opencode.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "opencode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let test = &parsed["mcpServers"]["test"];

    // Verify ${VAR} stays as ${VAR} in env
    let env = test["env"].as_object().unwrap();
    assert_eq!(env["API_KEY"].as_str().unwrap(), "${API_KEY}");
    assert_eq!(env["PORT"].as_str().unwrap(), "${PORT}");
    assert_eq!(env["WORKSPACE"].as_str().unwrap(), "${WORKSPACE_ROOT}");
}

#[test]
fn test_import_gemini_env_var_passthrough() {
    let config_content = r#"{
  "mcpServers": {
    "test": {
      "command": "npx",
      "args": ["-y", "server"],
      "env": {
        "GEMINI_API_KEY": "${GEMINI_API_KEY}",
        "LOG_LEVEL": "${LOG_LEVEL}",
        "DATA_DIR": "${DATA_DIR}"
      }
    }
  }
}"#;

    let project = TestProject::new("gemini", ".gemini", "settings.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "gemini",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let test = &parsed["mcpServers"]["test"];

    // Verify ${VAR} stays as ${VAR} in env
    let env = test["env"].as_object().unwrap();
    assert_eq!(env["GEMINI_API_KEY"].as_str().unwrap(), "${GEMINI_API_KEY}");
    assert_eq!(env["LOG_LEVEL"].as_str().unwrap(), "${LOG_LEVEL}");
    assert_eq!(env["DATA_DIR"].as_str().unwrap(), "${DATA_DIR}");
}

#[test]
fn test_import_kilocode_env_var_passthrough() {
    let config_content = r#"{
  "mcpServers": {
    "test": {
      "command": "npx",
      "args": ["server"],
      "env": {
        "API_KEY": "${API_KEY}",
        "CONFIG": "${CONFIG}",
        "WORKSPACE": "${WORKSPACE_ROOT}"
      }
    }
  }
}"#;

    let project = TestProject::new("kilocode", ".kilocode", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "kilocode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server", ""],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
    let test = &parsed["mcpServers"]["test"];

    // Verify ${VAR} stays as ${VAR} in env
    let env = test["env"].as_object().unwrap();
    assert_eq!(env["API_KEY"].as_str().unwrap(), "${API_KEY}");
    assert_eq!(env["CONFIG"].as_str().unwrap(), "${CONFIG}");
    assert_eq!(env["WORKSPACE"].as_str().unwrap(), "${WORKSPACE_ROOT}");
}

#[test]
fn test_import_custom_features_selection() {
    let config_content = r#"{
  "mcpServers": {
    "tools-only": {
      "command": "npx",
      "args": ["@mcp/server-tools"]
    },
    "resources-only": {
      "command": "npx",
      "args": ["@mcp/server-resources"]
    }
  }
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec![
            "Resources server", // Description for resources-only (sorted first)
            "n",                // Customize features
            "n",                // Disable tools
            "y",                // Enable resources
            "n",                // Disable prompts
            "Tools server",     // Description for tools-only (sorted second)
            "n",                // Customize features
            "y",                // Enable tools
            "n",                // Disable resources
            "n",                // Disable prompts
        ],
    );

    assert!(
        output.status.success(),
        "Import should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(output_path.exists(), "Output file should be created");

    let output_content = fs::read_to_string(&output_path).expect("Failed to read output file");
    let parsed: serde_json::Value =
        serde_json::from_str(&output_content).expect("Output should be valid JSON");

    let servers = parsed["mcpServers"].as_object().unwrap();
    assert_eq!(servers.len(), 2);

    // Verify tools-only server has only tools enabled
    let tools_only = &servers["tools-only"];
    assert_eq!(tools_only["description"].as_str().unwrap(), "Tools server");
    let features = &tools_only["features"];
    assert_eq!(features["tools"].as_bool().unwrap(), true);
    assert_eq!(features["resources"].as_bool().unwrap(), false);
    assert_eq!(features["prompts"].as_bool().unwrap(), false);

    // Verify resources-only server has only resources enabled
    let resources_only = &servers["resources-only"];
    assert_eq!(
        resources_only["description"].as_str().unwrap(),
        "Resources server"
    );
    let features = &resources_only["features"];
    assert_eq!(features["tools"].as_bool().unwrap(), false);
    assert_eq!(features["resources"].as_bool().unwrap(), true);
    assert_eq!(features["prompts"].as_bool().unwrap(), false);
}

#[test]
fn test_import_default_all_features_enabled() {
    let config_content = r#"{
  "mcpServers": {
    "test": {
      "command": "npx",
      "args": ["server"]
    }
  }
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", config_content);
    let output_path = project.output_path();

    let output = run_import_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec![
            "Test server", // Description
            "",            // Keep all features (default Y)
        ],
    );

    assert!(
        output.status.success(),
        "Import should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_content = fs::read_to_string(&output_path).expect("Failed to read output file");
    let parsed: serde_json::Value =
        serde_json::from_str(&output_content).expect("Output should be valid JSON");

    let test_server = &parsed["mcpServers"]["test"];

    // When features are not customized, they should all be true (but serialized with defaults)
    // The Features struct uses #[serde(default)] so enabled features are serialized as true
    let features = &test_server["features"];
    assert_eq!(features["tools"].as_bool().unwrap(), true);
    assert_eq!(features["resources"].as_bool().unwrap(), true);
    assert_eq!(features["prompts"].as_bool().unwrap(), true);
}
