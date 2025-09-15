use core::convert::Infallible;

use serde::Serialize;
use veecle_ipc_protocol::EncodedStorable;
use veecle_os_runtime::{InitializedReader, Storable};

use crate::Connector;

/// An actor that will take any values of type `T` written by other actors and send them out via
/// the provided [`Connector`].
#[veecle_os_runtime::actor]
pub async fn output<T>(
    #[init_context] connector: &Connector,
    mut reader: InitializedReader<'_, T>,
) -> Infallible
where
    T: Storable<DataType: Serialize> + 'static,
{
    let output = connector.output();
    loop {
        let value = reader.wait_for_update().await.read(|value| {
            veecle_ipc_protocol::Message::Storable(EncodedStorable::new(value).unwrap())
        });
        output.try_send(value).unwrap();
    }
}
