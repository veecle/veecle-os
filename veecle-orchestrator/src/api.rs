use std::fmt::Debug;
use std::ops::ControlFlow;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use eyre::WrapErr;
use futures::future::BoxFuture;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tempfile::{Builder, TempPath};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::Instrument;
use veecle_net_utils::{AsyncSocketStream, UnresolvedMultiSocketAddress};
use veecle_orchestrator_protocol::{Info, InstanceId, Request, Response};

use crate::distributor::Distributor;
use crate::runtime::Conductor;

type Responder = Box<
    dyn FnOnce(
            Framed<AsyncSocketStream, LinesCodec>,
        ) -> BoxFuture<
            'static,
            eyre::Result<ControlFlow<(), (Framed<AsyncSocketStream, LinesCodec>, String)>>,
        > + Send,
>;

/// Handles a [`Request::AddWithBinary`] message.
///
/// Reads and verifies the binary data from the stream, then adds the instance to the conductor.
async fn handle_add_with_binary(
    stream: &mut AsyncSocketStream,
    conductor: Arc<Conductor>,
    id: InstanceId,
    length: usize,
    hash: [u8; 32],
    privileged: bool,
) -> eyre::Result<()> {
    let path = read_binary_to_temp_file(stream, length, hash)
        .await
        .wrap_err("reading binary data")?;

    conductor
        .add(id, path.into(), privileged)
        .await
        .wrap_err("adding binary instance")?;

    Ok(())
}

/// Reads and verifies binary data from a stream into a temporary executable file.
///
/// Creates a new temporary file, reads `length` bytes from the stream, validates the SHA-256 hash,
/// sets executable permissions, and returns a [`TempPath`] that will clean up the file when dropped.
async fn read_binary_to_temp_file(
    stream: &mut AsyncSocketStream,
    length: usize,
    hash: [u8; 32],
) -> eyre::Result<TempPath> {
    let mut file = tokio::task::spawn_blocking(|| {
        Builder::new()
            .prefix("veecle-runtime-")
            .suffix(".bin")
            .make(|path| std::fs::File::create(path).map(File::from))
            .wrap_err("creating temporary file")
    })
    .await??;

    let mut hasher = Sha256::new();
    let mut remaining = length;
    let mut buffer = [0u8; 8192];

    while remaining > 0 {
        let chunk_size = buffer.len().min(remaining);

        let bytes_read = stream
            .read(&mut buffer[..chunk_size])
            .await
            .wrap_err("reading binary data from stream")?;

        if bytes_read == 0 {
            eyre::bail!("connection closed before receiving all binary data");
        }

        let chunk = &buffer[..bytes_read];
        hasher.update(chunk);
        file.as_file_mut()
            .write_all(chunk)
            .await
            .wrap_err("writing binary data to temporary file")?;

        remaining -= bytes_read;
    }

    let computed_hash: [u8; 32] = hasher.finalize().into();
    if computed_hash != hash {
        eyre::bail!("binary data hash verification failed");
    }

    file.as_file_mut()
        .sync_all()
        .await
        .wrap_err("syncing temporary file")?;

    let path = file.into_temp_path();

    // `0o755` is equivalent to `u=rwx,go=rx`; the owner can read, write and execute, all others
    // can only read and execute. This is necessary so we can run the written binary.
    tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).await?;

    Ok(path)
}

/// Handles a single API request, returning an encoded response and optionally a closure that will take over the stream
/// after sending the initial response.
#[tracing::instrument(skip_all, fields(request.variant))]
async fn handle_request(
    request: &str,
    distributor: &Distributor,
    conductor: &Arc<Conductor>,
) -> eyre::Result<(String, Option<Responder>)> {
    tracing::debug!(request.unparsed = %request);

    let request: Request = serde_json::from_str(request).wrap_err("parsing request")?;

    tracing::info!(request.parsed = ?request);
    tracing::Span::current().record(
        "request.variant",
        format_args!("{}", request.variant_name()),
    );

    fn encode(response: impl Serialize + Debug) -> eyre::Result<String> {
        tracing::info!(?response);
        Ok(serde_json::to_string(&Response::Ok(response))?)
    }

    let response = match request {
        Request::Version => encode(env!("CARGO_PKG_VERSION"))?,
        Request::Add {
            id,
            path,
            privileged,
        } => {
            conductor
                .add(id, path.into(), privileged)
                .await
                .wrap_err("adding instance")?;
            encode(())?
        }
        Request::AddWithBinary {
            id,
            length,
            hash,
            privileged,
        } => {
            let conductor = Arc::clone(conductor);

            let responder: Responder = Box::new(move |mut stream| {
                Box::pin(async move {
                    // Technically the `.get_mut()` here doesn't include the `Framed` read buffer.
                    // But this is ok because:
                    //  * the client sends the request line
                    //  * we send the initial response line
                    //  * the client sends the binary data
                    // As long as the client waits to receive the response line we shouldn't have
                    // received any of the binary data into the read buffer.
                    match handle_add_with_binary(
                        stream.get_mut(),
                        conductor,
                        id,
                        length,
                        hash,
                        privileged,
                    )
                    .await
                    {
                        Ok(()) => Ok(ControlFlow::Continue((stream, encode(())?))),
                        Err(error) => {
                            tracing::warn!(?error);
                            let response = serde_json::to_string(&Response::<()>::err(&*error))
                                .wrap_err("encoding error response")?;
                            Ok(ControlFlow::Continue((stream, response)))
                        }
                    }
                })
            });

            return Ok((encode(())?, Some(responder)));
        }
        Request::Remove(id) => {
            conductor.remove(id).await.wrap_err("removing instance")?;
            encode(())?
        }
        Request::Start { id, priority } => {
            conductor
                .start(id, priority)
                .await
                .wrap_err("starting instance")?;
            encode(())?
        }
        Request::Stop(id) => {
            conductor.stop(id).await.wrap_err("stopping instance")?;
            encode(())?
        }
        Request::Link { type_name, to } => {
            distributor
                .link(type_name, to)
                .await
                .wrap_err("linking instances")?;
            encode(())?
        }
        Request::Info => encode(Info {
            runtimes: conductor.info().await?,
            links: distributor.info().await?,
        })?,
        Request::Clear => {
            conductor.clear().await;
            distributor.clear().await.wrap_err("clearing distributor")?;
            encode(())?
        }
    };

    Ok((response, None))
}

/// Handles all API requests from a single client.
async fn handle_client(
    stream: AsyncSocketStream,
    distributor: &Distributor,
    conductor: &Arc<Conductor>,
) -> eyre::Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());

    tracing::info!("client connected");

    while let Some(line) = stream
        .next()
        .await
        .transpose()
        .wrap_err("receiving request")?
    {
        match handle_request(&line, distributor, conductor).await {
            Ok((response, responder)) => {
                stream.send(response).await.wrap_err("sending response")?;
                if let Some(responder) = responder {
                    stream = match responder(stream).await.context("in responder")? {
                        ControlFlow::Continue((mut stream, response)) => {
                            stream
                                .send(response)
                                .await
                                .wrap_err("sending post-operation response")?;
                            stream
                        }
                        ControlFlow::Break(()) => {
                            break;
                        }
                    }
                }
            }
            Err(error) => {
                tracing::warn!(?error, "error handling client request");
                let response = serde_json::to_string(&Response::<()>::err(&*error))
                    .wrap_err("encoding error response")?;
                stream
                    .send(response)
                    .await
                    .wrap_err("sending error response")?;
            }
        }
    }

    tracing::info!("client disconnected");

    Ok(())
}

/// Serves the API defined in [`veecle_orchestrator_protocol`] on the specified socket address.
#[tracing::instrument(skip_all, fields(%address))]
pub async fn run(
    address: UnresolvedMultiSocketAddress,
    distributor: Arc<Distributor>,
    conductor: Arc<Conductor>,
) -> eyre::Result<()> {
    let listener = address.bind_async().await.wrap_err("binding socket")?;
    let mut connection_ids = 0..u64::MAX;

    tracing::info!("listening");
    loop {
        let (stream, client_address) = listener.accept().await.wrap_err("accepting connection")?;
        let connection_id = connection_ids.next().unwrap();
        let distributor = distributor.clone();
        let conductor = conductor.clone();
        tokio::spawn(
            async move {
                if let Err(error) = handle_client(stream, &distributor, &conductor).await {
                    tracing::error!(?error, "handling client failed");
                }
            }
            .instrument(tracing::info_span!(
                parent: None,
                "api_connection",
                connection.id = connection_id,
                connection.client = %client_address,
            )),
        );
    }
}
