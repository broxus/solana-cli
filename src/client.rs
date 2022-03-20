use indicatif::ProgressBar;
use solana_client::rpc_client::RpcClient;
use solana_program::bpf_loader_upgradeable;
use solana_program::bpf_loader_upgradeable::UpgradeableLoaderState;
use solana_program::message::Message;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

use crate::error::Result;
use crate::utils;

/// Establishes a RPC connection with the solana cluster configured by
/// `solana config set --url <URL>`. Information about what cluster
/// has been configured is gleened from the solana config file
/// `~/.config/solana/cli/config.yml`.
pub fn establish_connection() -> Result<RpcClient> {
    let rpc_url = utils::get_rpc_url()?;
    Ok(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    ))
}

pub fn create_buffer(
    payer: &Keypair,
    buffer: &Keypair,
    authority_address: &Pubkey,
    program_path: &str,
    connection: &RpcClient,
) -> Result<()> {
    utils::print_header("Creating buffer");

    let program_data = utils::read_elf(program_path)?;

    let buffer_data_len = program_data.len();

    let minimum_balance = connection.get_minimum_balance_for_rent_exemption(
        UpgradeableLoaderState::programdata_len(buffer_data_len)?,
    )?;

    let mut transaction = Transaction::new_with_payer(
        &bpf_loader_upgradeable::create_buffer(
            &payer.pubkey(),
            &buffer.pubkey(),
            authority_address,
            minimum_balance,
            program_data.len(),
        )?,
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, buffer], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    println!("Buffer: {}", buffer.pubkey());

    Ok(())
}

pub fn write_buffer(
    payer: &Keypair,
    buffer_pubkey: &Pubkey,
    program_path: &str,
    connection: &RpcClient,
) -> Result<()> {
    utils::print_header("Writing buffer");

    let blockhash = connection.get_latest_blockhash()?;

    let create_msg = |offset: u32, bytes: Vec<u8>| {
        let instruction =
            bpf_loader_upgradeable::write(buffer_pubkey, &payer.pubkey(), offset, bytes);
        Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash)
    };

    let program_data = utils::read_elf(program_path)?;

    let mut write_messages = vec![];
    let chunk_size = utils::calculate_max_chunk_size(&create_msg);
    for (chunk, i) in program_data.chunks(chunk_size).zip(0..) {
        write_messages.push(create_msg((i * chunk_size) as u32, chunk.to_vec()));
    }

    let pb = ProgressBar::new(write_messages.len() as u64);
    for message in write_messages {
        pb.inc(1);

        let transaction =
            Transaction::new(&vec![payer], message, connection.get_latest_blockhash()?);
        connection.send_and_confirm_transaction(&transaction)?;
    }
    pb.finish_with_message("done");

    Ok(())
}

pub fn set_buffer_authority(
    payer: &Keypair,
    current_authority: &Keypair,
    buffer_address: &Pubkey,
    new_authority_address: &Pubkey,
    connection: &RpcClient,
) -> Result<()> {
    utils::print_header("Setting buffer authority");

    let mut transaction = Transaction::new_with_payer(
        &[bpf_loader_upgradeable::set_buffer_authority(
            buffer_address,
            &current_authority.pubkey(),
            new_authority_address,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    println!("Authority: {}", new_authority_address);

    Ok(())
}

pub fn deploy(
    payer: &Keypair,
    program: &Keypair,
    buffer_pubkey: &Pubkey,
    program_path: &str,
    connection: &RpcClient,
) -> Result<()> {
    utils::print_header("Deploying program");

    let program_data = utils::read_elf(program_path)?;

    let mut transaction = Transaction::new_with_payer(
        &bpf_loader_upgradeable::deploy_with_max_program_len(
            &payer.pubkey(),
            &program.pubkey(),
            buffer_pubkey,
            &payer.pubkey(),
            connection
                .get_minimum_balance_for_rent_exemption(UpgradeableLoaderState::program_len()?)?,
            program_data.len(),
        )?,
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, program], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    println!("Program: {}", program.pubkey());

    Ok(())
}

pub fn set_program_authority(
    current_authority: &Keypair,
    program_address: &Pubkey,
    new_authority_address: &Pubkey,
    connection: &RpcClient,
) -> Result<()> {
    utils::print_header("Setting program authority");

    let mut transaction = Transaction::new_with_payer(
        &[bpf_loader_upgradeable::set_upgrade_authority(
            program_address,
            &current_authority.pubkey(),
            Some(new_authority_address),
        )],
        Some(&current_authority.pubkey()),
    );
    transaction.sign(&[current_authority], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    println!("Authority: {}", new_authority_address);

    Ok(())
}
