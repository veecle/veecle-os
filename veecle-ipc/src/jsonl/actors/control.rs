use core::convert::Infallible;

use futures::future::join;
use veecle_os_runtime::{Reader, Writer};

use crate::jsonl::Connector;
use crate::{ControlRequest, ControlResponse};

/// An actor that forwards [`ControlRequest`] from the store to the orchestrator and routes
/// [`ControlResponse`] back.
#[veecle_os_runtime::actor]
pub async fn control_handler(
    #[init_context] connector: &Connector,
    requests: Reader<'_, ControlRequest>,
    mut responses: Writer<'_, ControlResponse>,
) -> Infallible {
    let (output, mut input) = connector.control_channels();

    let result: (Infallible, Infallible) = join(
        async move {
            let mut requests = requests.wait_init().await;
            loop {
                let request = requests.wait_for_update().await.read_cloned();

                if let Err(error) = output.send(request).await {
                    veecle_telemetry::error!(
                        "failed to send control message to orchestrator",
                        error = format!("{error:?}")
                    );
                }
            }
        },
        async move {
            loop {
                let Some(response) = input.recv().await else {
                    veecle_telemetry::error!("control response channel closed");
                    panic!("control response channel shouldn't ever close");
                };
                responses.write(response).await;
            }
        },
    )
    .await;

    match result {}
}
