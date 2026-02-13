use crate::auth::OAuthClient;
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
    pub async fn new(
        command: &str,
        args: Option<&Vec<String>>,
        env: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<Self> {
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

        // Create process in new process group for proper cleanup
        #[cfg(unix)]
        {
            unsafe {
                cmd.pre_exec(|| {
                    libc::setpgid(0, 0);
                    Ok(())
                });
            }
        }

        #[cfg(windows)]
        {
            // CREATE_NEW_PROCESS_GROUP = 0x00000200
            // This allows us to send Ctrl+C/Ctrl+Break to the entire process tree
            cmd.creation_flags(0x00000200);
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn command: {}", command))?;

        let stdin = child.stdin.take().context("Failed to capture stdin")?;
        let stdout = child.stdout.take().context("Failed to capture stdout")?;

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

            if !trimmed.starts_with('{') {
                tracing::debug!("Skipping non-JSON output: {}", trimmed);
                continue;
            }

            match serde_json::from_str::<JsonRpcResponse>(trimmed) {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if value.get("error").is_some() {
                            let id_value = value.get("id").cloned();
                            if id_value.is_none()
                                || matches!(id_value, Some(serde_json::Value::Null))
                            {
                                tracing::warn!(
                                    "Received error response with null id, skipping: {}",
                                    trimmed
                                );
                                continue;
                            }
                            return Ok(JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: id_value.unwrap(),
                                result: None,
                                error: serde_json::from_value(value.get("error").unwrap().clone())
                                    .ok(),
                            });
                        }
                    }
                    tracing::warn!("Failed to parse JSON-RPC response: {}. Raw: {}", e, trimmed);
                    continue;
                }
            }
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        let mut child = self.child.lock().await;

        // Attempt graceful shutdown first, then force kill
        #[cfg(unix)]
        {
            if let Some(pid) = child.id() {
                unsafe {
                    // Send SIGTERM to the entire process group
                    libc::kill(-(pid as i32), libc::SIGTERM);
                }
            }
        }

        #[cfg(windows)]
        {
            if let Some(pid) = child.id() {
                // Send Ctrl+C event to process group for graceful shutdown
                unsafe {
                    use windows_sys::Win32::System::Console::GenerateConsoleCtrlEvent;
                    // CTRL_C_EVENT = 0, pid as process group ID
                    let _ = GenerateConsoleCtrlEvent(0, pid);
                }

                // Give process brief time to handle Ctrl+C (100ms)
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        // Force kill if still running
        child.kill().await?;
        Ok(())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.try_lock() {
            // Force kill on drop (cleanup)
            #[cfg(unix)]
            {
                if let Some(pid) = child.id() {
                    unsafe {
                        // Send SIGKILL to entire process group
                        libc::kill(-(pid as i32), libc::SIGKILL);
                    }
                }
            }

            #[cfg(windows)]
            {
                if let Some(pid) = child.id() {
                    // Force terminate the process and its children
                    unsafe {
                        use windows_sys::Win32::Foundation::CloseHandle;
                        use windows_sys::Win32::System::Threading::{
                            OpenProcess, TerminateProcess, PROCESS_TERMINATE,
                        };

                        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
                        if !handle.is_null() {
                            let _ = TerminateProcess(handle, 1);
                            CloseHandle(handle);
                        }
                    }
                }
            }

            let _ = child.start_kill();
        }
    }
}

pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
    headers: std::collections::HashMap<String, String>,
    session_id: Arc<Mutex<Option<String>>>,
    protocol_version: Arc<Mutex<String>>,
}

impl HttpTransport {
    pub async fn new(
        url: &str,
        headers: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<Self> {
        let headers_map = headers.cloned().unwrap_or_default();

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(2)
            .build()?;

        Ok(Self {
            client,
            url: url.to_string(),
            headers: headers_map,
            session_id: Arc::new(Mutex::new(None)),
            protocol_version: Arc::new(Mutex::new("2024-11-05".to_string())),
        })
    }

    fn set_session_id(&self, session_id: String) {
        if let Ok(mut sid) = self.session_id.try_lock() {
            *sid = Some(session_id);
        }
    }

    pub fn set_protocol_version(&self, version: String) {
        if let Ok(mut pv) = self.protocol_version.try_lock() {
            *pv = version;
        }
    }

    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let protocol_ver = if let Ok(pv) = self.protocol_version.try_lock() {
            pv.clone()
        } else {
            "2024-11-05".to_string()
        };

        let mut req = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("MCP-Protocol-Version", protocol_ver);

        // Add session ID if initialized
        if let Ok(session_id_lock) = self.session_id.try_lock() {
            if let Some(ref session_id) = *session_id_lock {
                req = req.header("MCP-Session-Id", session_id);
            }
        }

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        let response = req
            .json(request)
            .send()
            .await
            .context("Failed to send HTTP request")?;

        let status = response.status();

        // Check Content-Type header for SSE detection
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        let response_text = response
            .text()
            .await
            .context("Failed to read HTTP response")?;

        if !status.is_success() {
            anyhow::bail!(
                "HTTP request failed with status {}: {}",
                status,
                response_text
            );
        }

        // Handle both JSON and SSE responses according to MCP Streamable HTTP spec
        // Server can return either Content-Type: application/json or Content-Type: text/event-stream
        let json_response: JsonRpcResponse = if content_type.contains("text/event-stream")
            || response_text.trim_start().starts_with("event:")
            || response_text.trim_start().starts_with("data:")
        {
            // Parse SSE format: event: message\ndata: {...}
            let mut data_content = String::new();

            for line in response_text.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    data_content.push_str(data);
                } else if line.starts_with("event:") {
                    // Skip event lines
                    continue;
                } else if line.starts_with("data:") {
                    // Handle data without space after colon
                    if let Some(data) = line.strip_prefix("data:") {
                        data_content.push_str(data.trim());
                    }
                }
            }

            if data_content.is_empty() {
                anyhow::bail!("No data found in SSE response: {}", response_text);
            }

            // Parse the JSON data from SSE
            serde_json::from_str(&data_content)
                .with_context(|| format!("Failed to parse SSE data as JSON: {}", data_content))?
        } else {
            // Parse plain JSON response
            serde_json::from_str(&response_text).with_context(|| {
                format!("Failed to parse HTTP response as JSON: {}", response_text)
            })?
        };

        // Check for JSON-RPC errors in the response
        if let Some(error) = &json_response.error {
            anyhow::bail!("JSON-RPC error (code {}): {}", error.code, error.message);
        }

        Ok(json_response)
    }

    pub async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct SseTransport {
    client: reqwest::Client,
    url: String,
    headers: std::collections::HashMap<String, String>,
    session_id: Arc<Mutex<Option<String>>>,
    protocol_version: Arc<Mutex<String>>,
    last_event_id: Arc<Mutex<Option<String>>>,
}

impl SseTransport {
    pub async fn new(
        url: &str,
        headers: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<Self> {
        let headers_map = headers.cloned().unwrap_or_default();

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(2)
            .build()?;

        Ok(Self {
            client,
            url: url.to_string(),
            headers: headers_map,
            session_id: Arc::new(Mutex::new(None)),
            protocol_version: Arc::new(Mutex::new("2024-11-05".to_string())),
            last_event_id: Arc::new(Mutex::new(None)),
        })
    }

    fn set_session_id(&self, session_id: String) {
        if let Ok(mut sid) = self.session_id.try_lock() {
            *sid = Some(session_id);
        }
    }

    pub fn set_protocol_version(&self, version: String) {
        if let Ok(mut pv) = self.protocol_version.try_lock() {
            *pv = version;
        }
    }

    fn set_last_event_id(&self, event_id: String) {
        if let Ok(mut id) = self.last_event_id.try_lock() {
            *id = Some(event_id);
        }
    }

    fn parse_sse_response(&self, sse_text: &str) -> Result<(JsonRpcResponse, Option<String>)> {
        // Parse SSE format: id: <id>\nevent: message\ndata: {...}
        let mut data_content = String::new();
        let mut event_id: Option<String> = None;

        for line in sse_text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(id) = line.strip_prefix("id: ") {
                event_id = Some(id.to_string());
            } else if let Some(data) = line.strip_prefix("data: ") {
                data_content.push_str(data);
            } else if line.starts_with("event:") {
                // Skip event lines
                continue;
            } else if line.starts_with("data:") {
                // Handle data without space after colon
                if let Some(data) = line.strip_prefix("data:") {
                    data_content.push_str(data.trim());
                }
            } else if line.starts_with("id:") {
                // Handle id without space after colon
                if let Some(id) = line.strip_prefix("id:") {
                    event_id = Some(id.trim().to_string());
                }
            }
        }

        if data_content.is_empty() {
            anyhow::bail!("No data found in SSE response: {}", sse_text);
        }

        // Parse the JSON data
        let json_response: JsonRpcResponse = serde_json::from_str(&data_content)
            .with_context(|| format!("Failed to parse SSE data as JSON: {}", data_content))?;

        Ok((json_response, event_id))
    }

    pub async fn send_request(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let protocol_ver = if let Ok(pv) = self.protocol_version.try_lock() {
            pv.clone()
        } else {
            "2024-11-05".to_string()
        };

        let mut req = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("MCP-Protocol-Version", protocol_ver);

        if let Ok(session_id_lock) = self.session_id.try_lock() {
            if let Some(ref session_id) = *session_id_lock {
                req = req.header("MCP-Session-Id", session_id);
            }
        }

        if let Ok(last_event_id_lock) = self.last_event_id.try_lock() {
            if let Some(ref last_event_id) = *last_event_id_lock {
                req = req.header("Last-Event-ID", last_event_id);
            }
        }

        for (key, value) in &self.headers {
            req = req.header(key, value);
        }

        let response = req
            .json(request)
            .send()
            .await
            .context("Failed to send SSE request")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read SSE response")?;

        if !status.is_success() {
            anyhow::bail!(
                "SSE request failed with status {}: {}",
                status,
                response_text
            );
        }

        let (json_response, event_id) = self.parse_sse_response(&response_text)?;

        if let Some(id) = event_id {
            self.set_last_event_id(id);
        }

        // Check for JSON-RPC errors in the response
        if let Some(error) = &json_response.error {
            anyhow::bail!("JSON-RPC error (code {}): {}", error.code, error.message);
        }

        Ok(json_response)
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
    pub async fn new(config: &McpServerConfig, server_name: &str) -> Result<Self> {
        match config {
            McpServerConfig::Stdio {
                command, args, env, ..
            } => {
                let transport = StdioTransport::new(command, args.as_ref(), env.as_ref()).await?;
                Ok(Transport::Stdio(transport))
            }
            McpServerConfig::Http {
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                ..
            } => {
                let mut final_headers = headers.clone().unwrap_or_default();

                if let Some(client_id) = oauth_client_id {
                    let oauth_client = OAuthClient::new()?;
                    let token = oauth_client
                        .authenticate(server_name, url, client_id, oauth_scopes.clone())
                        .await?;

                    final_headers.insert(
                        "Authorization".to_string(),
                        format!("Bearer {}", token.access_token),
                    );

                    tracing::debug!("Added OAuth token to HTTP transport for {}", server_name);
                }

                let transport = HttpTransport::new(url, Some(&final_headers)).await?;
                Ok(Transport::Http(transport))
            }
            McpServerConfig::Sse {
                url,
                headers,
                oauth_client_id,
                oauth_scopes,
                ..
            } => {
                let mut final_headers = headers.clone().unwrap_or_default();

                if let Some(client_id) = oauth_client_id {
                    let oauth_client = OAuthClient::new()?;
                    let token = oauth_client
                        .authenticate(server_name, url, client_id, oauth_scopes.clone())
                        .await?;

                    final_headers.insert(
                        "Authorization".to_string(),
                        format!("Bearer {}", token.access_token),
                    );

                    tracing::debug!("Added OAuth token to SSE transport for {}", server_name);
                }

                let transport = SseTransport::new(url, Some(&final_headers)).await?;
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

    pub fn set_session_id(&self, session_id: String) {
        match self {
            Transport::Stdio(_) => {}
            Transport::Http(t) => t.set_session_id(session_id),
            Transport::Sse(t) => t.set_session_id(session_id),
        }
    }

    pub fn set_protocol_version(&self, version: String) {
        match self {
            Transport::Stdio(_) => {}
            Transport::Http(t) => t.set_protocol_version(version),
            Transport::Sse(t) => t.set_protocol_version(version),
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
    use crate::config::schema::{Features, Timeout};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_http_transport_creation() {
        let config = McpServerConfig::Http {
            description: "Test HTTP server".to_string(),
            url: "http://localhost:8080/mcp".to_string(),
            headers: None,
            oauth_client_id: None,
            oauth_scopes: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let result = Transport::new(&config, "test_server").await;
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
            oauth_client_id: None,
            oauth_scopes: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let result = Transport::new(&config, "test_server").await;
        assert!(result.is_ok(), "HTTP transport with headers should succeed");
    }

    #[tokio::test]
    async fn test_sse_transport_creation() {
        let config = McpServerConfig::Sse {
            description: "Test SSE server".to_string(),
            url: "http://localhost:8080/sse".to_string(),
            headers: None,
            oauth_client_id: None,
            oauth_scopes: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let result = Transport::new(&config, "test_server").await;
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
            oauth_client_id: None,
            oauth_scopes: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let result = Transport::new(&config, "test_server").await;
        assert!(result.is_ok(), "SSE transport with headers should succeed");
    }

    #[tokio::test]
    async fn test_stdio_transport_still_works() {
        let config = McpServerConfig::Stdio {
            description: "Test stdio server".to_string(),
            command: "echo".to_string(),
            args: Some(vec!["test".to_string()]),
            env: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let result = Transport::new(&config, "test_server").await;
        assert!(result.is_ok(), "Stdio transport should still work");
    }

    #[test]
    fn test_transport_variants_exist() {
        use std::mem::discriminant;

        let http_config = McpServerConfig::Http {
            description: "".to_string(),
            url: "http://test".to_string(),
            headers: None,
            oauth_client_id: None,
            oauth_scopes: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let sse_config = McpServerConfig::Sse {
            description: "".to_string(),
            url: "http://test".to_string(),
            headers: None,
            oauth_client_id: None,
            oauth_scopes: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        let stdio_config = McpServerConfig::Stdio {
            description: "".to_string(),
            command: "test".to_string(),
            args: None,
            env: None,
            features: Features::default(),
            enabled: true,
            timeout: Timeout::default(),
        };

        assert!(discriminant(&http_config) != discriminant(&sse_config));
        assert!(discriminant(&http_config) != discriminant(&stdio_config));
        assert!(discriminant(&sse_config) != discriminant(&stdio_config));
    }

    #[tokio::test]
    async fn test_sse_last_event_id_tracking() {
        let transport = SseTransport::new("http://localhost:8080/sse", None)
            .await
            .expect("Failed to create SSE transport");

        let sse_response =
            "id: test-event-123\ndata: {\"jsonrpc\": \"2.0\", \"id\": 1, \"result\": {}}";
        let (_, event_id) = transport
            .parse_sse_response(sse_response)
            .expect("Failed to parse SSE response");

        assert_eq!(event_id, Some("test-event-123".to_string()));
    }

    #[tokio::test]
    async fn test_sse_last_event_id_without_id() {
        let transport = SseTransport::new("http://localhost:8080/sse", None)
            .await
            .expect("Failed to create SSE transport");

        let sse_response = "data: {\"jsonrpc\": \"2.0\", \"id\": 1, \"result\": {}}";
        let (_, event_id) = transport
            .parse_sse_response(sse_response)
            .expect("Failed to parse SSE response");

        assert_eq!(event_id, None);
    }

    #[tokio::test]
    async fn test_sse_last_event_id_storage() {
        let transport = SseTransport::new("http://localhost:8080/sse", None)
            .await
            .expect("Failed to create SSE transport");

        transport.set_last_event_id("event-456".to_string());

        {
            let lock = transport.last_event_id.try_lock();
            assert!(lock.is_ok(), "Failed to acquire lock on last_event_id");
            if let Ok(id_guard) = lock {
                assert_eq!(*id_guard, Some("event-456".to_string()));
            }
        }
    }

    #[tokio::test]
    async fn test_sse_last_event_id_with_compact_format() {
        let transport = SseTransport::new("http://localhost:8080/sse", None)
            .await
            .expect("Failed to create SSE transport");

        let sse_response =
            "id:test-event-789\ndata:{\"jsonrpc\": \"2.0\", \"id\": 1, \"result\": {}}";
        let (_, event_id) = transport
            .parse_sse_response(sse_response)
            .expect("Failed to parse SSE response");

        assert_eq!(event_id, Some("test-event-789".to_string()));
    }
}
