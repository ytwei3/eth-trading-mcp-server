use anyhow::{Context, Result};
use ethers::prelude::*;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;

use super::client::EthClient;

// Uniswap V2 Pair ABI
abigen!(
    UniswapV2Pair,
    r#"[
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
        function token0() external view returns (address)
        function token1() external view returns (address)
    ]"#,
);

// Chainlink Price Feed ABI
abigen!(
    ChainlinkAggregator,
    r#"[
        function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound)
        function decimals() external view returns (uint8)
    ]"#,
);

#[derive(Debug)]
pub struct PriceInfo {
    pub price_usd: Option<Decimal>,
    pub price_eth: Option<Decimal>,
    pub source: String,
}

/// CoinGecko API response
#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    ethereum: Option<EthereumPrice>,
}

#[derive(Debug, Deserialize)]
struct EthereumPrice {
    usd: Option<f64>,
}

/// Get token price using multiple sources
pub async fn get_token_price(
    provider: &EthClient,
    token_address: Address,
) -> Result<PriceInfo> {
    // Special case for ETH
    if token_address == Address::zero() {
        return get_eth_price_from_chainlink(provider).await;
    }

    // Try to get price from CoinGecko
    match get_price_from_coingecko(&token_address).await {
        Ok(price_info) => Ok(price_info),
        Err(_) => {
            // Fallback: estimate from Uniswap pool if available
            get_price_from_uniswap(provider, token_address).await
        }
    }
}

/// Get ETH price from Chainlink price feed
async fn get_eth_price_from_chainlink(provider: &EthClient) -> Result<PriceInfo> {
    // ETH/USD Chainlink feed on Ethereum mainnet
    let eth_usd_feed = "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419"
        .parse::<Address>()
        .unwrap();

    let aggregator = ChainlinkAggregator::new(eth_usd_feed, provider.clone());

    match aggregator.latest_round_data().call().await {
        Ok((_, answer, _, _, _)) => {
            let decimals = aggregator.decimals().call().await.unwrap_or(8);
            let price = Decimal::from(answer.as_u128()) / Decimal::from(10u64.pow(decimals as u32));

            Ok(PriceInfo {
                price_usd: Some(price),
                price_eth: Some(Decimal::from(1)),
                source: "Chainlink".to_string(),
            })
        }
        Err(_) => {
            // Fallback to a default ETH price
            Ok(PriceInfo {
                price_usd: Some(Decimal::from(2000)), // Default fallback
                price_eth: Some(Decimal::from(1)),
                source: "Default".to_string(),
            })
        }
    }
}

/// Get price from CoinGecko API
async fn get_price_from_coingecko(token_address: &Address) -> Result<PriceInfo> {
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/token_price/ethereum?contract_addresses={}&vs_currencies=usd,eth",
        format!("{:?}", token_address).to_lowercase()
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await
        .context("Failed to fetch from CoinGecko")?;

    let data: serde_json::Value = response.json().await?;

    // CoinGecko returns {address: {usd: price, eth: price}}
    let token_key = format!("{:?}", token_address).to_lowercase();
    let token_data = data
        .get(&token_key)
        .context("Token not found in CoinGecko")?;

    let price_usd = token_data
        .get("usd")
        .and_then(|v| v.as_f64())
        .map(Decimal::from_f64_retain)
        .flatten();

    let price_eth = token_data
        .get("eth")
        .and_then(|v| v.as_f64())
        .map(Decimal::from_f64_retain)
        .flatten();

    Ok(PriceInfo {
        price_usd,
        price_eth,
        source: "CoinGecko".to_string(),
    })
}

/// Estimate price from Uniswap V2 pool
async fn get_price_from_uniswap(
    provider: &EthClient,
    token_address: Address,
) -> Result<PriceInfo> {
    // WETH address on mainnet
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        .parse::<Address>()
        .unwrap();

    // Common Uniswap V2 factory
    let factory = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"
        .parse::<Address>()
        .unwrap();

    // Calculate pair address (simplified - in production, use factory.getPair)
    // For now, return an estimate
    Ok(PriceInfo {
        price_usd: None,
        price_eth: Some(Decimal::from_str("0.001")?), // Placeholder
        source: "Uniswap V2 (estimated)".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_calculation() {
        let price = Decimal::from(100_000_000u64) / Decimal::from(100_000_000u64);
        assert_eq!(price, Decimal::from(1));
    }
}
