#!/bin/bash

cd "$(dirname "$0")/../.."

echo "Testing dynamic-mcp Server..."
echo

echo "Building release binary..."
cargo build --release --quiet 2>&1 | grep -v "warning:" || true
echo ""

# Create minimal test config without upstream servers (to avoid connection errors)
TEST_CONFIG="tests/fixtures/test_mcp.json"
mkdir -p "$(dirname "$TEST_CONFIG")"
cat >"$TEST_CONFIG" <<EOF
{
  "mcpServers": {}
}
EOF

echo "1. Testing initialize request:"
result=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | timeout 5 ./target/release/dmcp "$TEST_CONFIG" 2>&1 | grep -v "ERROR\|WARN" | head -1)
if echo "$result" | grep -q '"protocolVersion"'; then
	echo "   ✅ Initialize successful"
else
	echo "   ❌ Initialize failed"
	echo "   Response: $result"
	rm -f "$TEST_CONFIG"
	exit 1
fi
echo

echo "2. Testing tools/list request:"
result=$({
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
	sleep 0.1
	echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
} | timeout 5 ./target/release/dmcp "$TEST_CONFIG" 2>&1 | grep -v "ERROR\|WARN" | tail -1)

if echo "$result" | grep -q 'get_dynamic_tools'; then
	echo "   ✅ Tools list successful"
	echo "$result" | jq '.result.tools[] | .name' 2>/dev/null || echo "   Tools: get_dynamic_tools, call_dynamic_tool"
else
	echo "   ❌ Tools list failed"
	echo "   Response: $result"
	rm -f "$TEST_CONFIG"
	exit 1
fi
echo

rm -f "$TEST_CONFIG"
echo "✅ Server is responding to MCP requests!"
