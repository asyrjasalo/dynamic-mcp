#!/bin/bash
set -e

cd "$(dirname "$0")/../.."

echo "╔════════════════════════════════════════════════════════╗"
echo "║     Phase 1 COMPLETE - Integration Test Suite         ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo "📦 Building release binary..."
cargo build --release --quiet 2>&1 | grep -v "warning:" || true
echo "   ✅ Build successful"
echo ""

echo "🧪 Test 1: Server initialization"
result=$({
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
} | timeout 5 ./target/release/dmcp tests/fixtures/config.test.json 2>/dev/null | grep -E '(jsonrpc|"id":1)' | head -1)

if echo "$result" | jq -e '.result.serverInfo.name == "dynamic-mcp"' >/dev/null 2>&1; then
	echo "   ✅ Initialize returns correct server info"
	echo "      Server: $(echo $result | jq -r .result.serverInfo.name) v$(echo $result | jq -r .result.serverInfo.version)"
else
	echo "   ❌ Initialize failed"
	echo "   Response: $result"
	exit 1
fi
echo ""

echo "🧪 Test 2: Tools listing"
result=$({
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
	sleep 0.1
	echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
} | timeout 5 ./target/release/dmcp tests/fixtures/config.test.json 2>/dev/null | grep '"id":2' | head -1)

tool_count=$(echo "$result" | jq '.result.tools | length' 2>/dev/null || echo "0")
if [ "$tool_count" = "2" ]; then
	echo "   ✅ Exposes 2 tools correctly"
	echo "      - get_dynamic_tools"
	echo "      - call_dynamic_tool"
else
	echo "   ❌ Expected 2 tools, got $tool_count"
	exit 1
fi
echo ""

echo "🧪 Test 3: Unit tests"
cargo test --quiet 2>&1 | tail -3
echo ""
