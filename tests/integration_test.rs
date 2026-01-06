use std::process::Command;

#[test]
fn test_project_builds() {
    let output = Command::new("cargo")
        .args(&["build", "--quiet"])
        .output()
        .expect("Failed to build");
    
    assert!(output.status.success(), "Build should succeed");
}

#[test]
fn test_config_example_exists() {
    assert!(std::path::Path::new("config.example.json").exists(), 
            "config.example.json should exist");
}

#[test]
fn test_binary_exists_after_build() {
    let _ = Command::new("cargo")
        .args(&["build"])
        .output()
        .expect("Failed to build");
    
    let binary_path = if cfg!(windows) {
        "target/debug/modular-mcp.exe"
    } else {
        "target/debug/modular-mcp"
    };
    
    assert!(std::path::Path::new(binary_path).exists(), 
            "Binary should exist after build");
}
