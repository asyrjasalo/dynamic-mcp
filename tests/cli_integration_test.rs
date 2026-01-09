use std::process::Command;

#[test]
fn test_project_builds() {
    let output = Command::new("cargo")
        .args(["build", "--quiet"])
        .output()
        .expect("Failed to build");

    assert!(output.status.success(), "Build should succeed");
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
    assert!(stdout.contains("import") || stdout.contains("Commands"));
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
