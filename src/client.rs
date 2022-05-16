use borsh::BorshSerialize;
use std::sync::Arc;

use solana_bridge::round_loader::RelayRoundProposalEventWithLen;
use solana_client::rpc_client::RpcClient;
use solana_client::tpu_client::{TpuClient, TpuClientConfig};
use solana_program::bpf_loader_upgradeable;
use solana_program::bpf_loader_upgradeable::UpgradeableLoaderState;
use solana_program::message::Message;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

use crate::error::{Error, Result};
use crate::utils;

/// Establishes a RPC connection with the solana cluster configured by
/// `solana config set --url <URL>`. Information about what cluster
/// has been configured is gleened from the solana config file
/// `~/.config/solana/cli/config.yml`.
pub fn establish_connection() -> Result<Arc<RpcClient>> {
    let rpc_url = utils::get_rpc_url()?;
    Ok(Arc::new(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    )))
}

pub fn create_buffer(
    payer: &Keypair,
    buffer: &Keypair,
    authority_address: &Pubkey,
    program_len: usize,
    connection: &Arc<RpcClient>,
) -> Result<()> {
    utils::print_header("Creating buffer");

    let minimum_balance = connection.get_minimum_balance_for_rent_exemption(
        UpgradeableLoaderState::programdata_len(program_len)?,
    )?;

    let mut transaction = Transaction::new_with_payer(
        &bpf_loader_upgradeable::create_buffer(
            &payer.pubkey(),
            &buffer.pubkey(),
            authority_address,
            minimum_balance,
            program_len,
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
    program_data: &[u8],
    connection: &Arc<RpcClient>,
) -> Result<()> {
    utils::print_header("Writing buffer");

    let blockhash = connection.get_latest_blockhash()?;

    // Get messages
    let create_msg = |offset: u32, bytes: Vec<u8>| {
        let instruction =
            bpf_loader_upgradeable::write(buffer_pubkey, &payer.pubkey(), offset, bytes);
        Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash)
    };

    let mut write_messages = vec![];
    let chunk_size = utils::calculate_max_chunk_size(&create_msg);
    for (chunk, i) in program_data.chunks(chunk_size).zip(0..) {
        write_messages.push(create_msg((i * chunk_size) as u32, chunk.to_vec()));
    }

    // Send message
    let websocket_url = utils::get_ws_url()?;
    let tpu_client = TpuClient::new(
        connection.clone(),
        &websocket_url,
        TpuClientConfig::default(),
    )
    .map_err(Error::TpuSenderError)?;

    let transaction_errors = tpu_client
        .send_and_confirm_messages_with_spinner(&write_messages, &[payer])
        .map_err(Error::TpuSenderError)?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    if !transaction_errors.is_empty() {
        for transaction_error in &transaction_errors {
            eprintln!("{:?}", transaction_error);
        }
        return Err(Error::WriteTransactions(transaction_errors.len()));
    }

    Ok(())
}

pub fn set_buffer_authority(
    payer: &Keypair,
    current_authority: &Keypair,
    buffer_address: &Pubkey,
    new_authority_address: &Pubkey,
    connection: &Arc<RpcClient>,
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
    max_data_len: usize,
    connection: &Arc<RpcClient>,
) -> Result<()> {
    utils::print_header("Deploying program");

    let mut transaction = Transaction::new_with_payer(
        &bpf_loader_upgradeable::deploy_with_max_program_len(
            &payer.pubkey(),
            &program.pubkey(),
            buffer_pubkey,
            &payer.pubkey(),
            connection
                .get_minimum_balance_for_rent_exemption(UpgradeableLoaderState::program_len()?)?,
            max_data_len,
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
    connection: &Arc<RpcClient>,
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

pub fn create_relay_round_proposal(
    payer: &Keypair,
    event_timestamp: u32,
    event_transaction_lt: u64,
    event_configuration: Pubkey,
    connection: &Arc<RpcClient>,
) -> Result<()> {
    utils::print_header("Create Relay Round Proposal");

    let mut transaction = Transaction::new_with_payer(
        &[solana_bridge::round_loader::create_proposal_ix(
            &payer.pubkey(),
            event_timestamp,
            event_transaction_lt,
            event_configuration,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    let setting_address = solana_bridge::round_loader::get_settings_address();
    let proposal_address = solana_bridge::round_loader::get_proposal_address(
        &payer.pubkey(),
        &setting_address,
        event_timestamp,
        event_transaction_lt,
        &event_configuration,
    );

    println!("Proposal address: {}", proposal_address);

    Ok(())
}

pub fn write_relay_round_proposal(
    payer: &Keypair,
    event_timestamp: u32,
    event_transaction_lt: u64,
    event_configuration: Pubkey,
    proposal_data: RelayRoundProposalEventWithLen,
    connection: &Arc<RpcClient>,
) -> Result<()> {
    utils::print_header("Writing Relay Round Proposal");

    let blockhash = connection.get_latest_blockhash()?;

    let create_msg = |offset: u32, bytes: Vec<u8>| {
        let instruction = solana_bridge::round_loader::write_proposal_ix(
            &payer.pubkey(),
            event_timestamp,
            event_transaction_lt,
            event_configuration,
            offset,
            bytes,
        );
        Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash)
    };

    let mut write_messages = vec![];
    let chunk_size = utils::calculate_max_chunk_size(&create_msg);
    for (chunk, i) in proposal_data.try_to_vec()?.chunks(chunk_size).zip(0..) {
        write_messages.push(create_msg((i * chunk_size) as u32, chunk.to_vec()));
    }

    // Send message
    let websocket_url = utils::get_ws_url()?;
    let tpu_client = TpuClient::new(
        connection.clone(),
        &websocket_url,
        TpuClientConfig::default(),
    )
    .map_err(Error::TpuSenderError)?;

    let transaction_errors = tpu_client
        .send_and_confirm_messages_with_spinner(&write_messages, &[payer])
        .map_err(Error::TpuSenderError)?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    if !transaction_errors.is_empty() {
        for transaction_error in &transaction_errors {
            eprintln!("{:?}", transaction_error);
        }
        return Err(Error::WriteTransactions(transaction_errors.len()));
    }

    Ok(())
}

pub fn finalize_relay_round_proposal(
    payer: &Keypair,
    event_timestamp: u32,
    event_transaction_lt: u64,
    event_configuration: Pubkey,
    round_number: u32,
    connection: &Arc<RpcClient>,
) -> Result<()> {
    utils::print_header("Finalize Relay Round Proposal");

    let mut transaction = Transaction::new_with_payer(
        &[solana_bridge::round_loader::finalize_proposal_ix(
            &payer.pubkey(),
            event_timestamp,
            event_transaction_lt,
            event_configuration,
            round_number,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer], connection.get_latest_blockhash()?);

    connection.send_and_confirm_transaction(&transaction)?;

    let setting_address = solana_bridge::round_loader::get_settings_address();
    let proposal_address = solana_bridge::round_loader::get_proposal_address(
        &payer.pubkey(),
        &setting_address,
        event_timestamp,
        event_transaction_lt,
        &event_configuration,
    );

    println!("Proposal address: {}", proposal_address);

    Ok(())
}
