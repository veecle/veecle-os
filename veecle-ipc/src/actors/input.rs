use core::convert::Infallible;

use serde::de::DeserializeOwned;
use veecle_os_runtime::{Storable, Writer};

use crate::Connector;

/// An actor that will receive values of type `T` from the provided [`Connector`] and send them to other actors.
#[veecle_os_runtime::actor]
pub async fn input<T>(
    #[init_context] connector: &Connector,
    mut writer: Writer<'_, T>,
) -> Infallible
where
    T: Storable<DataType: DeserializeOwned> + 'static,
{
    let mut input = connector.storable_input(std::any::type_name::<T>());
    loop {
        let value = input.recv().await.unwrap();
        match serde_json::from_str(&value) {
            Ok(value) => writer.write(value).await,
            Err(error) => {
                let error = anyhow::Error::new(error).context(format!(
                    "invalid ipc input for {}",
                    std::any::type_name::<T>()
                ));
                veecle_telemetry::error!("error", error = format!("{error:?}"));
            }
        }
    }
}
