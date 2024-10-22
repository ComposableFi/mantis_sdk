# Mantis SDK Intent CLI: 

## 🚀 Quick Start

```bash
cargo run -- [COMMAND]

Commands:
  solana           Solana -> Solana single domain intent
  solana-ethereum  Solana -> Ethereum cross-domain intent
  ethereum         Ethereum -> Ethereum single domain intent
  ethereum-solana  Ethereum -> Solana cross-domain intent
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## 🗺️ Navigation

Our CLI offers the following pathways:

- 🌞 `new-intent`: Submit a new intent into the network
- 🌞 `query-quote`: Submit a new intent into the network

Beneath each of these commands there are 4 sub-commands:

- 🌞 `solana`: For Solana -> Solana single domain interactions
- 🌙 `ethereum`: For Ethereum -> Ethereum single domain interactions
- 🌠 `solana-ethereum`: For the daring Solana -> Ethereum cross-domain 
- 🌌 `ethereum-solana`: For the brave Ethereum -> Solana cross-domain

## 🧭 new-intent Command Details

### 🌞 Solana Single Domain

```bash
cargo run -- solana <amount_in> <token_in> <token_out> <amount_out> <timeout>
```

### 🌙 Ethereum Single Domain

```bash
cargo run -- ethereum <token_in> <amount_in> <token_out> <amount_out> <timeout>
```

#### 🔓 Ethereum Approval
Before your first Ethereum transaction, you'll need to send this approval:

```rust
pub async fn _approve_erc20()
```

This only needs to be done once to grant the necessary permissions.

### 🌠 Solana to Ethereum

```bash
cargo run -- solana-ethereum <amount_in> <token_in> <token_out> <amount_out> <timeout> <dst_user>
```

### 🌌 Ethereum to Solana

```bash
cargo run -- ethereum-solana <token_in> <amount_in> <token_out> <amount_out> <timeout> <dst_user>
```

## 🎭 Arguments Explained

- `amount_in`: The amount you're sending (in tokens)
- `token_in`: The address of your input token
- `token_out`: The address of your desired output token
- `amount_out`: The amount you expect to receive (in tokens)
- `timeout`: The duration in UNIX timestamp before you can withdraw token_in
- `dst_user`: The address of the recipient (for cross-domain only)

## 🎭 query-quote Command details

This command builds a quotes query and dispatches it to the auctioneer given by
the `AUCTIONEER_URL` environment variable (see below). It waits for the results
to arrive and presents them. Sub-commands (`ethereum`, `solana`, etc.) determine
to which networks input token and output token should belong.

Quotes query returns an amount of output tokens that can be swapped for the given
amount of input token by various solvers available on the network. The result
contains a list of solvers who can handle the intent along with the amount of
output tokens each of them is willing to swap for.

## 🎭 Arguments Explained

- `token_in`: The address of your input token
- `src_address`: The address to send input tokens from
- `amount`: The amount of tokens to swap
- `dst_address`: The address to send output tokens to
- `token_out`: The address of your output token

## 🗝️ Environment Variables

Make sure to set up your .env file with these keys:

```env
ETHEREUM_RPC=""      # Your Ethereum node RPC URL
ETHEREUM_PKEY=""     # Your Ethereum private key
SOLANA_KEYPAIR=""    # Your Solana wallet private key (e.g., Phantom wallet private key)
AUCTIONEER_URL=""    # Auctioneer to query quotes from
```

## 🌟 Examples

1. 🌞 Solana Single Domain
   ```bash
   cargo run -- solana 100 So11111111111111111111111111111111111111112 EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v 95 3600
   ```

2. 🌙 Ethereum Single Domain
   ```bash
   cargo run -- ethereum 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 1000 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 0.5 7200
   ```

3. 🌠 Solana to Ethereum
   ```bash
   cargo run -- solana-ethereum 50 So11111111111111111111111111111111111111112 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 49 3600 0x742d35Cc6634C0532925a3b844Bc454e4438f44e
   ```

4. 🌌 Ethereum to Solana
   ```bash
   cargo run -- ethereum-solana 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 100 EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v 99 7200 9ZNTfG4NyQgxy2SWjSiQoUyBPEvXT2xo7fKc5hPYYJ7b
   ```
