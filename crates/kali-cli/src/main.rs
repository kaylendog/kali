use std::path::PathBuf;

use clap::Parser;
use kali_runtime::Runtime;
use kali_stack::Compile;

/// Command line interface for the Kali programming language.
#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    subcommand: SubCommand,
    /// Enable verbose output.
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(name = "run")]
    Run(Run),
}

/// Run a Kali program.
#[derive(Parser)]
struct Run {
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    match args.subcommand {
        SubCommand::Run(run) => {
            let ast = kali_parse::parse_file(&run.path).unwrap();
            // compile to stack machine
            let mut unit = kali_stack::StackTranslationUnit::new();
            ast.compile(&mut unit);
            // print stack machine
            if args.verbose {
                println!("{:?}", unit);
            }
            // execute
            let mut runtime = Runtime::new(unit.into_inner());
            runtime.run();

            println!("{:?}", runtime.stack());
        }
    }
}
