use bit_vec::BitVec;
use eyre::Result;
use futures::future::join_all;
use rand::Rng;
use serde::Serialize;
use tokio::time::{sleep, Duration};

use alloy::{
    network::TransactionBuilder,
    primitives::*,
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::TransactionRequest,
    transports::http::{reqwest::Url, Client, Http},
};

#[macro_use]
extern crate lazy_static;

/// EVMBloom is a 256 bit bloom filter encoding a chainId's EVM opcode support.
#[derive(Debug, Serialize)]
pub struct EVMBloom {
    chain_id: u64,
    bloom: String,
    version: String,
    unknown_opcodes: Vec<u8>,
}

impl EVMBloom {
    /// Creates the [EVMBloom] of RPC URL `rpc_url`.
    pub async fn new(rpc_url: Url) -> Result<EVMBloom> {
        let provider = ProviderBuilder::new().on_http(rpc_url);
        let chain_id = provider.get_chain_id().await?;

        // Asynchronously check each opcode's support.
        let mut futures_results = vec![];
        for i in 0u8..=255 {
            futures_results.push(check_opcode(i, &provider));
        }
        let results: Vec<bool> = join_all(futures_results).await;

        // Generate bloom.
        let mut bloom = BitVec::from_elem(256, false);
        for (i, result) in results.iter().enumerate() {
            bloom.set(i, *result)
        }

        // Find latest EVM version supported and find unknown opcodes supported.
        let versions = evm_versions();
        let mut latest = "unknown";
        let mut unknown_opcodes: Vec<u8> = vec![];
        for version in versions {
            // Check whether version is supported.
            //
            // Version is supported if `bloom | version.bloom == bloom`
            let mut bloom_copy = bloom.clone();
            bloom_copy.or(&version.bloom);
            if bloom_copy == bloom {
                // Update latest version.
                latest = version.name;

                // Search for unknown opcodes.
                //
                // If `bloom ^ version.bloom != 0` every non-zero bit is an opcode
                // supported by the chain but not the EVM version.
                bloom_copy = bloom.clone();
                bloom_copy.xor(&version.bloom);
                if !bloom_copy.is_empty() {
                    let mut found = vec![];
                    for i in 0u8..=255 {
                        match bloom_copy.get(i as usize) {
                            Some(val) => {
                                if val {
                                    found.push(i);
                                }
                            }
                            None => unreachable!(),
                        };
                    }
                    unknown_opcodes = found;
                }
            }
        }

        Ok(EVMBloom {
            chain_id,
            bloom: format!("0x{}", hex::encode(bloom.to_bytes())),
            version: latest.to_string(),
            unknown_opcodes,
        })
    }
}

/// Checks whether opcode `opcode` is supported by provider `provider`.
///
/// Whether an opcode is supported is tested by executing the opcode in a deploy tx.
async fn check_opcode(opcode: u8, provider: &RootProvider<Http<Client>>) -> bool {
    // Note to always support STOP and INVALID opcodes.
    if opcode == 0x00 || opcode == 0xFE {
        return true;
    }

    // Add randomized delay to prevent hitting rate limits.
    sleep(Duration::from_millis(
        rand::thread_rng().gen_range(1..1000) * (opcode as u64),
    ))
    .await;

    // Construct deploy tx trying to execute the opcode.
    let tx = TransactionRequest::default()
        .with_from(address!("0000000000000000000000000000000000000000"))
        .with_deploy_code(vec![opcode]);

    // Perform RPC call.
    let is_supported = match provider.call(&tx).await {
        Ok(_) => true,
        Err(e) => {
            // Return whether error indicates opcode is supported.
            let ok = SUPPORTED_ERRORS
                .iter()
                .any(|pattern| e.to_string().contains(pattern));

            // Enabled for debugging.
            if !ok {
                eprintln!("{}", e);
            }

            ok
        }
    };

    is_supported
}

lazy_static! {
    static ref SUPPORTED_ERRORS: Vec<String> = vec![
        String::from("stack underflow"),
        String::from("StackUnderflow"),
    ];
}

struct EVMVersion {
    name: &'static str,
    bloom: BitVec,
}

fn evm_versions() -> Vec<EVMVersion> {
    let mut versions = vec![];

    // -- Frontier --
    let mut bloom_frontier = BitVec::from_elem(256, false);
    bloom_frontier.set(0x00, true); // STOP
    bloom_frontier.set(0x01, true); // ADD
    bloom_frontier.set(0x02, true); // MUL
    bloom_frontier.set(0x03, true); // SUB
    bloom_frontier.set(0x04, true); // DIV
    bloom_frontier.set(0x05, true); // SDIV
    bloom_frontier.set(0x06, true); // MOD
    bloom_frontier.set(0x07, true); // SMOD
    bloom_frontier.set(0x08, true); // ADDMOD
    bloom_frontier.set(0x09, true); // MULMOD
    bloom_frontier.set(0x0A, true); // EXP
    bloom_frontier.set(0x0B, true); // SIGNEXTEND

    bloom_frontier.set(0x10, true); // LT
    bloom_frontier.set(0x11, true); // GT
    bloom_frontier.set(0x12, true); // SLT
    bloom_frontier.set(0x13, true); // SGT
    bloom_frontier.set(0x14, true); // EG
    bloom_frontier.set(0x15, true); // ISZERO
    bloom_frontier.set(0x16, true); // AND
    bloom_frontier.set(0x17, true); // OR
    bloom_frontier.set(0x18, true); // XOR
    bloom_frontier.set(0x19, true); // NOT
    bloom_frontier.set(0x1A, true); // BYTE

    bloom_frontier.set(0x20, true); // KECCAK256

    bloom_frontier.set(0x30, true); // ADDRESS
    bloom_frontier.set(0x31, true); // BALANCE
    bloom_frontier.set(0x32, true); // ORIGIN
    bloom_frontier.set(0x33, true); // CALLER
    bloom_frontier.set(0x34, true); // CALLVALUE
    bloom_frontier.set(0x35, true); // CALLDATALOAD
    bloom_frontier.set(0x36, true); // CALLDATASIZE
    bloom_frontier.set(0x37, true); // CALLDATACOPY
    bloom_frontier.set(0x38, true); // CALLSIZE
    bloom_frontier.set(0x39, true); // CODECOPY
    bloom_frontier.set(0x3A, true); // GASPRICE
    bloom_frontier.set(0x3B, true); // EXTCODESIZE
    bloom_frontier.set(0x3C, true); // EXTCODECOPY

    bloom_frontier.set(0x40, true); // BLOCKHASH
    bloom_frontier.set(0x41, true); // COINBASE
    bloom_frontier.set(0x42, true); // TIMESTAMP
    bloom_frontier.set(0x43, true); // NUMBER
    bloom_frontier.set(0x44, true); // DIFFICULTY
    bloom_frontier.set(0x45, true); // GASLIMIT

    bloom_frontier.set(0x50, true); // POP
    bloom_frontier.set(0x51, true); // MLOAD
    bloom_frontier.set(0x52, true); // MSTORE
    bloom_frontier.set(0x53, true); // MSTORE8
    bloom_frontier.set(0x54, true); // SLOAD
    bloom_frontier.set(0x55, true); // SSTORE
    bloom_frontier.set(0x56, true); // JUMP
    bloom_frontier.set(0x57, true); // JUMPI
    bloom_frontier.set(0x58, true); // PC
    bloom_frontier.set(0x59, true); // MSIZE
    bloom_frontier.set(0x5A, true); // GAS
    bloom_frontier.set(0x5B, true); // JUMPDEST

    bloom_frontier.set(0x60, true); // PUSH1
    bloom_frontier.set(0x61, true); // PUSH2
    bloom_frontier.set(0x62, true); // PUSH3
    bloom_frontier.set(0x63, true); // PUSH4
    bloom_frontier.set(0x64, true); // PUSH5
    bloom_frontier.set(0x65, true); // PUSH6
    bloom_frontier.set(0x66, true); // PUSH7
    bloom_frontier.set(0x67, true); // PUSH8
    bloom_frontier.set(0x68, true); // PUSH9
    bloom_frontier.set(0x69, true); // PUSH10
    bloom_frontier.set(0x6A, true); // PUSH11
    bloom_frontier.set(0x6B, true); // PUSH12
    bloom_frontier.set(0x6C, true); // PUSH13
    bloom_frontier.set(0x6D, true); // PUSH14
    bloom_frontier.set(0x6E, true); // PUSH15
    bloom_frontier.set(0x6F, true); // PUSH16
    bloom_frontier.set(0x70, true); // PUSH17
    bloom_frontier.set(0x71, true); // PUSH18
    bloom_frontier.set(0x72, true); // PUSH19
    bloom_frontier.set(0x73, true); // PUSH20
    bloom_frontier.set(0x74, true); // PUSH21
    bloom_frontier.set(0x75, true); // PUSH22
    bloom_frontier.set(0x76, true); // PUSH23
    bloom_frontier.set(0x77, true); // PUSH24
    bloom_frontier.set(0x78, true); // PUSH25
    bloom_frontier.set(0x79, true); // PUSH26
    bloom_frontier.set(0x7A, true); // PUSH27
    bloom_frontier.set(0x7B, true); // PUSH28
    bloom_frontier.set(0x7C, true); // PUSH29
    bloom_frontier.set(0x7D, true); // PUSH30
    bloom_frontier.set(0x7E, true); // PUSH31
    bloom_frontier.set(0x7F, true); // PUSH32
    bloom_frontier.set(0x80, true); // DUP1
    bloom_frontier.set(0x81, true); // DUP2
    bloom_frontier.set(0x82, true); // DUP3
    bloom_frontier.set(0x83, true); // DUP4
    bloom_frontier.set(0x84, true); // DUP5
    bloom_frontier.set(0x85, true); // DUP6
    bloom_frontier.set(0x86, true); // DUP7
    bloom_frontier.set(0x87, true); // DUP8
    bloom_frontier.set(0x88, true); // DUP9
    bloom_frontier.set(0x89, true); // DUP10
    bloom_frontier.set(0x8A, true); // DUP11
    bloom_frontier.set(0x8B, true); // DUP12
    bloom_frontier.set(0x8C, true); // DUP13
    bloom_frontier.set(0x8D, true); // DUP14
    bloom_frontier.set(0x8E, true); // DUP15
    bloom_frontier.set(0x8F, true); // DUP16
    bloom_frontier.set(0x90, true); // SWAP1
    bloom_frontier.set(0x91, true); // SWAP2
    bloom_frontier.set(0x92, true); // SWAP3
    bloom_frontier.set(0x93, true); // SWAP4
    bloom_frontier.set(0x94, true); // SWAP5
    bloom_frontier.set(0x95, true); // SWAP6
    bloom_frontier.set(0x96, true); // SWAP7
    bloom_frontier.set(0x97, true); // SWAP8
    bloom_frontier.set(0x98, true); // SWAP9
    bloom_frontier.set(0x99, true); // SWAP10
    bloom_frontier.set(0x9A, true); // SWAP11
    bloom_frontier.set(0x9B, true); // SWAP12
    bloom_frontier.set(0x9C, true); // SWAP13
    bloom_frontier.set(0x9D, true); // SWAP14
    bloom_frontier.set(0x9E, true); // SWAP15
    bloom_frontier.set(0x9F, true); // SWAP16
    bloom_frontier.set(0xA0, true); // LOG0
    bloom_frontier.set(0xA1, true); // LOG1
    bloom_frontier.set(0xA2, true); // LOG2
    bloom_frontier.set(0xA3, true); // LOG3
    bloom_frontier.set(0xA4, true); // LOG4

    bloom_frontier.set(0xF0, true); // CREATE
    bloom_frontier.set(0xF1, true); // CALL
    bloom_frontier.set(0xF2, true); // CALLCODE
    bloom_frontier.set(0xF3, true); // RETURN

    bloom_frontier.set(0xFE, true); // INVALID
    bloom_frontier.set(0xFF, true); // SELFDESTRUCT
    versions.push(EVMVersion {
        name: "frontier",
        bloom: bloom_frontier,
    });

    // -- Homestead --
    let mut bloom_homestead = versions[versions.len() - 1].bloom.clone();
    bloom_homestead.set(0xF4, true); // DELEGATECALL
    versions.push(EVMVersion {
        name: "homestead",
        bloom: bloom_homestead,
    });

    // -- TangerineWhistle --
    let mut bloom_tangerine_whistle = versions[versions.len() - 1].bloom.clone();
    bloom_tangerine_whistle.set(0x31, true); // BALANCE
    versions.push(EVMVersion {
        name: "tangerine_whistle",
        bloom: bloom_tangerine_whistle,
    });

    // -- Spurious Dragon --
    let bloom_spurious_dragon = versions[versions.len() - 1].bloom.clone();
    versions.push(EVMVersion {
        name: "spurious_dragon",
        bloom: bloom_spurious_dragon,
    });

    // -- Byzantium --
    let mut bloom_byzantium = versions[versions.len() - 1].bloom.clone();
    bloom_byzantium.set(0x3D, true); // RETURNDATASIZE
    bloom_byzantium.set(0x3E, true); // RETURNDATACOPY
    bloom_byzantium.set(0xFA, true); // STATICCALL
    bloom_byzantium.set(0xFD, true); // REVERT
    versions.push(EVMVersion {
        name: "byzantium",
        bloom: bloom_byzantium,
    });

    // -- Constantinople --
    let mut bloom_constantinople = versions[versions.len() - 1].bloom.clone();
    bloom_constantinople.set(0x1B, true); // SHL
    bloom_constantinople.set(0x1C, true); // SHR
    bloom_constantinople.set(0x1D, true); // SAR
    bloom_constantinople.set(0x3F, true); // EXTCODEHASH
    bloom_constantinople.set(0xF5, true); // CREATE2
    versions.push(EVMVersion {
        name: "constantinople",
        bloom: bloom_constantinople,
    });

    // -- Instanbul --
    let mut bloom_instanbul = versions[versions.len() - 1].bloom.clone();
    bloom_instanbul.set(0x46, true); // CHAINID
    bloom_instanbul.set(0x47, true); // SELFBALANCE
    versions.push(EVMVersion {
        name: "instanbul",
        bloom: bloom_instanbul,
    });

    // -- Berlin --
    let bloom_berlin = versions[versions.len() - 1].bloom.clone();
    versions.push(EVMVersion {
        name: "berlin",
        bloom: bloom_berlin,
    });

    // -- London --
    let mut bloom_london = versions[versions.len() - 1].bloom.clone();
    bloom_london.set(0x48, true); // BASEFEE
    versions.push(EVMVersion {
        name: "london",
        bloom: bloom_london,
    });

    // -- Merge --
    let bloom_merge = versions[versions.len() - 1].bloom.clone();
    versions.push(EVMVersion {
        name: "merge",
        bloom: bloom_merge,
    });

    // -- Shanghai --
    let mut bloom_shanghai = versions[versions.len() - 1].bloom.clone();
    bloom_shanghai.set(0x5F, true); // PUSH0
    versions.push(EVMVersion {
        name: "shanghai",
        bloom: bloom_shanghai.clone(),
    });

    // -- Cancun --
    let mut bloom_cancun = versions[versions.len() - 1].bloom.clone();
    bloom_cancun.set(0x49 as usize, true); // BLOBHASH
    bloom_cancun.set(0x4A as usize, true); // BLOBBASEFEE
    bloom_cancun.set(0x5C as usize, true); // TLOAD
    bloom_cancun.set(0x5D as usize, true); // TSTORE
    bloom_cancun.set(0x5E as usize, true); // MCOPY
    versions.push(EVMVersion {
        name: "cancun",
        bloom: bloom_cancun,
    });

    versions
}
