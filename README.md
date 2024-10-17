# Mantis SDK Intent CLI: 


## 🚀 Quick Start

To embark on your cross-chain adventure, simply use the following incantation:

```bash
cargo run -- [COMMAND]
```

## 🗺️ Navigation

Our magical CLI offers the following pathways:

- 🌞 `solana`: For Solana -> Solana single domain intents
- 🌙 `ethereum`: For Ethereum -> Ethereum single domain intents
- 🌠 `solana-ethereum`: For the daring Solana -> Ethereum cross-domain journey
- 🌌 `ethereum-solana`: For the brave Ethereum -> Solana cross-domain expedition

## 🧭 Command Details

### 🌞 Solana Single Domain

```bash
cargo run -- solana <amount_in> <token_in> <token_out> <amount_out> <timeout>
```

### 🌙 Ethereum Single Domain

```bash
cargo run -- ethereum <token_in> <amount_in> <token_out> <amount_out> <timeout>
```

### 🌠 Solana to Ethereum

```bash
cargo run -- solana-ethereum <amount_in> <token_in> <token_out> <amount_out> <timeout> <dst_user>
```

### 🌌 Ethereum to Solana

```bash
cargo run -- ethereum-solana <token_in> <amount_in> <token_out> <amount_out> <timeout> <dst_user>
```

## 🎭 Arguments Explained

- `amount_in`: The treasure you're sending (in tokens)
- `token_in`: The magical address of your input token
- `token_out`: The mystical address of your desired output token
- `amount_out`: The treasure you expect to receive (in tokens)
- `timeout`: The duration (in seconds) before your intent turns into a pumpkin 🎃
- `dst_user`: The address of the lucky recipient (for cross-domain spells only)

## 🌟 Examples

Here are some examples to get you started on your cross-chain odyssey:

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

## 🎩 Final Words

Remember, with great power comes great responsibility. Use this magical tool wisely, and may your cross-chain adventures be prosperous! If you encounter any mystical bugs or have ideas for new spells, feel free to open an issue or submit a pull request.

Happy bridging! 🌈🚀
