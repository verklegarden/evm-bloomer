//! EVM-Bloomer ...

use clap::Parser;
use eyre::Result;

use evm_bloomer::EVMBloom;

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

    let rpc_url = match args.rpc_url.parse() {
        Ok(rpc) => rpc,
        Err(e) => panic!("Invalid RPC URL provided: {}", e),
    };

    // Create EVMBloom for given rpc url.
    let bloom = EVMBloom::create(rpc_url).await?;
    println!("{}", bloom);
    println!("{}", bloom.to_table());

    println!("Supports cancun? {}", bloom.supports_cancun());
    println!("Is cancun? {}", bloom.is_version_cancun());

    if !bloom.supports_cancun() {
        // Find non-supported opcodes.
    }

    // Compute distance to mainnet.
    //let distance = bloom.compute_distance(&evm_bloomer::evm_bloom::EVMBloom::ethereum());
    //println!("Distance to mainnet: {}", distance);

    // Visualize.
    //bloom.visualize()?;

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
