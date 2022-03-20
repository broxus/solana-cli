use solana_program::message::Message;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use yaml_rust::YamlLoader;

use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::keypair::read_keypair_file;
use solana_sdk::transaction::Transaction;

use crate::error::{Error, Result};

pub fn get_config() -> Result<yaml_rust::Yaml> {
    let path = match home::home_dir() {
        Some(mut path) => {
            path.push(".config/solana/cli/config.yml");
            path
        }
        None => {
            return Err(Error::ConfigReadError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "failed to locate homedir and thus can not locoate solana config",
            )));
        }
    };
    let config = std::fs::read_to_string(path).map_err(Error::ConfigReadError)?;
    let mut config = YamlLoader::load_from_str(&config)?;
    match config.len() {
        1 => Ok(config.remove(0)),
        l => Err(Error::InvalidConfig(format!(
            "expected one yaml document got ({})",
            l
        ))),
    }
}

pub fn get_rpc_url() -> Result<String> {
    let config = get_config()?;
    match config["json_rpc_url"].as_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(Error::InvalidConfig(
            "missing `json_rpc_url` field".to_string(),
        )),
    }
}

pub fn get_payer() -> Result<Keypair> {
    let config = get_config()?;
    let path = match config["keypair_path"].as_str() {
        Some(s) => s,
        None => {
            return Err(Error::InvalidConfig(
                "missing `keypair_path` field".to_string(),
            ))
        }
    };
    read_keypair_file(path).map_err(|e| {
        Error::InvalidConfig(format!("failed to read keypair file ({}): ({})", path, e))
    })
}

pub fn read_elf(program_location: &str) -> Result<Vec<u8>> {
    let mut file = File::open(program_location).map_err(Error::ProgramOpenError)?;
    let mut program_data = Vec::new();
    file.read_to_end(&mut program_data)
        .map_err(Error::ProgramReadError)?;

    Ok(program_data)
}

pub fn calculate_max_chunk_size<F>(create_msg: &F) -> usize
where
    F: Fn(u32, Vec<u8>) -> Message,
{
    let baseline_msg = create_msg(0, Vec::new());
    let tx_size = bincode::serialized_size(&Transaction {
        signatures: vec![
            Signature::default();
            baseline_msg.header.num_required_signatures as usize
        ],
        message: baseline_msg,
    })
    .unwrap() as usize;
    // add 1 byte buffer to account for shortvec encoding
    solana_sdk::packet::PACKET_DATA_SIZE
        .saturating_sub(tx_size)
        .saturating_sub(1)
}

pub fn get_keypair_file(program_path: &str) -> PathBuf {
    let mut keypair_file = PathBuf::new();
    keypair_file.push(&program_path);

    let mut filename = keypair_file.file_stem().unwrap().to_os_string();
    filename.push("-keypair");

    keypair_file.set_file_name(filename);
    keypair_file.set_extension("json");

    keypair_file
}

pub fn print_header(header: &'static str) {
    println!();
    println!("===================================");
    println!();
    println!("    {}", header);
    println!();
    println!("===================================");
    println!();
}
