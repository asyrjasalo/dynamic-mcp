#!/bin/bash

echo "Testing connections to all upstream servers..."
echo ""

response=$( (
	echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
	sleep 3
	echo '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
) | timeout 15 ./target/release/dmcp 2>/dev/null)

echo "=== Listing available groups ==="
groups=$(echo "$response" | jq -r 'select(.id==2) | .result.tools[0].inputSchema.properties.group.enum[]' 2>/dev/null | sort)

if [ -z "$groups" ]; then
	echo "❌ No groups found. Server may not be connecting to upstream servers."
	exit 1
fi

echo "$groups"

echo ""
echo "=== Testing each server group ==="

for group in $groups; do
	echo -n "Testing $group... "

	response=$( (
		echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
		sleep 3
		echo "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"get_dynamic_tools\",\"arguments\":{\"group\":\"$group\"}}}"
	) | timeout 15 ./target/release/dmcp 2>/dev/null)

	# Check if there's an actual error in the JSON response
	has_error=$(echo "$response" | jq -e 'select(.id==2) | .error != null' 2>/dev/null)

	if [ "$has_error" = "true" ]; then
		error_msg=$(echo "$response" | jq -r 'select(.id==2) | .error.message' 2>/dev/null)
		echo "❌ FAILED"
		echo "  $error_msg"
	else
		tool_count=$(echo "$response" | jq -r 'select(.id==2) | .result.content[0].text' 2>/dev/null | jq 'length' 2>/dev/null || echo "?")
		echo "✅ OK (found $tool_count tools)"
	fi
done

echo ""
echo "=== Server logs (last 20 lines) ==="
tail -20 /tmp/mcp-server.log
