use crate::env;
use crate::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::Client;
use anchor_lang::system_program;
use anchor_spl::associated_token;
use anchor_spl::associated_token::get_associated_token_address;
use anyhow::Result;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::system_instruction;
use spl_associated_token_account::instruction;
use spl_token::instruction::sync_native;
use std::rc::Rc;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::time::sleep;
use tokio::time::Duration;
use {
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{hash::Hash, transaction::Transaction},
};

pub fn escrow_and_store_intent_solana(
    src_user: &Rc<Keypair>,
    auctioneer_state: Pubkey,
    client: &Client<Rc<Keypair>>,
    intent_id: String,
    amount_in: u64,
    mut token_in: Pubkey,
    dst_user: Pubkey,
    token_out: String,
    amount_out: String,
    timeout_duration: u64,
    single_domain: bool,
) -> Result<Signature, String> {
    let program = match client.program(bridge_escrow::ID) {
        Ok(prog) => prog,
        Err(err) => return Err(format!("Failed to get program: {}", err)),
    };

    if token_in == Pubkey::from_str("11111111111111111111111111111111").unwrap() {
        token_in = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let _ = ensure_wsol_balance_blocking(src_user, amount_in).unwrap();
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
        let sig = program
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
            .signer(&*src_user)
            .send_with_spinner_and_config(RpcSendTransactionConfig {
                skip_preflight: true,
                ..Default::default()
            });

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

pub fn escrow_and_store_intent_cross_chain_solana(
    src_user: &Rc<Keypair>,
    auctioneer_state: Pubkey,
    client: &Client<Rc<Keypair>>,
    intent_id: String,
    amount_in: u64,
    mut token_in: Pubkey,
    dst_user: String,
    token_out: String,
    amount_out: String,
    timeout_duration: u64,
    single_domain: bool,
) -> Result<Signature, String> {
    let program = match client.program(bridge_escrow::ID) {
        Ok(prog) => prog,
        Err(err) => return Err(format!("Failed to get program: {}", err)),
    };

    if token_in == Pubkey::from_str("11111111111111111111111111111111").unwrap() {
        token_in = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let _ = ensure_wsol_balance_blocking(src_user, amount_in).unwrap();
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
        let sig = program
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
            .signer(&*src_user)
            .send_with_spinner_and_config(RpcSendTransactionConfig {
                skip_preflight: true,
                ..Default::default()
            });

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

pub fn ensure_wsol_balance_blocking(fee_payer: &Rc<Keypair>, amount_in: u64) -> Result<(), String> {
    let rpc_url = env::var("SOLANA_RPC").expect("SOLANA_RPC must be set");
    let rpc_client = solana_client::rpc_client::RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::confirmed(),
    );

    // WSOL mint address
    let wsol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let wsol_token_address = get_associated_token_address(&fee_payer.pubkey(), &wsol_mint);

    // Check if WSOL account exists
    let account_data = rpc_client.get_account(&wsol_token_address);
    let current_balance = match account_data {
        Ok(_) => {
            // Account exists, fetch balance
            let token_balance = rpc_client
                .get_token_account_balance(&wsol_token_address)
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
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .map_err(|e| format!("Failed to fetch blockhash: {}", e))?;
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&fee_payer.pubkey()));
    transaction.sign(&[fee_payer], recent_blockhash);

    loop {
        let sig = rpc_client.send_and_confirm_transaction_with_spinner(&transaction);

        match sig {
            Ok(_) => return Ok(()), // Transaction succeeded, exit loop
            Err(err) if err.to_string().contains("unable to confirm transaction") => {
                eprintln!("Transaction failed: {}. Retrying...", err);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Err(err) => {
                return Err(format!(
                    "Transaction failed due to a non-retryable error: {}",
                    err
                ))
            } // Break on other errors
        }
    }
}

// fn _user_cancel_intent_solana(
//     wallet: Rc<Keypair>,
//     auctioneer_state: Pubkey,
//     client: Client<Rc<Keypair>>,
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
