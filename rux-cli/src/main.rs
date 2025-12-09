// CLI tool for RUX

mod commands;

use clap::Parser;
use commands::{Cli, Commands, handle_build, handle_dev, handle_new, handle_check};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Build(args) => handle_build(args)?,
        Commands::Dev(args) => handle_dev(args)?,
        Commands::New(args) => handle_new(args)?,
        Commands::Check(args) => handle_check(args)?,
    }
    
    Ok(())
}
