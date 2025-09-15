//! CAN features for Veecle OS' CLI

#![forbid(unsafe_code)]

pub mod codegen;

#[derive(clap::Parser, Debug)]
/// Arguments for the CAN CLI.
pub struct Arguments {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
/// Commands for the CAN CLI.
enum Commands {
    /// Read a CAN-DBC file and generate code defining the data types and actors to handle them.
    Codegen(codegen::Arguments),
}

impl Arguments {
    /// Runs the CAN command.
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Codegen(command) => command.run(),
        }
    }
}
