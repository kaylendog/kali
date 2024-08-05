use std::path::PathBuf;

use clap::Parser;
use kali_ir::Compile;
use kali_type::Typed;
use kali_vm::Runtime;
use rustyline::DefaultEditor;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

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
    /// Run a Kali program.
    #[clap(name = "run")]
    Run(Run),
    #[clap(name = "repl")]
    /// Start a REPL session.
    Repl,
}

/// Run a Kali program.
#[derive(Parser)]
struct Run {
    /// Path to the Kali program.
    path: PathBuf,
}

fn main() {
    let args = Args::parse();

    // initialise tracing
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match args.subcommand {
        SubCommand::Run(run) => {
            let ast = kali_parse::parse_file(&run.path).unwrap();
            // compile to stack machine
            let mut unit = kali_ir::StackTranslationUnit::new();
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
            let mut verbose = args.verbose;
            loop {
                let line = rl.readline("kali>> ");
                match line {
                    Ok(line) => {
                        if line.is_empty() {
                            continue;
                        }

                        match line.as_str() {
                            "quit" | "exit" | "q" => break,
                            "debug" => {
                                verbose = true;
                                continue;
                            }
                            _ => {}
                        }

                        rl.add_history_entry(line.as_str())
                            .expect("failed to add history entry");
                        let ast = kali_parse::parse_str(&line).unwrap();

                        // assert stack is well typed
                        let mut ctx = kali_type::Context::default();
                        let ty = match ast.ty(&mut ctx).and_then(|ty| ty.resolve(&mut ctx)) {
                            Ok(ty) => ty,
                            Err(e) => {
                                eprintln!("Type Error: {}", e);
                                continue;
                            }
                        };

                        if !ty.is_monotype() {
                            eprintln!("Type Error: expected definite type, found {:?}", ty);
                            continue;
                        }

                        println!("\nType: {:?}\n", ty);

                        // compile to stack machine
                        let mut unit = kali_ir::StackTranslationUnit::new();
                        ast.compile(&mut unit);

                        // print stack machine
                        if verbose {
                            println!("{:?}", unit);
                        }

                        // execute
                        let mut runtime = Runtime::new(unit.into_inner());
                        if verbose {
                            runtime.run_debug()
                        } else {
                            runtime.run()
                        }

                        println!("{:?}", runtime.stack());
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
