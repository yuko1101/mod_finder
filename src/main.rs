mod command;
mod util;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    #[clap(arg_required_else_help = true)]
    Start {
        #[arg(required = true)]
        mods_dir: PathBuf,
        #[arg(required = false)]
        consistent_mods: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);

    match args.cmd {
        SubCommand::Start { mods_dir, consistent_mods } => {
            command::start::start(mods_dir, consistent_mods)?;
        }
    }

    Ok(())
}
