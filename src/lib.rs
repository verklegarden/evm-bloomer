//! EVMBloomer ...

use std::fmt;
use std::path::Path;

use alloy::{
    hex,
    network::TransactionBuilder,
    primitives::*,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    transports::http::reqwest::Url,
};
use bit_vec::BitVec;
use eyre::Result;
use image::{ImageBuffer, Rgb};

mod evm_bloom;

/*
/// EVMBloom
pub struct EVMBloom {
    bloom: BitVec,
}

impl EVMBloom {
    /// Creates the [EVMBloom] for given RPC URL.
    pub async fn create(rpc_url: Url) -> Result<Self> {
        let provider = ProviderBuilder::new().on_http(rpc_url);

        // Create bloom by trying to execute each possible opcode.
        // Note that execution errors are expected, eg due to stack underflow.
        let mut bloom = BitVec::from_elem(256, false);
        for i in 0u8..=255 {
            let bytecode = hex::decode(format!("{:02x}", i))?;

            let tx = TransactionRequest::default()
                .with_from(address!("0000000000000000000000000000000000000000"))
                .with_deploy_code(bytecode);

            match provider.call(&tx).await {
                Ok(_) => {
                    bloom.set(usize::from(i), true);
                }
                Err(e) => {
                    let e_str = e.to_string();

                    // Opcode is considered not supported if error contains any
                    // of the following patterns.
                    let not_supported_patterns = vec!["invalid opcode", "is not supported"];

                    let is_supported = not_supported_patterns
                        .iter()
                        .all(|&pattern| !e_str.contains(pattern));

                    if is_supported {
                        bloom.set(usize::from(i), true);

                        // Note to print unknown errors for debugging.
                        if !e_str.contains("stack underflow") {
                            println!("unknown execution error: {}", e);
                        }
                    }
                }
            }
        }

        Ok(EVMBloom { bloom })
    }

    /// Returns Ethereum mainnet's [EVMBloom].
    pub fn ethereum() -> Self {
        // Ethereum is currently running the Cancun version.
        Self::version_cancun()
    }

    /// Returns [EVMBloom] of EVM version Cancun.
    pub fn version_cancun() -> Self {
        let bloom = BitVec::from_bytes(&[
            0b11111111, 0b11110000, 0b11111111, 0b11111100, 0b10000000, 0b00000000, 0b11111111,
            0b11111111, 0b11111111, 0b11100000, 0b11111111, 0b11111111, 0b11111111, 0b11111111,
            0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111111, 0b11111000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b11111100, 0b00100101,
        ]);

        EVMBloom { bloom }
    }

    /// Returns [EVMBloom] of EVM version Shanghai.
    pub fn version_shanghai() -> Self {
        EVMBloom {
            bloom: BitVec::from_elem(256, false),
        }
    }

    /// Returns [EVMBloom] of EVM version Paris.
    pub fn version_paris() -> Self {
        EVMBloom {
            bloom: BitVec::from_elem(256, false),
        }
    }

    /// Computes the Manhattan distance to [EVMBloom] other.
    pub fn compute_distance(&self, other: &EVMBloom) -> u64 {
        let mut got = self.bloom.clone();

        got.xor(&other.bloom);
        got.count_ones()
    }

    /// Visualizes the [EVMBloom] and stores the image at path.
    pub fn visualize(self /*path: &Path*/) -> Result<()> {
        // Create a 16x16 image.
        let imgx = 16;
        let imgy = 16;
        let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(imgx, imgy);

        // Iterate over each pixel position.
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            // Calculate bit index.
            let bit_index = (y * imgx + x) as usize;

            // Get bit value.
            let bit = match self.bloom.get(bit_index) {
                Some(b) => b,
                None => false,
            };

            // Convert bit value to color (black or white).
            let color_value = if bit { 0 } else { 255 };
            *pixel = Rgb([color_value, color_value, color_value]);
        }

        // Save as image.
        img.save("output.png")?;

        Ok(())
    }
}

impl fmt::Display for EVMBloom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //for i in 0..=255 {
        //    if i % 8 == 0 {
        //        write!(f, "\n\n")?;
        //    }
        //
        //    write!(f, "{}", usize::from(self.bloom[i]))?;
        //}
        //Ok(())

        write!(f, "{}", self.bloom)
    }
}
*/
