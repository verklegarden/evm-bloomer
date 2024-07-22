//! EVMBloomer ...
#[macro_use]
extern crate lazy_static;
mod asm_parser;
use std::fmt;
use std::ops::{Deref, DerefMut};
use tokio::time::{sleep, Duration};

use alloy::providers::RootProvider;
use alloy::transports::http::{Client, Http};
use alloy::{
    network::TransactionBuilder,
    primitives::*,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    transports::http::reqwest::Url,
};
use futures::future::join_all;

use bit_vec::BitVec;
use eyre::Result;
use image::{ImageBuffer, Rgb};

/// EVMBloom is a 256 bit bloom filter encoding an EVM's opcode support.
#[derive(Debug)]
pub struct EVMBloom(BitVec);

lazy_static! {
    static ref SUPPORTED_ERRORS: Vec<String> = vec![String::from("stack underflow")];
    static ref UNSUPPORTED_ERRORS: Vec<String> = vec![
        String::from("invalid opcode"),
        String::from("is not supported")
    ];
}

impl EVMBloom {
    pub fn new() -> EVMBloom {
        EVMBloom(BitVec::from_elem(256, false))
    }
    /// Creates the [EVMBloom] of rpc url `rpc_url`.
    pub async fn create(rpc_url: Url) -> Result<Self> {
        let provider = ProviderBuilder::new().on_http(rpc_url);

        // Create bloom by trying to execute each possible opcode.
        let mut bloom = EVMBloom::new();

        let mut futures_results = vec![];
        for i in 0u8..=255 {
            futures_results.push(check_opcode(i, &provider));
        }
        let results: Vec<Result<bool>> = join_all(futures_results).await;
        for (index, result) in results.iter().enumerate() {
            match result {
                Ok(val) => {
                    bloom.set(index, *val);
                }
                Err(_) => {
                    bloom.set(index, false);
                }
            }
        }

        Ok(bloom)
    }

    /// Returns an image visualization of [EVMBloom].
    pub fn visualize(&self) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        // Create a 16x16 image.
        let imgx = 16;
        let imgy = 16;
        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(imgx, imgy);

        let mut counter = 0;
        let mut pos: [i64; 2] = [7, 7];
        let mut mag = 1;
        let mut dir: [i64; 2] = [1, 0];
        'outer: loop {
            // Take "mag" steps in 2 consecutive directions
            for _ in 0..2 {
                // Take mage steps in direction "dir"
                for _ in 0..mag {
                        if counter >= 256 {
                            break 'outer;
                        }
                        let bit = match self.get(counter) {
                            Some(b) => b,
                            None => false,
                        };
                        let color_value = if bit { 0 } else { 255 };
                        println!("{:?}",pos);
                        let pixel = img.get_pixel_mut(pos[0] as u32, pos[1] as u32);
                        *pixel = Rgb([color_value, color_value, color_value]);
                        pos[0] += dir[0];
                        pos[1] += dir[1];
                        counter += 1;
                }
                // Rotate "dir" 90 degrees
                let tmp = dir[0];
                dir[0] = -1 * dir[1];
                dir[1] = tmp;
            }
            // Increase step size "mag"
            mag += 1;
        }

        // // Iterate over each pixel position.
        // for (x, y, pixel) in img.enumerate_pixels_mut() {
        //     // Calculate bit index.
        //     let bit_index = (y * imgx + x) as usize;

        //     // Get bit value.
        //     let bit = match self.get(bit_index) {
        //         Some(b) => b,
        //         None => false,
        //     };

        //     // Convert bit value to color (black or white).
        //     let color_value = if bit { 0 } else { 255 };
        //     *pixel = Rgb([color_value, color_value, color_value]);
        // }

        Ok(img)
    }

    /// Returns [EVMBloom] as stringified table.
    pub fn to_table(&self) -> String {
        let mut result = String::from("Opcode | Mnemonic        | Supported?\n");
        result.push_str("------------------------------------\n");
        for i in 0..=255 {
            let mnemonic = MNEMONICS[i];

            let supported = match self.get(i) {
                Some(b) => b,
                None => unreachable!(),
            };

            result.push_str(format!("0x{:02X}   | {:15} | {}\n", i, mnemonic, supported).as_str());
        }

        result
    }

    pub fn to_json(&self, chain_id: u64) -> String {
        // TODO: Implement json serde functions.
        //       Bloom should be hex number?
        //
        // { chain_id: 1, bloom: "0x..." }
        format!("{}", chain_id)
    }
}

async fn check_opcode(opcode: u8, provider: &RootProvider<Http<Client>>) -> Result<bool> {
    // Add a sleep to prevent exceeding the max api rate :(
    sleep(Duration::from_millis(50 * (opcode as u64))).await;
    let bytecode = vec![opcode];
    let tx = TransactionRequest::default()
        .with_from(address!("0000000000000000000000000000000000000000"))
        .with_deploy_code(bytecode);
    match provider.call(&tx).await {
        Ok(_) => {
            return Ok(true);
        }
        Err(e) => {
            let e_str = e.to_string();
            println!("{}", e_str);

            let is_supported = SUPPORTED_ERRORS
                .iter()
                .any(|pattern| e_str.contains(pattern));

            if is_supported {
                // Note to print unknown errors for debugging.
                if !e_str.contains("stack underflow") {
                    println!("unknown execution error: {}", e);
                }
                return Ok(true);
            }
            return Ok(false);
        }
    }
}

// Offers EVM version comparison functions.
// TODO: This should be removed. Easy enough to do via bit operations outside of
//       module. Don't over provide!
impl EVMBloom {
    /// Returns whether [EVMBloom] is the Cancun EVM version.
    pub fn is_version_cancun(&self) -> bool {
        self.eq(&Self::version_cancun())
    }

    /// Returns whether [EVMBloom] supports the Cancun EVM version.
    pub fn supports_cancun(&self) -> bool {
        let cancun = Self::version_cancun();

        // Return whether (cancun & self.bloom) == cancun.
        let mut conjuction = cancun.clone();
        if !conjuction.and(&self) {
            panic!("EVMBloom::supports_cancun: bitwise and failed");
        }
        conjuction.eq(&cancun)
    }
}

// Offers the different EVM version blooms.
impl EVMBloom {
    /// Returns Ethereum mainnet's [EVMBloom].
    pub fn ethereum() -> Self {
        Self::version_cancun()
    }

    /// Returns the [EVMBloom] of the Cancun EVM version.
    pub fn version_cancun() -> Self {
        EVMBloom(BitVec::from_bytes(&[
            0b11111111, 0b11110000, 0b11111111, 0b11111100, 0b10000000, 0b00000000, 0b11111111,
            0b11111111, 0b11111111, 0b11100000, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
            0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b11111100, 0b00100101,
        ]))
    }
}

impl Deref for EVMBloom {
    type Target = BitVec;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EVMBloom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for EVMBloom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EVMBloom: {}", self.0)
    }
}

// Maps an opcode to its mnemonic or emty string if unknown.
pub static MNEMONICS: [&str; 256] = [
    "STOP",            // 0x00
    "ADD",             // 0x01
    "MUL",             // 0x02
    "SUB",             // 0x03
    "DIV",             // 0x04
    "SDIV",            // 0x05
    "MOD",             // 0x06
    "SMOD",            // 0x07
    "ADDMOD",          // 0x08
    "MULMOD",          // 0x09
    "EXP",             // 0x0A
    "SIGNEXTEND",      // 0x0B
    "",                // 0x0C
    "",                // 0x0D
    "",                // 0x0E
    "",                // 0x0F
    "LT",              // 0x10
    "GT",              // 0x11
    "SLT",             // 0x12
    "SGT",             // 0x13
    "EQ",              // 0x14
    "ISZERO",          // 0x15
    "AND",             // 0x16
    "OR",              // 0x17
    "XOR",             // 0x18
    "NOT",             // 0x19
    "BYTE",            // 0x1A
    "SHL",             // 0x1B
    "SHR",             // 0x1C
    "SAR",             // 0x1D
    "",                // 0x1E
    "",                // 0x1F
    "KECCAK256",       // 0x20
    "",                // 0x21
    "",                // 0x22
    "",                // 0x23
    "",                // 0x24
    "",                // 0x25
    "",                // 0x26
    "",                // 0x27
    "",                // 0x28
    "",                // 0x29
    "",                // 0x2A
    "",                // 0x2B
    "",                // 0x2C
    "",                // 0x2D
    "",                // 0x2E
    "",                // 0x2F
    "ADDRESS",         // 0x30
    "BALANCE",         // 0x31
    "ORIGIN",          // 0x32
    "CALLER",          // 0x33
    "CALLVALUE",       // 0x34
    "CALLDATALOAD",    // 0x35
    "CALLDATASIZE",    // 0x36
    "CALLDATACOPY",    // 0x37
    "CODESIZE",        // 0x38
    "CODECOPY",        // 0x39
    "GASPRICE",        // 0x3A
    "EXTCODESIZE",     // 0x3B
    "EXTCODECOPY",     // 0x3C
    "RETURNDATASIZE",  // 0x3D
    "RETURNDATACOPY",  // 0x3E
    "EXTCODEHASH",     // 0x3F
    "BLOCKHASH",       // 0x40
    "COINBASE",        // 0x41
    "TIMESTAMP",       // 0x42
    "NUMBER",          // 0x43
    "DIFFICULTY",      // 0x44
    "GASLIMIT",        // 0x45
    "CHAINID",         // 0x46
    "SELFBALANCE",     // 0x47
    "BASEFEE",         // 0x48
    "BLOBHASH",        // 0x49
    "BLOBBASEFEE",     // 0x4A
    "",                // 0x4B
    "",                // 0x4C
    "",                // 0x4D
    "",                // 0x4E
    "",                // 0x4F
    "POP",             // 0x50
    "MLOAD",           // 0x51
    "MSTORE",          // 0x52
    "MSTORE8",         // 0x53
    "SLOAD",           // 0x54
    "SSTORE",          // 0x55
    "JUMP",            // 0x56
    "JUMPI",           // 0x57
    "PC",              // 0x58
    "MSIZE",           // 0x59
    "GAS",             // 0x5A
    "JUMPDEST",        // 0x5B
    "TLOAD",           // 0x5C
    "TSTORE",          // 0x5D
    "MCOPY",           // 0x5E
    "PUSH0",           // 0x5F
    "PUSH1",           // 0x60
    "PUSH2",           // 0x61
    "PUSH3",           // 0x62
    "PUSH4",           // 0x63
    "PUSH5",           // 0x64
    "PUSH6",           // 0x65
    "PUSH7",           // 0x66
    "PUSH8",           // 0x67
    "PUSH9",           // 0x68
    "PUSH10",          // 0x69
    "PUSH11",          // 0x6A
    "PUSH12",          // 0x6B
    "PUSH13",          // 0x6C
    "PUSH14",          // 0x6D
    "PUSH15",          // 0x6E
    "PUSH16",          // 0x6F
    "PUSH17",          // 0x70
    "PUSH18",          // 0x71
    "PUSH19",          // 0x72
    "PUSH20",          // 0x73
    "PUSH21",          // 0x74
    "PUSH22",          // 0x75
    "PUSH23",          // 0x76
    "PUSH24",          // 0x77
    "PUSH25",          // 0x78
    "PUSH26",          // 0x79
    "PUSH27",          // 0x7A
    "PUSH28",          // 0x7B
    "PUSH29",          // 0x7C
    "PUSH30",          // 0x7D
    "PUSH31",          // 0x7E
    "PUSH32",          // 0x7F
    "DUP1",            // 0x80
    "DUP2",            // 0x81
    "DUP3",            // 0x82
    "DUP4",            // 0x83
    "DUP5",            // 0x84
    "DUP6",            // 0x85
    "DUP7",            // 0x86
    "DUP8",            // 0x87
    "DUP9",            // 0x88
    "DUP10",           // 0x89
    "DUP11",           // 0x8A
    "DUP12",           // 0x8B
    "DUP13",           // 0x8C
    "DUP14",           // 0x8D
    "DUP15",           // 0x8E
    "DUP16",           // 0x8F
    "SWAP1",           // 0x90
    "SWAP2",           // 0x91
    "SWAP3",           // 0x92
    "SWAP4",           // 0x93
    "SWAP5",           // 0x94
    "SWAP6",           // 0x95
    "SWAP7",           // 0x96
    "SWAP8",           // 0x97
    "SWAP9",           // 0x98
    "SWAP10",          // 0x99
    "SWAP11",          // 0x9A
    "SWAP12",          // 0x9B
    "SWAP13",          // 0x9C
    "SWAP14",          // 0x9D
    "SWAP15",          // 0x9E
    "SWAP16",          // 0x9F
    "LOG0",            // 0xA0
    "LOG1",            // 0xA1
    "LOG2",            // 0xA2
    "LOG3",            // 0xA3
    "LOG4",            // 0xA4
    "",                // 0xA5
    "",                // 0xA6
    "",                // 0xA7
    "",                // 0xA8
    "",                // 0xA9
    "",                // 0xAA
    "",                // 0xAB
    "",                // 0xAC
    "",                // 0xAD
    "",                // 0xAE
    "",                // 0xAF
    "",                // 0xB0
    "",                // 0xB1
    "",                // 0xB2
    "",                // 0xB3
    "",                // 0xB4
    "",                // 0xB5
    "",                // 0xB6
    "",                // 0xB7
    "",                // 0xB8
    "",                // 0xB9
    "",                // 0xBA
    "",                // 0xBB
    "",                // 0xBC
    "",                // 0xBD
    "",                // 0xBE
    "",                // 0xBF
    "",                // 0xC0
    "",                // 0xC1
    "",                // 0xC2
    "",                // 0xC3
    "",                // 0xC4
    "",                // 0xC5
    "",                // 0xC6
    "",                // 0xC7
    "",                // 0xC8
    "",                // 0xC9
    "",                // 0xCA
    "",                // 0xCB
    "",                // 0xCC
    "",                // 0xCD
    "",                // 0xCE
    "",                // 0xCF
    "DATALOAD",        // 0xD0
    "DATALOADN",       // 0xD1
    "DATASIZE",        // 0xD2
    "DATACOPY",        // 0xD3
    "",                // 0xD4
    "",                // 0xD5
    "",                // 0xD6
    "",                // 0xD7
    "",                // 0xD8
    "",                // 0xD9
    "",                // 0xDA
    "",                // 0xDB
    "",                // 0xDC
    "",                // 0xDD
    "",                // 0xDE
    "",                // 0xDF
    "RJUMP",           // 0xE0
    "JUMPI",           // 0xE1
    "JUMPV",           // 0xE2
    "CALLF",           // 0xE3
    "RETF",            // 0xE4
    "JUMPF",           // 0xE5
    "DUPN",            // 0xE6
    "SWAPN",           // 0xE7
    "EXCHANGE",        // 0xE8
    "",                // 0xE9
    "",                // 0xEA
    "",                // 0xEB
    "EOFCREATE",       // 0xEC
    "",                // 0xED
    "RETURNCONTRACT",  // 0xEE
    "",                // 0xEF
    "CREATE",          // 0xF0
    "CALL",            // 0xF1
    "CALLCODE",        // 0xF2
    "RETURN",          // 0xF3
    "DELEGATECALL",    // 0xF4
    "CREATE2",         // 0xF5
    "",                // 0xF6
    "RETURNDATALOAD",  // 0xF7
    "EXTCALL",         // 0xF8
    "EXTDELEGATECALL", // 0xF9
    "STATICCALL",      // 0xFA
    "EXTSTATICCALL",   // 0xFB
    "",                // 0xFC
    "REVERT",          // 0xFD
    "INVALID",         // 0xFE
    "SELFDESTRUCT",    // 0xFF
];
