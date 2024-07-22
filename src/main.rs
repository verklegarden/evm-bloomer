//! EVM-Bloomer ...
//!
#[macro_use]
extern crate lazy_static;
mod asm_parser;
use alloy::transports::http::reqwest::Url;
use clap::Parser;
use eyre::Result;

use evm_bloomer::EVMBloom;

use crate::asm_parser::verify_project::{verify_project_bloom_filter,get_project_bloom};

#[derive(Parser)]
struct Cli {
    /// The RPC URL of the EVM chain.
    rpc_url: String,
}

/**
 * Important operations:
 * - Create bloom for chain
 * - Compare chain's bloom against mainnet
 * - Compare chain's bloom against other chain's bloom
 * - Compare chain's bloom against specific version
 *
 * Questions:
 * - Are we mainnet EVM compatible?
 * - On which version is this EVM chain?
 *
 */

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let rpc_url: Url = match args.rpc_url.parse() {
        Ok(rpc) => rpc,
        Err(e) => panic!("Invalid RPC URL provided: {}", e),
    };

    // Create EVMBloom for given rpc url.
    let bloom = EVMBloom::create(rpc_url).await?;
    bloom.visualize()?.save("polygon2.png").expect("");
    // println!("{}", bloom);
    // println!("{}", bloom.to_table());

    // println!("Supports cancun? {}", bloom.supports_cancun());
    // println!("Is cancun? {}", bloom.is_version_cancun());

    // if !bloom.supports_cancun() {
    //     // Find non-supported opcodes.
    // }

    // Compute distance to mainnet.
    //let distance = bloom.compute_distance(&evm_bloomer::evm_bloom::EVMBloom::ethereum());
    //println!("Distance to mainnet: {}", distance);

    // Visualize.
    // bloom.visualize()?.save("sep.png").expect("Failed to save");


    ////////////////// Check Project uses valid ASM //////////////////
    // Compile a project with forge build --extra-output-files evm.assembly
    // Then provide the absolute path to the project "out" directory
    // println!("{:?}",verify_project_bloom_filter("/home/max/tmp/chronicle-dao/out",bloom));
    let bloom = get_project_bloom("/home/max/tmp/chronicle-dao/out");
    println!("{:?}",bloom);
    bloom.unwrap().visualize()?.save("timelock2.png").expect("");
    //////////////////////////////////////////////////////////////////

    Ok(())

}

/*
 *
 *
 * Arbitrum:
 * 1111111111110000111111111111110010000000000000001111111111111111111111111100000011111111111111111111111111111111111111111111111111111111111111111111111111111111111110000000000000000000000000000000000000000000000000000000000000000000000000001111110000100101
 *
 * Polygon PoS:
 * 1111111111110000111111111111110010000000000000001111111111111111111111111000000011111111111111111111111111111111111111111111111111111111111111111111111111111111111110000000000000000000000000000000000000000000000000000000000000000000000000001111110000100101
 *
 *
 *
 */
