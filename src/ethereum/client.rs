use anyhow::{Context, Result};
use ethers::prelude::*;
use std::sync::Arc;

pub type EthClient = Arc<Provider<Http>>;

/// Create an Ethereum provider from RPC URL
pub async fn create_provider(rpc_url: &str) -> Result<EthClient> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .context("Failed to create provider")?
        .interval(std::time::Duration::from_millis(10u64));

    Ok(Arc::new(provider))
}

/// Create a wallet from private key
pub fn create_wallet(private_key: &str) -> Result<LocalWallet> {
    let wallet = private_key
        .parse::<LocalWallet>()
        .context("Failed to parse private key")?;

    Ok(wallet)
}

/// Get wallet with provider (signer)
pub fn create_signer(
    wallet: LocalWallet,
    provider: EthClient,
    chain_id: u64,
) -> SignerMiddleware<Provider<Http>, LocalWallet> {
    let wallet = wallet.with_chain_id(chain_id);
    SignerMiddleware::new((*provider).clone(), wallet)
}
