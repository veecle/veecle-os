use serde::Serialize;
use veecle_ipc_protocol::EncodedStorable;
use veecle_os_runtime::{InitializedReader, Never, Storable};

use crate::{Connector, SendPolicy};

/// An actor that will take any values of type `T` written by other actors and send them out via
/// the provided [`Connector`].
///
/// # Send Policy
///
/// The behavior when the IPC output channel is full can be controlled via [`SendPolicy`]:
///
/// - **Default behavior** ([`SendPolicy::Panic`]): Panics if the channel is full, making
///   buffer exhaustion immediately visible during testing.
///
/// - **Drop behavior** ([`SendPolicy::Drop`]): Messages are dropped with a warning if the
///   channel is full. Use this for non-critical data like telemetry.
///
/// # Examples
///
/// ```no_run
/// # use serde::{Serialize, Deserialize};
/// # use veecle_os_runtime::Storable;
/// # #[derive(Debug, Storable, Serialize, Deserialize)]
/// # struct CriticalData;
/// # #[derive(Debug, Storable, Serialize, Deserialize)]
/// # struct TelemetryData;
/// # async fn example() {
/// # let connector: &'static veecle_ipc::Connector = todo!();
/// use veecle_ipc::SendPolicy;
///
/// veecle_os::runtime::execute! {
///     store: [CriticalData, TelemetryData],
///     actors: [
///         // Default: panics on full buffer (for testing/critical paths)
///         veecle_ipc::Output::<CriticalData>: connector.into(),
///         // Explicitly drop telemetry when buffer is full
///         veecle_ipc::Output::<TelemetryData>: (connector, SendPolicy::Drop).into(),
///     ],
/// }
/// # .await;
/// # }
/// ```
#[veecle_os_runtime::actor]
pub async fn output<T>(
    #[init_context] config: OutputConfig<'_>,
    mut reader: InitializedReader<'_, T>,
) -> Never
where
    T: Storable<DataType: Serialize> + 'static,
{
    let output = config.connector.storable_output();
    let send_policy = config.send_policy;

    loop {
        let value = reader
            .wait_for_update()
            .await
            .read(|value| EncodedStorable::new(value).unwrap());

        match send_policy {
            SendPolicy::Drop => {
                if let Err(error) = output.try_send(value) {
                    veecle_telemetry::warn!(
                        "dropped IPC message due to full channel",
                        type_name = std::any::type_name::<T>(),
                        error = format!("{error:?}")
                    );
                }
            }
            SendPolicy::Panic => {
                output.try_send(value).expect("IPC output channel is full");
            }
        }
    }
}

/// Configuration for the [`Output`] actor.
#[derive(Debug, Clone, Copy)]
pub struct OutputConfig<'a> {
    connector: &'a Connector,
    send_policy: SendPolicy,
}

impl<'a> OutputConfig<'a> {
    /// Creates a new output configuration.
    pub fn new(connector: &'a Connector, send_policy: SendPolicy) -> Self {
        Self {
            connector,
            send_policy,
        }
    }
}

impl<'a> From<&'a Connector> for OutputConfig<'a> {
    fn from(connector: &'a Connector) -> Self {
        Self {
            connector,
            send_policy: SendPolicy::default(),
        }
    }
}

impl<'a> From<(&'a Connector, SendPolicy)> for OutputConfig<'a> {
    fn from((connector, send_policy): (&'a Connector, SendPolicy)) -> Self {
        Self {
            connector,
            send_policy,
        }
    }
}
