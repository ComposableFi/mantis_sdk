mod cli;
mod ethereum;
mod solana;

use std::env;
use std::sync::Arc;

use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::{Client, Cluster};
use anyhow::Result;
use clap::ArgMatches;
use ethers::types::Address;
use ethers::types::U256;
use rand::{distributions::Alphanumeric, Rng};
use solana_sdk::bs58;
use solana_sdk::signature::Signer;
use std::str::FromStr;

use crate::cli::parse_cli;
use crate::cli::parse_common_args;
use crate::ethereum::escrow_and_store_intent_ethereum;
use crate::solana::{escrow_and_store_intent_cross_chain_solana, escrow_and_store_intent_solana};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let matches = parse_cli();

    // Execute the appropriate function based on the subcommand used
    if let Some(solana_matches) = matches.subcommand_matches("solana") {
        let solana_matches_cloned = solana_matches.clone();
        handle_solana_single_domain_intent(&solana_matches_cloned)
            .await
            .expect("Failed to execute blocking code on solana");
    } else if let Some(solana_ethereum_matches) = matches.subcommand_matches("solana-ethereum") {
        let solana_ethereum_matches_cloned = solana_ethereum_matches.clone();
        handle_solana_ethereum_cross_domain_intent(&solana_ethereum_matches_cloned)
            .await
            .expect("Failed to execute blocking code on solana-ethereum");
    } else if let Some(ethereum_matches) = matches.subcommand_matches("ethereum") {
        let ethereum_matches_cloned = ethereum_matches.clone();
        handle_ethereum_single_domain_intent(&ethereum_matches_cloned)
            .await
            .unwrap();
    } else if let Some(ethereum_solana_matches) = matches.subcommand_matches("ethereum-solana") {
        handle_ethereum_solana_cross_domain_intent(&ethereum_solana_matches)
            .await
            .unwrap();
    }

    Ok(())
}

/// Handle the Ethereum -> Ethereum single domain intent.
async fn handle_ethereum_single_domain_intent(matches: &ArgMatches) -> Result<()> {
    let token_in = Address::from_str(matches.get_one::<String>("token_in").unwrap()).unwrap();
    let amount_in = U256::from(*matches.get_one::<u64>("amount_in").unwrap());
    let token_out = matches.get_one::<String>("token_out").unwrap().to_string();
    let amount_out = U256::from(*matches.get_one::<u64>("amount_out").unwrap());
    let timeout = U256::from(*matches.get_one::<u64>("timeout").unwrap());

    // Call the escrow function for single domain
    match escrow_and_store_intent_ethereum(
        token_in,
        amount_in,
        token_out,
        amount_out,
        String::default(),
        true,
        timeout,
    )
    .await
    {
        Ok(receipt) => {
            println!(
                "Transaction successful, receipt: {:?}",
                receipt.transaction_hash
            );
        }
        Err(e) => {
            println!("Transaction failed ** Remember you need to approve <TOKEN_IN> to Escrow SC 0x64E78873057769a5fd9A2278E6820666ec7e87f9 **: {:?}", e);
        }
    }

    Ok(())
}

/// Handle the Ethereum -> Solana cross-domain intent.
async fn handle_ethereum_solana_cross_domain_intent(matches: &ArgMatches) -> Result<()> {
    let token_in = Address::from_str(matches.get_one::<String>("token_in").unwrap()).unwrap();
    let amount_in = U256::from(*matches.get_one::<u64>("amount_in").unwrap());
    let token_out = matches.get_one::<String>("token_out").unwrap().to_string();
    let amount_out = U256::from(*matches.get_one::<u64>("amount_out").unwrap());
    let timeout = U256::from(*matches.get_one::<u64>("timeout").unwrap());
    let dst_user = matches.get_one::<String>("dst_user").unwrap().to_string();

    // Call the escrow function for cross domain
    match escrow_and_store_intent_ethereum(
        token_in, amount_in, token_out, amount_out, dst_user, false, timeout,
    )
    .await
    {
        Ok(receipt) => {
            println!(
                "Transaction successful, receipt: {:?}",
                receipt.transaction_hash
            );
        }
        Err(e) => {
            println!("Transaction failed ** Remember you need to approve <TOKEN_IN> to Escrow SC 0x64E78873057769a5fd9A2278E6820666ec7e87f9 **: {:?}", e);
        }
    }

    Ok(())
}

/// Handle the Solana -> Solana intent.
async fn handle_solana_single_domain_intent(matches: &ArgMatches) -> Result<()> {
    let private_key_bytes =
        bs58::decode(env::var("SOLANA_KEYPAIR").expect("SOLANA_KEYPAIR must be set"))
            .into_vec()
            .expect("Failed to decode Base58 private key");

    let wallet =
        Arc::new(Keypair::from_bytes(&private_key_bytes).expect("Failed to create keypair"));

    let auctioneer_state = Pubkey::find_program_address(&[b"auctioneer"], &bridge_escrow::ID).0;

    let client = Client::new_with_options(
        Cluster::Mainnet,
        wallet.clone(),
        CommitmentConfig::processed(),
    );

    let intent_id = generate_random_intent_id();
    let (amount_in, token_in, token_out, amount_out, timeout_duration) = parse_common_args(matches);

    let dst_user = wallet.pubkey();
    let single_domain = true;

    match escrow_and_store_intent_solana(
        &wallet,
        auctioneer_state,
        &client,
        intent_id,
        amount_in,
        token_in,
        dst_user,
        token_out,
        amount_out,
        timeout_duration,
        single_domain,
    )
    .await
    {
        Ok(sig) => {
            println!("Transaction successful, signature: {}", sig);
            sig
        }
        Err(e) => {
            println!("Transaction failed ** Remember you need to create a token_account for Escrow Program first **: {}", e);
            return Err(anyhow::anyhow!(e));
        }
    };

    Ok(())
}

/// Handle the Solana -> Ethereum cross-domain intent.
async fn handle_solana_ethereum_cross_domain_intent(matches: &ArgMatches) -> Result<()> {
    let private_key_bytes =
        bs58::decode(env::var("SOLANA_KEYPAIR").expect("SOLANA_KEYPAIR must be set"))
            .into_vec()
            .expect("Failed to decode Base58 private key");

    let wallet =
        Arc::new(Keypair::from_bytes(&private_key_bytes).expect("Failed to create keypair"));

    let auctioneer_state = Pubkey::find_program_address(&[b"auctioneer"], &bridge_escrow::ID).0;

    let client = Client::new_with_options(
        Cluster::Mainnet,
        wallet.clone(),
        CommitmentConfig::processed(),
    );

    let intent_id = generate_random_intent_id();
    let (amount_in, token_in, token_out, amount_out, timeout_duration) = parse_common_args(matches);

    let dst_user = matches.get_one::<String>("dst_user").unwrap().to_string();
    let single_domain = false;

    match escrow_and_store_intent_cross_chain_solana(
        &wallet,
        auctioneer_state,
        &client,
        intent_id,
        amount_in,
        token_in,
        dst_user,
        token_out,
        amount_out,
        timeout_duration,
        single_domain,
    )
    .await
    {
        Ok(sig) => {
            println!("Transaction successful, signature: {}", sig);
            sig
        }
        Err(e) => {
            println!("Transaction failed ** Remember you need to create a token_account for Escrow Program first **: {}", e);
            return Err(anyhow::anyhow!(e));
        }
    };

    Ok(())
}

/// Generate a random intent ID.
fn generate_random_intent_id() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}
