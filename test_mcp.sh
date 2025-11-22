#!/bin/bash

# Test the MCP server with sample requests

echo "Testing MCP Server..."
echo ""

# Start the server in the background
cargo run --release 2>/dev/null &
SERVER_PID=$!

# Give it time to start
sleep 2

# Test 1: Initialize
echo "1. Initialize:"
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
echo ""

# Test 2: List tools
echo "2. List Tools:"
echo '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
echo ""

# Test 3: Get balance (Vitalik's address)
echo "3. Get ETH Balance for Vitalik's address:"
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_balance","arguments":{"wallet_address":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"}}}'
echo ""

# Clean up
kill $SERVER_PID 2>/dev/null

echo "Send these JSON requests to the server via stdin to test!"
