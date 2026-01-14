//! `veecle-os orchestrator` CLI

#![forbid(unsafe_code)]

use std::io::{BufRead, BufReader, Write};

use anyhow::Context;
use camino::Utf8PathBuf;
use comfy_table::{Cell, Color, Table};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use serde::de::DeserializeOwned;
use veecle_net_utils::{BlockingSocketStream, UnresolvedMultiSocketAddress};
use veecle_orchestrator_protocol::{
    BINARY_TRANSFER_CHUNK_SIZE, Info, InstanceId, LinkTarget, Priority, Request, Response,
};

/// Veecle OS Orchestrator CLI interface
///
/// Communicates with the control socket of a local Veecle OS Orchestrator.
#[derive(clap::Parser, Debug)]
#[command(disable_help_subcommand = true, version)]
pub struct Arguments {
    /// The socket address to connect to (Unix path or TCP host:port), can be set via environment for easy sharing between the orchestrator and CLI.
    #[arg(long, env = "VEECLE_ORCHESTRATOR_SOCKET")]
    socket: UnresolvedMultiSocketAddress,

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

    /// Stop all active runtimes and clear all orchestrator state.
    Clear,
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

        /// Send the binary file content instead of just the path (useful for remote orchestrators).
        #[arg(long)]
        copy: bool,

        /// Mark this runtime as privileged, allowing it to send control messages.
        #[arg(long, default_value_t = false)]
        privileged: bool,
    },

    /// Remove the runtime instance with the passed id.
    Remove { id: InstanceId },

    /// Start the runtime instance with the passed id.
    Start {
        id: InstanceId,

        /// Priority level for the runtime process.
        #[arg(long)]
        priority: Option<Priority>,
    },

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

/// Reads, deserializes and checks [`Response::Err`] for a <code>[Response]\<T></code> from `stream`.
fn receive<T>(stream: &mut BufReader<BlockingSocketStream>) -> anyhow::Result<T>
where
    T: DeserializeOwned + 'static,
{
    let response = stream
        .lines()
        .next()
        .context("receiving response")?
        .context("receiving response")?;
    let response: Response<T> = serde_json::from_str(&response).context("parsing response")?;

    Ok(response.into_result()?)
}

/// Serializes and sends `request` then reads, deserializes and checks a <code>[Response]\<T></code>
/// to/from `stream`.
fn send<T>(stream: &mut BufReader<BlockingSocketStream>, request: Request) -> anyhow::Result<T>
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

    receive(stream)
}

/// Sends a [`Request::AddWithBinary`] followed by the binary data with progress reporting.
fn send_add_with_binary(
    stream: &mut BufReader<BlockingSocketStream>,
    id: InstanceId,
    data: &[u8],
    privileged: bool,
) -> anyhow::Result<()> {
    let () = send(stream, Request::add_with_binary(id, data, privileged))
        .context("sending AddWithBinary request, receiving initial response")?;

    // get progress bar for binary upload
    let pb = ProgressBar::new(data.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .expect("valid progress bar template")
            .progress_chars("█▓░"),
    );

    // send data in chunks to enable progress reporting
    for chunk in data.chunks(BINARY_TRANSFER_CHUNK_SIZE) {
        stream
            .get_mut()
            .write_all(chunk)
            .context("sending binary data")?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_and_clear();

    let () = receive(stream).context("receiving final response")?;

    Ok(())
}

impl Arguments {
    /// Runs the command.
    pub fn run(self) -> anyhow::Result<()> {
        let mut stream = BufReader::new(
            self.socket
                .connect_blocking()
                .context("connecting to orchestrator socket")?,
        );

        match self.command {
            Command::Version => {
                let version: String = send(&mut stream, Request::Version)?;
                println!("server version: {version}");
            }
            Command::Runtime(Runtime::Add {
                path,
                id,
                copy,
                privileged,
            }) => {
                let id = id.unwrap_or_else(InstanceId::new);
                if copy {
                    let data = std::fs::read(&path)
                        .with_context(|| format!("reading binary file '{path}'"))?;
                    send_add_with_binary(&mut stream, id, &data, privileged)?;
                    println!("added instance {id} (sent {} bytes)", data.len());
                } else {
                    let () = send(
                        &mut stream,
                        Request::Add {
                            path,
                            id,
                            privileged,
                        },
                    )?;
                    println!("added instance {id}");
                }
            }
            Command::Runtime(Runtime::Remove { id }) => {
                let () = send(&mut stream, Request::Remove(id))?;
                println!("removed instance {id}");
            }
            Command::Runtime(Runtime::Start { id, priority }) => {
                let () = send(&mut stream, Request::Start { id, priority })?;
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
            Command::Clear => {
                let () = send(&mut stream, Request::Clear)?;
                println!("cleared orchestrator state");
            }
        }
        Ok(())
    }
}
