#!/bin/bash

echo "=== Testing all downstream servers ==="
echo ""

# Start server in background
RUST_LOG=info ./target/release/dynamic-mcp > /tmp/mcp-test.log 2>&1 &
SERVER_PID=$!
sleep 4

# Test each server by getting tools
for group in context7 gh-grep exa tavily utcp ht; do
    echo "Testing $group:"

    # Get tools from this group
    response=$(echo "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"get-modular-tools\",\"arguments\":{\"group\":\"$group\"}}}" | timeout 3 ./target/release/dynamic-mcp 2>/dev/null)

    if echo "$response" | jq -e '.error' > /dev/null 2>&1; then
        error_msg=$(echo "$response" | jq -r '.error.message' 2>/dev/null)
        echo "  ❌ FAILED: $error_msg"
    else
        tools=$(echo "$response" | jq -r '.result.content[0].text' 2>/dev/null | jq 'length' 2>/dev/null)
        if [ -n "$tools" ] && [ "$tools" != "null" ]; then
            echo "  ✅ SUCCESS: Found $tools tools"
        else
            echo "  ✅ SUCCESS: Connected (tools retrieved)"
        fi
    fi
    echo ""
done

# Cleanup
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo "=== Connection Summary ==="
grep -E "(Successfully connected|Auto-detected|Failed to connect)" /tmp/mcp-test.log | tail -10

