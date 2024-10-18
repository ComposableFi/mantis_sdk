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
use spl_associated_token_account::instruction;
use std::rc::Rc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use {
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{hash::Hash, transaction::Transaction},
};

pub fn escrow_and_store_intent_solana(
    src_user: &Rc<Keypair>,
    auctioneer_state: Pubkey,
    client: &Client<Rc<Keypair>>,
    intent_id: String,     // Pass in the intent ID
    amount_in: u64,        // Pass in the input amount
    token_in: Pubkey,      // Pass in the input token
    dst_user: Pubkey,      // Pass in the destination user
    token_out: String,     // Pass in the output token
    amount_out: String,    // Pass in the output amount
    timeout_duration: u64, // Pass in the timeout duration
    single_domain: bool,   // Pass in the single domain flag
) -> Result<Signature, String> {
    let program = match client.program(bridge_escrow::ID) {
        Ok(prog) => prog,
        Err(err) => return Err(format!("Failed to get program: {}", err)),
    };

    let user_token_in_addr = get_associated_token_address(&src_user.pubkey(), &token_in);
    let token_in_escrow_addr = get_associated_token_address(&auctioneer_state, &token_in);

    let intent_state =
        Pubkey::find_program_address(&[b"intent", intent_id.as_bytes()], &bridge_escrow::ID).0;

    // Get the current timestamp
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Define the new intent payload
    let new_intent = bridge_escrow::IntentPayload {
        intent_id: intent_id.clone(),
        user_in: src_user.pubkey(), // Must match the ctx.accounts.user key in the contract
        user_out: dst_user.to_string(),
        token_in: token_in,
        amount_in: amount_in,
        token_out: token_out,
        amount_out: amount_out, // Amount out as a string
        timeout_timestamp_in_sec: current_timestamp + timeout_duration,
        single_domain: single_domain,
    };

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
        .args(bridge_escrow::instruction::EscrowAndStoreIntent { new_intent })
        .payer(src_user.clone())
        .signer(&*src_user)
        .send_with_spinner_and_config(RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        });

    match sig {
        Ok(signature) => Ok(signature),
        Err(err) => Err(format!("Transaction failed: {}", err)),
    }
}

pub fn escrow_and_store_intent_cross_chain_solana(
    src_user: &Rc<Keypair>,
    auctioneer_state: Pubkey,
    client: &Client<Rc<Keypair>>,
    intent_id: String,     // Pass in the intent ID
    amount_in: u64,        // Pass in the input amount
    token_in: Pubkey,      // Pass in the input token
    dst_user: String,      // Pass in the destination user (cross-chain)
    token_out: String,     // Pass in the output token (cross-chain)
    amount_out: String,    // Pass in the output amount (cross-chain)
    timeout_duration: u64, // Pass in the timeout duration
    single_domain: bool,   // Pass in the single domain flag
) -> Result<Signature, String> {
    let program = match client.program(bridge_escrow::ID) {
        Ok(prog) => prog,
        Err(err) => return Err(format!("Failed to get program: {}", err)),
    };
    let user_token_in_addr = get_associated_token_address(&src_user.pubkey(), &token_in);
    let token_in_escrow_addr = get_associated_token_address(&auctioneer_state, &token_in);

    let intent_state =
        Pubkey::find_program_address(&[b"intent", intent_id.as_bytes()], &bridge_escrow::ID).0;

    // Get the current timestamp
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Define the new intent payload
    let new_intent = bridge_escrow::IntentPayload {
        intent_id: intent_id.clone(),
        user_in: src_user.pubkey(), // Must match the ctx.accounts.user key in the contract
        user_out: dst_user,
        token_in: token_in,
        amount_in: amount_in,
        token_out: token_out,
        amount_out: amount_out, // Amount out as a string
        timeout_timestamp_in_sec: current_timestamp + timeout_duration,
        single_domain: single_domain,
    };

    let sig = program
        .request()
        // .instruction(ComputeBudgetInstruction::set_compute_unit_limit(1_000_000))
        // .instruction(ComputeBudgetInstruction::request_heap_frame(128 * 1024))
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
        .args(bridge_escrow::instruction::EscrowAndStoreIntent { new_intent })
        .payer(src_user.clone())
        .signer(&*src_user)
        .send_with_spinner_and_config(RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        });

    match sig {
        Ok(signature) => Ok(signature),
        Err(err) => Err(format!("Transaction failed: {}", err)),
    }
}
