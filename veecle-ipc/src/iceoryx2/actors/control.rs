use core::convert::Infallible;

use anyhow::Context;
use iceoryx2::pending_response::PendingResponse;
use iceoryx2::port::client::Client;
use iceoryx2::port::unable_to_deliver_strategy::UnableToDeliverStrategy;
use iceoryx2::service::ipc::Service;
use iceoryx2::service::service_name::ServiceName;
use veecle_os_runtime::{Reader, Writer};
use veecle_osal_std::time::{Duration, Time, TimeAbstraction};
use veecle_telemetry::future::FutureExt;
use veecle_telemetry::span;

use crate::{ControlRequest, ControlResponse};

use super::super::Connector;

#[veecle_telemetry::instrument]
fn send(
    client: &mut Client<Service, ControlRequest, (), ControlResponse, ()>,
    request: ControlRequest,
) -> anyhow::Result<PendingResponse<Service, ControlRequest, (), ControlResponse, ()>> {
    let buffer = client
        .loan_uninit()
        .context("failed to loan buffer for control request")?;
    let buffer = buffer.write_payload(request);
    let pending = buffer
        .send()
        .context("failed send control request to orchestrator")?;

    Ok(pending)
}

#[veecle_telemetry::instrument]
async fn receive(
    pending: PendingResponse<Service, ControlRequest, (), ControlResponse, ()>,
) -> anyhow::Result<ControlResponse> {
    loop {
        if let Some(response) = pending
            .receive()
            .context("failed to receive control response")?
        {
            break Ok(response.clone());
        };

        // No response yet, need to busy-loop.
        Time::sleep(Duration::from_millis(1))
            .with_span(span!("sleep"))
            .await
            .unwrap();
    }
}

#[veecle_telemetry::instrument]
async fn process(
    client: &mut Client<Service, ControlRequest, (), ControlResponse, ()>,
    request: ControlRequest,
) -> anyhow::Result<ControlResponse> {
    veecle_telemetry::trace!("sending control request", request = format!("{request:?}"));

    let pending = send(client, request)?;
    let response = receive(pending).await?;

    veecle_telemetry::trace!(
        "received control response",
        response = format!("{response:?}")
    );

    Ok(response)
}

/// An actor that forwards [`ControlRequest`] from the store to the orchestrator via iceoryx2
/// request/response and routes [`ControlResponse`] back using zero-copy.
#[veecle_os_runtime::actor]
pub async fn control_handler(
    #[init_context] connector: &Connector,
    mut requests: Reader<'_, ControlRequest>,
    mut responses: Writer<'_, ControlResponse>,
) -> Infallible {
    // TODO: I have no idea how we would do guarantees that it is the correct runtime connecting
    // here. (With JSONL we would just restrict access to the socket to that one runtime).
    let service_name = ServiceName::new(&format!(
        "veecle/runtime/{}/control",
        connector.runtime_id()
    ))
    .unwrap();

    let service = connector
        .node()
        .service_builder(&service_name)
        .request_response::<ControlRequest, ControlResponse>()
        .open_or_create()
        .unwrap();

    let mut client = service
        .client_builder()
        .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
        .create()
        .unwrap();

    // No idea what it's doing, but it seems it needs some time before sending the first
    // message or it will be missed by the server.
    Time::sleep(Duration::from_millis(100)).await.unwrap();

    loop {
        let request = requests.wait_for_update().await.read_cloned().unwrap();

        let response = process(&mut client, request).await.unwrap_or_else(|error| {
            let error = format!("{error:?}");
            let response = ControlResponse::error(&error);
            veecle_telemetry::error!("handling control request failed", error = error);
            response
        });

        responses.write(response).await;
    }
}
