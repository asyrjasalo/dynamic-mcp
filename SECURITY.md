# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |

## Security Features

### OAuth2 Token Storage

**Location**: `~/.dynamic-mcp/oauth-servers/`

**Security Measures**:
- Tokens stored in user's home directory (POSIX permissions: 0600)
- File-based storage with JSON serialization
- No encryption at rest (relies on filesystem permissions)
- Directory created with restrictive permissions (0700)

**Token Security**:
- PKCE (Proof Key for Code Exchange) used for authorization
- RFC 6749 compliant refresh token rotation
- Automatic token refresh before expiry (5-minute buffer)
- Tokens include expiry timestamp for validation

**Recommendations for Production**:
1. Ensure home directory has proper permissions (mode 0700)
2. Consider using OS keychain integration for token storage:
   - macOS: Keychain Access
   - Linux: Secret Service API (gnome-keyring, KWallet)
   - Windows: Credential Manager
3. Implement token encryption at rest if operating in multi-user environments
4. Rotate credentials regularly using OAuth refresh flow
5. Monitor token storage location for unauthorized access

### Environment Variable Handling

**Syntax**: Only `${VAR}` syntax supported (not `$VAR`)

**Security Considerations**:
- Environment variables exposed to child processes (stdio transport)
- No sanitization of env var values
- Undefined variables preserved as `${VAR}` with warning

**Recommendations**:
1. Avoid storing secrets directly in config files
2. Use environment variables for sensitive data
3. Set restrictive permissions on config files (0600)
4. Review environment variables passed to child processes
5. Consider using secret management tools (e.g., HashiCorp Vault)

### Network Security

**HTTP/SSE Transports**:
- HTTPS enforced for OAuth authentication flows
- No certificate pinning (uses system trust store)
- Custom headers supported (including `Authorization`)
- No request/response size limits enforced

**Recommendations**:
1. Use HTTPS for all remote MCP servers
2. Validate TLS certificates (default behavior)
3. Implement rate limiting on upstream servers
4. Monitor network traffic for anomalies

### Process Security

**Child Process Management** (stdio transport):
- Spawns child processes with inherited environment
- No process isolation or sandboxing
- Child processes run with same user privileges
- No resource limits enforced (CPU, memory, file descriptors)

**Recommendations**:
1. Review command and args in configuration before deployment
2. Use absolute paths for commands to prevent PATH hijacking
3. Consider using containerization for isolation
4. Implement resource limits at OS level (ulimit, cgroups)
5. Monitor child process behavior

### Configuration Security

**Config File**: `dynamic-mcp.json` or `config.json`

**Security Considerations**:
- Plain text JSON (no encryption)
- May contain sensitive data (URLs, OAuth client IDs)
- Read by multiple modules during startup
- Live reload feature watches config file

**Recommendations**:
1. Set restrictive permissions: `chmod 600 dynamic-mcp.json`
2. Store in secure location (not web-accessible directories)
3. Use environment variables for secrets
4. Add config files to `.gitignore`
5. Review changes before config reload

## Known Security Limitations

1. **Token Storage**: Tokens stored as plain text in filesystem (not encrypted)
2. **No Rate Limiting**: No built-in rate limiting for tool calls
3. **No Audit Logging**: Security events not logged separately
4. **No Input Validation**: Tool arguments passed through without validation
5. **Process Privileges**: Child processes inherit full privileges
6. **No Sandboxing**: No isolation between upstream MCP servers

## Security Best Practices

### For Operators

1. **Least Privilege**:
   ```bash
   # Run as dedicated user with minimal permissions
   useradd -r -s /bin/false dynamic-mcp
   sudo -u dynamic-mcp /usr/local/bin/dmcp config.json
   ```

2. **File Permissions**:
   ```bash
   # Config file
   chmod 600 dynamic-mcp.json
   chown dynamic-mcp:dynamic-mcp dynamic-mcp.json

   # Token storage
   chmod 700 ~/.dynamic-mcp
   chmod 600 ~/.dynamic-mcp/oauth-servers/*.json
   ```

3. **Network Isolation**:
   ```bash
   # Restrict network access with firewall rules
   # Allow only necessary outbound connections
   ```

4. **Monitoring**:
   ```bash
   # Enable debug logging
   RUST_LOG=debug dmcp config.json

   # Monitor for anomalies
   tail -f /var/log/dmcp.log | grep -E 'ERROR|WARN'
   ```

5. **Regular Updates**:
   ```bash
   # Check for updates regularly
   pip install --upgrade dmcp
   # or: cargo install dynamic-mcp --force
   ```

### For Developers

1. **Input Validation**:
   - Validate all tool arguments before passing to upstream
   - Sanitize file paths and command arguments
   - Implement schema validation for tool inputs

2. **Secret Management**:
   - Never log tokens or secrets
   - Use secure comparison for CSRF tokens
   - Implement token rotation policies

3. **Error Handling**:
   - Don't leak sensitive information in error messages
   - Log security events separately
   - Implement rate limiting for authentication

4. **Code Review**:
   - Security-focused code review for auth changes
   - Dependency audit: `cargo audit`
   - Static analysis: `cargo clippy`

5. **Testing**:
   - Security test cases in CI/CD
   - Fuzzing for input validation
   - Regular penetration testing

## References

- OAuth 2.0: [RFC 6749](https://tools.ietf.org/html/rfc6749)
- PKCE: [RFC 7636](https://tools.ietf.org/html/rfc7636)
- Token Refresh: [RFC 6749 Section 6](https://tools.ietf.org/html/rfc6749#section-6)
- MCP Specification: [Model Context Protocol](https://modelcontextprotocol.io/)

---

**Last Updated**: January 6, 2026
**Version**: 1.0.0
