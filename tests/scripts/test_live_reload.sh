#!/bin/bash

set -e

echo "Building dynamic-mcp..."
cargo build --release

echo ""
echo "Creating test config file..."
cat >config.test.json <<EOF
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
./target/release/dynamic-mcp config.test.json >/tmp/mcp-output.log 2>&1 &
MCP_PID=$!

echo "MCP started with PID: $MCP_PID"
sleep 2

echo ""
echo "Checking initial config load..."
if grep -q "test-server" /tmp/mcp-output.log; then
	echo "✅ Initial config loaded successfully"
else
	echo "❌ Initial config not loaded"
	kill $MCP_PID 2>/dev/null || true
	exit 1
fi

echo ""
echo "Modifying config file..."
cat >config.test.json <<EOF
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

echo "Waiting for reload..."
sleep 3

echo ""
echo "Checking reload events..."
if grep -q "Config file changed" /tmp/mcp-output.log; then
	echo "✅ Config change detected"
else
	echo "❌ Config change not detected"
fi

if grep -q "reloading" /tmp/mcp-output.log; then
	echo "✅ Config reload triggered"
else
	echo "❌ Config reload not triggered"
fi

if grep -q "reconnected" /tmp/mcp-output.log; then
	echo "✅ Downstream resources reconnected"
else
	echo "❌ Downstream resources not reconnected"
fi

echo ""
echo "Stopping MCP server..."
kill $MCP_PID 2>/dev/null || true
wait $MCP_PID 2>/dev/null || true

echo ""
echo "Test output log:"
echo "===================="
cat /tmp/mcp-output.log
echo "===================="

echo ""
echo "Test complete!"
