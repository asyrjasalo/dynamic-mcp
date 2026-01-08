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

fn run_migrate_with_input(
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
    let mut args = vec!["migrate", tool_name];

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

    let mut child = cmd.spawn().expect("Failed to spawn migrate command");

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
fn test_migrate_cursor_project_success() {
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

    let output = run_migrate_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["File operations on /tmp", "Git operations"],
    );

    assert!(
        output.status.success(),
        "Migration should succeed. stderr: {}",
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
fn test_migrate_opencode_jsonc_success() {
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

    let project = TestProject::new("opencode", ".opencode", "mcp.jsonc", config_content);
    let output_path = project.output_path();

    let output = run_migrate_with_input(
        "opencode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Web search functionality"],
    );

    assert!(
        output.status.success(),
        "Migration should succeed. stderr: {}",
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
fn test_migrate_vscode_env_var_normalization() {
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

    let output = run_migrate_with_input(
        "vscode",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["GitHub API access"],
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
fn test_migrate_claude_project_success() {
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

    let output = run_migrate_with_input(
        "claude",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["PostgreSQL database access"],
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
fn test_migrate_cline_success() {
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

    let output = run_migrate_with_input(
        "cline",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Brave search integration"],
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
fn test_migrate_force_flag_skips_overwrite_prompt() {
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

    let output = run_migrate_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Test server"],
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
fn test_migrate_missing_config_file_error() {
    let project = TempDir::new().expect("Failed to create temp dir");

    let output = run_migrate_with_input(
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
fn test_migrate_empty_description_error() {
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

    let output = run_migrate_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec![""],
    );

    assert!(
        !output.status.success(),
        "Should fail with empty description"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Description cannot be empty") || stderr.contains("empty"));
}

#[test]
fn test_migrate_invalid_json_error() {
    let invalid_config = r#"{
  "mcpServers": {
    "test": {
      "command": "node"
}"#;

    let project = TestProject::new("cursor", ".cursor", "mcp.json", invalid_config);
    let output_path = project.output_path();

    let output = run_migrate_with_input(
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
fn test_migrate_multiple_servers_interactive() {
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

    let output = run_migrate_with_input(
        "cursor",
        project.path(),
        output_path.to_str().unwrap(),
        true,
        false,
        vec!["Description 1", "Description 2", "Description 3"],
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
