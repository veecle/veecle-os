use futures::future::join;
use veecle_os_runtime::Never;
use veecle_os_runtime::single_writer::{Reader, Writer};

use crate::{Connector, ControlRequest, ControlResponse};

/// An actor that forwards [`ControlRequest`] from the store to the orchestrator and routes
/// [`ControlResponse`] back.
#[veecle_os_runtime::actor]
pub async fn control_handler(
    #[init_context] connector: &Connector,
    mut requests: Reader<'_, ControlRequest>,
    mut responses: Writer<'_, ControlResponse>,
) -> Never {
    let (output, mut input) = connector.control_channels();

    let result: (Never, Never) = join(
        async move {
            loop {
                let request = requests.read_updated_cloned().await;

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
