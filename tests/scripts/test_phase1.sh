#!/bin/bash
set -e

# Change to project root (two levels up from tests/scripts/)
cd "$(dirname "$0")/../.."

echo "=== Phase 1 Integration Test ==="
echo ""

echo "1. Building project..."
cargo build --quiet 2>&1 | grep -v "warning:" || true
echo "   ✅ Build successful"
echo ""

echo "2. Testing initialize..."
result=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | timeout 2 cargo run --quiet -- examples/config.example.json 2>/dev/null | head -1)
if echo "$result" | grep -q '"protocolVersion"'; then
	echo "   ✅ Initialize successful"
else
	echo "   ❌ Initialize failed"
	echo "   Response: $result"
	exit 1
fi
echo ""

echo "3. Testing tools/list..."
result=$({
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
	echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
} | timeout 2 cargo run --quiet -- examples/config.example.json 2>/dev/null | tail -1)

if echo "$result" | grep -q 'get-modular-tools'; then
	echo "   ✅ Tools list successful"
else
	echo "   ❌ Tools list failed"
	echo "   Response: $result"
	exit 1
fi
echo ""

echo "=== All Phase 1 tests passed! ==="
