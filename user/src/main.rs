mod cli;
mod ethereum;
mod quotes;
mod solana;

use std::env;
use std::rc::Rc;

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

    if let Some(new_intent_matches) = matches.subcommand_matches("new-intent") {
        // Execute the appropriate function based on the subcommand used
        if let Some(solana_matches) = new_intent_matches.subcommand_matches("solana") {
            let solana_matches_cloned = solana_matches.clone();
            tokio::task::spawn_blocking(move || {
                handle_solana_single_domain_intent(&solana_matches_cloned).unwrap();
            })
            .await
            .expect("Failed to execute blocking code on solana");
        } else if let Some(solana_ethereum_matches) =
            new_intent_matches.subcommand_matches("solana-ethereum")
        {
            let solana_ethereum_matches_cloned = solana_ethereum_matches.clone();
            tokio::task::spawn_blocking(move || {
                handle_solana_ethereum_cross_domain_intent(&solana_ethereum_matches_cloned)
                    .unwrap();
            })
            .await
            .expect("Failed to execute blocking code on solana-ethereum");
        } else if let Some(ethereum_matches) = new_intent_matches.subcommand_matches("ethereum") {
            let ethereum_matches_cloned = ethereum_matches.clone();
            handle_ethereum_single_domain_intent(&ethereum_matches_cloned)
                .await
                .unwrap();
        } else if let Some(ethereum_solana_matches) =
            new_intent_matches.subcommand_matches("ethereum-solana")
        {
            handle_ethereum_solana_cross_domain_intent(&ethereum_solana_matches)
                .await
                .unwrap();
        }
    } else if let Some(query_quote_matches) = matches.subcommand_matches("query-quote") {
        handle_quote_query(&query_quote_matches).await.unwrap();
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
            println!("Transaction successful, receipt: {:?}", receipt);
        }
        Err(e) => {
            println!("Transaction failed ** Remember you need to approve <TOKEN_IN> to Escrow SC 0x59880a68fafcE2E282866bdb741Cf0b20E95c1B7 **: {:?}", e);
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
            println!("Transaction successful, receipt: {:?}", receipt);
        }
        Err(e) => {
            println!("Transaction failed ** Remember you need to approve <TOKEN_IN> to Escrow SC 0x59880a68fafcE2E282866bdb741Cf0b20E95c1B7 **: {:?}", e);
        }
    }

    Ok(())
}

/// Handle the Solana -> Solana intent.
fn handle_solana_single_domain_intent(matches: &ArgMatches) -> Result<()> {
    let private_key_bytes =
        bs58::decode(env::var("SOLANA_KEYPAIR").expect("SOLANA_KEYPAIR must be set"))
            .into_vec()
            .expect("Failed to decode Base58 private key");

    let wallet =
        Rc::new(Keypair::from_bytes(&private_key_bytes).expect("Failed to create keypair"));

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
    ) {
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
fn handle_solana_ethereum_cross_domain_intent(matches: &ArgMatches) -> Result<()> {
    let private_key_bytes =
        bs58::decode(env::var("SOLANA_KEYPAIR").expect("SOLANA_KEYPAIR must be set"))
            .into_vec()
            .expect("Failed to decode Base58 private key");

    let wallet =
        Rc::new(Keypair::from_bytes(&private_key_bytes).expect("Failed to create keypair"));

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
    ) {
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

async fn handle_quote_query(matches: &ArgMatches) -> Result<()> {
    let auctioneer_url = env::var("AUCTIONEER_URL").expect("AUCTIONEER_URL must be set");
    let (subcmd, cmd_matches) = matches.subcommand().expect("No subcommand given");
    let networks: Vec<&str> = subcmd.split("-").collect();
    let (src_chain, dst_chain) =
        if networks.len() > 1 {
            (networks[0].to_string(), networks[1].to_string())
        } else {
            (networks[0].to_string(), networks[0].to_string())
        };    
    let query = quotes::Query {
        src_chain,
        dst_chain,
        token_in: cmd_matches.get_one::<String>("token_in").unwrap().to_string(),
        token_out: cmd_matches.get_one::<String>("token_out").unwrap().to_string(),
        amount: cmd_matches.get_one::<u64>("amount").unwrap().to_string(),
        src_address: cmd_matches.get_one::<String>("src_address").unwrap().to_string(),
        dst_address: cmd_matches.get_one::<String>("dst_address").unwrap().to_string(),
    };
    let http_client = reqwest::Client::new();
    let request = http_client.get(format!("{}/query_quote", auctioneer_url))
        .header("Content-Type", "application/json")
        .json(&query);
    let response = request.send().await?;
    let output: quotes::QuoteResponse = response.json().await?;

    let mut solver_width = 0;
    let mut token_width = 0;
    let mut amount_width = 0;
    for quote in output.outputs.iter() {
        solver_width = std::cmp::max(solver_width, quote.solver_id.len());
        token_width = std::cmp::max(token_width, quote.quote.token.len());
        amount_width = std::cmp::max(amount_width, quote.quote.amount.len());
    }
    let hline = format!(
        "+-{}-+-{}-+-{}-+",
        "-".repeat(solver_width),
        "-".repeat(token_width),
        "-".repeat(amount_width)
    );

    println!("Quotes:");
    println!("{}", hline);
    println!(
        "| {:<solver_width$} | {:<token_width$} | {:<amount_width$} |",
        "Solver",
        "Output token",
        "Amount",
        solver_width = solver_width,
        token_width = token_width,
        amount_width = amount_width
    );
    println!("{}", hline);
    for quote in output.outputs.iter() {
        println!(
            "| {:<solver_width$} | {:<token_width$} | {:<amount_width$} |",
            quote.solver_id,
            quote.quote.token,
            quote.quote.amount,
            solver_width = solver_width,
            token_width = token_width,
            amount_width = amount_width
        );
    }
    println!("{}", hline);
    println!(
        "| {:<solver_width$} | {:<token_width$} | {:<amount_width$} |",
        "",
        "Mean amount",
        output.mean_output.amount,
        solver_width = solver_width,
        token_width = token_width,
        amount_width = amount_width
    );
    println!("{}", hline);
    Ok(())
}
