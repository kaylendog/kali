mod debug;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::debug::{debug, DebugCommand};

/// Command line interface for the Kali programming language.
#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    #[clap(flatten)]
    opts: GlobalOpts,
}

#[derive(Parser)]
pub struct GlobalOpts {
    /// Enable verbose output.
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Parser)]
enum Command {
    /// Debugging commands.
    #[clap(subcommand)]
    Debug(DebugCommand),
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder().with_default_directive(LevelFilter::ERROR.into()).from_env_lossy(),
        )
        .init();

    let Args { command, opts } = Args::parse();

    match command {
        Command::Debug(command) => debug(command, &opts),
    };
}
