use std::path::PathBuf;

use clap::Parser;
use kali_type::Typed;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

// mod compiler;

/// Command line interface for the Kali programming language.
#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    /// Enable verbose output.
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Parser)]
enum Command {
    /// Debugging commands.
    Debug {
        /// The kind of debugging to perform.
        #[clap(subcommand)]
        kind: DebugKind,
    },
}

/// The kind of debugging to perform.
#[derive(Parser)]
enum DebugKind {
    Lex {
        /// The file to lex.
        file: PathBuf,
    },
    Parse {
        /// The file to parse.
        file: PathBuf,
    },
    Typecheck {
        /// The file to typecheck.
        file: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    // initialise tracing
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .from_env_lossy();

    tracing_subscriber::fmt().with_env_filter(filter).init();

    match args.command {
        Command::Debug { kind } => match kind {
            DebugKind::Lex { file } => {
                let contents = std::fs::read_to_string(&file).expect("could not read file");
                let tokens = kali_parse::lexer::lex_to_vec(&contents);
                println!("{:?}", tokens);
            }
            DebugKind::Parse { file } => {
                let contents = std::fs::read_to_string(&file).expect("could not read file");
                let module = kali_parse::parse_str(&contents).expect("could not parse file");
                println!("{:#?}", module);
            }
            DebugKind::Typecheck { file } => {
                let contents = std::fs::read_to_string(&file).expect("could not read file");
                let module = kali_parse::parse_str(&contents).expect("could not parse file");
                module.stmts.iter().for_each(|stmt| {
                    let ty = stmt.ty(&mut kali_type::Context::default());
                    match ty {
                        Ok(ty) => println!("{:?}", ty),
                        Err(e) => eprintln!("Type Error: {:?}", e),
                    }
                });
            }
        },
    }
}
