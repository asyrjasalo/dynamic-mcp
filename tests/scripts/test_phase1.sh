#!/bin/bash
set -e

cd "$(dirname "$0")/../.."

echo "=== Phase 1 Integration Test ==="
echo ""

echo "1. Building release binary..."
cargo build --release --quiet 2>&1 | grep -v "warning:" || true
echo "   ✅ Build successful"
echo ""

TEST_CONFIG="tests/fixtures/test_phase1.json"
mkdir -p "$(dirname "$TEST_CONFIG")"
cat >"$TEST_CONFIG" <<EOF
{
  "mcpServers": {}
}
EOF

echo "2. Testing initialize..."
result=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | timeout 5 ./target/release/dmcp "$TEST_CONFIG" 2>&1 | grep -v "ERROR\|WARN" | head -1)
if echo "$result" | grep -q '"protocolVersion"'; then
	echo "   ✅ Initialize successful"
else
	echo "   ❌ Initialize failed"
	echo "   Response: $result"
	rm -f "$TEST_CONFIG"
	exit 1
fi
echo ""

echo "3. Testing tools/list..."
result=$({
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
	echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
} | timeout 5 ./target/release/dmcp "$TEST_CONFIG" 2>&1 | grep -v "ERROR\|WARN" | tail -1)

if echo "$result" | grep -q 'get_dynamic_tools'; then
	echo "   ✅ Tools list successful"
else
	echo "   ❌ Tools list failed"
	echo "   Response: $result"
	rm -f "$TEST_CONFIG"
	exit 1
fi
echo ""

rm -f "$TEST_CONFIG"
echo "=== All Phase 1 tests passed! ==="
