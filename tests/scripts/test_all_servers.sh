#!/bin/bash

# Test script to verify all upstream servers are working

echo "Testing connections to all upstream servers..."
echo ""

# Start server in background
./target/release/dmcp > /tmp/mcp-server.log 2>&1 &
SERVER_PID=$!

# Wait for server to start
sleep 3

# List all available groups
echo "=== Listing available groups ==="
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | timeout 2 ./target/release/dmcp 2>/dev/null | jq -r '.result.tools[0].inputSchema.properties.group.enum[]' 2>/dev/null | sort

echo ""
echo "=== Testing each server group ==="

# Test each group
for group in context7 gh-grep exa tavily utcp ht; do
    echo -n "Testing $group... "

    # Try to get tools from this group
    result=$(echo "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"get_dynamic_tools\",\"arguments\":{\"group\":\"$group\"}}}" | timeout 3 ./target/release/dmcp 2>/dev/null | jq -r '.error // .result.content[0].text' 2>/dev/null)

    if echo "$result" | grep -q "error\|Failed"; then
        echo "❌ FAILED"
        echo "$result" | head -3
    else
        tool_count=$(echo "$result" | jq 'length' 2>/dev/null || echo "?")
        echo "✅ OK (found tools)"
    fi
done

# Cleanup
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo ""
echo "=== Server logs (last 20 lines) ==="
tail -20 /tmp/mcp-server.log

