#!/usr/bin/env python3
"""
Simple MCP client for testing the Ethereum Trading MCP Server
"""

import json
import subprocess
import sys

def send_request(method, params=None):
    """Send a JSON-RPC request to the MCP server"""
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params or {}
    }

    # Start the server
    proc = subprocess.Popen(
        ["cargo", "run", "--release"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    # Send request
    request_json = json.dumps(request)
    print(f"→ Request: {request_json}\n")

    stdout, stderr = proc.communicate(input=request_json + "\n", timeout=30)

    # Parse response
    if stdout.strip():
        response = json.loads(stdout.strip())
        print(f"← Response:")
        print(json.dumps(response, indent=2))
    else:
        print("No response received")

    if stderr and "error" in stderr.lower():
        print(f"\nErrors: {stderr}")

def main():
    print("=" * 60)
    print("MCP Server Test Client")
    print("=" * 60)
    print()

    if len(sys.argv) < 2:
        print("Usage examples:")
        print("  python client_example.py init")
        print("  python client_example.py list")
        print("  python client_example.py balance <address>")
        print("  python client_example.py price <token_address>")
        print("  python client_example.py swap <from> <to> <amount> <wallet>")
        return

    command = sys.argv[1]

    if command == "init":
        send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "python-client", "version": "1.0"}
        })

    elif command == "list":
        send_request("tools/list")

    elif command == "balance":
        if len(sys.argv) < 3:
            print("Usage: python client_example.py balance <wallet_address>")
            print("Example: python client_example.py balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            return

        send_request("tools/call", {
            "name": "get_balance",
            "arguments": {
                "wallet_address": sys.argv[2]
            }
        })

    elif command == "price":
        if len(sys.argv) < 3:
            print("Usage: python client_example.py price <token_address>")
            print("Example: python client_example.py price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            return

        send_request("tools/call", {
            "name": "get_token_price",
            "arguments": {
                "token_address": sys.argv[2]
            }
        })

    elif command == "swap":
        if len(sys.argv) < 6:
            print("Usage: python client_example.py swap <from_token> <to_token> <amount> <wallet_address>")
            print("Example: python client_example.py swap 0x0000000000000000000000000000000000000000 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 0.1 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            return

        send_request("tools/call", {
            "name": "swap_tokens",
            "arguments": {
                "from_token": sys.argv[2],
                "to_token": sys.argv[3],
                "amount": sys.argv[4],
                "wallet_address": sys.argv[5],
                "slippage_bps": 50
            }
        })

    else:
        print(f"Unknown command: {command}")

if __name__ == "__main__":
    main()
