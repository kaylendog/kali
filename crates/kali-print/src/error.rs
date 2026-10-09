/// A specialized `Result` type for printing operations, using the custom `Error` type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for printing operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),
}
