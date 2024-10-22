use clap::{Arg, ArgMatches, Command};
use std::str::FromStr;
use crate::Pubkey;

pub fn parse_cli() -> ArgMatches {
    let quote_query_args = quote_query_args();
    Command::new("Mantis SDK Intent CLI")
        .version("1.0")
        .about("Handles Solana and Ethereum escrow intents. Single Domain & Cross Domain")
        .subcommand(
            Command::new("submit-intent")
                .about("Submit a new intent to the network")
                .subcommand(
                    Command::new("solana")
                        .about("Solana -> Solana single domain intent")
                        .args(common_args()), // Use common_args for Solana
                )
                .subcommand(
                    Command::new("solana-ethereum")
                        .about("Solana -> Ethereum cross-domain intent")
                        .args(cross_domain_args()), // Use cross_domain_args for cross domain
                )
                .subcommand(
                    Command::new("ethereum")
                        .about("Ethereum -> Ethereum single domain intent")
                        .args(common_args_ethereum()), // Use common_args_ethereum for Ethereum
                )
                .subcommand(
                    Command::new("ethereum-solana")
                        .about("Ethereum -> Solana cross-domain intent")
                        .args(cross_domain_args_ethereum()), // Use Ethereum cross domain args
                )
        )
        .subcommand(
            Command::new("query-quote")
                .about("Query an amount of output tokens offered for a given amount of input tokens")
                .subcommand(
                    Command::new("solana")
                        .about("Solana -> Solana single domain swap")
                        .args(quote_query_args.clone()),
                )
                .subcommand(
                    Command::new("solana-ethereum")
                        .about("Solana -> Ethereum cross-domain swap")
                        .args(quote_query_args.clone()),
                )
                .subcommand(
                    Command::new("ethereum")
                        .about("Ethereum -> Ethereum single domain swap")
                        .args(quote_query_args.clone()),
                )
                .subcommand(
                    Command::new("ethereum-solana")
                        .about("Ethereum -> Solana cross-domain swap")
                        .args(quote_query_args),
                )
        )
        .get_matches()
}

/// Parse Solana common arguments.
pub fn parse_common_args(matches: &ArgMatches) -> (u64, Pubkey, String, String, u64) {
    let amount_in: u64 = *matches
        .get_one::<u64>("amount_in")
        .expect("amount_in is required");
    let token_in = Pubkey::from_str(matches.get_one::<String>("token_in").unwrap())
        .expect("Invalid token_in address");
    let token_out = matches.get_one::<String>("token_out").unwrap().to_string();
    let amount_out = matches.get_one::<String>("amount_out").unwrap().to_string();
    let timeout_duration: u64 = *matches
        .get_one::<u64>("timeout")
        .expect("timeout is required");

    (amount_in, token_in, token_out, amount_out, timeout_duration)
}

/// Solana common arguments for single and cross-domain intents.
pub fn common_args() -> Vec<Arg> {
    vec![
        Arg::new("amount_in")
            .required(true)
            .value_parser(clap::value_parser!(u64)) 
            .help("Amount in tokens"),
        Arg::new("token_in")
            .required(true)
            .value_parser(clap::value_parser!(String))
            .help("Token input address"),
        Arg::new("token_out")
            .required(true)
            .value_parser(clap::value_parser!(String)) 
            .help("Token output address"),
        Arg::new("amount_out")
            .required(true)
            .value_parser(clap::value_parser!(String)) 
            .help("Amount out in tokens"),
        Arg::new("timeout")
            .required(true)
            .value_parser(clap::value_parser!(u64)) 
            .help("Timeout duration in seconds"),
    ]
}

/// Additional argument for cross-domain intents.
fn cross_domain_args() -> Vec<Arg> {
    let mut args = common_args();
    args.push(
        Arg::new("dst_user")
            .required(true)
            .help("Destination user address"),
    );
    args
}

/// Ethereum common arguments for single and cross-domain intents.
fn common_args_ethereum() -> Vec<Arg> {
    vec![
        Arg::new("token_in")
            .required(true)
            .help("Token input address"),
        Arg::new("amount_in")
            .required(true)
            .value_parser(clap::value_parser!(u64)) 
            .help("Amount in tokens"),
        Arg::new("token_out")
            .required(true)
            .help("Token output address"),
        Arg::new("amount_out")
            .required(true)
            .value_parser(clap::value_parser!(u64)) 
            .help("Amount out in tokens"),
        Arg::new("timeout")
            .required(true)
            .value_parser(clap::value_parser!(u64)) 
            .help("Timeout duration in seconds"),
    ]
}

/// Additional argument for cross-domain Ethereum intents.
fn cross_domain_args_ethereum() -> Vec<Arg> {
    let mut args = common_args_ethereum();
    args.push(
        Arg::new("dst_user")
            .required(true)
            .help("Destination user address"),
    );
    args
}

fn quote_query_args() -> Vec<Arg> {
    vec![
        Arg::new("token_in")
            .required(true)
            .help("Input token address"),
        Arg::new("src_address")
            .required(true)
            .help("The address where input tokens are coming from"),
        Arg::new("amount")
            .required(true)
            .value_parser(clap::value_parser!(u64)) 
            .help("Amount of input tokens to be swapped"),
        Arg::new("dst_address")
            .required(true)
            .help("The address where output tokens are going to"),
        Arg::new("token_out")
            .required(true)
            .help("Output token address"),
    ]
}
