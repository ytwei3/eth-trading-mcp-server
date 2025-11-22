use anyhow::Result;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::ethereum::{get_eth_balance, get_token_balance, EthClient};
use crate::types::{Tool, ToolContent, ToolResult};

#[derive(Debug, Deserialize)]
pub struct GetBalanceParams {
    pub wallet_address: String,
    pub token_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub balance: String,
    pub symbol: String,
    pub decimals: u8,
    pub wallet_address: String,
    pub token_address: Option<String>,
}

pub fn get_tool_definition() -> Tool {
    Tool {
        name: "get_balance".to_string(),
        description: "Query ETH or ERC20 token balance for a wallet address".to_string(),
        input_schema: json!({
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
        }),
    }
}

pub async fn execute(provider: &EthClient, params: GetBalanceParams) -> Result<ToolResult> {
    let wallet_address = params
        .wallet_address
        .parse::<Address>()
        .map_err(|e| anyhow::anyhow!("Invalid wallet address: {}", e))?;

    let balance_info = if let Some(token_addr_str) = &params.token_address {
        let token_address = token_addr_str
            .parse::<Address>()
            .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;

        get_token_balance(provider, token_address, wallet_address).await?
    } else {
        get_eth_balance(provider, wallet_address).await?
    };

    let response = BalanceResponse {
        balance: balance_info.balance.to_string(),
        symbol: balance_info.symbol,
        decimals: balance_info.decimals,
        wallet_address: params.wallet_address,
        token_address: params.token_address,
    };

    let text = format!(
        "Balance: {} {}\nDecimals: {}\nWallet: {}\nRaw balance: {}",
        response.balance,
        response.symbol,
        response.decimals,
        response.wallet_address,
        balance_info.raw_balance
    );

    Ok(ToolResult {
        content: vec![ToolContent::text(text)],
        is_error: None,
    })
}
