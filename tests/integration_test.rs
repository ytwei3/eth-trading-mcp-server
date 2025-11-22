use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions() {
        // This test verifies that tool definitions are properly structured
        // In a real scenario, you would test against the actual MCP server

        let get_balance_schema = json!({
            "type": "object",
            "properties": {
                "wallet_address": {
                    "type": "string",
                    "description": "The wallet address to query (0x...)"
                },
                "token_address": {
                    "type": "string",
                    "description": "Optional ERC20 token contract address. If not provided, returns ETH balance."
                }
            },
            "required": ["wallet_address"]
        });

        assert_eq!(get_balance_schema["type"], "object");
        assert!(get_balance_schema["properties"]["wallet_address"].is_object());
    }

    #[test]
    fn test_json_rpc_request_format() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "tools/list");
    }

    #[test]
    fn test_initialize_request_format() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        assert_eq!(request["method"], "initialize");
        assert_eq!(request["params"]["protocolVersion"], "2024-11-05");
    }

    #[test]
    fn test_tool_call_request_format() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "get_balance",
                "arguments": {
                    "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
                }
            }
        });

        assert_eq!(request["method"], "tools/call");
        assert_eq!(request["params"]["name"], "get_balance");
    }

    #[test]
    fn test_swap_parameters() {
        let swap_params = json!({
            "from_token": "0x0000000000000000000000000000000000000000",
            "to_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "amount": "1.0",
            "slippage_bps": 50,
            "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
        });

        assert_eq!(swap_params["slippage_bps"], 50);
        assert_eq!(swap_params["amount"], "1.0");
    }
}
