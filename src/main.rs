//! EVM-Bloomer is a tool to inspect EVM versions of EVM compatible chains.
//!
//! The tool generates a bloom filter encoding the support of each possible
//! EVM opcode for a chain. Further, it outputs the latest version the chain's
//! EVM matches to and outputs the list of unknown opcodes supported.
//!
//! Example:
//!
//!     $ evm-bloomer -r $rpc_eth $rpc_oeth $rpc_arb1
//!
use clap::Parser;
use eyre::Result;
use futures::future::try_join_all;
use serde::Serialize;
use serde_json::json;

use alloy::transports::http::reqwest::Url;

use evm_bloomer::EVMBloom;

#[derive(Parser)]
struct Cli {
    /// The RPC URLs to report.
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub rpc_urls: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Report {
    // TODO: Timestamp
    blooms: Vec<EVMBloom>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    // Parse RPC URL arguments.
    let mut rpc_urls: Vec<Url> = vec![];
    for rpc_url in args.rpc_urls {
        match rpc_url.parse() {
            Ok(rpc) => rpc_urls.push(rpc),
            Err(e) => eprintln!("invalid rpc: {}: {}", e, rpc_url),
        }
    }

    // Asynchronously generate blooms.
    let mut futures_results = vec![];
    for rpc_url in rpc_urls {
        futures_results.push(EVMBloom::new(rpc_url.clone()));
    }
    let blooms: Vec<EVMBloom> = try_join_all(futures_results)
        .await
        .expect("failed to generate blooms");

    let report = Report { blooms };
    println!("{}", json!(report)); // .json());

    Ok(())
}
