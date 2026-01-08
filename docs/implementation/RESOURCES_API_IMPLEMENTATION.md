# Resources API Proxying Implementation Guide

> **Date**: January 8, 2026
> **Status**: 🔍 Analysis Complete - Ready for Implementation
> **Priority**: 🟢 LOW (Optional per MCP spec)
> **Complexity**: Medium (~800-1200 LOC across 4 modules)

## Executive Summary

Implementing Resources API proxying would add support for `resources/list` and `resources/read` to dynamic-mcp, allowing downstream clients to access file-like resources exposed by upstream MCP servers.

**Current Status**: Not implemented (flagged as optional in MCP_SPEC_COMPLIANCE.md)

**Scope**: Moderate feature addition requiring changes to:
1. **Type definitions** (`src/proxy/types.rs`) - Add resource data structures
2. **Client proxy logic** (`src/proxy/client.rs`) - Forward resources/list and resources/read calls
3. **Server handler** (`src/server.rs`) - Add request routing for resource methods
4. **Tests** - Add integration and unit test coverage

---

## 1. Official MCP Specification Reference

### 1.1 Required Methods

Per https://modelcontextprotocol.io/specification/2025-11-25/server/resources:

#### `resources/list` Request
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "resources/list",
  "params": {
    "cursor": "optional-cursor-value"
  }
}
```

**Response Format**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "resources": [
      {
        "uri": "file:///project/src/main.rs",
        "name": "main.rs",
        "title": "Rust Software Application Main File",
        "description": "Primary application entry point",
        "mimeType": "text/x-rust",
        "icons": [...],
        "annotations": {
          "audience": ["user", "assistant"],
          "priority": 0.8,
          "lastModified": "2025-01-12T15:00:58Z"
        }
      }
    ],
    "nextCursor": "next-page-cursor"
  }
}
```

#### `resources/read` Request
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "resources/read",
  "params": {
    "uri": "file:///project/src/main.rs"
  }
}
```

**Response Format**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "contents": [
      {
        "uri": "file:///project/src/main.rs",
        "mimeType": "text/x-rust",
        "text": "fn main() { ... }"
      }
    ]
  }
}
```

### 1.2 Optional Methods

- `resources/templates/list` - Resource templates with URI templates (RFC 6570)
- Notifications:
  - `notifications/resources/list_changed` - List changed notification
  - `notifications/resources/updated` - Resource update notification
- `resources/subscribe` / `resources/unsubscribe` - Subscription support

### 1.3 Capabilities Declaration

**Initialization Response Must Include** (in `src/server.rs`):
```json
{
  "capabilities": {
    "tools": {},
    "resources": {
      "subscribe": false,
      "listChanged": false
    }
  }
}
```

### 1.4 Error Codes

Standard JSON-RPC errors:
- `-32002`: Resource not found
- `-32603`: Internal error

---

## 2. Implementation Architecture

### 2.1 Data Flow

```
MCP Client (downstream)
    ↓
resources/list → dynamic-mcp server.rs
    ↓
Route to specific group → client.rs proxy
    ↓
transport.rs → Upstream MCP Server
    ↓
Response flows back through same path
```

### 2.2 Required Changes by Module

| Module | Changes | Impact |
|--------|---------|--------|
| `src/proxy/types.rs` | +3 new types (Resource, ResourceContent, Annotation) | ~100 LOC |
| `src/proxy/client.rs` | Add `proxy_resources_list()` and `proxy_resources_read()` methods | ~80 LOC |
| `src/server.rs` | Route `resources/list` and `resources/read` to client | ~120 LOC |
| `tests/` | Add integration tests for both endpoints | ~150 LOC |

**Total Estimated LOC**: ~450 LOC (excluding tests)

---

## 3. Implementation Steps

### Phase 1: Type Definitions

**File**: `src/proxy/types.rs`

**Add Structs**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<ResourceIcon>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceIcon {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>, // ["user", "assistant"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>, // 0.0 - 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>, // ISO 8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>, // base64-encoded binary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}
```

**Tests to Add**:
- Serialization roundtrip for Resource
- Serialization roundtrip for ResourceContent
- Optional field handling (title, description, etc.)

---

### Phase 2: Client Proxy Methods

**File**: `src/proxy/client.rs`

**Add Methods to `ModularMcpClient`**:

```rust
/// Proxy resources/list to the appropriate upstream server
pub async fn proxy_resources_list(
    &self,
    group_name: &str,
    cursor: Option<String>,
) -> Result<JsonRpcResponse> {
    let group_state = self.groups.get(group_name)
        .ok_or_else(|| anyhow::anyhow!("Group not found: {}", group_name))?;

    match group_state {
        GroupState::Connected { transport, .. } => {
            let request = JsonRpcRequest::new(4, "resources/list");
            let request = if let Some(cursor) = cursor {
                request.with_params(json!({ "cursor": cursor }))
            } else {
                request
            };

            transport.send_request(&request)
                .await
                .context("Failed to list resources from upstream server")
        }
        GroupState::Failed { .. } => {
            anyhow::bail!("Group {} is not connected", group_name)
        }
    }
}

/// Proxy resources/read to the appropriate upstream server
pub async fn proxy_resources_read(
    &self,
    group_name: &str,
    uri: String,
) -> Result<JsonRpcResponse> {
    let group_state = self.groups.get(group_name)
        .ok_or_else(|| anyhow::anyhow!("Group not found: {}", group_name))?;

    match group_state {
        GroupState::Connected { transport, .. } => {
            let request = JsonRpcRequest::new(5, "resources/read")
                .with_params(json!({ "uri": uri }));

            transport.send_request(&request)
                .await
                .context("Failed to read resource from upstream server")
        }
        GroupState::Failed { .. } => {
            anyhow::bail!("Group {} is not connected", group_name)
        }
    }
}
```

**Tests to Add**:
- Test successful resource list from connected group
- Test resource list from failed group (error case)
- Test successful resource read
- Test resource read with invalid URI (error propagation)
- Test cursor pagination handling

---

### Phase 3: Server Request Routing

**File**: `src/server.rs`

**Modify `handle_request()` method** (currently ~26-41):

```rust
pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => self.handle_initialize(request).await,
        "tools/list" => self.handle_list_tools(request).await,
        "tools/call" => self.handle_call_tool(request).await,
        "resources/list" => self.handle_resources_list(request).await,  // NEW
        "resources/read" => self.handle_resources_read(request).await,   // NEW
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}
```

**Add New Handlers**:

```rust
async fn handle_resources_list(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let client = self.client.read().await;

    // Extract group name from params - requires client to specify which group
    // Two options for UX:
    // Option A: Required "group" param
    // Option B: Try all connected groups and aggregate results

    let group_name = match request.params.as_ref()
        .and_then(|p| p.get("group"))
        .and_then(|g| g.as_str()) {
        Some(name) => name.to_string(),
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing required parameter: group".to_string(),
                    data: None,
                }),
            };
        }
    };

    let cursor = request.params
        .as_ref()
        .and_then(|p| p.get("cursor"))
        .and_then(|c| c.as_str())
        .map(String::from);

    match client.proxy_resources_list(&group_name, cursor).await {
        Ok(response) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: response.result,
            error: response.error,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to list resources: {}", e),
                data: None,
            }),
        },
    }
}

async fn handle_resources_read(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    let client = self.client.read().await;

    let (group_name, uri) = match request.params.as_ref() {
        Some(params) => {
            let group = params.get("group")
                .and_then(|g| g.as_str())
                .map(String::from);
            let uri = params.get("uri")
                .and_then(|u| u.as_str())
                .map(String::from);

            match (group, uri) {
                (Some(g), Some(u)) => (g, u),
                _ => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: "Missing required parameters: group, uri".to_string(),
                            data: None,
                        }),
                    };
                }
            }
        }
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Missing params object".to_string(),
                    data: None,
                }),
            };
        }
    };

    match client.proxy_resources_read(&group_name, uri).await {
        Ok(response) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: response.result,
            error: response.error,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: format!("Failed to read resource: {}", e),
                data: None,
            }),
        },
    }
}
```

**Update `handle_initialize()` to advertise capabilities**:

```rust
async fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {
                    "subscribe": false,
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": self.name,
                "version": self.version
            }
        })),
        error: None,
    }
}
```

**Tests to Add**:
- Test initialize response includes resources capability
- Test resources/list without group parameter (error)
- Test resources/list with valid group (success)
- Test resources/read parameter validation
- Test upstream server error propagation

---

### Phase 4: Documentation Updates

**Update**: `docs/MCP_SPEC_COMPLIANCE.md`

Change from:
```markdown
### Resources API
**Status**: ❌ Not implemented
**Priority**: 🟢 LOW
**Notes**: Not required for tool-only proxying (current use case).
```

To:
```markdown
### Resources API
**Status**: ✅ Implemented (v1.3.0)
**Priority**: 🟢 LOW
**Implementation**: `resources/list` and `resources/read` proxying

**Features**:
- ✅ List resources from upstream servers
- ✅ Read resource contents (text and binary)
- ✅ Pagination support (cursor-based)
- ✅ Proper error handling (-32002 for not found, -32603 for internal)
- ⏳ Subscriptions not implemented (optional)
- ⏳ Templates not implemented (optional)

**Usage**:
```json
{
  "method": "resources/list",
  "params": {
    "group": "filesystem",
    "cursor": "optional-pagination-cursor"
  }
}
```

```json
{
  "method": "resources/read",
  "params": {
    "group": "filesystem",
    "uri": "file:///path/to/file"
  }
}
```
```

**Update**: `docs/implementation/STATUS.md`

Add to "Completed Features":
- Resources API proxying (resources/list, resources/read)

Update metrics:
- LOC: ~4,765 → ~5,215 (+450)
- Test count: 74 → 84+ (depends on test coverage)

---

## 4. Design Decisions & Alternatives

### 4.1 Group Parameter Requirement

**Decision**: Require explicit `group` parameter in `resources/list` and `resources/read`

**Rationale**:
1. Mirrors `tools/call` pattern (requires group selection)
2. Prevents ambiguity when multiple groups expose same resources
3. Enables granular access control per group

**Alternative 1**: Auto-discover and aggregate from all groups
- **Pro**: More transparent to clients
- **Con**: Resource URI collisions, unclear which group serves which resource

**Alternative 2**: Global resource discovery across all groups
- **Pro**: Single unified view
- **Con**: Complex deduplication, slower list operations

### 4.2 Error Handling Strategy

**Decision**: Propagate upstream errors with standard JSON-RPC codes

```rust
- Upstream error → Pass through with original code
- Server not connected → -32603 (Internal error)
- Invalid parameters → -32602 (Invalid params)
- Resource not found → -32002 (Resource not found, if specified by upstream)
```

### 4.3 Optional Features Not Implemented

**Deferred** (Optional per MCP spec, can be added later):

1. **Subscriptions** (`resources/subscribe`, `resources/unsubscribe`)
   - Requires notification infrastructure (differs from one-off requests)
   - Low demand in current use cases

2. **List Change Notifications** (`notifications/resources/list_changed`)
   - Requires server-initiated notification capability
   - Can be added as separate feature

3. **Resource Templates** (`resources/templates/list`)
   - URI templates with parameters (RFC 6570)
   - Less common than basic resource list/read

4. **Binary Content Streaming**
   - Current implementation uses base64 encoding
   - Large files may impact performance
   - Can optimize later with chunking if needed

---

## 5. Testing Strategy

### 5.1 Unit Tests (in `src/proxy/types.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "file:///test.txt".to_string(),
            name: "test.txt".to_string(),
            title: Some("Test File".to_string()),
            description: None,
            mime_type: Some("text/plain".to_string()),
            icons: None,
            annotations: None,
        };

        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["uri"], "file:///test.txt");
        assert_eq!(json["name"], "test.txt");
    }

    #[test]
    fn test_resource_content_with_text() {
        let content = ResourceContent {
            uri: "file:///test.txt".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: Some("Hello, world!".to_string()),
            blob: None,
            annotations: None,
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["text"], "Hello, world!");
        assert!(json["blob"].is_null());
    }

    #[test]
    fn test_resource_annotations() {
        let annotations = ResourceAnnotations {
            audience: Some(vec!["user".to_string(), "assistant".to_string()]),
            priority: Some(0.8),
            last_modified: Some("2025-01-12T15:00:58Z".to_string()),
        };

        let json = serde_json::to_value(&annotations).unwrap();
        assert_eq!(json["priority"], 0.8);
    }
}
```

### 5.2 Integration Tests (in `tests/resources_integration_test.rs`)

```rust
#[tokio::test]
async fn test_resources_list_success() {
    // Setup: Create mock upstream server that exposes resources
    let server = MockMcpServer::new()
        .with_resources(vec![
            Resource {
                uri: "file:///test.txt".to_string(),
                // ...
            },
        ]);

    let response = client.proxy_resources_list("test_group", None).await;

    assert!(response.is_ok());
    let resp = response.unwrap();
    assert!(resp.result.is_some());
}

#[tokio::test]
async fn test_resources_list_invalid_group() {
    let response = client.proxy_resources_list("nonexistent", None).await;

    assert!(response.is_err());
}

#[tokio::test]
async fn test_resources_read_success() {
    let response = client
        .proxy_resources_read("test_group", "file:///test.txt".to_string())
        .await;

    assert!(response.is_ok());
}
```

### 5.3 Server Handler Tests

- Test `handle_resources_list()` routing
- Test parameter validation
- Test error propagation
- Test initialization response includes resources capability

---

## 6. Compatibility & Backwards Compatibility

### 6.1 Backwards Compatibility

**No breaking changes**:
- New optional capability in `initialize` response
- New methods don't affect existing tools/list or tools/call
- Existing clients continue to work without modification

### 6.2 Upstream Server Compatibility

**Requirements**:
- Upstream server MUST declare `resources` capability in `initialize`
- If capability missing, resources calls will fail gracefully

**Implementation**:
```rust
// In future: Check upstream server capabilities before proxying
let upstream_caps = /* extract from initialize response */
if !upstream_caps.get("resources").is_some() {
    return JsonRpcResponse::error(-32601, "Upstream server does not support resources");
}
```

---

## 7. Future Enhancements

### 7.1 Phase 2 Features (Optional)

1. **Subscriptions & Notifications**
   - Requires notification queue infrastructure
   - Estimated: 200-300 LOC

2. **Resource Templates**
   - Add `resources/templates/list`
   - URI template expansion with parameters
   - Estimated: 150-200 LOC

3. **Binary Content Streaming**
   - Chunked transfer for large files
   - Range request support
   - Estimated: 100-150 LOC

4. **Resource Caching**
   - Cache frequently accessed resources
   - TTL-based invalidation
   - Estimated: 80-120 LOC

### 7.2 Optimization Opportunities

1. **Lazy Resource Loading**
   - Don't load resource contents until requested
   - Current design already does this

2. **Resource Size Limits**
   - Add configurable max resource size
   - Prevent OOM with large files

3. **Pagination Efficiency**
   - Batch multiple page requests
   - Resume from arbitrary cursors

---

## 8. Implementation Checklist

- [ ] **Phase 1: Types**
  - [ ] Add Resource struct
  - [ ] Add ResourceContent struct
  - [ ] Add ResourceAnnotations struct
  - [ ] Add ResourceIcon struct
  - [ ] Write serialization tests
  - [ ] Run `cargo test` (types pass)

- [ ] **Phase 2: Client Proxy**
  - [ ] Implement `proxy_resources_list()`
  - [ ] Implement `proxy_resources_read()`
  - [ ] Write unit tests
  - [ ] Run `cargo test` (client tests pass)
  - [ ] Run `cargo clippy` (no warnings)

- [ ] **Phase 3: Server Routing**
  - [ ] Add `handle_resources_list()` handler
  - [ ] Add `handle_resources_read()` handler
  - [ ] Update `handle_request()` match statement
  - [ ] Update `handle_initialize()` to advertise capability
  - [ ] Write handler tests
  - [ ] Run full `cargo test` suite
  - [ ] Run `cargo clippy` (no warnings)

- [ ] **Phase 4: Documentation**
  - [ ] Update `MCP_SPEC_COMPLIANCE.md`
  - [ ] Update `docs/implementation/STATUS.md`
  - [ ] Update `CHANGELOG.md`
  - [ ] Update `README.md` (optional, if user-facing)

- [ ] **Phase 5: Verification**
  - [ ] `cargo build --release` (no errors)
  - [ ] `cargo test` (all tests pass, including new ones)
  - [ ] `cargo clippy` (zero warnings)
  - [ ] Manual testing with real upstream server (if available)
  - [ ] Integration test with mocked upstream

---

## 9. Estimated Effort

| Phase | Task | Estimated Time |
|-------|------|-----------------|
| 1 | Type definitions & tests | 1-2 hours |
| 2 | Client proxy methods | 2-3 hours |
| 3 | Server request handlers | 2-3 hours |
| 4 | Documentation updates | 1 hour |
| 5 | Testing & verification | 1-2 hours |
| **Total** | | **~8-12 hours** |

---

## 10. References

- **MCP Specification (Resources)**: https://modelcontextprotocol.io/specification/2025-11-25/server/resources
- **RFC 3986 (URI)**: https://datatracker.ietf.org/doc/html/rfc3986
- **RFC 6570 (URI Templates)**: https://datatracker.ietf.org/doc/html/rfc6570
- **Current Implementation**: `src/server.rs:26-41`, `src/proxy/client.rs`
- **Spec Compliance Doc**: `docs/MCP_SPEC_COMPLIANCE.md` (section 3.2)

---

## Appendix A: Code Snippets for Copy-Paste

### A.1 Full Resource Type Definition

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Vec<ResourceIcon>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceIcon {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}
```

---

**Document Version**: 1.0
**Status**: 🔍 Complete Analysis
**Next Step**: Execute Phase 1 (Type Definitions)
