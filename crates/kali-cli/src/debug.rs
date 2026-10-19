use std::path::PathBuf;

use clap::Subcommand;

use crate::GlobalOpts;

/// The kind of debugging to perform.
#[derive(Subcommand)]
pub enum DebugCommand {
    Parse {
        /// The file to parse.
        file: PathBuf,
    },
    /// Parse a file and print it back as formatted source.
    Format {
        /// The file to print.
        file: PathBuf,
    },
    /// Type check a file and print the inferred types.
    TypeCheck {
        /// The file to type check.
        file: PathBuf,
    },
}

pub fn debug(command: DebugCommand, _opts: &GlobalOpts) {
    let mut context = kali_main::Context::default();
    if let Err(err) = handle(command, &mut context) {
        for e in err {
            e.into_report().eprint(&mut context).unwrap();
        }
    }
}

fn handle(
    command: DebugCommand,
    context: &mut kali_main::Context,
) -> Result<(), Vec<kali_main::Error>> {
    match command {
        DebugCommand::Parse { file } => {
            let id = context.load(&file)?;
            context.parse(id)?;
            context.get_module(id).map(|module| {
                println!("{:#?}", module);
            });
            Ok(())
        }
        DebugCommand::Format { file } => {
            let id = context.load(&file)?;
            context.parse(id)?;
            let formatted = context.format_one_default(id)?;
            println!("{formatted}");
            Ok(())
        }
        DebugCommand::TypeCheck { file } => {
            let id = context.load(&file)?;
            context.parse(id)?;
            context.typecheck(id)?;
            Ok(())
        }
    }
}
