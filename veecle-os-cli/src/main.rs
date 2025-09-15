//! The Veecle OS command line tool.

#![forbid(unsafe_code)]

use clap::Parser;
use veecle_os_data_support_can_cli as can;

/// The Veecle OS command line.
#[derive(Debug, Parser)]
#[command()]
struct VeecleOsCli {
    /// Subcommands to execute.
    #[command(subcommand)]
    command: Commands,
}

/// List of supported subcommands.
#[derive(Debug, clap::Subcommand)]
enum Commands {
    Can(can::Arguments),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = VeecleOsCli::parse();

    match arguments.command {
        Commands::Can(command) => command.run(),
    }
}
