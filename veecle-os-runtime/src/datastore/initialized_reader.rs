use core::cell::Ref;
use core::marker::PhantomData;

use crate::datastore::{Storable, slot};

/// Reader for a [`Storable`] type.
///
/// Allows [`Actor`]s to read a value of a type written by another actor.
/// The generic type `T` from the reader specifies the type of the value that is being read.
///
/// This reader can be requested directly as an actor input in simple cases, this will mean your actor does not start
/// running until all `InitializedReader`s it takes have been initialized by their writers.
/// If you need to do something more complex (e.g. you have interdependencies between actors so one must write an
/// initial value earlier) then you can take a `Reader` and convert via [`Reader::wait_init`][super::Reader::wait_init]
/// when ready.
/// By ensuring the presence of a value for `T` has been written at least once, this reader avoids `Option` when
/// reading.
///
/// # Example
///
/// ```rust
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, Reader, InitializedReader};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// #[veecle_os_runtime::actor]
/// async fn foo_reader(mut reader: InitializedReader<'_, Foo>) -> veecle_os_runtime::Never {
///     loop {
///         let processed_value = reader.wait_for_update().await.read(|value: &Foo| {
///             // Do something with the value.
///         });
///     }
/// }
///
/// #[veecle_os_runtime::actor]
/// async fn foo_reader_complex(mut reader: Reader<'_, Foo>) -> veecle_os_runtime::Never {
///     // Do some initialization that must be completed before waiting for the reader to have an initial value.
///     let mut reader = reader.wait_init().await;
///     loop {
///         let processed_value = reader.wait_for_update().await.read(|value: &Foo| {
///             // Do something with the value.
///         });
///     }
/// }
/// ```
///
/// [`Actor`]: crate::actor::Actor
#[derive(Debug)]
pub struct InitializedReader<'a, T>
where
    T: Storable + 'static,
{
    waiter: slot::Waiter<'a, T>,

    marker: PhantomData<fn(T)>,
}

impl<T> InitializedReader<'_, T>
where
    T: Storable + 'static,
{
    /// Reads the current value of a type.
    ///
    /// Can be combined with [`Self::wait_for_update`] to wait for the value to be updated before reading it.
    ///
    /// This method takes a closure to ensure the reference is not held across await points.
    #[veecle_telemetry::instrument]
    pub fn read<U>(&self, f: impl FnOnce(&T::DataType) -> U) -> U {
        self.waiter.read(|value| {
            let value = value
                .as_ref()
                .expect("initialized reader should only access initialized values");

            veecle_telemetry::trace!("Slot read", value = format_args!("{value:?}"));
            f(value)
        })
    }

    /// Reads and clones the current value of a type.
    ///
    /// This is a wrapper around [`Self::read`] that additionally clones the value.
    /// You can use it instead of `reader.read(|c| c.clone())`.
    pub fn read_cloned(&self) -> T::DataType
    where
        T::DataType: Clone,
    {
        self.read(|t| t.clone())
    }

    /// Waits for any write to occur.
    ///
    /// This future resolving does not imply that `previous_value != new_value`, just that a
    /// [`Writer`][super::Writer] has written a value of `T` since the last time this future resolved.
    ///
    /// This returns `&mut Self` to allow chaining a call to methods accessing the value, for example
    /// [`read`][Self::read`].
    #[veecle_telemetry::instrument]
    pub async fn wait_for_update(&mut self) -> &mut Self {
        self.waiter.wait().await;
        self.waiter.update_generation();
        self
    }
    /// Returns `true` if a unseen value is available.
    pub fn is_updated(&self) -> bool {
        self.waiter.is_updated()
    }
}

impl<'a, T> InitializedReader<'a, T>
where
    T: Storable + 'static,
{
    /// Creates a new `InitializedReader` from a [`Waiter`][slot::Waiter].
    pub(crate) fn new(waiter: slot::Waiter<'a, T>) -> Self {
        Self {
            waiter,
            marker: Default::default(),
        }
    }
}

impl<T> super::combined_readers::Sealed for InitializedReader<'_, T> where T: Storable {}

impl<T> super::combined_readers::CombinableReader for InitializedReader<'_, T>
where
    T: Storable,
{
    type ToBeRead = T::DataType;
    type ToBeWaitRead = T::DataType;

    fn borrow(&mut self) -> Ref<'_, Self::ToBeRead> {
        self.waiter.update_generation();
        Ref::map(self.waiter.borrow(), |t| t.as_ref().unwrap())
    }

    fn borrow_for_updated(&mut self) -> Ref<'_, Self::ToBeWaitRead> {
        self.waiter.update_generation();
        Ref::map(self.waiter.borrow(), |t| t.as_ref().unwrap())
    }

    async fn wait_for_update(&mut self) {
        self.wait_for_update().await;
    }

    fn is_updated(&self) -> bool {
        self.is_updated()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::pin::pin;
    use futures::FutureExt;

    use crate::datastore::{Reader, Slot, Storable, Writer, generational};

    #[test]
    fn read() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor(u8);

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Sensor>::new());

        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let reader = Reader::from_slot(slot.as_ref());

        assert!(reader.wait_init().now_or_never().is_none());

        source.as_ref().increment_generation();
        writer.write(Sensor(5)).now_or_never().unwrap();

        let reader = Reader::from_slot(slot.as_ref())
            .wait_init()
            .now_or_never()
            .unwrap();

        assert_eq!(reader.read(|x: &Sensor| x.clone()), Sensor(5));
        assert_eq!(reader.read_cloned(), Sensor(5));
    }

    #[test]
    fn wait_for_update() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor(u8);

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Sensor>::new());

        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let reader = Reader::from_slot(slot.as_ref());

        source.as_ref().increment_generation();
        writer.write(Sensor(1)).now_or_never().unwrap();

        let mut reader = reader.wait_init().now_or_never().unwrap();

        assert!(reader.wait_for_update().now_or_never().is_some());
        assert!(reader.wait_for_update().now_or_never().is_none());

        source.as_ref().increment_generation();
        writer.write(Sensor(1)).now_or_never().unwrap();

        reader
            .wait_for_update()
            .now_or_never()
            .unwrap()
            .read(|x| assert_eq!(x, &Sensor(1)));
    }

    #[test]
    fn wait_init_wait_for_update() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor(u8);

        let source = pin!(generational::Source::new());
        let slot = pin!(Slot::<Sensor>::new());

        let mut writer = Writer::new(source.as_ref().waiter(), slot.as_ref());
        let reader = Reader::from_slot(slot.as_ref());

        let mut wait_init_fut = pin!(reader.wait_init());
        assert!(wait_init_fut.as_mut().now_or_never().is_none());
        // Increment generation to allow the writer to write.
        source.as_ref().increment_generation();
        writer.write(Sensor(1)).now_or_never().unwrap();

        let mut reader = wait_init_fut.now_or_never().unwrap();

        // If `wait_init` does not increment the waiter generation, `now_or_never` must return `Some`.
        reader
            .wait_for_update()
            .now_or_never()
            .unwrap()
            .read(|x| assert_eq!(x, &Sensor(1)));
    }
}
