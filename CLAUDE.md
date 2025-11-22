## Overview

Build a Model Context Protocol (MCP) server in Rust that enables AI agents to query balances and execute token swaps on Ethereum.


## Requirements

### Core Functionality

Implement an MCP server with the following tools:

1. **`get_balance`** - Query ETH and ERC20 token balances
    - Input: wallet address, optional token contract address
    - Output: balance information with proper decimals
2. **`get_token_price`** - Get current token price in USD or ETH
    - Input: token address or symbol
    - Output: price data
3. **`swap_tokens`** - Execute a token swap on Uniswap V2 or V3
    - Input: from_token, to_token, amount, slippage tolerance
    - Output: simulation result showing estimated output and gas costs
    - **Important**: Construct a real Uniswap transaction and submit it to the blockchain for simulation (using `eth_call` or similar). The transaction should NOT be executed on-chain.

### Technical Stack

**Required:**

- Rust with async runtime (tokio)
- Ethereum RPC client library (ethers-rs or alloy)
- MCP SDK for Rust ([rmcp](https://github.com/modelcontextprotocol/rust-sdk)) or implement JSON-RPC 2.0 manually
- Structured logging (tracing)

### Constraints

- Must connect to real Ethereum RPC (use public endpoints or Infura/Alchemy)
- Balance queries must fetch real on-chain data
- For swaps: construct real Uniswap V2/V3 swap transactions and simulate them using RPC methods
- Transaction signing: implement basic wallet management (e.g., private key via environment variable or config file)
- Use `rust_decimal` or similar for financial precision

## Deliverables

1. **Working code** - Rust project that compiles and runs ✅
2. **README** with: ✅
    - Setup instructions (dependencies, env vars, how to run)
    - Example MCP tool call (show JSON request/response)
    - Design decisions (3-5 sentences on your approach)
    - Known limitations or assumptions
3. **Tests** - Demonstrate core functionality ✅

## Integration with AI Agents

This MCP server is designed to be consumed by AI agents. Here's how to set it up:

### Claude Desktop Configuration

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Configure Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):
   ```json
   {
     "mcpServers": {
       "ethereum-trading": {
         "command": "/path/to/eth-trading-mcp-server/target/release/eth-trading-mcp-server",
         "env": {
           "ETH_RPC_URL": "https://eth.llamarpc.com"
         }
       }
     }
   }
   ```

3. **Test by chatting with Claude:**
   - "What's the ETH balance of 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045?"
   - "Get me the price of USDC (0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48)"
   - "Simulate swapping 0.1 ETH to USDC"

The AI agent will call the appropriate tools and present results in natural language.

## Manual Testing (Without AI Agent)

### Python Test Client

A Python test client (`client_example.py`) is provided for testing the server directly:

```bash
# Make the script executable
chmod +x client_example.py

# Initialize the server
python3 client_example.py init

# List available tools
python3 client_example.py list

# Test balance query (example: Vitalik's address)
python3 client_example.py balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

# Test token price (example: USDC)
python3 client_example.py price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

# Test swap simulation (ETH -> USDC)
python3 client_example.py swap 0x0000000000000000000000000000000000000000 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 0.1 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
```

**Available commands:**
- `init` - Initialize the MCP server connection
- `list` - List all available tools
- `balance <wallet_address>` - Query wallet balance
- `price <token_address>` - Get token price (use `0x0000000000000000000000000000000000000000` for ETH)
- `swap <from_token> <to_token> <amount> <wallet_address>` - Simulate token swap

### Bash Test Script

A simple bash script (`test_mcp.sh`) is also available for quick testing:

```bash
chmod +x test_mcp.sh
./test_mcp.sh
```

### Rust Tests

Run the integrated test suite:
```bash
cargo test
cargo test -- --nocapture  # With output
```

## Development Approach

You're **encouraged** to use AI assistants (Cursor, Claude Code, GitHub Copilot, etc.) while working on this assignment. However, the solution should demonstrate your understanding of:

- Rust and async programming
- Ethereum fundamentals
- System design and architecture

The code will be reviewed for comprehension and design decisions.

## Submission

Create a GitHub repository and share the link. Ensure:

- `cargo build` compiles successfully
- `cargo test` passes
- README has clear setup instructions
- Code is well-organized and readable
