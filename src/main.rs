use std::str::FromStr;

use borsh::BorshSerialize;
use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};
use solana_bridge::round_loader::{RelayRoundProposalEventWithLen, MAX_RELAYS, MIN_RELAYS};

use solana_clap_utils::input_parsers::{value_of, values_of};
use solana_clap_utils::input_validators::{is_keypair, is_valid_pubkey};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, write_keypair_file, Keypair, Signer};

use solana_cli::client::*;
use solana_cli::error::*;
use solana_cli::utils::*;

fn main() -> anyhow::Result<()> {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("deploy")
                .about("Deploy program ")
                .arg(
                    Arg::with_name("program-path")
                        .long("program-path")
                        .value_name("PROGRAM_PATH")
                        .takes_value(true)
                        .required(true)
                        .help("Path to the program"),
                )
                .arg(
                    Arg::with_name("authority")
                        .long("authority")
                        .validator(is_valid_pubkey)
                        .value_name("AUTHORITY")
                        .takes_value(true)
                        .required(true)
                        .help("Multisig address"),
                )
                .arg(
                    Arg::with_name("payer-keypair")
                        .long("payer-keypair")
                        .validator(is_keypair)
                        .value_name("PAYER_KEYPAIR")
                        .takes_value(true)
                        .required(false)
                        .help("Path to the payer keypair"),
                )
                .arg(
                    Arg::with_name("program-keypair")
                        .long("program-keypair")
                        .validator(is_keypair)
                        .value_name("PROGRAM_KEYPAIR")
                        .takes_value(true)
                        .required(false)
                        .help("Path to the program keypair"),
                )
                .arg(
                    Arg::with_name("program-size")
                        .long("program-size")
                        .value_name("PROGRAM_SIZE")
                        .takes_value(true)
                        .required(true)
                        .help("Program size"),
                ),
        )
        .subcommand(
            SubCommand::with_name("upload-program-buffer")
                .about("Upload program to buffer account")
                .arg(
                    Arg::with_name("program-path")
                        .long("program-path")
                        .value_name("PROGRAM_PATH")
                        .takes_value(true)
                        .required(true)
                        .help("Path to the program"),
                )
                .arg(
                    Arg::with_name("authority")
                        .long("authority")
                        .validator(is_valid_pubkey)
                        .value_name("AUTHORITY")
                        .takes_value(true)
                        .required(true)
                        .help("Multsig address"),
                )
                .arg(
                    Arg::with_name("payer-keypair")
                        .long("payer-keypair")
                        .validator(is_keypair)
                        .value_name("PAYER_KEYPAIR")
                        .takes_value(true)
                        .required(false)
                        .help("Path to the payer keypair"),
                ),
        )
        .subcommand(
            SubCommand::with_name("set-program-authority")
                .about("Set a program's authority.")
                .arg(
                    Arg::with_name("program")
                        .long("program")
                        .validator(is_valid_pubkey)
                        .value_name("PROGRAM")
                        .takes_value(true)
                        .required(true)
                        .help("Program address"),
                )
                .arg(
                    Arg::with_name("current-authority-keypair")
                        .long("current-authority-keypair")
                        .validator(is_keypair)
                        .value_name("CURRENT_AUTHORITY_KEYPAIR")
                        .takes_value(true)
                        .required(false)
                        .help("Path to the current authority keypair"),
                )
                .arg(
                    Arg::with_name("new-authority")
                        .long("new-authority")
                        .validator(is_valid_pubkey)
                        .value_name("NEW_AUTHORITY")
                        .takes_value(true)
                        .required(true)
                        .help("New authority address"),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-relay-round")
                .about("Set a program's authority.")
                .arg(
                    Arg::with_name("event_timestamp")
                        .long("event-timestamp")
                        .value_name("EVENT_TIMESTAMP")
                        .takes_value(true)
                        .required(true)
                        .help("Everscale event timestamp"),
                )
                .arg(
                    Arg::with_name("transaction_lt")
                        .long("transaction-lt")
                        .value_name("TRANSACTION_LT")
                        .takes_value(true)
                        .required(true)
                        .help("Everscale event transaction lt"),
                )
                .arg(
                    Arg::with_name("configuration")
                        .long("configuration")
                        .value_name("CONFIGURATION")
                        .takes_value(true)
                        .required(true)
                        .help("Everscale event configuration"),
                )
                .arg(
                    Arg::with_name("round_number")
                        .long("round-number")
                        .value_name("ROUND_NUMBER")
                        .takes_value(true)
                        .required(true)
                        .help("Current Relay Round number"),
                )
                .arg(
                    Arg::with_name("proposal_round_number")
                        .long("proposal-round-number")
                        .value_name("PROPOSAL_ROUND_NUMBER")
                        .takes_value(true)
                        .required(true)
                        .help("Relay Round number in proposal"),
                )
                .arg(
                    Arg::with_name("proposal_relays")
                        .long("proposal-relays")
                        .value_name("PROPOSAL_RELAYS")
                        .takes_value(true)
                        .required(true)
                        .min_values(MIN_RELAYS as u64)
                        .max_values(MAX_RELAYS as u64)
                        .help("List of Relays in proposal"),
                )
                .arg(
                    Arg::with_name("proposal_round_end")
                        .long("proposal-round-end")
                        .value_name("PROPOSAL_ROUND_END")
                        .takes_value(true)
                        .required(true)
                        .help("Round end value in proposal"),
                )
                .arg(
                    Arg::with_name("payer_keypair")
                        .long("payer-keypair")
                        .validator(is_keypair)
                        .value_name("PAYER_KEYPAIR")
                        .takes_value(true)
                        .required(false)
                        .help("Path to the payer keypair"),
                ),
        )
        .get_matches();

    let connection = establish_connection()?;
    println!(
        "Connected to remote solana node running version ({}).",
        connection.get_version()?
    );

    let (sub_command, sub_matches) = app_matches.subcommand();

    match (sub_command, sub_matches) {
        ("deploy", Some(arg_matches)) => {
            let payer = match value_of::<String>(arg_matches, "payer-keypair") {
                None => get_payer()?,
                Some(path) => read_keypair_file(&path)
                    .map_err(|_| anyhow::Error::new(Error::KeypairReadError))?,
            };
            println!("Deploying with key: {}", payer.pubkey());

            let buffer = Keypair::new();
            println!("Buffer key: {}", buffer.pubkey());

            let authority_pubkey = Pubkey::from_str(
                value_of::<String>(arg_matches, "authority")
                    .ok_or(Error::InvalidPubkey)?
                    .as_str(),
            )?;
            println!("Program authority: {}", authority_pubkey);

            let program_path =
                value_of::<String>(arg_matches, "program-path").ok_or(Error::InvalidProgramPath)?;

            let program_data = read_elf(&program_path)?;

            let max_data_len = match value_of::<usize>(arg_matches, "program-size") {
                Some(len) => len * 1000,
                None => program_data.len(),
            };

            create_buffer(&payer, &buffer, &payer.pubkey(), max_data_len, &connection)?;

            write_buffer(&payer, &buffer.pubkey(), &program_data, &connection)?;

            let program = match value_of::<String>(arg_matches, "program-keypair") {
                None => {
                    let program = Keypair::new();
                    let keypair_file = get_keypair_file(&program_path);
                    write_keypair_file(&program, keypair_file)
                        .map_err(|_| anyhow::Error::new(Error::KeypairReadError))?;
                    program
                }
                Some(path) => read_keypair_file(&path)
                    .map_err(|_| anyhow::Error::new(Error::KeypairReadError))?,
            };

            deploy(
                &payer,
                &program,
                &buffer.pubkey(),
                max_data_len,
                &connection,
            )?;

            set_program_authority(&payer, &program.pubkey(), &authority_pubkey, &connection)?;
        }
        ("upload-program-buffer", Some(arg_matches)) => {
            let payer = match value_of::<String>(arg_matches, "payer-keypair") {
                None => get_payer()?,
                Some(path) => read_keypair_file(&path)
                    .map_err(|_| anyhow::Error::new(Error::KeypairReadError))?,
            };
            println!("Uploading with key: {}", payer.pubkey());

            let buffer = Keypair::new();
            println!("Buffer key: {}", buffer.pubkey());

            let authority_pubkey = Pubkey::from_str(
                value_of::<String>(arg_matches, "authority")
                    .ok_or(Error::InvalidPubkey)?
                    .as_str(),
            )?;
            println!("Buffer authority: {}", authority_pubkey);

            let program_path =
                value_of::<String>(arg_matches, "program-path").ok_or(Error::InvalidProgramPath)?;

            let program_data = read_elf(&program_path)?;

            create_buffer(
                &payer,
                &buffer,
                &payer.pubkey(),
                program_data.len(),
                &connection,
            )?;

            write_buffer(&payer, &buffer.pubkey(), &program_data, &connection)?;

            set_buffer_authority(
                &payer,
                &payer,
                &buffer.pubkey(),
                &authority_pubkey,
                &connection,
            )?;
        }
        ("set-program-authority", Some(arg_matches)) => {
            let current_authority =
                match value_of::<String>(arg_matches, "current-authority-keypair") {
                    None => get_payer()?,
                    Some(path) => read_keypair_file(&path)
                        .map_err(|_| anyhow::Error::new(Error::KeypairReadError))?,
                };
            println!("Current authority: {}", current_authority.pubkey());

            let program_pubkey = Pubkey::from_str(
                value_of::<String>(arg_matches, "program")
                    .ok_or(Error::InvalidPubkey)?
                    .as_str(),
            )?;
            println!("Program: {}", program_pubkey);

            let new_authority_pubkey = Pubkey::from_str(
                value_of::<String>(arg_matches, "new-authority")
                    .ok_or(Error::InvalidPubkey)?
                    .as_str(),
            )?;
            println!("Program: {}", program_pubkey);

            set_program_authority(
                &current_authority,
                &program_pubkey,
                &new_authority_pubkey,
                &connection,
            )?;
        }
        ("create-relay-round", Some(arg_matches)) => {
            let payer = match value_of::<String>(arg_matches, "payer-keypair") {
                None => get_payer()?,
                Some(path) => read_keypair_file(&path)
                    .map_err(|_| anyhow::Error::new(Error::KeypairReadError))?,
            };
            println!("Creating proposal with key: {}", payer.pubkey());

            let event_timestamp = value_of::<u32>(arg_matches, "event_timestamp")
                .ok_or(Error::InvalidEventTimestamp)?;

            let event_transaction_lt = value_of::<u64>(arg_matches, "transaction_lt")
                .ok_or(Error::InvalidTransactionLt)?;

            let event_configuration = {
                let bytes = hex::decode(
                    value_of::<String>(arg_matches, "configuration")
                        .ok_or(Error::InvalidConfiguration)?
                        .as_str(),
                )?;
                Pubkey::try_from(bytes.as_slice())?
            };

            let round_number =
                value_of::<u32>(arg_matches, "round_number").ok_or(Error::InvalidRoundNumber)?;

            let proposal_round_num = value_of::<u32>(arg_matches, "proposal_round_number")
                .ok_or(Error::InvalidProposalRoundNumber)?;

            let proposal_relays = values_of::<String>(arg_matches, "proposal_relays")
                .ok_or(Error::InvalidProposalRelays)?;

            let mut relays = vec![];
            for proposal_relay in proposal_relays {
                let bytes = hex::decode(&proposal_relay)?;
                relays.push(Pubkey::try_from(bytes.as_slice())?);
            }

            let proposal_round_end = value_of::<u32>(arg_matches, "proposal_round_end")
                .ok_or(Error::InvalidRoundNumber)?;

            let proposal =
                RelayRoundProposalEventWithLen::new(proposal_round_num, relays, proposal_round_end);

            let proposal_pubkey = solana_bridge::round_loader::get_proposal_address(
                round_number,
                event_timestamp,
                event_transaction_lt,
                &event_configuration,
                &proposal.data.try_to_vec()?,
            );

            println!("Proposal address: {}", proposal_pubkey);

            create_relay_round_proposal(
                &payer,
                round_number,
                event_timestamp,
                event_transaction_lt,
                event_configuration,
                &proposal,
                &connection,
            )?;

            write_relay_round_proposal(&payer, &proposal_pubkey, &proposal, &connection)?;

            finalize_relay_round_proposal(&payer, &proposal_pubkey, round_number, &connection)?;
        }
        _ => {}
    };

    Ok(())
}
