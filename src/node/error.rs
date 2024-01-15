#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Node HelpMessage...")]
    NodeHelpMessage(),
    #[error("Unknown Argument: {:?}", .0)]
    UnknownArgument(String),
    #[error("Could not access home directory")]
    NoHomeDir(),

    #[error("Attempt to sign transaction failed.")]
    CouldNotSignTransaction(),

    #[error("Wallet not specified, use -wallet= or include wallet= in config file.")]
    NoWallet(),

    #[error(transparent)]
    BitcoinConsensusError(#[from] bitcoin::consensus::encode::Error),
    #[error(transparent)]
    BitcoinAddressError(#[from] bitcoin::address::Error),
    #[error(transparent)]
    HexError(#[from] hex::FromHexError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    BitcoincoreRPCError(#[from] bitcoincore_rpc::Error),
    #[error(transparent)]
    SQLiteError(#[from] sqlite::Error),
    #[error(transparent)]
    JSONError(#[from] serde_json::Error),
    #[error(transparent)]
    ParseError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
}
