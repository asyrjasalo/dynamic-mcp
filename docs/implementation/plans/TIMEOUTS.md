# MCP Timeout Best Practices

**Last Updated**: 2026-01-12

## Overview

This document captures research findings about MCP timeout best practices, current community efforts to standardize timeouts, and implications for dynamic-mcp.

## Current State of Timeout Standardization

### RFC #1492: Standardize Timeouts Between Servers and Clients

**Status**: Active proposal (not yet merged as of 2026-01-12)

**Source**: https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1492

#### Key Proposal Elements

1. **Responder-declared timeouts**: Servers declare timeout requirements during initialization phase

   - Aligns with MCP's capability negotiation philosophy
   - Provides clients with server timing expectations upfront

2. **Per-method timeout declarations**: Different timeouts for different operations

   - `tools/list`: Expected to be fast
   - `tools/call`: Variable based on tool complexity
   - `resources/list`, `prompts/list`: Separate per-method timeouts
   - Protocol-wide defaults as baseline

3. **Timeout negotiation**:

   - Servers provide recommended timeouts
   - Clients retain control and may impose stricter limits
   - Avoids "impedance mismatch" between expectations and reality

4. **Protocol-wide defaults**: Recommended baseline timeouts (e.g., 60s for most operations)

#### Philosophy

Unlike other protocols (gRPC, HTTP) where requesters set deadlines, MCP's unique initialization phase allows **capability-based timeout declaration**. The server, knowing its typical response times, provides information to help clients choose appropriate timeouts rather than forcing failure through arbitrary limits.

### Opposition and Discussion

- **Requester-defined timeout argument**: Some commenters (e.g., simonrussell) suggest per-request deadlines like gRPC
  - Counter-argument: Requester-declared timeouts don't make operations faster, just ensure failure
  - Responder-declared timeouts increase success rate by informing realistic expectations

## Real-World Timeout Issues

### Vendor Fragmentation

| Implementation | `tools/list` Timeout | Other Operations      | Notes                                  |
| -------------- | -------------------- | --------------------- | -------------------------------------- |
| Anthropic MCP  | 5s                   | 60s for other `/list` | Documented cause of flakiness          |
| TypeScript SDK | Hardcoded            | Hardcoded             | Users requesting configurable timeouts |
| Python SDK     | Different defaults   | Different defaults    | Inconsistent behavior across SDKs      |

### Community Demand for Longer/Configurable Timeouts

**13+ reactions**: [TypeScript SDK #245](https://github.com/modelcontextprotocol/typescript-sdk/issues/245) - requesting configurable timeouts
**21+ reactions**: [Inspector #142](https://github.com/modelcontextprotocol/inspector/issues/142) - requesting configurable timeouts

**Tool-specific issues**:

- [Continue #5325](https://github.com/continuedev/continue/issues/5325), [#7509](https://github.com/continuedev/continue/issues/7509): "We have advanced MCP tools facing timeout problems"
- [LM Studio #727](https://github.com/lmstudio-ai/lmstudio-bug-tracker/issues/727): Fixed, non-configurable timeout prevents MCP tools > default time
- [Claude Code #424](https://github.com/anthropics/claude-code/issues/424): Request for config option / env var for MCP tool call timeout
- [Cline #1306](https://github.com/cline/cline/issues/1306), [#2296](https://github.com/cline/cline/issues/2296): SDK hard-coded timeout, users can't extend
- [Codex #2346](https://github.com/openai/codex/issues/2346): Request to make MCP request timeout configurable
- [Gemini CLI #3130](https://github.com/google-gemini/gemini-cli/issues/3130): MCP Server timeout issues on Windows
- [N8N Community](https://community.n8n.io/t/mcp-server-timeout-configuration): Requesting timeouts up to **15 minutes** for ETL workflows

### Common Use Cases Needing Long Timeouts

- **ETL jobs**: Data extraction, transformation, loading (minutes to hours)
- **Large file conversions**: PDF/image processing (minutes)
- **Multi-step provisioning**: Cloud resource setup (minutes)
- **Batch operations**: Bulk API calls (minutes)
- **Complex analysis**: Codebase search, dependency analysis (minutes)

## Emerging Solutions

### MCP Async Tasks (Experimental)

**Specification**: SEP-1686, introduced in 2025-11-25 spec revision
**Source**: https://workos.com/blog/mcp-async-tasks-ai-agent-workflows

#### Concept

**"Call-now, fetch-later"**: MCP requests return immediately with a durable task handle, while real work continues in background. Results can be polled or subscribed to later.

#### Key Features

1. **Capability negotiation**: Both sides advertise which request types support task augmentation

   ```json
   {
     "capabilities": {
       "tasks": {
         "list": {},
         "cancel": {},
         "requests": {
           "tools": { "call": {} }
         }
       }
     }
   }
   ```

2. **Tool-level control**: Each tool can declare task support

   - `"forbidden"`: Must stay synchronous (default if missing)
   - `"optional"`: Requester may choose sync or task
   - `"required"`: Must use tasks or call is invalid

3. **Task lifecycle**: Small durable state machine

   - `working`: Actively executing
   - `input_required`: Needs additional user input
   - `completed`: Success, result available
   - `failed`: Execution failed
   - `cancelled`: Requester cancelled

4. **Task creation**: Add `task` field with `ttl` (time-to-live)

   ```json
   {
     "jsonrpc": "2.0",
     "id": 1,
     "method": "tools/call",
     "params": {
       "name": "long_running_tool",
       "arguments": { ... },
       "task": { "ttl": 60000 }
     }
   }
   ```

5. **Polling and results**:

   - `tasks/get`: Check task status (respect `pollInterval` hint)
   - `tasks/result`: Fetch final result (blocking until terminal)
   - `tasks/cancel`: Stop in-flight task

#### Implementation Patterns

**Server side**:

- Durable task store (database, job queue, workflow engine)
- Append-only state transitions (no regressions)
- Respect but may override TTL
- Idempotency for retry detection

**Client / agent side**:

- Poll `tasks/get` as source of truth
- Use notifications for UX (speed), polling for correctness
- Parallelize multiple tasks easily

#### Security Considerations

- Task IDs are sensitive capability handles
- Bind tasks to authorization context (user/tenant)
- Filter `tasks/list` by caller's context
- Cryptographically unguessable task IDs if no auth context

## Current dynamic-mcp Timeout Behavior

### Connection Phase Timeouts (with Retries)

These timeouts apply during the initial connection setup and are **automatically retried**:

| Phase                               | Timeout | Retry Count | Retry Backoff | Description                                   |
| ----------------------------------- | ------- | ----------- | ------------- | --------------------------------------------- |
| Transport creation                  | 5s      | 3 attempts  | 2s, 4s, 8s    | Creating HTTP/SSE client or stdio process     |
| Initialize request                  | 5s      | 3 attempts  | 2s, 4s, 8s    | MCP protocol handshake (`initialize` method)  |
| Initialize retry (version mismatch) | 5s      | 3 attempts  | 2s, 4s, 8s    | Retry with server's protocol version          |
| `tools/list`                        | 5s      | 3 attempts  | 2s, 4s, 8s    | Listing available tools during initialization |

**Retry Flow for Connections**:

1. First attempt fails
2. Wait 2 seconds
3. Second attempt
4. Wait 4 seconds
5. Third attempt
6. Wait 8 seconds
7. Final attempt (if all fail, server marked as failed)

**Periodic Background Retry**:

- Failed groups are retried every **30 seconds** indefinitely
- Continues until connection succeeds or config is reloaded
- Allows automatic recovery from transient failures

### MCP Operation Timeouts (No Retries)

These timeouts apply to individual MCP operations and are **NOT retried**:

| Operation        | Timeout | Retries | Error Behavior                      |
| ---------------- | ------- | ------- | ----------------------------------- |
| `tools/call`     | 30s     | **0**   | Returns error immediately to caller |
| `resources/list` | 10s     | **0**   | Returns error immediately to caller |
| `resources/read` | 10s     | **0**   | Returns error immediately to caller |
| `prompts/list`   | 10s     | **0**   | Returns error immediately to caller |
| `prompts/get`    | 10s     | **0**   | Returns error immediately to caller |

**Key Distinction**:

- **Connection timeouts**: Applied during startup, retried up to 3 times with exponential backoff
- **Operation timeouts**: Applied to individual tool/resource/prompt calls, **no automatic retry**
- If an operation times out, the error is returned to the calling agent/LLM without retry

### HTTP/SSE Client Configuration

The underlying HTTP client (reqwest) has these fixed timeouts:

| Setting                  | Value      | Purpose                                        |
| ------------------------ | ---------- | ---------------------------------------------- |
| `connect_timeout`        | 5 seconds  | TCP connection establishment                   |
| `timeout`                | 10 seconds | Overall HTTP request (including data transfer) |
| `pool_idle_timeout`      | 90 seconds | Keep idle connections in pool                  |
| `pool_max_idle_per_host` | 2          | Max idle connections per host                  |

### Summary of Timeout Behavior

**What Gets Retried**:

- ✅ Transport creation (stdio process start, HTTP connection)
- ✅ Initialize handshake
- ✅ Tools/list (during connection setup)

**What Does NOT Get Retried**:

- ❌ Tool calls (`tools/call`)
- ❌ Resources operations (`resources/list`, `resources/read`)
- ❌ Prompts operations (`prompts/list`, `prompts/get`)

**Why This Matters**:

- A slow or temporarily unavailable upstream server causes tool calls to fail immediately
- Network blips during operation execution result in errors to the user
- No graceful degradation for transient failures during normal operation
- Long-running tools (>30s) cannot be used through dynamic-mcp

### Current Gaps

1. **No operation retries**: MCP operations (tool calls, resources, prompts) fail immediately on timeout

   - No graceful degradation for transient failures
   - No retry for network blips

2. **No Async Tasks support**: Long-running operations (ETL, file conversion) can't use dynamic-mcp

   - No "call-now, fetch-later" pattern
   - Users blocked by 30s tool call timeout

3. **Fixed timeouts**: No way to configure timeouts per operation

   - 30s may be too short for complex tools
   - No negotiation with upstream server expectations

## Comparison: dynamic-mcp vs Community Best Practices

| Aspect                | dynamic-mcp   | RFC/Community Best Practices                 |
| --------------------- | ------------- | -------------------------------------------- |
| Timeout source        | Hardcoded     | Server-declared (negotiated)                 |
| Per-method timeouts   | Fixed         | Variable per operation type                  |
| Long-running ops      | Not supported | Async Tasks (experimental)                   |
| Operation retries     | No            | Not specified (but connection retries exist) |
| Timeout configuration | None          | Optional, server-guided                      |

## Recommendations for dynamic-mcp

### Short-term (Non-breaking)

1. **Add configurable timeouts**:

   - Allow users to set operation-specific timeouts via config
   - Provide sensible defaults while allowing customization

2. **Add operation retries**:

   - Retry failed MCP operations (not just connections)
   - Use exponential backoff for transient failures
   - Make retry count configurable

3. **Improve logging**:

   - Distinguish timeout errors from other failures
   - Log timeout durations for observability
   - Help users identify timeout issues

### Medium-term (Breaking Changes)

1. **Implement MCP Async Tasks** (when stable):

   - Support `tasks/create`, `tasks/get`, `tasks/result`, `tasks/cancel`
   - Enable long-running workflows
   - Maintain compatibility with synchronous mode

2. **Negotiate timeouts** (per RFC #1492):

   - Read server timeout declarations from capabilities
   - Use as guidance (not hard limit)
   - Allow user overrides when needed

### Long-term (Protocol Evolution)

1. **Participate in RFC discussions**:

   - Contribute to timeout standardization
   - Advocate for real-world use cases (ETL, batch ops)
   - Ensure proxy use cases are represented

2. **Monitor spec changes**:

   - Track SEP-1686 (Async Tasks) stability
   - Watch RFC #1492 acceptance/changes
   - Adopt ratified features promptly

## References

- **RFC #1492**: https://github.com/modelcontextprotocol/modelcontextprotocol/pull/1492
- **SEP-1686 (Async Tasks)**: https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1686
- **WorkOS Async Tasks Guide**: https://workos.com/blog/mcp-async-tasks-ai-agent-workflows
- **MCP Best Practices**: https://mcp-best-practice.github.io/mcp-best-practice/
- **Related Issues**:
  - [TypeScript SDK #245: Configurable timeouts](https://github.com/modelcontextprotocol/typescript-sdk/issues/245)
  - [Inspector #142: Configurable timeouts](https://github.com/modelcontextprotocol/inspector/issues/142)
  - [Continue #5325: Timeout problems](https://github.com/continuedev/continue/issues/5325)
  - [Continue #7509: Timeout problems](https://github.com/continuedev/continue/issues/7509)
  - [LM Studio #727: Fixed, non-configurable timeout](https://github.com/lmstudio-ai/lmstudio-bug-tracker/issues/727)
  - [Claude Code #424: Config option for MCP timeout](https://github.com/anthropics/claude-code/issues/424)
  - [Cline #1306: Hard-coded timeout](https://github.com/cline/cline/issues/1306)
  - [Cline #2296: Hard-coded timeout](https://github.com/cline/cline/issues/2296)
  - [Codex #2346: Configurable timeout](https://github.com/openai/codex/issues/2346)
