use std::{io::Write, path::PathBuf};

use clap::Parser;
use kali_runtime::Runtime;
use kali_stack::Compile;
use rustyline::DefaultEditor;

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

        SubCommand::Repl => {
            let mut rl = DefaultEditor::new().expect("failed to initialise repl");
            loop {
                let line = rl.readline("kali>> ");
                match line {
                    Ok(line) => {
                        if line.is_empty() {
                            continue;
                        }

                        match line.as_str() {
                            "quit" | "exit" | "q" => break,
                            _ => {}
                        }

                        rl.add_history_entry(line.as_str())
                            .expect("failed to add history entry");
                        let ast = kali_parse::parse_str(&line).unwrap();

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
                    Err(_) => break,
                }
            }
        }
    }
}
