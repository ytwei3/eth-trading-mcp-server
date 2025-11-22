use anyhow::Result;
use serde_json::{json, Value};

use crate::ethereum::EthClient;
use crate::tools;
use crate::types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, ToolResult, MCP_VERSION};

pub struct McpServer {
    provider: EthClient,
}

impl McpServer {
    pub fn new(provider: EthClient) -> Self {
        Self { provider }
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        tracing::info!("Handling request: {}", request.method);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params).await,
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tool_call(&request.params).await,
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(result),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    async fn handle_initialize(&self, _params: &Value) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "protocolVersion": MCP_VERSION,
            "serverInfo": {
                "name": "eth-trading-mcp-server",
                "version": "0.1.0"
            },
            "capabilities": {
                "tools": {}
            }
        }))
    }

    async fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        let tools = tools::get_all_tools();
        Ok(json!({
            "tools": tools
        }))
    }

    async fn handle_tool_call(&self, params: &Value) -> Result<Value, JsonRpcError> {
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing tool name".to_string(),
                data: None,
            })?;

        let default_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&default_args);

        tracing::info!("Calling tool: {} with args: {}", tool_name, arguments);

        let result = self.execute_tool(tool_name, arguments).await?;

        Ok(json!(result))
    }

    async fn execute_tool(&self, name: &str, args: &Value) -> Result<ToolResult, JsonRpcError> {
        let result = match name {
            "get_balance" => {
                let params: tools::get_balance::GetBalanceParams =
                    serde_json::from_value(args.clone()).map_err(|e| JsonRpcError {
                        code: -32602,
                        message: format!("Invalid parameters: {}", e),
                        data: None,
                    })?;

                tools::get_balance::execute(&self.provider, params)
                    .await
                    .map_err(|e| self.error_to_json_rpc_error(e))
            }
            "get_token_price" => {
                let params: tools::get_token_price::GetTokenPriceParams =
                    serde_json::from_value(args.clone()).map_err(|e| JsonRpcError {
                        code: -32602,
                        message: format!("Invalid parameters: {}", e),
                        data: None,
                    })?;

                tools::get_token_price::execute(&self.provider, params)
                    .await
                    .map_err(|e| self.error_to_json_rpc_error(e))
            }
            "swap_tokens" => {
                let params: tools::swap_tokens::SwapTokensParams =
                    serde_json::from_value(args.clone()).map_err(|e| JsonRpcError {
                        code: -32602,
                        message: format!("Invalid parameters: {}", e),
                        data: None,
                    })?;

                tools::swap_tokens::execute(&self.provider, params)
                    .await
                    .map_err(|e| self.error_to_json_rpc_error(e))
            }
            _ => {
                return Err(JsonRpcError {
                    code: -32601,
                    message: format!("Unknown tool: {}", name),
                    data: None,
                })
            }
        };

        result
    }

    fn error_to_json_rpc_error(&self, error: anyhow::Error) -> JsonRpcError {
        JsonRpcError {
            code: -32000,
            message: error.to_string(),
            data: None,
        }
    }
}
