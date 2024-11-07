use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use std::sync::Arc;
use crate::env;

abigen!(
    Escrow,
    r#"[
    {
        "constant": false,
        "inputs": [
            {
                "components": [
                    { "name": "tokenIn", "type": "address" },
                    { "name": "amountIn", "type": "uint256" },
                    { "name": "srcUser", "type": "address" },
                    { "name": "tokenOut", "type": "string" },
                    { "name": "amountOut", "type": "uint256" },
                    { "name": "dstUser", "type": "string" },
                    { "name": "winnerSolver", "type": "string" },
                    { "name": "timeout", "type": "uint256" }
                ],
                "name": "newIntentInfo",
                "type": "tuple"
            }
        ],
        "name": "escrowFunds",
        "outputs": [{ "name": "", "type": "uint256" }],
        "payable": true,
        "stateMutability": "payable",
        "type": "function"
    }
    ]"#
);

abigen!(
    ERC20,
    r#"[
    {
        "constant": false,
        "inputs": [
            { "name": "_spender", "type": "address" },
            { "name": "_value", "type": "uint256" }
        ],
        "name": "approve",
        "outputs": [{ "name": "", "type": "bool" }],
        "type": "function"
    }]"#
);

pub async fn escrow_and_store_intent_ethereum(
    token_in: Address,
    amount_in: U256,
    token_out: String,
    amount_out: U256,
    mut dst_user: String,
    single_domain: bool,
    timeout: U256,
) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
    let contract_address = "0x393D402F48F0F468030082b5410a58cA2231FD34"; // Escrow Contract
    let private_key = env::var("ETHEREUM_PKEY").expect("ETHEREUM_PKEY must be set");
    let rpc_url = env::var("ETHEREUM_RPC").expect("ETHEREUM_RPC must be set");

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let provider = Arc::new(provider);

    let wallet: LocalWallet = private_key.parse()?;
    let wallet = wallet.with_chain_id(1u64); // Mainnet
    let wallet = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
    let src_user = wallet.address();

    let contract_address = contract_address.parse::<Address>()?;
    let contract = Escrow::new(contract_address, wallet.clone());

    // Set dst_user based on whether it's a single domain or cross domain transaction
    if single_domain {
        dst_user = format!("0x{:x}", src_user);
    }

    let intent = (
        token_in,         // token_in passed as parameter
        amount_in,        // amount_in passed as parameter
        src_user,         // src_user passed as parameter
        token_out,        // token_out passed as parameter
        amount_out,       // amount_out passed as parameter
        dst_user,         // dst_user determined based on single/cross domain
        "".to_string(),   // winner_solver (currently empty, can be passed as needed)
        timeout,          // timeout passed as parameter
    );

    // Call contract function with the constructed intent
    let contract = contract.escrow_funds(intent).value(U256::zero());
    let pending_tx = contract.send().await?;

    let tx_receipt = pending_tx
        .await?
        .expect("Failed to fetch transaction receipt");

    Ok(tx_receipt)
}

pub async fn _approve_erc20(
    provider_url: &str,
    private_key: &str,
    token_address: &str,
    spender_address: &str,
    amount: &str,
) -> Result<(), String> {
    let provider = Provider::<Http>::try_from(provider_url)
        .map_err(|e| format!("Failed to create provider: {}", e))?;
    let provider = Arc::new(provider);

    let wallet: LocalWallet = private_key
        .parse()
        .map_err(|e| format!("Failed to parse private key: {}", e))?;
    let wallet = wallet.with_chain_id(1u64); // Mainnet
    let wallet = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    let token_address = token_address
        .parse::<Address>()
        .map_err(|e| format!("Failed to parse token address: {}", e))?;
    let erc20 = ERC20::new(token_address, wallet.clone());

    let spender: Address = spender_address
        .parse::<Address>()
        .map_err(|e| format!("Failed to parse spender address: {}", e))?;
    let amount =
        U256::from_dec_str(amount).map_err(|e| format!("Failed to parse amount: {}", e))?;

    let tx = erc20.approve(spender, amount);
    let pending_tx = tx
        .send()
        .await
        .map_err(|e| format!("Failed to send transaction: {}", e))?;

    pending_tx
        .await
        .map_err(|e| format!("Transaction failed: {}", e))?;

    Ok(())
}
