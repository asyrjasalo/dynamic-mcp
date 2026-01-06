use crate::config::McpServerConfig;
use crate::proxy::types::{JsonRpcRequest, JsonRpcResponse};
use anyhow::{Context, Result};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct StdioTransport {
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl StdioTransport {
    pub async fn new(config: &McpServerConfig) -> Result<Self> {
        match config {
            McpServerConfig::Stdio { command, args, env, .. } => {
                let mut cmd = Command::new(command);
                
                if let Some(args) = args {
                    cmd.args(args);
                }
                
                if let Some(env_vars) = env {
                    cmd.envs(env_vars);
                }
                
                cmd.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit());
                
                let mut child = cmd.spawn()
                    .with_context(|| format!("Failed to spawn command: {}", command))?;
                
                let stdin = child.stdin.take()
                    .context("Failed to capture stdin")?;
                let stdout = child.stdout.take()
                    .context("Failed to capture stdout")?;
                
                Ok(Self {
                    child,
                    stdin: Arc::new(Mutex::new(stdin)),
                    stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
                })
            }
            _ => anyhow::bail!("Only stdio transport is supported in Phase 1"),
        }
    }
    
    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let request_json = serde_json::to_string(request)?;
        
        // Send request
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(request_json.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        }
        
        // Read response
        let mut line = String::new();
        {
            let mut stdout = self.stdout.lock().await;
            stdout.read_line(&mut line).await?;
        }
        
        let response: JsonRpcResponse = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse response: {}", line))?;
        
        Ok(response)
    }
    
    pub async fn close(&mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }
}
