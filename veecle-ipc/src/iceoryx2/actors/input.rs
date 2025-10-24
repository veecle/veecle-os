use core::convert::Infallible;
use core::fmt::Debug;
use iceoryx2::prelude::ZeroCopySend;
use veecle_os_runtime::{Storable, Writer};
use veecle_osal_std::time::{Duration, Time, TimeAbstraction};
use veecle_telemetry::future::FutureExt;
use veecle_telemetry::span;

use super::super::Connector;

/// An actor that will receive values of type `T` from the provided [`Connector`] and send them to other actors.
#[veecle_os_runtime::actor]
pub async fn input<T>(
    #[init_context] connector: &Connector,
    mut writer: Writer<'_, T>,
) -> Infallible
where
    T: Storable<DataType: Debug + ZeroCopySend + Clone> + 'static,
{
    let service = connector.storable::<T::DataType>(std::any::type_name::<T>());
    let subscriber = service.subscriber_builder().create().unwrap();

    loop {
        while let Some(value) = subscriber.receive().unwrap() {
            writer.write(value.payload().clone()).await;
        }

        // There is no way to register interest for new values, so we just busy-poll with a slight
        // delay.
        Time::sleep(Duration::from_millis(1))
            .with_span(span!("sleep"))
            .await
            .unwrap();
    }
}
