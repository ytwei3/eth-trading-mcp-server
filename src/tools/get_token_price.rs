use anyhow::Result;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::ethereum::{get_token_price, EthClient};
use crate::types::{Tool, ToolContent, ToolResult};

#[derive(Debug, Deserialize)]
pub struct GetTokenPriceParams {
    pub token_address: String,
}

#[derive(Debug, Serialize)]
pub struct PriceResponse {
    pub token_address: String,
    pub price_usd: Option<String>,
    pub price_eth: Option<String>,
    pub source: String,
}

pub fn get_tool_definition() -> Tool {
    Tool {
        name: "get_token_price".to_string(),
        description: "Get current token price in USD and ETH from various price oracles"
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "token_address": {
                    "type": "string",
                    "description": "The token contract address (0x...). Use 0x0000000000000000000000000000000000000000 for ETH."
                }
            },
            "required": ["token_address"]
        }),
    }
}

pub async fn execute(provider: &EthClient, params: GetTokenPriceParams) -> Result<ToolResult> {
    let token_address = params
        .token_address
        .parse::<Address>()
        .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;

    let price_info = get_token_price(provider, token_address).await?;

    let response = PriceResponse {
        token_address: params.token_address,
        price_usd: price_info.price_usd.map(|p| p.to_string()),
        price_eth: price_info.price_eth.map(|p| p.to_string()),
        source: price_info.source,
    };

    let text = format!(
        "Token: {}\nPrice (USD): {}\nPrice (ETH): {}\nSource: {}",
        response.token_address,
        response
            .price_usd
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("N/A"),
        response
            .price_eth
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("N/A"),
        response.source
    );

    Ok(ToolResult {
        content: vec![ToolContent::text(text)],
        is_error: None,
    })
}
