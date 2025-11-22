# Ethereum Trading MCP Server

A Model Context Protocol (MCP) server implementation in Rust that enables AI agents to query balances and simulate token swaps on Ethereum.

## Features

- **Balance Queries**: Query ETH and ERC20 token balances for any address
- **Price Feeds**: Get real-time token prices from CoinGecko and Chainlink oracles
- **Swap Simulation**: Simulate Uniswap V2 swaps with gas estimation (no on-chain execution)
- **MCP Protocol**: Full JSON-RPC 2.0 implementation following MCP specification
- **Production-Ready**: Uses ethers-rs for Ethereum interactions, rust_decimal for financial precision

## Prerequisites

- Rust 1.70 or higher
- An Ethereum RPC endpoint (public or from Infura/Alchemy)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd eth-trading-mcp-server
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` and configure your Ethereum RPC URL:
```bash
ETH_RPC_URL=https://eth.llamarpc.com
# Or use your own Infura/Alchemy endpoint:
# ETH_RPC_URL=https://mainnet.infura.io/v3/YOUR_API_KEY
```

4. Build the project:
```bash
cargo build --release
```

## Running the Server

The MCP server communicates via stdio (standard input/output):

```bash
cargo run --release
```

The server will:
1. Connect to the Ethereum RPC endpoint
2. Verify the connection by fetching the chain ID
3. Listen for JSON-RPC requests on stdin
4. Send responses on stdout

## Available Tools

### 1. get_balance

Query ETH or ERC20 token balance for a wallet address.

**Parameters:**
- `wallet_address` (string, required): The wallet address to query (0x...)
- `token_address` (string, optional): ERC20 token contract address. If not provided, returns ETH balance.

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_balance",
    "arguments": {
      "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Balance: 1.234567890123456789 ETH\nDecimals: 18\nWallet: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb\nRaw balance: 1234567890123456789"
      }
    ]
  }
}
```

### 2. get_token_price

Get current token price in USD and ETH from price oracles.

**Parameters:**
- `token_address` (string, required): Token contract address. Use `0x0000000000000000000000000000000000000000` for ETH.

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "get_token_price",
    "arguments": {
      "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Token: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48\nPrice (USD): 1.00\nPrice (ETH): 0.0005\nSource: CoinGecko"
      }
    ]
  }
}
```

### 3. swap_tokens

Simulate a token swap on Uniswap V2 without executing the transaction.

**Parameters:**
- `from_token` (string, required): Source token address. Use `0x0000000000000000000000000000000000000000` for ETH.
- `to_token` (string, required): Destination token address
- `amount` (string, required): Amount to swap in token units (e.g., "1.5")
- `slippage_bps` (number, optional): Slippage tolerance in basis points (default: 50 = 0.5%)
- `wallet_address` (string, required): Wallet address for simulation

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "swap_tokens",
    "arguments": {
      "from_token": "0x0000000000000000000000000000000000000000",
      "to_token": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
      "amount": "1.0",
      "slippage_bps": 50,
      "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Swap Simulation:\nFrom: 0x0000000000000000000000000000000000000000\nTo: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48\nAmount In: 1.0\nEstimated Output: 2000.5\nMinimum Output (with slippage): 1990.4975\nEstimated Gas: 150000\nSlippage Tolerance: 50 bps (0.5%)\nRoute: 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 -> 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
      }
    ]
  }
}
```

## MCP Protocol Flow

1. **Initialize**: Client sends `initialize` request
2. **List Tools**: Client requests available tools with `tools/list`
3. **Call Tool**: Client invokes tools with `tools/call`

Example initialization:
```json
{
  "jsonrpc": "2.0",
  "id": 0,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "example-client",
      "version": "1.0.0"
    }
  }
}
```

## Design Decisions

### Architecture

1. **Modular Design**: Separated concerns into distinct modules (ethereum/, tools/, types.rs, mcp.rs) for maintainability and testability.

2. **Async-First**: Built on Tokio for efficient handling of concurrent RPC calls and future scalability.

3. **Simulation Only**: The `swap_tokens` tool uses `eth_estimateGas` and `getAmountsOut` to simulate swaps without executing transactions, ensuring safety for AI agents.

4. **Financial Precision**: Uses `rust_decimal` throughout to avoid floating-point errors in financial calculations.

5. **Price Oracle Strategy**: Implements a fallback chain (CoinGecko → Chainlink → Uniswap pools) to maximize price data availability.

### Implementation Details

- **Uniswap Integration**: Uses Uniswap V2 Router for swap simulations due to its simplicity and widespread adoption
- **ABI Generation**: Leverages ethers-rs `abigen!` macro for type-safe contract interactions
- **Error Handling**: Comprehensive error handling with anyhow for internal errors and JSON-RPC error codes for client responses
- **Logging**: Structured logging with tracing, output to stderr to avoid interfering with stdio protocol

## Known Limitations

1. **Mainnet Only**: Currently configured for Ethereum mainnet. Would need modifications for L2s or testnets.

2. **Price Feeds**: CoinGecko API has rate limits. For production, implement caching or use paid API tiers.

3. **Swap Routing**: Uses simple direct paths (token A → token B) or single-hop through WETH. Production systems should implement multi-hop routing for better prices.

4. **Gas Estimation**: May be inaccurate for complex scenarios. The `eth_estimateGas` call can fail if the wallet lacks sufficient balance.

5. **No Transaction Execution**: This server only simulates swaps. To execute real transactions, you'd need to:
   - Add proper wallet management with secure key storage
   - Implement transaction signing and broadcasting
   - Add confirmation tracking
   - Handle nonce management

6. **Uniswap V2 Only**: Does not support Uniswap V3 or other DEXs. V3's concentrated liquidity would provide better pricing but requires more complex integration.

## Testing

Run the test suite:
```bash
cargo test
```

Run with logging:
```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Development

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

Format code:
```bash
cargo fmt
```

Run linter:
```bash
cargo clippy
```

## Security Considerations

- Never commit `.env` files containing private keys
- This server simulates transactions only - no private keys are required for basic operation
- For production use, implement proper secret management (e.g., HashiCorp Vault, AWS Secrets Manager)
- Always validate and sanitize inputs, especially addresses and amounts

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
