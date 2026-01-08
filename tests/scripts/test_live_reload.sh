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
	for _ in {1..15}; do
		sleep 0.5
		echo '{"jsonrpc":"2.0","id":999,"method":"tools/list"}' 2>/dev/null || break
	done
) | ./target/release/dmcp "$TEST_CONFIG" >/tmp/mcp-output.log 2>&1 &
MCP_PID=$!

echo "MCP started with PID: $MCP_PID"
sleep 2

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
sleep 2

echo ""
echo "Checking reload events..."

reload_detected=false
if grep -qE "(Config file changed|reloading|config changed)" /tmp/mcp-output.log; then
	echo "✅ Config change detected and reload triggered"
	reload_detected=true
else
	echo "ℹ️  Config change log message not found (checking for implicit reload)"
fi

reload_verified=false
if grep -q "test-server: Test server modified" /tmp/mcp-output.log || grep -q "another-server: Another test server" /tmp/mcp-output.log; then
	echo "✅ Config reload verified - new server descriptions loaded"
	reload_verified=true
else
	echo "⚠️  Could not verify new server descriptions in log"
fi

server_count_another=$(grep -c "another-server" /tmp/mcp-output.log | head -1 || echo "0")

if [ "$server_count_another" -gt 0 ]; then
	echo "✅ New server 'another-server' detected in config after reload"
	reload_verified=true
fi

if [ "$reload_detected" = true ] || [ "$reload_verified" = true ]; then
	echo ""
	echo "✅ Live reload test PASSED - Config reload is working!"
	TEST_PASSED=true
else
	echo ""
	echo "❌ Live reload test FAILED - Could not verify reload"
	echo "Debug info:"
	echo "  reload_detected=$reload_detected"
	echo "  reload_verified=$reload_verified"
	echo "  server_count_another=$server_count_another"
	TEST_PASSED=false
fi

echo ""
echo "Stopping MCP server..."
# Kill the process group (server and keepalive loop)
kill -TERM $MCP_PID 2>/dev/null || true
# Kill any child processes
pkill -P $MCP_PID 2>/dev/null || true
sleep 0.5
# Force kill if still running
pkill -f "dynamic-mcp.*config.test.json" 2>/dev/null || true
kill -KILL $MCP_PID 2>/dev/null || true
wait $MCP_PID 2>/dev/null || true

# Exit with appropriate code
if [ "$TEST_PASSED" = true ]; then
	exit 0
else
	echo ""
	echo "Test output log:"
	echo "===================="
	cat /tmp/mcp-output.log
	echo "===================="
	echo ""
	exit 1
fi
