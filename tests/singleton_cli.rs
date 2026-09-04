//! Black-box CLI tests for the v1.8.0 singleton / double-launch detection flags.
//!
//! These run the compiled `dmcp` binary (via `CARGO_BIN_EXE_dmcp`) and assert on
//! process exit behavior, without linking against the crate internals.
use std::process::Command;
use std::time::Duration;

/// `--no-evict` must only be accepted with `--transport http`. With any other
/// transport it should fail fast (exit non-zero) and explain why.
#[test]
fn no_evict_rejected_without_http_transport() {
    let bin = env!("CARGO_BIN_EXE_dmcp");
    for transport in ["stdio", "both"] {
        let out = Command::new(bin)
            .args([
                "--transport",
                transport,
                "--no-evict",
                "--http-endpoint",
                "127.0.0.1:38082/dynamic-mcp",
                "tests/fixtures/singleton_dummy.json",
            ])
            .output()
            .expect("failed to spawn dmcp");
        assert!(
            !out.status.success(),
            "`--no-evict` with `--transport {transport}` should fail (exit non-zero)"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("--no-evict"),
            "error for `--transport {transport}` should mention --no-evict, got: {stderr}"
        );
    }
}

/// `--no-evict` with `--transport http` passes validation and proceeds to run
/// (it does not fail fast). We confirm it is still alive shortly after start.
#[test]
fn no_evict_accepted_with_http_transport() {
    let bin = env!("CARGO_BIN_EXE_dmcp");
    let mut child = Command::new(bin)
        .args([
            "--transport",
            "http",
            "--no-evict",
            "--http-endpoint",
            "127.0.0.1:38082/dynamic-mcp",
            "tests/fixtures/singleton_dummy.json",
        ])
        .spawn()
        .expect("failed to spawn dmcp");
    // Give it a moment: if validation had rejected it, the process would have
    // exited already with a --no-evict error in stderr.
    std::thread::sleep(Duration::from_millis(1500));
    let status = child.try_wait().expect("failed to poll dmcp");
    assert!(
        status.is_none(),
        "`--no-evict` with `--transport http` should pass validation and keep running"
    );
    let _ = child.kill();
    let _ = child.wait();
}
