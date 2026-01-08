#!/bin/bash

echo "=== Testing all upstream servers ==="
echo ""

response=$( (
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
	sleep 3
	echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
) | timeout 15 ./target/release/dmcp 2>/dev/null)

groups=$(echo "$response" | jq -r 'select(.id==2) | .result.tools[0].inputSchema.properties.group.enum[]' 2>/dev/null | sort)

for group in $groups; do
	echo "Testing $group:"

	result=$( (
		echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
		sleep 3
		echo "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"get_dynamic_tools\",\"arguments\":{\"group\":\"$group\"}}}"
	) | timeout 15 ./target/release/dmcp 2>/dev/null)

	if echo "$result" | jq -e 'select(.id==2) | .error' >/dev/null 2>&1; then
		error_msg=$(echo "$result" | jq -r 'select(.id==2) | .error.message' 2>/dev/null)
		echo "  ❌ FAILED: $error_msg"
	else
		tools=$(echo "$result" | jq -r 'select(.id==2) | .result.content[0].text' 2>/dev/null | jq 'length' 2>/dev/null)
		if [ "$tools" != "" ] && [ "$tools" != "null" ]; then
			echo "  ✅ SUCCESS: Found $tools tools"
		else
			echo "  ✅ SUCCESS: Connected"
		fi
	fi
	echo ""
done

echo "=== Connection Summary ==="
grep -E "(Successfully connected|Auto-detected|Failed to connect)" /tmp/mcp-test.log | tail -10
