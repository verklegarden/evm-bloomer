use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use walkdir::WalkDir;
use eyre::Result;
use crate::EVMBloom;

use super::opcodes::EVM_OPCODES;

pub fn verify_project_bloom_filter(project_path: &str, correct_bloom: EVMBloom) -> Result<bool> {
    let mut project_opcode_bloom = EVMBloom::new();
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "asm" {
                    println!("Reading file: {:?}", path);
                    if let Ok(file) = File::open(path) {
                        let reader = io::BufReader::new(file);
                        for (index, line) in reader.lines().enumerate() {
                            let line = line?;
                            check_asm_line(&line, &mut project_opcode_bloom);
                            // println!("Line {}: {}", index + 1, line);
                        }
                    } else {
                        println!("Failed to open file: {:?}", path);
                    }
                }
            }
        }
    }
    println!("{:?}", correct_bloom);
    println!("{:?}", project_opcode_bloom);
    let mut combined_bloom = project_opcode_bloom.clone();
    combined_bloom.or(&correct_bloom);
    Ok(combined_bloom.eq(&correct_bloom))
}

fn check_asm_line(line: &str, bloom_filter: &mut EVMBloom) {
    let modified_line = line.to_uppercase().replace("_", "");
    for opcode_str in EVM_OPCODES.keys() {
        if modified_line.contains(opcode_str) {
            bloom_filter.set(*(EVM_OPCODES.get(opcode_str).unwrap()) as usize, true);
            break;
        }
    }
}
