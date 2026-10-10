use ariadne::{sources, Color, Label, Report, ReportKind};
use std::{ops::Range, path::PathBuf};

use anyhow::{Context, Result};
use clap::Subcommand;
use kali_parse::{chumsky::error::Rich, Token};

use crate::GlobalOpts;

/// The kind of debugging to perform.
#[derive(Subcommand)]
pub enum DebugCommand {
    Lex {
        /// The file to lex.
        file: PathBuf,
    },
    Parse {
        /// The file to parse.
        file: PathBuf,
    },
    /// Parse a file and print it back as formatted source.
    Print {
        /// The file to print.
        file: PathBuf,
        /// Whether to use pretty mode.
        #[clap(long)]
        pretty: bool,
    },
}

pub fn debug(command: DebugCommand, opts: &GlobalOpts) -> Result<()> {
    match command {
        DebugCommand::Lex { file } => lex(file),
        DebugCommand::Parse { file } => parse(file),
        DebugCommand::Print { file, pretty } => print(file, pretty),
    }
}

fn lex(path: PathBuf) -> Result<()> {
    let src = std::fs::read_to_string(&path).context("Failed to read file")?;
    println!("{:#?}", kali_parse::lex_str(&src));
    Ok(())
}

fn parse(path: PathBuf) -> Result<()> {
    let src = std::fs::read_to_string(&path).context("Failed to read file")?;
    let (module, errors) = kali_parse::parse_str(&src);
    let path = path.display().to_string();
    if !errors.is_empty() {
        build_report(path.clone(), &errors)
            .iter()
            .for_each(|report| {
                report
                    .eprint(sources([(path.clone(), src.clone())]))
                    .unwrap();
            });
    }
    println!("{:#?}", module);

    Ok(())
}

fn print(path: PathBuf, pretty: bool) -> Result<()> {
    let src = std::fs::read_to_string(&path).context("Failed to read file")?;
    let (module, errors) = kali_parse::parse_str(&src);
    let path = path.display().to_string();
    for report in build_report(path.clone(), &errors) {
        report.eprint(sources([(path.clone(), src.clone())]))?;
    }
    let module = module.context("Nothing to print, parsing failed")?;
    if pretty {
        kali_print::print_module_pretty(&module, &src, &mut std::io::stdout().lock())?;
    } else {
        kali_print::print_module(&module, &src, &mut std::io::stdout().lock())?;
    }
    Ok(())
}

/// Build an error report for the given error.
fn build_report<'src>(
    filename: impl AsRef<str>,
    errs: &[Rich<'src, Token<'src>>],
) -> Vec<Report<'src, (String, Range<usize>)>> {
    errs.iter()
        .map(|e| {
            Report::build(
                ReportKind::Error,
                (filename.as_ref().to_owned(), e.span().into_range()),
            )
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_message(e.to_string())
            .with_label(
                Label::new((filename.as_ref().to_owned(), e.span().into_range()))
                    .with_message(e.reason().to_string())
                    .with_color(Color::Red),
            )
            .with_labels(e.contexts().map(|(label, span)| {
                Label::new((filename.as_ref().to_owned(), span.into_range()))
                    .with_message(format!("while parsing this {label}"))
                    .with_color(Color::Yellow)
            }))
            .finish()
        })
        .collect()
}
