use anyhow::{Context, Result};
use ethers::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;

use super::client::EthClient;

// ERC20 ABI for balanceOf and decimals
abigen!(
    ERC20,
    r#"[
        function balanceOf(address account) external view returns (uint256)
        function decimals() external view returns (uint8)
        function symbol() external view returns (string)
        function name() external view returns (string)
    ]"#,
);

#[derive(Debug)]
pub struct BalanceInfo {
    pub balance: Decimal,
    pub symbol: String,
    pub decimals: u8,
    pub raw_balance: U256,
}

/// Get ETH balance for an address
pub async fn get_eth_balance(provider: &EthClient, address: Address) -> Result<BalanceInfo> {
    let balance = provider
        .get_balance(address, None)
        .await
        .context("Failed to get ETH balance")?;

    let decimals = 18u8;
    let balance_decimal = wei_to_decimal(balance, decimals)?;

    Ok(BalanceInfo {
        balance: balance_decimal,
        symbol: "ETH".to_string(),
        decimals,
        raw_balance: balance,
    })
}

/// Get ERC20 token balance for an address
pub async fn get_token_balance(
    provider: &EthClient,
    token_address: Address,
    wallet_address: Address,
) -> Result<BalanceInfo> {
    let contract = ERC20::new(token_address, provider.clone());

    // Get balance
    let balance = contract
        .balance_of(wallet_address)
        .call()
        .await
        .context("Failed to call balanceOf")?;

    // Get decimals
    let decimals = contract
        .decimals()
        .call()
        .await
        .context("Failed to get token decimals")?;

    // Get symbol
    let symbol = contract
        .symbol()
        .call()
        .await
        .unwrap_or_else(|_| "UNKNOWN".to_string());

    let balance_decimal = wei_to_decimal(balance, decimals)?;

    Ok(BalanceInfo {
        balance: balance_decimal,
        symbol,
        decimals,
        raw_balance: balance,
    })
}

/// Convert wei amount to decimal with given decimals
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
    fn test_wei_to_decimal() {
        // 1 ETH = 1e18 wei
        let one_eth = U256::from_dec_str("1000000000000000000").unwrap();
        let result = wei_to_decimal(one_eth, 18).unwrap();
        assert_eq!(result, Decimal::from(1));

        // 1000 USDC (6 decimals) = 1000e6
        let thousand_usdc = U256::from_dec_str("1000000000").unwrap();
        let result = wei_to_decimal(thousand_usdc, 6).unwrap();
        assert_eq!(result, Decimal::from(1000));
    }
}
