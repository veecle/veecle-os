use core::convert::Infallible;
use core::fmt::Debug;
use iceoryx2::prelude::ZeroCopySend;
use veecle_os_runtime::{Reader, Storable};

use super::super::Connector;

/// An actor that will take any values of type `T` written by other actors and send them out via
/// the provided [`Connector`].
#[veecle_os_runtime::actor]
pub async fn output<T>(#[init_context] connector: &Connector, reader: Reader<'_, T>) -> Infallible
where
    T: Storable<DataType: Debug + ZeroCopySend + Clone> + 'static,
{
    let service = connector.storable(std::any::type_name::<T>());
    let publisher = service.publisher_builder().create().unwrap();
    let mut reader = reader.wait_init().await;
    loop {
        reader.wait_for_update().await.read(|value| {
            let sample = publisher.loan_uninit().unwrap();
            let sample = sample.write_payload(value.clone());
            sample.send().unwrap();
        });
    }
}
