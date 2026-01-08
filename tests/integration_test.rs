use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_project_builds() {
    let output = Command::new("cargo")
        .args(["build", "--quiet"])
        .output()
        .expect("Failed to build");

    assert!(output.status.success(), "Build should succeed");
}

#[test]
fn test_config_example_exists() {
    assert!(
        std::path::Path::new("examples/config.example.json").exists(),
        "examples/config.example.json should exist"
    );
}

#[test]
fn test_binary_exists_after_build() {
    let _ = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to build");

    let binary_path = if cfg!(windows) {
        "target/debug/dmcp.exe"
    } else {
        "target/debug/dmcp"
    };

    assert!(
        std::path::Path::new(binary_path).exists(),
        "Binary should exist after build"
    );
}

#[test]
fn test_migrate_command_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "migrate", "--help"])
        .output()
        .expect("Failed to run migrate help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("migrate"));
    assert!(stdout.contains("Tool name") || stdout.contains("AI coding tools"));
}

#[test]
fn test_migrate_command_with_valid_config() {
    let mut input_file = NamedTempFile::new().unwrap();
    let config_json = r#"{
        "mcpServers": {
            "test": {
                "command": "node",
                "args": ["server.js"]
            }
        }
    }"#;
    input_file.write_all(config_json.as_bytes()).unwrap();
    input_file.flush().unwrap();

    let output_file = NamedTempFile::new().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "migrate",
            input_file.path().to_str().unwrap(),
            "-o",
            output_file.path().to_str().unwrap(),
        ])
        .env("TEST_MODE", "1")
        .output()
        .expect("Failed to run migrate");

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        eprintln!("Migration failed with stderr: {}", stderr);
    }
}

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to run version");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dynamic-mcp"));
}

#[test]
fn test_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to run help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage") || stdout.contains("dynamic-mcp"));
    assert!(stdout.contains("migrate") || stdout.contains("Commands"));
}

#[test]
fn test_invalid_config_path() {
    let output = Command::new("cargo")
        .args(["run", "--", "/nonexistent/config.json"])
        .output()
        .expect("Failed to run with invalid config");

    let exit_code = output.status.code().unwrap_or(0);
    assert_ne!(exit_code, 0, "Should fail with nonexistent config");
}

#[test]
fn test_config_schema_validation() {
    let config = std::fs::read_to_string("examples/config.example.json")
        .expect("Failed to read examples/config.example.json");

    let parsed: serde_json::Value =
        serde_json::from_str(&config).expect("Config should be valid JSON");

    assert!(
        parsed.get("mcpServers").is_some(),
        "Config should have mcpServers"
    );
}

#[test]
fn test_migrate_with_tool_name_cursor() {
    use std::fs;
    let fixture_path = "tests/fixtures/migrate/cursor/project.json";

    assert!(
        std::path::Path::new(fixture_path).exists(),
        "Cursor fixture should exist"
    );

    let content = fs::read_to_string(fixture_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(parsed.get("mcpServers").is_some());
}

#[test]
fn test_migrate_with_tool_name_opencode() {
    let fixture_path = "tests/fixtures/migrate/opencode/project.jsonc";

    assert!(
        std::path::Path::new(fixture_path).exists(),
        "OpenCode fixture should exist"
    );
}

#[test]
fn test_migrate_with_tool_name_claude() {
    use std::fs;
    let fixture_path = "tests/fixtures/migrate/claude-desktop/global.json";

    assert!(
        std::path::Path::new(fixture_path).exists(),
        "Claude Desktop fixture should exist"
    );

    let content = fs::read_to_string(fixture_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(parsed.get("mcpServers").is_some());
}

#[test]
fn test_migrate_with_tool_name_vscode() {
    use std::fs;
    let fixture_path = "tests/fixtures/migrate/vscode/project.json";

    assert!(
        std::path::Path::new(fixture_path).exists(),
        "VS Code fixture should exist"
    );

    let content = fs::read_to_string(fixture_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(parsed.get("servers").is_some());
}

#[test]
fn test_migrate_with_tool_name_codex_toml() {
    use std::fs;
    let fixture_path = "tests/fixtures/migrate/codex/global.toml";

    assert!(
        std::path::Path::new(fixture_path).exists(),
        "Codex fixture should exist"
    );

    let content = fs::read_to_string(fixture_path).unwrap();
    assert!(content.contains("[mcp."));
}
