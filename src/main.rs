mod ethereum;
mod mcp;
mod tools;
mod types;

use anyhow::{Context, Result};
use ethers::prelude::Middleware;
use std::io::{self, BufRead, Write};
use tracing_subscriber::EnvFilter;

use ethereum::create_provider;
use mcp::McpServer;
use types::JsonRpcRequest;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting Ethereum Trading MCP Server");

    // Get Ethereum RPC URL from environment
    let rpc_url = std::env::var("ETH_RPC_URL")
        .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());

    tracing::info!("Connecting to Ethereum RPC: {}", rpc_url);

    // Create provider
    let provider = create_provider(&rpc_url)
        .await
        .context("Failed to create Ethereum provider")?;

    // Test connection
    let chain_id = provider.get_chainid().await?;
    tracing::info!("Connected to chain ID: {}", chain_id);

    // Create MCP server
    let server = McpServer::new(provider);

    tracing::info!("MCP Server ready, listening on stdio");

    // Read from stdin and write to stdout
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        tracing::debug!("Received: {}", line);

        // Parse request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse request: {}", e);
                let error_response = types::JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: serde_json::Value::Null,
                    result: None,
                    error: Some(types::JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
                continue;
            }
        };

        // Handle request
        let response = server.handle_request(request).await;

        // Send response
        let response_json = serde_json::to_string(&response)?;
        tracing::debug!("Sending: {}", response_json);
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    tracing::info!("MCP Server shutting down");

    Ok(())
}
