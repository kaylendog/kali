use std::{io::Write, path::PathBuf};

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
    #[clap(name = "repl")]
    /// Start a REPL session.
    Repl,
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

        SubCommand::Repl => loop {
            // print prompt
            print!("> ");
            std::io::stdout().flush().unwrap();

            // read input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            // parse and compile
            let ast = kali_parse::parse_str(&input).unwrap();
            let mut unit = kali_stack::StackTranslationUnit::new();
            ast.compile(&mut unit);

            // print stack machine
            if args.verbose {
                println!("{:?}", unit);
            }

            // execute
            let mut runtime = Runtime::new(unit.into_inner());
            runtime.run();
            // print output
            println!("{:?}", runtime.stack());
        },
    }
}
