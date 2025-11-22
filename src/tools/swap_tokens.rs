use anyhow::Result;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::ethereum::{simulate_swap, EthClient};
use crate::types::{Tool, ToolContent, ToolResult};

#[derive(Debug, Deserialize)]
pub struct SwapTokensParams {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
    #[serde(default = "default_slippage")]
    pub slippage_bps: u32,
    pub wallet_address: String,
}

fn default_slippage() -> u32 {
    50 // 0.5% default slippage
}

#[derive(Debug, Serialize)]
pub struct SwapResponse {
    pub from_token: String,
    pub to_token: String,
    pub amount_in: String,
    pub estimated_output: String,
    pub minimum_output: String,
    pub estimated_gas: String,
    pub slippage_bps: u32,
    pub route: Vec<String>,
}

pub fn get_tool_definition() -> Tool {
    Tool {
        name: "swap_tokens".to_string(),
        description:
            "Simulate a token swap on Uniswap V2. Returns estimated output and gas costs without executing."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "from_token": {
                    "type": "string",
                    "description": "Source token address (0x...). Use 0x0000000000000000000000000000000000000000 for ETH."
                },
                "to_token": {
                    "type": "string",
                    "description": "Destination token address (0x...)"
                },
                "amount": {
                    "type": "string",
                    "description": "Amount to swap (in token units, e.g., '1.5' for 1.5 tokens)"
                },
                "slippage_bps": {
                    "type": "number",
                    "description": "Slippage tolerance in basis points (e.g., 50 = 0.5%). Default: 50",
                    "default": 50
                },
                "wallet_address": {
                    "type": "string",
                    "description": "Wallet address for simulation (0x...)"
                }
            },
            "required": ["from_token", "to_token", "amount", "wallet_address"]
        }),
    }
}

pub async fn execute(provider: &EthClient, params: SwapTokensParams) -> Result<ToolResult> {
    let from_token = params
        .from_token
        .parse::<Address>()
        .map_err(|e| anyhow::anyhow!("Invalid from_token address: {}", e))?;

    let to_token = params
        .to_token
        .parse::<Address>()
        .map_err(|e| anyhow::anyhow!("Invalid to_token address: {}", e))?;

    let wallet_address = params
        .wallet_address
        .parse::<Address>()
        .map_err(|e| anyhow::anyhow!("Invalid wallet address: {}", e))?;

    let amount = params
        .amount
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;

    let simulation = simulate_swap(
        provider,
        from_token,
        to_token,
        amount,
        params.slippage_bps,
        wallet_address,
    )
    .await?;

    let response = SwapResponse {
        from_token: params.from_token,
        to_token: params.to_token,
        amount_in: params.amount,
        estimated_output: simulation.estimated_output.to_string(),
        minimum_output: simulation.minimum_output.to_string(),
        estimated_gas: simulation.estimated_gas.to_string(),
        slippage_bps: params.slippage_bps,
        route: simulation.route.iter().map(|addr| format!("{:?}", addr)).collect(),
    };

    let text = format!(
        "Swap Simulation:\n\
        From: {}\n\
        To: {}\n\
        Amount In: {}\n\
        Estimated Output: {}\n\
        Minimum Output (with slippage): {}\n\
        Estimated Gas: {}\n\
        Slippage Tolerance: {} bps ({}%)\n\
        Route: {}",
        response.from_token,
        response.to_token,
        response.amount_in,
        response.estimated_output,
        response.minimum_output,
        response.estimated_gas,
        response.slippage_bps,
        (response.slippage_bps as f64) / 100.0,
        response.route.join(" -> ")
    );

    Ok(ToolResult {
        content: vec![ToolContent::text(text)],
        is_error: None,
    })
}
