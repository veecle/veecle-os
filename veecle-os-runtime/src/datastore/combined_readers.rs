use core::future::{Future, poll_fn};
use core::pin::pin;
use core::task::Poll;

/// Allows combining (nearly) arbitrary amounts of [`Reader`]s, [`ExclusiveReader`]s or [`InitializedReader`]s.
///
/// [`ExclusiveReader`]: super::ExclusiveReader
/// [`InitializedReader`]: super::InitializedReader
/// [`Reader`]: super::Reader
pub trait CombineReaders {
    /// The (tuple) value that will be read from the combined readers.
    type ToBeRead<'b>;

    /// Reads a tuple of values from all combined readers in the provided function.
    fn read<U>(&mut self, f: impl FnOnce(Self::ToBeRead<'_>) -> U) -> U;

    /// Observes the combined readers for updates.
    ///
    /// Will return if **any** of the readers is updated.
    ///
    /// This returns `&mut Self` to allow chaining a call to [`read`][Self::read].
    #[allow(async_fn_in_trait)]
    async fn wait_for_update(&mut self) -> &mut Self;

    /// Returns `true` if **any** of the readers was updated.
    fn is_updated(&self) -> bool;
}

pub(super) trait Sealed {}

#[allow(private_bounds)]
/// A marker trait for types that can be used with [`CombineReaders`], see that for more details.
pub trait CombinableReader: Sealed {
    /// The (owned) type that this type reads, will be exposed as a reference in the [`CombineReaders::read`] callback.
    type ToBeRead: 'static;

    /// Internal implementation details.
    ///
    /// Borrows the value of the reader from the slot's internal [`RefCell`][core::cell::RefCell].
    #[doc(hidden)]
    fn borrow(&mut self) -> core::cell::Ref<'_, Self::ToBeRead>;

    /// Internal implementation details.
    ///
    /// See [`Reader::wait_for_update`] for more.
    ///
    /// [`Reader::wait_for_update`]: super::Reader::wait_for_update
    #[doc(hidden)]
    #[allow(async_fn_in_trait)]
    async fn wait_for_update(&mut self) -> &mut Self;

    /// Internal implementation details.
    ///
    /// See [`Reader::is_updated`] for more.
    ///
    /// [`Reader::is_updated`]: super::Reader::is_updated
    fn is_updated(&self) -> bool;
}

/// Implements [`CombineReaders`] for provided types for the various reader types.
macro_rules! impl_combined_reader_helper {
    (
        tuples: [
            $(($($generic_type:ident)*),)*
        ],
    ) => {
        $(
            impl<$($generic_type,)*> CombineReaders for ( $( &mut $generic_type, )* )
            where
                $($generic_type: CombinableReader,)*
            {
                type ToBeRead<'x> = (
                    $(&'x <$generic_type as CombinableReader>::ToBeRead,)*
                );

                #[allow(non_snake_case)]
                #[veecle_telemetry::instrument]
                fn read<A>(&mut self, f: impl FnOnce(Self::ToBeRead<'_>) -> A) -> A {
                    let ($($generic_type,)*) = self;
                    let ($($generic_type,)*) = ($({
                        $generic_type.borrow()
                    },)*);
                    f(($(&*$generic_type,)*))
                }

                #[allow(non_snake_case)]
                #[veecle_telemetry::instrument]
                async fn wait_for_update(&mut self) -> &mut Self {
                    {
                        let ($($generic_type,)*) = self;
                        let ($(mut $generic_type,)*) = ($(pin!($generic_type.wait_for_update()),)*);
                        poll_fn(move |cx| {
                            // We check every reader to increment the generation for every reader.
                            let mut update_available = false;
                            $(
                                if $generic_type.as_mut().poll(cx).is_ready() {
                                    update_available = true;
                                }
                            )*
                            if update_available {
                                Poll::Ready(())
                            } else {
                                Poll::Pending
                            }
                        }).await;
                    }
                    self
                }

                #[allow(non_snake_case)]
                #[veecle_telemetry::instrument]
                fn is_updated(&self) -> bool {
                    let ($($generic_type,)*) = self;
                    let result = $({
                        $generic_type.is_updated()
                    })||*;
                    result
                }
            }
        )*
    };
}

impl_combined_reader_helper!(
    tuples: [
        // We don't implement this for a tuple with only one type, as that is just a reader.
        (T U),
        (T U V),
        (T U V W),
        (T U V W X),
        (T U V W X Y),
        (T U V W X Y Z),
    ],
);

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::pin::pin;
    use futures::FutureExt;

    use crate::datastore::{
        CombineReaders, ExclusiveReader, Reader, Slot, Storable, Writer, generational,
    };

    #[test]
    fn read_exclusive_reader() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor0(u8);
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor1(u8);

        let slot0 = pin!(Slot::<Sensor0>::new());
        let slot1 = pin!(Slot::<Sensor1>::new());

        let mut reader0 = ExclusiveReader::from_slot(slot0.as_ref());
        let mut reader1 = ExclusiveReader::from_slot(slot1.as_ref());

        (&mut reader0, &mut reader1).read(|(a, b)| assert_eq!(a.is_none(), b.is_none()));
    }

    #[test]
    fn wait_for_update_exclusive_reader() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor0(u8);
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor1(u8);

        let source = pin!(generational::Source::new());
        let slot0 = pin!(Slot::<Sensor0>::new());
        let slot1 = pin!(Slot::<Sensor1>::new());

        let mut writer0 = Writer::new(source.as_ref().waiter(), slot0.as_ref());
        let mut writer1 = Writer::new(source.as_ref().waiter(), slot1.as_ref());
        let mut reader0 = ExclusiveReader::from_slot(slot0.as_ref());
        let mut reader1 = ExclusiveReader::from_slot(slot1.as_ref());

        assert!(
            (&mut reader0, &mut reader1)
                .wait_for_update()
                .now_or_never()
                .is_none()
        );

        source.as_ref().increment_generation();
        writer0.write(Sensor0(2)).now_or_never().unwrap();
        writer1.write(Sensor1(2)).now_or_never().unwrap();

        assert!(
            (&mut reader0, &mut reader1)
                .wait_for_update()
                .now_or_never()
                .is_some()
        );
    }

    #[test]
    fn read() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor0(u8);
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor1(u8);

        let source = pin!(generational::Source::new());
        let slot0 = pin!(Slot::<Sensor0>::new());
        let slot1 = pin!(Slot::<Sensor1>::new());

        let mut reader0 = Reader::from_slot(slot0.as_ref());
        let mut reader1 = Reader::from_slot(slot1.as_ref());

        (&mut reader0, &mut reader1).read(|(a, b)| assert_eq!(a.is_none(), b.is_none()));

        let mut writer0 = Writer::new(source.as_ref().waiter(), slot0.as_ref());
        let mut writer1 = Writer::new(source.as_ref().waiter(), slot1.as_ref());
        source.as_ref().increment_generation();
        writer0.write(Sensor0(2)).now_or_never().unwrap();
        writer1.write(Sensor1(2)).now_or_never().unwrap();

        let mut reader0 = reader0.wait_init().now_or_never().unwrap();
        let mut reader1 = reader1.wait_init().now_or_never().unwrap();

        (&mut reader0, &mut reader1).read(|(a, b)| assert_eq!(a.0, b.0));
    }

    #[test]
    fn wait_for_update() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor0(u8);
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor1(u8);

        let source = pin!(generational::Source::new());
        let slot0 = pin!(Slot::<Sensor0>::new());
        let slot1 = pin!(Slot::<Sensor1>::new());

        let mut writer0 = Writer::new(source.as_ref().waiter(), slot0.as_ref());
        let mut writer1 = Writer::new(source.as_ref().waiter(), slot1.as_ref());
        let mut reader0 = Reader::from_slot(slot0.as_ref());
        let mut reader1 = Reader::from_slot(slot1.as_ref());

        assert!(
            (&mut reader0, &mut reader1)
                .wait_for_update()
                .now_or_never()
                .is_none()
        );

        source.as_ref().increment_generation();
        writer0.write(Sensor0(2)).now_or_never().unwrap();
        writer1.write(Sensor1(2)).now_or_never().unwrap();

        assert!(
            (&mut reader0, &mut reader1)
                .wait_for_update()
                .now_or_never()
                .is_some()
        );

        let mut reader0 = reader0.wait_init().now_or_never().unwrap();
        let mut reader1 = reader1.wait_init().now_or_never().unwrap();

        assert!(
            (&mut reader0, &mut reader1)
                .wait_for_update()
                .now_or_never()
                .is_none()
        );

        source.as_ref().increment_generation();
        writer0.write(Sensor0(3)).now_or_never().unwrap();
        writer1.write(Sensor1(3)).now_or_never().unwrap();

        (&mut reader0, &mut reader1)
            .wait_for_update()
            .now_or_never()
            .unwrap();
        assert!((&mut reader0, &mut reader1).is_updated());
    }

    #[test]
    fn read_mixed() {
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor0(u8);
        #[derive(Eq, PartialEq, Debug, Clone, Storable)]
        #[storable(crate = crate)]
        struct Sensor1(u8);

        let source = pin!(generational::Source::new());
        let slot0 = pin!(Slot::<Sensor0>::new());
        let slot1 = pin!(Slot::<Sensor1>::new());

        let mut reader0 = Reader::from_slot(slot0.as_ref());
        let mut reader1 = ExclusiveReader::from_slot(slot1.as_ref());

        (&mut reader0, &mut reader1).read(|(a, b)| assert_eq!(a.is_none(), b.is_none()));

        let mut writer0 = Writer::new(source.as_ref().waiter(), slot0.as_ref());
        let mut writer1 = Writer::new(source.as_ref().waiter(), slot1.as_ref());
        source.as_ref().increment_generation();
        writer0.write(Sensor0(2)).now_or_never().unwrap();
        writer1.write(Sensor1(2)).now_or_never().unwrap();

        let mut reader0 = reader0.wait_init().now_or_never().unwrap();

        (&mut reader0, &mut reader1)
            .read(|(a, b): (&Sensor0, &Option<Sensor1>)| assert_eq!(a.0, b.as_ref().unwrap().0));
    }
}
