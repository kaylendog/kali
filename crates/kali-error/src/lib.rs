//! Error types for Kali

use ariadne::Report;

pub enum Error {
    /// A type error occurred.
    Type(kali_type::Error),
}

impl Error {
    pub fn into_report(self) -> Report<'static> {
        todo!()
    }
}
