#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Malformed Response: {:?}", .0)]
    MalformedStream(String),
    #[error("Unexpected Stream Shutdown")]
    UnexpectedShutdown(),
    #[error("Must specify at least one argument.")]
    TooFewArgs(),



    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    BitcoincoreRPCError(#[from] bitcoincore_rpc::Error),
    #[error(transparent)]
    SQLiteError(#[from] sqlite::Error),
    #[error(transparent)]
    JSONError(#[from] serde_json::Error),
}
