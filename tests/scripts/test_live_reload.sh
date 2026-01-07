#!/bin/bash

set -e

# Change to project root
cd "$(dirname "$0")/../.."

# Use config file in fixtures directory
TEST_CONFIG="tests/fixtures/config.test.json"

echo "Building dynamic-mcp..."
cargo build --release

echo ""
echo "Creating test config file..."
cat >"$TEST_CONFIG" <<EOF
{
  "\$schema": "https://raw.githubusercontent.com/d-kimuson/dynamic-mcp/refs/heads/main/config-schema.json",
  "mcpServers": {
    "test-server": {
      "type": "stdio",
      "description": "Test server initial",
      "command": "echo",
      "args": ["Initial config"]
    }
  }
}
EOF

echo ""
echo "Starting dynamic-mcp in background..."
# Start server and keep it alive by sending periodic keepalive messages
(
	# Send initialize to establish connection
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
	# Keep connection alive with periodic messages (enough for the test duration)
	for i in {1..15}; do
		sleep 1
		echo '{"jsonrpc":"2.0","id":999,"method":"tools/list"}' 2>/dev/null || break
	done
) | ./target/release/dmcp "$TEST_CONFIG" >/tmp/mcp-output.log 2>&1 &
MCP_PID=$!

echo "MCP started with PID: $MCP_PID"
sleep 4

echo ""
echo "Checking initial config load..."
# Check for successful connection or config loading messages
if grep -qE "(test-server|Successfully connected|MCP server config loaded|✅ Successfully connected)" /tmp/mcp-output.log; then
	echo "✅ Initial config loaded successfully"
else
	echo "⚠️  Initial config check - checking log:"
	tail -10 /tmp/mcp-output.log || true
	# Don't exit, continue with reload test
fi

echo ""
echo "Modifying config file..."
cat >"$TEST_CONFIG" <<EOF
{
  "\$schema": "https://raw.githubusercontent.com/d-kimuson/dynamic-mcp/refs/heads/main/config-schema.json",
  "mcpServers": {
    "test-server": {
      "type": "stdio",
      "description": "Test server modified",
      "command": "echo",
      "args": ["Modified config"]
    },
    "another-server": {
      "type": "stdio",
      "description": "Another test server",
      "command": "echo",
      "args": ["Another server"]
    }
  }
}
EOF

echo "Waiting for reload (file watcher needs time to detect changes)..."
sleep 6

echo ""
echo "Checking reload events..."

# Check for reload log messages
reload_detected=false
if grep -qE "(Config file changed|reloading)" /tmp/mcp-output.log; then
	echo "✅ Config change detected and reload triggered (found in log)"
	reload_detected=true
else
	echo "⚠️  Config change log messages not found"
fi

# More reliable: Check if tool descriptions changed (showing reload happened)
# Get tool descriptions before and after reload
initial_desc=$(grep -o "test-server: Test server initial" /tmp/mcp-output.log | head -1)
modified_desc=$(grep -o "test-server: Test server modified" /tmp/mcp-output.log | head -1)
another_desc=$(grep -o "another-server: Another test server" /tmp/mcp-output.log | head -1)

if [ -n "$modified_desc" ] || [ -n "$another_desc" ]; then
	echo "✅ Config reload verified - tool descriptions updated"
	echo "   Found: $modified_desc"
	echo "   Found: $another_desc"
	reload_verified=true
else
	echo "⚠️  Could not verify reload via tool description changes"
	reload_verified=false
fi

# Check for reconnection messages
if grep -qE "(Successfully reconnected|reconnected to MCP group)" /tmp/mcp-output.log; then
	echo "✅ Upstream MCP servers reconnected"
elif [ "$reload_verified" = true ]; then
	echo "⚠️  Reconnection messages not found (test servers may have failed to connect)"
	echo "   This is expected - 'echo' command is not a real MCP server"
else
	echo "ℹ️  Skipping reconnection check (reload not verified)"
fi

# Final verdict
if [ "$reload_detected" = true ] || [ "$reload_verified" = true ]; then
	echo ""
	echo "✅ Live reload test PASSED - Config reload is working!"
	TEST_PASSED=true
else
	echo ""
	echo "❌ Live reload test FAILED - Could not verify reload"
	TEST_PASSED=false
fi

echo ""
echo "Stopping MCP server..."
# Kill the process group (server and keepalive loop)
kill -TERM $MCP_PID 2>/dev/null || true
# Kill any child processes
pkill -P $MCP_PID 2>/dev/null || true
sleep 1
# Force kill if still running
pkill -f "dynamic-mcp.*config.test.json" 2>/dev/null || true
kill -KILL $MCP_PID 2>/dev/null || true
wait $MCP_PID 2>/dev/null || true

# Exit with appropriate code
if [ "$TEST_PASSED" = true ]; then
	exit 0
else
	exit 1
fi

echo ""
echo "Test output log:"
echo "===================="
cat /tmp/mcp-output.log
echo "===================="

echo ""
echo "Test complete!"
