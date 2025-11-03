use core::convert::Infallible;

use serde::Serialize;
use veecle_ipc_protocol::EncodedStorable;
use veecle_os_runtime::{InitializedReader, Storable};

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
    #[init_context] config: OutputConfig,
    mut reader: InitializedReader<'_, T>,
) -> Infallible
where
    T: Storable<DataType: Serialize> + 'static,
{
    let output = config.connector.output();
    let policy = config.policy;

    loop {
        let value = reader.wait_for_update().await.read(|value| {
            veecle_ipc_protocol::Message::Storable(EncodedStorable::new(value).unwrap())
        });

        match policy {
            SendPolicy::Drop => {
                if let Err(error) = output.try_send(value) {
                    veecle_telemetry::warn!(
                        "dropped ipc message due to full channel",
                        type_name = std::any::type_name::<T>(),
                        error = format!("{error:?}")
                    );
                }
            }
            SendPolicy::Panic => {
                output
                    .try_send(value)
                    .expect("IPC output channel is full - this indicates buffer exhaustion");
            }
        }
    }
}

/// Configuration for the [`Output`] actor.
#[derive(Debug, Clone, Copy)]
pub struct OutputConfig {
    connector: &'static Connector,
    policy: SendPolicy,
}

impl OutputConfig {
    /// Creates a new output configuration.
    pub fn new(connector: &'static Connector, policy: SendPolicy) -> Self {
        Self { connector, policy }
    }
}

impl From<&'static Connector> for OutputConfig {
    fn from(connector: &'static Connector) -> Self {
        Self {
            connector,
            policy: SendPolicy::default(),
        }
    }
}

impl From<(&'static Connector, SendPolicy)> for OutputConfig {
    fn from((connector, policy): (&'static Connector, SendPolicy)) -> Self {
        Self { connector, policy }
    }
}
