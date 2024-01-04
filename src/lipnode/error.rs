#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Node HelpMessage...")]
    NodeHelpMessage(),
    #[error("Unknown Argument: {:?}", .0)]
    UnknownArgument(String),
    #[error("Could not access home directory")]
    NoHomeDir(),
    #[error("Vectors Max Size: {}", .0)]
    MaxVectorSize(u32),

    #[error("Wallet not specified, use -wallet= or include wallet= in config file.")]
    NoWallet(),
    #[error("Wallet({}) Was expected to be watchonly, please move wallet or select another.", .0)]
    WalletNotWatchOnly(String),
    #[error("Wallet({}) Was not expected to be watchonly, please move wallet or select another.", .0)]
    WalletWatchOnly(String),

    #[error("Value is not a String")]
    NotString(),
    #[error("Value is not a std::vector")]
    NotVector(),
    #[error("Value is not an i64")]
    NotI64(),
    #[error("Value is not a u64")]
    NotU64(),

    #[error(transparent)]
    HTBE(#[from] bitcoin::hex::parse::HexToBytesError),
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
