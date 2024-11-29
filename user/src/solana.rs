use crate::env;
use crate::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::Client;
use anchor_lang::system_program;
use anchor_spl::associated_token;
use anchor_spl::associated_token::get_associated_token_address;
use anyhow::{anyhow, Result};
use jito_protos::searcher::SubscribeBundleResultsRequest;
use solana_sdk::pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::system_instruction;
use spl_associated_token_account::instruction;
use spl_token::instruction::sync_native;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use clap::builder::OsStr;
use solana_client::rpc_config::RpcSendTransactionConfig;
use tokio::time::sleep;
use tokio::time::Duration;
use strum::EnumString;
use strum_macros::{Display, IntoStaticStr};
use {
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{hash::Hash, instruction::Instruction, transaction::Transaction},
};

pub const JITO_ADDRESS: Pubkey =
    solana_program::pubkey!("96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5");
pub const JITO_TIP_AMOUNT: u64 = 10000;
pub const JITO_BLOCK_ENGINE_URL: &str = "https://mainnet.block-engine.jito.wtf";
pub const RETRIES: u8 = 5;

#[derive(Debug, Clone, Copy, Default, EnumString, Display, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum TxSendMethod {
    #[default]
    JITO,
    RPC,
}

impl From<&TxSendMethod> for OsStr {
    fn from(value: &TxSendMethod) -> Self {
        let string: &'static str = value.into();
        OsStr::from(string)
    }
}

pub async fn escrow_and_store_intent_solana(
    src_user: &Arc<Keypair>,
    auctioneer_state: Pubkey,
    client: &Client<Arc<Keypair>>,
    intent_id: String,
    amount_in: u64,
    mut token_in: Pubkey,
    dst_user: Pubkey,
    token_out: String,
    amount_out: String,
    timeout_duration: u64,
    single_domain: bool,
    tx_send_method: TxSendMethod,
) -> Result<Signature, String> {
    let program = match client.program(bridge_escrow::ID) {
        Ok(prog) => prog,
        Err(err) => return Err(format!("Failed to get program: {}", err)),
    };

    if token_in == Pubkey::from_str("11111111111111111111111111111111").unwrap() {
        token_in = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        ensure_wsol_balance_blocking(src_user, amount_in, tx_send_method)
            .await
            .unwrap();
    }

    let user_token_in_addr = get_associated_token_address(&src_user.pubkey(), &token_in);
    let token_in_escrow_addr = get_associated_token_address(&auctioneer_state, &token_in);

    let intent_state =
        Pubkey::find_program_address(&[b"intent", intent_id.as_bytes()], &bridge_escrow::ID).0;

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    println!("intent_id: {intent_id}");

    let new_intent = bridge_escrow::IntentPayload {
        intent_id: intent_id.clone(),
        user_in: src_user.pubkey(),
        user_out: dst_user.to_string(),
        token_in: token_in,
        amount_in: amount_in,
        token_out: token_out,
        amount_out: amount_out,
        timeout_timestamp_in_sec: current_timestamp + timeout_duration,
        single_domain: single_domain,
    };

    loop {
        let instructions = program
            .request()
            .accounts(bridge_escrow::accounts::EscrowAndStoreIntent {
                user: src_user.pubkey(),
                user_token_account: user_token_in_addr,
                auctioneer_state,
                token_mint: token_in,
                escrow_token_account: token_in_escrow_addr,
                intent: intent_state,
                token_program: anchor_spl::token::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            })
            .args(bridge_escrow::instruction::EscrowAndStoreIntent {
                new_intent: new_intent.clone(),
            })
            .payer(src_user.clone())
            .instructions()
            .unwrap();
        let sig = submit(&program.async_rpc(), src_user.clone(), instructions, tx_send_method).await;

        match sig {
            Ok(signature) => break Ok(signature), // Transaction succeeded, exit loop
            Err(err) if err.to_string().contains("unable to confirm transaction") => {
                eprintln!("Transaction failed: {}. Retrying...", err);
                let _ = sleep(Duration::from_secs(1));
            }
            Err(err) => {
                break Err(format!(
                    "Transaction failed due to a non-retryable error: {}",
                    err
                ))
            } // Break on other errors
        }
    }
}

pub async fn escrow_and_store_intent_cross_chain_solana(
    src_user: &Arc<Keypair>,
    auctioneer_state: Pubkey,
    client: &Client<Arc<Keypair>>,
    intent_id: String,
    amount_in: u64,
    mut token_in: Pubkey,
    dst_user: String,
    token_out: String,
    amount_out: String,
    timeout_duration: u64,
    single_domain: bool,
    tx_send_method: TxSendMethod,
) -> Result<Signature, String> {
    let program = match client.program(bridge_escrow::ID) {
        Ok(prog) => prog,
        Err(err) => return Err(format!("Failed to get program: {}", err)),
    };

    if token_in == Pubkey::from_str("11111111111111111111111111111111").unwrap() {
        token_in = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        ensure_wsol_balance_blocking(src_user, amount_in, tx_send_method)
            .await
            .unwrap();
    }

    let user_token_in_addr = get_associated_token_address(&src_user.pubkey(), &token_in);
    let token_in_escrow_addr = get_associated_token_address(&auctioneer_state, &token_in);

    let intent_state =
        Pubkey::find_program_address(&[b"intent", intent_id.as_bytes()], &bridge_escrow::ID).0;

    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    println!("intent_id: {intent_id}");

    let new_intent = bridge_escrow::IntentPayload {
        intent_id: intent_id.clone(),
        user_in: src_user.pubkey(),
        user_out: dst_user,
        token_in: token_in,
        amount_in: amount_in,
        token_out: token_out,
        amount_out: amount_out,
        timeout_timestamp_in_sec: current_timestamp + timeout_duration,
        single_domain: single_domain,
    };

    loop {
        let instructions = program
            .request()
            .accounts(bridge_escrow::accounts::EscrowAndStoreIntent {
                user: src_user.pubkey(),
                user_token_account: user_token_in_addr,
                auctioneer_state,
                token_mint: token_in,
                escrow_token_account: token_in_escrow_addr,
                intent: intent_state,
                token_program: anchor_spl::token::ID,
                associated_token_program: associated_token::ID,
                system_program: system_program::ID,
            })
            .args(bridge_escrow::instruction::EscrowAndStoreIntent {
                new_intent: new_intent.clone(),
            })
            .payer(src_user.clone())
            .instructions()
            .unwrap();

        let sig = submit(&program.async_rpc(), src_user.clone(), instructions, tx_send_method).await;

        match sig {
            Ok(signature) => break Ok(signature), // Transaction succeeded, exit loop
            Err(err) if err.to_string().contains("unable to confirm transaction") => {
                eprintln!("Transaction failed: {}. Retrying...", err);
                let _ = sleep(Duration::from_secs(1));
            }
            Err(err) => {
                break Err(format!(
                    "Transaction failed due to a non-retryable error: {}",
                    err
                ))
            } // Break on other errors
        }
    }
}

pub async fn ensure_wsol_balance_blocking(
    fee_payer: &Arc<Keypair>,
    amount_in: u64,
    tx_send_method: TxSendMethod,
) -> Result<(), String> {
    let rpc_url = env::var("SOLANA_RPC").expect("SOLANA_RPC must be set");
    let rpc_client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());

    // WSOL mint address
    let wsol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let wsol_token_address = get_associated_token_address(&fee_payer.pubkey(), &wsol_mint);

    // Check if WSOL account exists
    let account_data = rpc_client.get_account(&wsol_token_address).await;
    let current_balance = match account_data {
        Ok(_) => {
            // Account exists, fetch balance
            let token_balance = rpc_client
                .get_token_account_balance(&wsol_token_address)
                .await
                .map_err(|e| format!("Failed to fetch WSOL balance: {}", e))?;
            token_balance.amount.parse::<u64>().unwrap_or(0)
        }
        Err(_) => {
            // Account doesn't exist, create it
            // _create_token_account(&fee_payer.pubkey(), &wsol_mint, fee_payer, &rpc_client)
            //     .await
            //     .map_err(|e| format!("Failed to create WSOL token account: {}", e))?;
            0 // New account, so balance starts at 0
        }
    };

    // If the current balance is sufficient, do nothing
    if current_balance >= amount_in {
        return Ok(());
    }

    // Wrap SOL into WSOL
    let additional_amount = amount_in - current_balance;

    // Transfer SOL to WSOL account
    let transfer_sol_to_wsol_ix =
        system_instruction::transfer(&fee_payer.pubkey(), &wsol_token_address, additional_amount);

    // Sync WSOL balance
    let sync_wsol_balance_ix = sync_native(&spl_token::ID, &wsol_token_address);

    // Build and send the transaction
    let instructions = vec![transfer_sol_to_wsol_ix, sync_wsol_balance_ix.unwrap()];

    submit(&rpc_client, fee_payer.clone(), instructions, tx_send_method).await?;
    Ok(())
}

// fn _user_cancel_intent_solana(
//     wallet: Arc<Keypair>,
//     auctioneer_state: Pubkey,
//     client: Client<Arc<Keypair>>,
//     intent_id: String,
// ) {
//     // Derive necessary accounts
//     let user = wallet.pubkey();
//     let user_token_in = get_associated_token_address(
//         &user,
//         &Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap(),
//     );
//     let auctioneer_token_in = get_associated_token_address(
//         &auctioneer_state,
//         &Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap(),
//     );

//     // Call the on_timeout method to cancel intent
//     let program = client.program(bridge_escrow::ID);
//     let auctioneer = Pubkey::from_str("5zCZ3jk8EZnJyG7fhDqD6tmqiYTLZjik5HUpGMnHrZfC").unwrap();

//     let _sig = program
//         .unwrap()
//         .request()
//         .instruction(ComputeBudgetInstruction::set_compute_unit_limit(1_000_000)) // Optional: Increase compute units if needed
//         .accounts(bridge_escrow::accounts::OnTimeout {
//             user: wallet.pubkey(),
//             auctioneer_state,
//             auctioneer: auctioneer,
//             intent: Some(
//                 Pubkey::find_program_address(
//                     &[b"intent", intent_id.as_bytes()],
//                     &bridge_escrow::ID,
//                 )
//                 .0,
//             ),
//             token_in: Some(
//                 Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap(),
//             ), // Modify if cross-chain
//             user_token_account: Some(user_token_in),
//             escrow_token_account: Some(auctioneer_token_in),
//             token_program: anchor_spl::token::ID,
//             associated_token_program: anchor_spl::associated_token::ID,
//             system_program: solana_program::system_program::ID,
//         })
//         .args(bridge_escrow::instruction::UserCancelIntent {
//             intent_id: intent_id.clone(),
//         })
//         .payer(wallet.clone())
//         .signer(&*wallet)
//         .send_with_spinner_and_config(RpcSendTransactionConfig {
//             skip_preflight: true,
//             ..Default::default()
//         })
//         .unwrap();
// }

pub async fn _create_token_account(
    owner: &Pubkey,
    mint: &Pubkey,
    fee_payer: &Keypair,
    rpc_client: &RpcClient,
) -> Result<Signature, String> {
    let create_account_ix = instruction::create_associated_token_account(
        &fee_payer.pubkey(),
        owner,
        mint,
        &pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
    );

    let mut transaction =
        Transaction::new_with_payer(&[create_account_ix], Some(&fee_payer.pubkey()));

    let recent_blockhash: Hash = rpc_client.get_latest_blockhash().await.unwrap();
    transaction.sign(&[fee_payer], recent_blockhash);

    rpc_client.simulate_transaction(&transaction).await.unwrap();

    loop {
        let sig = rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .await;

        match sig {
            Ok(signature) => break Ok(signature), // Transaction succeeded, exit loop
            Err(err) if err.to_string().contains("unable to confirm transaction") => {
                eprintln!("Transaction failed: {}. Retrying...", err);
                let _ = sleep(Duration::from_secs(1));
            }
            Err(err) => {
                break Err(format!(
                    "Transaction failed due to a non-retryable error: {}",
                    err
                ))
            } // Break on other errors
        }
    }
}

pub async fn submit(
    rpc_client: &RpcClient,
    fee_payer: Arc<Keypair>,
    instructions: Vec<Instruction>,
    tx_send_method: TxSendMethod,
) -> Result<Signature, String> {
    match tx_send_method {
        TxSendMethod::JITO => submit_jito(rpc_client, fee_payer, instructions).await,
        TxSendMethod::RPC => submit_default(rpc_client, fee_payer, instructions).await,
    }.map_err(|e| e.to_string())
}

pub async fn submit_default(
    rpc_client: &RpcClient,
    fee_payer: Arc<Keypair>,
    instructions: Vec<Instruction>,
) -> Result<Signature> {

    let mut current_try = 0;
    loop {
        current_try += 1;

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .await
            .map_err(|e| anyhow!("Failed to fetch blockhash: {}", e))?;
        let transaction = Transaction::new_signed_with_payer(&instructions, Some(&fee_payer.pubkey()), &[&*fee_payer], recent_blockhash);

        let sig = rpc_client.send_and_confirm_transaction_with_spinner_and_config(&transaction, rpc_client.commitment(), RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        }).await;

        match sig {
            Ok(sig) => return Ok(sig), // Transaction succeeded, exit loop
            Err(err) if err.to_string().contains("unable to confirm transaction") => {
                eprintln!("Transaction failed: {}. Retrying...", err);
                if current_try == RETRIES {
                    return Err(anyhow!("Failed to send transaction: {}", err));
                }
                std::thread::sleep(Duration::from_secs(1));
            }
            Err(err) => {
                return Err(anyhow!(
                    "Transaction failed due to a non-retryable error: {}",
                    err
                ))
            } // Break on other errors
        }
    }
}

pub async fn submit_jito(
    rpc_client: &RpcClient,
    fee_payer: Arc<Keypair>,
    instructions: Vec<Instruction>,
) -> Result<Signature> {
    let ix = system_instruction::transfer(
        &fee_payer.pubkey(),
        &JITO_ADDRESS,
        JITO_TIP_AMOUNT,
    );

    let mut all_instructions = vec![ix];
    all_instructions.extend_from_slice(instructions.as_slice());

    let tx = Transaction::new_with_payer(all_instructions.as_slice(), Some(&fee_payer.pubkey()));

    let mut current_try = 0;
    let mut signature: Signature = Signature::default();
    while current_try < RETRIES {
        let mut cloned_tx = tx.clone();
        let mut client =
            jito_searcher_client::get_searcher_client(&JITO_BLOCK_ENGINE_URL, &fee_payer)
                .await?;
        let mut bundle_results_subscription = client
            .subscribe_bundle_results(SubscribeBundleResultsRequest {})
            .await?
            .into_inner();

        let blockhash = rpc_client.get_latest_blockhash().await?;
        cloned_tx.sign(&[&*fee_payer], blockhash);

        let signatures = jito_searcher_client::send_bundle_with_confirmation(
            &[cloned_tx.into()],
            &rpc_client,
            &mut client,
            &mut bundle_results_subscription,
        )
        .await
        .or_else(|e| {
            println!("This is error {:?}", e);
            Err(e)
        });

        if let Ok(sigs) = signatures {
            signature = *sigs.first().ok_or_else(|| anyhow!("No signature found"))?;
            return Ok(signature);
        } else {
            current_try += 1;
            continue;
        }
    }
    if current_try == RETRIES {
        println!("Failed to send transaction with the tries, Sending it through RPC Now");
        submit_default(rpc_client, fee_payer, instructions).await
    } else {
        Ok(signature)
    }
}
