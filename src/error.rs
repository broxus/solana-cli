use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read solana config file: ({0})")]
    ConfigReadError(std::io::Error),
    #[error("failed to parse solana config file: ({0})")]
    ConfigParseError(#[from] yaml_rust::ScanError),
    #[error("invalid config: ({0})")]
    InvalidConfig(String),
    #[error("invalid pubkey")]
    InvalidPubkey,
    #[error("invalid program path")]
    InvalidProgramPath,
    #[error("failed to open program file: ({0})")]
    ProgramOpenError(std::io::Error),
    #[error("failed to read program file: ({0})")]
    ProgramReadError(std::io::Error),
    #[error("failed to read keypair file")]
    KeypairReadError,
    #[error("failed to write keypair file")]
    KeypairWriteError,

    #[error("solana client error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),

    #[error("solana instruction error: ({0})")]
    InstructionError(#[from] solana_program::instruction::InstructionError),
}

pub type Result<T> = std::result::Result<T, Error>;
