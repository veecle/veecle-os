//! `veecle-os orchestrator` CLI

#![forbid(unsafe_code)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

use anyhow::Context;
use camino::Utf8PathBuf;
use comfy_table::{Cell, Color, Table};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use veecle_orchestrator_protocol::{Info, Instance, InstanceId, LinkTarget, Request, Response};

/// Veecle OS Orchestrator CLI interface
///
/// Communicates with the control socket of a local Veecle OS Orchestrator.
#[derive(clap::Parser, Debug)]
#[command(disable_help_subcommand = true)]
pub struct Arguments {
    /// The path to the control socket, can be set via environment for easy sharing between the orchestrator and CLI.
    #[arg(long, env = "VEECLE_ORCHESTRATOR_SOCKET")]
    socket: Utf8PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Queries the version of the server.
    Version,

    #[command(subcommand)]
    Runtime(Runtime),

    #[command(subcommand)]
    Link(Link),
}

/// Manage runtimes registered on the orchestrator.
#[derive(clap::Subcommand, Debug)]
enum Runtime {
    /// Add a new runtime instance with the passed binary path.
    Add {
        path: Utf8PathBuf,

        /// Force a specific instance id, otherwise a new one will be generated.
        #[arg(long)]
        id: Option<InstanceId>,
    },

    /// Remove the runtime instance with the passed id.
    Remove { id: InstanceId },

    /// Start the runtime instance with the passed id.
    Start { id: InstanceId },

    /// Stop the runtime instance with the passed id.
    Stop { id: InstanceId },

    /// List known runtime instances.
    List,
}

/// Manage IPC links on the orchestrator.
#[derive(clap::Subcommand, Debug)]
enum Link {
    /// Link IPC for a data type to a runtime instance.
    Add {
        /// The type name identifying the data.
        #[arg(long = "type")]
        type_name: String,

        /// The instance that will receive the data.
        #[arg(long)]
        to: LinkTarget,
    },

    /// List configured IPC links.
    List,
}

/// Connects to the Veecle OS Orchestrator control `socket` and sends the `request`, expecting to get back a response wrapping
/// a `T` value that will be deserialized.
fn send<T>(stream: &mut BufReader<UnixStream>, request: Request) -> anyhow::Result<T>
where
    T: DeserializeOwned + 'static,
{
    stream
        .get_mut()
        .write_all(
            serde_json::to_string(&request)
                .context("encoding request")?
                .as_bytes(),
        )
        .context("sending request")?;
    stream
        .get_mut()
        .write_all(b"\n")
        .context("sending request")?;

    let response = stream
        .lines()
        .next()
        .context("receiving response")?
        .context("receiving response")?;
    let response: Response<T> = serde_json::from_str(&response).context("parsing response")?;

    Ok(response.into_result()?)
}

impl Arguments {
    /// Runs the command.
    pub fn run(self) -> anyhow::Result<()> {
        let mut stream = BufReader::new(
            UnixStream::connect(&self.socket).context("connecting to orchestrator socket")?,
        );

        match self.command {
            Command::Version => {
                let version: String = send(&mut stream, Request::Version)?;
                println!("server version: {version}");
            }
            Command::Runtime(Runtime::Add { path, id }) => {
                let id = id.unwrap_or_else(InstanceId::new);
                let () = send(&mut stream, Request::Add(Instance { path, id }))?;
                println!("added instance {id}");
            }
            Command::Runtime(Runtime::Remove { id }) => {
                let () = send(&mut stream, Request::Remove(id))?;
                println!("removed instance {id}");
            }
            Command::Runtime(Runtime::Start { id }) => {
                let () = send(&mut stream, Request::Start(id))?;
                println!("started instance {id}");
            }
            Command::Runtime(Runtime::Stop { id }) => {
                let () = send(&mut stream, Request::Stop(id))?;
                println!("stopped instance {id}");
            }
            Command::Runtime(Runtime::List) => {
                let info: Info = send(&mut stream, Request::Info)?;

                println!(
                    "{}",
                    Table::new()
                        .load_preset(comfy_table::presets::UTF8_FULL)
                        .set_header(["Id", "Binary", "Running"])
                        .add_rows(info.runtimes.iter().map(|(id, info)| {
                            [
                                id.into(),
                                (&info.binary).into(),
                                Cell::new(info.running).fg(if info.running {
                                    Color::DarkGreen
                                } else {
                                    Color::DarkRed
                                }),
                            ]
                        }))
                );
            }
            Command::Link(Link::Add { type_name, to }) => {
                let () = send(
                    &mut stream,
                    Request::Link {
                        type_name: type_name.clone(),
                        to,
                    },
                )?;
                println!("linked {type_name} to {to}");
            }
            Command::Link(Link::List) => {
                let info: Info = send(&mut stream, Request::Info)?;

                println!(
                    "{}",
                    Table::new()
                        .load_preset(comfy_table::presets::UTF8_FULL)
                        .set_header(["For Type", "To Instance(s)"])
                        .add_rows(
                            info.links
                                .iter()
                                .map(|(ty, to)| { [ty.to_string(), to.iter().join("\n")] })
                        )
                );
            }
        }
        Ok(())
    }
}
