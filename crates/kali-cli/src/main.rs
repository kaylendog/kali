use std::path::PathBuf;

use clap::Parser;
use kali_ast::Stmt;
use kali_type::Typed;
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
            let stmts = kali_parse::parse_file(&run.path).unwrap();
            // type check the program
            let mut ctx = kali_type::Context::default();

            // declare top-level types
            for stmt in stmts
                .iter()
                .filter(|stmt| matches!(stmt.inner, Stmt::Type(..)))
            {
                let ty = stmt.ty(&mut ctx);
                if let Err(e) = ty {
                    eprintln!("Type Error: {}", e);
                }
            }

            // type check declarations
            for stmt in stmts
                .iter()
                .filter(|stmt| !matches!(stmt.inner, Stmt::Type(..)))
            {
                let ty = stmt.ty(&mut ctx);
                if let Err(e) = ty {
                    eprintln!("Type Error: {}", e);
                }
            }
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
                            "debug" => {
                                continue;
                            }
                            _ => {}
                        }

                        rl.add_history_entry(line.as_str())
                            .expect("failed to add history entry");
                        let ast = kali_parse::parse_str(&line).unwrap();

                        // // assert program is well typed
                        // let mut ctx = kali_type::Context::default();
                        // let ty = match ast.ty(&mut ctx).and_then(|ty| ty.resolve(&mut ctx)) {
                        //     Ok(ty) => ty,
                        //     Err(e) => {
                        //         eprintln!("Type Error: {}", e);
                        //         continue;
                        //     }
                        // };

                        // if !ty.is_monotype() {
                        //     eprintln!("Type Error: expected definite type, found {:?}", ty);
                        //     continue;
                        // }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}
