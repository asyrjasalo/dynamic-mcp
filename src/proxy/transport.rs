use crate::config::McpServerConfig;
use crate::proxy::types::{JsonRpcRequest, JsonRpcResponse};
use anyhow::{Context, Result};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

pub struct StdioTransport {
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
}

impl StdioTransport {
    pub async fn new(command: &str, args: Option<&Vec<String>>, env: Option<&std::collections::HashMap<String, String>>) -> Result<Self> {
        let mut cmd = Command::new(command);
        
        if let Some(args) = args {
            cmd.args(args);
        }
        
        if let Some(env_vars) = env {
            cmd.envs(env_vars);
        }
        
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        
        let mut child = cmd.spawn()
            .with_context(|| format!("Failed to spawn command: {}", command))?;
        
        let stdin = child.stdin.take()
            .context("Failed to capture stdin")?;
        let stdout = child.stdout.take()
            .context("Failed to capture stdout")?;
        
        Ok(Self {
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
        })
    }
    
    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let request_json = serde_json::to_string(request)?;
        
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(request_json.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
        }
        
        let mut stdout = self.stdout.lock().await;
        loop {
            let mut line = String::new();
            let bytes_read = stdout.read_line(&mut line).await?;
            
            if bytes_read == 0 {
                anyhow::bail!("Connection closed before receiving response");
            }
            
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(trimmed) {
                return Ok(response);
            }
        }
    }
    
    pub async fn close(&mut self) -> Result<()> {
        let mut child = self.child.lock().await;
        child.kill().await?;
        Ok(())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.try_lock() {
            let _ = child.start_kill();
        }
    }
}

pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
    headers: std::collections::HashMap<String, String>,
}

impl HttpTransport {
    pub async fn new(url: &str, headers: Option<&std::collections::HashMap<String, String>>) -> Result<Self> {
        let headers_map = headers.cloned().unwrap_or_default();
        
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
            headers: headers_map,
        })
    }
    
    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let mut req = self.client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");
        
        for (key, value) in &self.headers {
            req = req.header(key, value);
        }
        
        let response = req
            .json(request)
            .send()
            .await
            .context("Failed to send HTTP request")?;
        
        let response_text = response.text().await
            .context("Failed to read HTTP response")?;
        
        let json_response: JsonRpcResponse = serde_json::from_str(&response_text)
            .with_context(|| format!("Failed to parse HTTP response as JSON: {}", response_text))?;
        
        Ok(json_response)
    }
    
    pub async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct SseTransport {
    transport: Arc<Mutex<rmcp::transport::StreamableHttpClientTransport<reqwest::Client>>>,
}

impl SseTransport {
    pub async fn new(url: &str, headers: Option<&std::collections::HashMap<String, String>>) -> Result<Self> {
        let mut config = rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig::with_uri(url);
        
        if let Some(headers_map) = headers {
            if let Some(auth) = headers_map.get("Authorization") {
                config.auth_header = Some(auth.clone());
            }
        }
        
        Ok(Self {
            transport: Arc::new(Mutex::new(
                rmcp::transport::StreamableHttpClientTransport::with_client(
                    reqwest::Client::new(),
                    config,
                )
            )),
        })
    }
    
    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        use rmcp::transport::Transport as RmcpTransport;
        
        let message = serde_json::to_value(request)?;
        let client_message: rmcp::model::ClientJsonRpcMessage = serde_json::from_value(message)?;
        
        let response_message = {
            let mut transport = self.transport.lock().await;
            
            transport.send(client_message).await
                .context("Failed to send request via SSE transport")?;
            
            transport.receive().await
                .ok_or_else(|| anyhow::anyhow!("Transport closed unexpectedly"))?
        };
        
        let response_value = serde_json::to_value(response_message)?;
        let response: JsonRpcResponse = serde_json::from_value(response_value)?;
        
        Ok(response)
    }
    
    pub async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

pub enum Transport {
    Stdio(StdioTransport),
    Http(HttpTransport),
    Sse(SseTransport),
}

impl Transport {
    pub async fn new(config: &McpServerConfig) -> Result<Self> {
        match config {
            McpServerConfig::Stdio { command, args, env, .. } => {
                let transport = StdioTransport::new(command, args.as_ref(), env.as_ref()).await?;
                Ok(Transport::Stdio(transport))
            }
            McpServerConfig::Http { url, headers, .. } => {
                let transport = HttpTransport::new(url, headers.as_ref()).await?;
                Ok(Transport::Http(transport))
            }
            McpServerConfig::Sse { url, headers, .. } => {
                let transport = SseTransport::new(url, headers.as_ref()).await?;
                Ok(Transport::Sse(transport))
            }
        }
    }
    
    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        match self {
            Transport::Stdio(t) => t.send_request(request).await,
            Transport::Http(t) => t.send_request(request).await,
            Transport::Sse(t) => t.send_request(request).await,
        }
    }
    
    pub async fn close(&mut self) -> Result<()> {
        match self {
            Transport::Stdio(t) => t.close().await,
            Transport::Http(t) => t.close().await,
            Transport::Sse(t) => t.close().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_transport_creation() {
        let config = McpServerConfig::Http {
            description: "Test HTTP server".to_string(),
            url: "http://localhost:8080/mcp".to_string(),
            headers: None,
        };
        
        let result = Transport::new(&config).await;
        assert!(result.is_ok(), "HTTP transport creation should succeed");
    }

    #[tokio::test]
    async fn test_http_transport_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
        
        let config = McpServerConfig::Http {
            description: "Test HTTP server with auth".to_string(),
            url: "http://localhost:8080/mcp".to_string(),
            headers: Some(headers),
        };
        
        let result = Transport::new(&config).await;
        assert!(result.is_ok(), "HTTP transport with headers should succeed");
    }

    #[tokio::test]
    async fn test_sse_transport_creation() {
        let config = McpServerConfig::Sse {
            description: "Test SSE server".to_string(),
            url: "http://localhost:8080/sse".to_string(),
            headers: None,
        };
        
        let result = Transport::new(&config).await;
        assert!(result.is_ok(), "SSE transport creation should succeed");
    }

    #[tokio::test]
    async fn test_sse_transport_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
        
        let config = McpServerConfig::Sse {
            description: "Test SSE server with auth".to_string(),
            url: "http://localhost:8080/sse".to_string(),
            headers: Some(headers),
        };
        
        let result = Transport::new(&config).await;
        assert!(result.is_ok(), "SSE transport with headers should succeed");
    }

    #[tokio::test]
    async fn test_stdio_transport_still_works() {
        let config = McpServerConfig::Stdio {
            description: "Test stdio server".to_string(),
            command: "echo".to_string(),
            args: Some(vec!["test".to_string()]),
            env: None,
        };
        
        let result = Transport::new(&config).await;
        assert!(result.is_ok(), "Stdio transport should still work");
    }

    #[test]
    fn test_transport_variants_exist() {
        use std::mem::discriminant;
        
        let http_config = McpServerConfig::Http {
            description: "".to_string(),
            url: "http://test".to_string(),
            headers: None,
        };
        
        let sse_config = McpServerConfig::Sse {
            description: "".to_string(),
            url: "http://test".to_string(),
            headers: None,
        };
        
        let stdio_config = McpServerConfig::Stdio {
            description: "".to_string(),
            command: "test".to_string(),
            args: None,
            env: None,
        };
        
        assert!(discriminant(&http_config) != discriminant(&sse_config));
        assert!(discriminant(&http_config) != discriminant(&stdio_config));
        assert!(discriminant(&sse_config) != discriminant(&stdio_config));
    }
}
