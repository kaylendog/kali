//! Error types for Kali

use ariadne::Report;
pub enum Error {
    /// A type error occurred.
    TypeError(kali_type::TypeInferenceError),
}

impl Error {
    pub fn into_report(self) -> Report<'static> {
        todo!()
    }
}
