use anyhow::{Context, Result};
use ethers::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use super::client::EthClient;

// Uniswap V2 Router ABI
abigen!(
    UniswapV2Router,
    r#"[
        function swapExactTokensForTokens(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
        function swapExactETHForTokens(uint amountOutMin, address[] calldata path, address to, uint deadline) external payable returns (uint[] memory amounts)
        function swapExactTokensForETH(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline) external returns (uint[] memory amounts)
        function getAmountsOut(uint amountIn, address[] calldata path) external view returns (uint[] memory amounts)
        function WETH() external pure returns (address)
    ]"#,
);

// ERC20 for approvals
abigen!(
    IERC20,
    r#"[
        function approve(address spender, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function decimals() external view returns (uint8)
    ]"#,
);

#[derive(Debug)]
pub struct SwapSimulation {
    pub estimated_output: Decimal,
    pub estimated_gas: U256,
    pub minimum_output: Decimal,
    pub price_impact: Decimal,
    pub route: Vec<Address>,
}

/// Simulate a token swap on Uniswap V2
pub async fn simulate_swap(
    provider: &EthClient,
    from_token: Address,
    to_token: Address,
    amount_in: Decimal,
    slippage_bps: u32, // basis points (e.g., 50 = 0.5%)
    wallet_address: Address,
) -> Result<SwapSimulation> {
    // Uniswap V2 Router on Ethereum mainnet
    let router_address = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
        .parse::<Address>()
        .unwrap();

    let router = UniswapV2Router::new(router_address, provider.clone());
    let weth = router.weth().call().await?;

    // Build the swap path
    let path = build_swap_path(from_token, to_token, weth);

    // Get decimals for from_token
    let from_decimals = if from_token == Address::zero() {
        18u8
    } else {
        let token = IERC20::new(from_token, provider.clone());
        token.decimals().call().await.unwrap_or(18)
    };

    // Convert amount to wei
    let amount_in_wei = decimal_to_wei(amount_in, from_decimals)?;

    // Get estimated output amounts
    let amounts_out = router
        .get_amounts_out(amount_in_wei, path.clone())
        .call()
        .await
        .context("Failed to get amounts out from router")?;

    let estimated_output_wei = amounts_out
        .last()
        .copied()
        .context("No output amount")?;

    // Get decimals for to_token
    let to_decimals = if to_token == Address::zero() {
        18u8
    } else {
        let token = IERC20::new(to_token, provider.clone());
        token.decimals().call().await.unwrap_or(18)
    };

    let estimated_output = wei_to_decimal(estimated_output_wei, to_decimals)?;

    // Calculate minimum output with slippage
    let slippage_multiplier = Decimal::from(10000 - slippage_bps) / Decimal::from(10000);
    let minimum_output = estimated_output * slippage_multiplier;
    let min_output_wei = decimal_to_wei(minimum_output, to_decimals)?;

    // Estimate gas by simulating the transaction
    let estimated_gas = estimate_swap_gas(
        provider,
        &router,
        from_token,
        to_token,
        amount_in_wei,
        min_output_wei,
        path.clone(),
        wallet_address,
    )
    .await?;

    // Calculate price impact (simplified)
    let price_impact = Decimal::from(0); // Would need pool reserves for accurate calculation

    Ok(SwapSimulation {
        estimated_output,
        estimated_gas,
        minimum_output,
        price_impact,
        route: path,
    })
}

/// Build swap path (direct or through WETH)
fn build_swap_path(from_token: Address, to_token: Address, weth: Address) -> Vec<Address> {
    let from = if from_token == Address::zero() {
        weth
    } else {
        from_token
    };

    let to = if to_token == Address::zero() {
        weth
    } else {
        to_token
    };

    // Simple path: from -> to
    // In production, could optimize routing
    vec![from, to]
}

/// Estimate gas for a swap transaction
async fn estimate_swap_gas(
    provider: &EthClient,
    router: &UniswapV2Router<Provider<Http>>,
    from_token: Address,
    to_token: Address,
    amount_in: U256,
    amount_out_min: U256,
    path: Vec<Address>,
    wallet_address: Address,
) -> Result<U256> {
    // Set deadline to 20 minutes from now (in Unix timestamp)
    let deadline = U256::from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 1200,
    );

    // Build the transaction based on token types
    let tx = if from_token == Address::zero() {
        // ETH -> Token
        router
            .swap_exact_eth_for_tokens(amount_out_min, path, wallet_address, deadline)
            .value(amount_in)
            .tx
    } else if to_token == Address::zero() {
        // Token -> ETH
        router
            .swap_exact_tokens_for_eth(amount_in, amount_out_min, path, wallet_address, deadline)
            .tx
    } else {
        // Token -> Token
        router
            .swap_exact_tokens_for_tokens(
                amount_in,
                amount_out_min,
                path,
                wallet_address,
                deadline,
            )
            .tx
    };

    // Estimate gas using eth_estimateGas
    match provider.estimate_gas(&tx, None).await {
        Ok(gas) => Ok(gas),
        Err(_) => {
            // Return a default estimate if simulation fails
            Ok(U256::from(300000)) // Conservative default
        }
    }
}

/// Convert decimal to wei
fn decimal_to_wei(amount: Decimal, decimals: u8) -> Result<U256> {
    let multiplier = Decimal::from(10u64.pow(decimals as u32));
    let amount_wei = amount * multiplier;
    let amount_str = amount_wei.trunc().to_string();

    U256::from_dec_str(&amount_str).context("Failed to convert decimal to U256")
}

/// Convert wei to decimal
fn wei_to_decimal(amount: U256, decimals: u8) -> Result<Decimal> {
    let amount_str = amount.to_string();
    let amount_decimal = Decimal::from_str(&amount_str)?;
    let divisor = Decimal::from(10u64.pow(decimals as u32));

    Ok(amount_decimal / divisor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_conversions() {
        let amount = Decimal::from(1);
        let wei = decimal_to_wei(amount, 18).unwrap();
        assert_eq!(wei, U256::from_dec_str("1000000000000000000").unwrap());

        let back = wei_to_decimal(wei, 18).unwrap();
        assert_eq!(back, amount);
    }

    #[test]
    fn test_slippage_calculation() {
        let output = Decimal::from(100);
        let slippage_bps = 50; // 0.5%
        let slippage_multiplier = Decimal::from(10000 - slippage_bps) / Decimal::from(10000);
        let min_output = output * slippage_multiplier;

        assert_eq!(min_output, Decimal::from_str("99.5").unwrap());
    }
}
