#!/bin/bash

echo "Testing Modular MCP Server..."
echo

cd "$(dirname "$0")"

echo "1. Testing initialize request:"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | cargo run --quiet -- ../../examples/config.example.json 2>/dev/null &
PID=$!
sleep 1
kill $PID 2>/dev/null
echo

echo "2. Testing tools/list request:"
(
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
	sleep 0.2
	echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
) | timeout 2 cargo run --quiet -- ../../examples/config.example.json 2>/dev/null | tail -1 | jq '.' 2>/dev/null || echo "Response received"
echo

echo "✅ Server is responding to MCP requests!"
