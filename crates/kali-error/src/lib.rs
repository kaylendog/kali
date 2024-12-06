//! Error types for Kali

use std::{env::current_dir, path::Path};

use ariadne::{Color, Label, Report, ReportKind};
use kali_parse::ParseError;
use kali_type::TypeInferenceError;

pub enum Error<'src> {
    /// A type error occurred.
    TypeError(kali_type::TypeInferenceError),
    /// A syntax error occurred.
    SyntaxError(kali_parse::ParseError<'src>),
}

impl Error<'_> {
    pub fn into_report(self) -> Report<'static> {
        // initialise builder
        let builder = match self {
            Error::TypeError(err) => match err {
                TypeInferenceError::UnificationFailed(_, _, _) => todo!(),
                TypeInferenceError::Multiple(_) => todo!(),
                TypeInferenceError::Mismatch { expected, found } => todo!(),
                TypeInferenceError::ResolutionFailed(_) => todo!(),
            },
            Error::SyntaxError(err) => match err {
                ParseError::UnrecognizedToken { expected, token } => {
                    Report::build(ReportKind::Error, (), token.0)
                        .with_message("unrecognized token")
                        .with_label(Label::new(token.0..token.2).with_color(Color::Red))
                        .with_note(format!("expected {}", expected.join(", ")))
                }
                ParseError::UnrecognizedEof { location, expected } => {
                    Report::build(ReportKind::Error, (), 0)
                        .with_message("unexpected eof")
                        .with_note(format!("expected \"{}\"", expected.join("\", \"")))
                }
                ParseError::ExtraToken { token } => Report::build(ReportKind::Error, (), token.0)
                    .with_message("unexpected extra token")
                    .with_label(Label::new(token.0..token.2).with_color(Color::Red))
                    .with_help("try removing this token, or review your source code"),
                ParseError::InvalidToken { location } => {
                    Report::build(ReportKind::Error, (), location)
                        .with_label(Label::new(location..location + 1).with_color(Color::Red))
                        .with_message("invalid token")
                }
                ParseError::User { .. } => unreachable!("custom parser error"),
            },
        };

        builder.finish()
    }
}
