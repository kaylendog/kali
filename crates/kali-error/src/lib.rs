//! Error types for Kali

use ariadne::Report;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A type error occurred.
    #[error("type error: {0}")]
    TypeError(#[from] kali_type::TypeInferenceError),
}
