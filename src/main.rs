use std::str::FromStr;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};

use solana_clap_utils::input_parsers::value_of;
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
                        .value_name("AUTHORITY")
                        .takes_value(true)
                        .required(true)
                        .help("Multsig address"),
                )
                .arg(
                    Arg::with_name("payer-path")
                        .long("payer-path")
                        .value_name("PAYER_PATH")
                        .takes_value(true)
                        .required(false)
                        .help("Path to the payer keypair"),
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
                        .value_name("AUTHORITY")
                        .takes_value(true)
                        .required(true)
                        .help("Multsig address"),
                )
                .arg(
                    Arg::with_name("payer-path")
                        .long("payer-path")
                        .value_name("PAYER_PATH")
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

    let _ = match (sub_command, sub_matches) {
        ("deploy", Some(arg_matches)) => {
            let payer = match value_of::<String>(arg_matches, "payer-path") {
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

            create_buffer(&payer, &buffer, &payer.pubkey(), &program_path, &connection)?;

            write_buffer(&payer, &buffer.pubkey(), &program_path, &connection)?;

            let program = Keypair::new();
            let keypair_file = get_keypair_file(&program_path);
            write_keypair_file(&program, keypair_file)
                .map_err(|_| anyhow::Error::new(Error::KeypairReadError))?;

            deploy(
                &payer,
                &program,
                &buffer.pubkey(),
                &program_path,
                &connection,
            )?;

            set_program_authority(&payer, &program.pubkey(), &authority_pubkey, &connection)?;
        }
        ("upload-program-buffer", Some(arg_matches)) => {
            let payer = match value_of::<String>(arg_matches, "payer-path") {
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

            create_buffer(&payer, &buffer, &payer.pubkey(), &program_path, &connection)?;

            write_buffer(&payer, &buffer.pubkey(), &program_path, &connection)?;

            set_buffer_authority(
                &payer,
                &payer,
                &buffer.pubkey(),
                &authority_pubkey,
                &connection,
            )?;
        }
        _ => {}
    };

    Ok(())
}
