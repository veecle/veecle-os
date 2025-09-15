//! `veecle-os orchestrator` CLI

use clap::Parser;

fn main() -> anyhow::Result<()> {
    veecle_orchestrator_cli::Arguments::parse().run()
}
