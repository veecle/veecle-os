use std::fmt::Debug;
use std::sync::Arc;

use eyre::WrapErr;
use futures::future::BoxFuture;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use serde::Serialize;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::Instrument;
use veecle_net_utils::{AsyncSocketStream, UnresolvedMultiSocketAddress};
use veecle_orchestrator_protocol::{Info, Request, Response};

use crate::conductor::Conductor;
use crate::distributor::Distributor;

type Responder = Box<
    dyn FnOnce(Framed<AsyncSocketStream, LinesCodec>) -> BoxFuture<'static, eyre::Result<()>>
        + Send,
>;

/// Handles a single API request, returning an encoded response and optionally a closure that will take over the stream
/// after sending the initial response.
#[tracing::instrument(skip_all, fields(request.variant))]
async fn handle_request(
    request: &str,
    distributor: &Distributor,
    conductor: &Conductor,
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
        Request::Add(instance) => {
            conductor.add(instance).await.wrap_err("adding instance")?;
            encode(())?
        }
        Request::Remove(id) => {
            conductor.remove(id).wrap_err("removing instance")?;
            encode(())?
        }
        Request::Start(id) => {
            conductor.start(id).wrap_err("starting instance")?;
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
            runtimes: conductor.info(),
            links: distributor.info().await?,
        })?,
    };

    Ok((response, None))
}

/// Handles all API requests from a single client.
async fn handle_client(
    stream: AsyncSocketStream,
    distributor: &Distributor,
    conductor: &Conductor,
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
                    responder(stream).await?;
                    break;
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
