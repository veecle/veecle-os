//! Veecle OS Orchestrator.

#![forbid(unsafe_code)]

use std::sync::Arc;

use camino::Utf8PathBuf;
use clap::Parser;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

use veecle_orchestrator::UnresolvedSocketAddr;

use self::conductor::Conductor;
use self::distributor::Distributor;
use self::telemetry::Exporter;

mod api;
mod conductor;
mod distributor;
mod external;
mod eyre_tracing_error;
mod listener;
mod telemetry;

#[derive(Parser)]
struct Arguments {
    #[arg(long, env = "VEECLE_ORCHESTRATOR_SOCKET")]
    control_socket: Utf8PathBuf,

    #[arg(long)]
    ipc_socket: Option<UnresolvedSocketAddr>,

    #[arg(long, env = "VEECLE_TELEMETRY_SOCKET")]
    telemetry_socket: Option<UnresolvedSocketAddr>,
}

// 16 arbitrarily chosen for channel sizing because it looks nice.
const ARBITRARY_CHANNEL_BUFFER: usize = 16;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Arguments::parse();

    eyre::set_hook(Box::new(eyre_tracing_error::Handler::default_with))?;

    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .with_env_var("VEECLE_ORCHESTRATOR_LOG")
                    .from_env()?,
            )
            .with_writer(std::io::stderr)
            .compact()
            .finish()
            .with(tracing_error::ErrorLayer::default()),
    )?;

    let exporter = if let Some(address) = args.telemetry_socket {
        Some(Arc::new(Exporter::new(address)?))
    } else {
        None
    };

    let (distributor, external) = if let Some(ipc_socket) = args.ipc_socket {
        let (external_output_tx, external_output_rx) =
            tokio::sync::mpsc::channel(ARBITRARY_CHANNEL_BUFFER);

        let distributor = Arc::new(Distributor::new(Some(external_output_tx)));

        let external = Some(tokio::spawn(external::run(
            ipc_socket,
            distributor.sender(),
            external_output_rx,
        )));

        (distributor, external)
    } else {
        (Arc::new(Distributor::new(None)), None)
    };

    let conductor = Arc::new(Conductor::new(distributor.clone(), exporter.clone())?);

    let api = tokio::spawn(api::run(
        args.control_socket,
        distributor.clone(),
        conductor.clone(),
    ));

    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    tokio::select! {
        _ = sigint.recv() => {
            tracing::info!("received SIGINT, shutting down");
        }
        _ = sigterm.recv() => {
            tracing::info!("received SIGTERM, shutting down");
        }
    }

    if let Some(external) = external {
        external.abort();
    }
    api.abort();

    conductor.shutdown().await;

    // Shut down telemetry exporter to flush remaining messages.
    if let Some(exporter) = exporter {
        exporter.shutdown().await;
    }

    Ok(())
}
