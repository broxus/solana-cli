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
    #[error("invalid event timestamp")]
    InvalidEventTimestamp,
    #[error("invalid transaction lt")]
    InvalidTransactionLt,
    #[error("invalid configuration")]
    InvalidConfiguration,
    #[error("invalid round number")]
    InvalidRoundNumber,
    #[error("invalid proposal round number")]
    InvalidProposalRoundNumber,
    #[error("invalid proposal relays")]
    InvalidProposalRelays,
    #[error("({0}) write transactions failed")]
    WriteTransactions(usize),

    #[error("solana client error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),

    #[error("tpu sender error: ({0})")]
    TpuSenderError(#[from] solana_client::tpu_client::TpuSenderError),

    #[error("solana instruction error: ({0})")]
    InstructionError(#[from] solana_program::instruction::InstructionError),

    #[error("std error: ({0})")]
    StdIoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
