//! Central communication hub for [`Actor`]s.
//!
//! Actors access data through readers and writers provided by slot implementations.
//! See the [`single_writer`] module for a slot implementation with one writer and multiple readers.
//!
//! [`Actor`]: crate::actor::Actor

mod combine_readers;
mod modify;
pub mod mpsc;
pub mod single_writer;
mod slot;
mod storable;
mod store_request;
pub(crate) mod sync;

pub use self::combine_readers::{CombinableReader, CombineReaders};
pub use self::modify::Modify;
pub use self::slot::DefinesSlot;
pub(crate) use self::slot::{SlotTrait, format_types};
pub use self::storable::Storable;
pub use self::store_request::StoreRequest;
#[doc(inline)]
pub use veecle_os_runtime_macros::Storable;

use crate::datastore::sync::generational;
use crate::single_writer::{ExclusiveReader, Reader, Slot, Writer};

use core::pin::Pin;

use self::mpsc::Slot as MpscSlot;

#[doc(hidden)]
/// Internal trait to abstract out type-erased and concrete data stores.
pub trait Datastore {
    /// Returns a generational source tracking the global datastore generation.
    ///
    /// This is used to ensure that every reader has had (or will have) a chance to read a value before a writer may
    /// overwrite it.
    fn source(self: Pin<&Self>) -> Pin<&generational::Source>;

    /// Returns a reference to a slot of a specific type.
    ///
    /// # Panics
    ///
    /// If there is no slot of type `S` in the datastore.
    ///
    /// `requestor` will be included in the panic message for context.
    #[expect(private_bounds, reason = "the methods are internal")]
    fn slot<S>(self: Pin<&Self>, requestor: &'static str) -> Pin<&S>
    where
        S: SlotTrait;
}

pub(crate) trait DatastoreExt<'a>: Copy {
    /// Returns the [`Reader`] for a specific slot.
    ///
    /// # Panics
    ///
    /// * If there is no [`Slot`] for `T` in the [`Datastore`].
    ///
    /// `requestor` will be included in the panic message for context.
    fn reader<T>(self, requestor: &'static str) -> Reader<'a, T>
    where
        T: Storable + 'static;

    /// Returns the [`ExclusiveReader`] for a specific slot.
    ///
    /// Exclusivity of the reader is not guaranteed by this method and must be ensured via other means (e.g.
    /// [`crate::execute::make_store_and_validate`]).
    ///
    /// # Panics
    ///
    /// * If there is no [`Slot`] for `T` in the [`Datastore`].
    ///
    /// `requestor` will be included in the panic message for context.
    fn exclusive_reader<T>(self, requestor: &'static str) -> ExclusiveReader<'a, T>
    where
        T: Storable + 'static;

    /// Returns the [`Writer`] for a specific slot.
    ///
    /// # Panics
    ///
    /// * If the [`Writer`] for this slot has already been acquired.
    ///
    /// * If there is no [`Slot`] for `T` in the [`Datastore`].
    ///
    /// `requestor` will be included in the panic message for context.
    fn writer<T>(self, requestor: &'static str) -> Writer<'a, T>
    where
        T: Storable + 'static;

    /// Returns an [`mpsc::Writer`] for a specific mpsc slot.
    ///
    /// # Panics
    ///
    /// * If writer capacity `N` is exceeded.
    ///
    /// * If there is no [`MpscSlot`] for `T` in the [`Datastore`].
    ///
    /// `requestor` will be included in the panic message for context.
    fn mpsc_writer<T, const N: usize>(self, requestor: &'static str) -> mpsc::Writer<'a, T, N>
    where
        T: Storable + 'static;

    /// Returns an [`mpsc::Reader`] for a specific mpsc slot.
    ///
    /// # Panics
    ///
    /// * If there is no [`MpscSlot`] for `T` in the [`Datastore`].
    ///
    /// `requestor` will be included in the panic message for context.
    fn mpsc_reader<T, const N: usize>(self, requestor: &'static str) -> mpsc::Reader<'a, T, N>
    where
        T: Storable + 'static;
}

impl<'a, S> DatastoreExt<'a> for Pin<&'a S>
where
    S: Datastore,
{
    fn reader<T>(self, requestor: &'static str) -> Reader<'a, T>
    where
        T: Storable + 'static,
    {
        Reader::from_slot(self.slot::<Slot<T>>(requestor))
    }

    fn exclusive_reader<T>(self, requestor: &'static str) -> ExclusiveReader<'a, T>
    where
        T: Storable + 'static,
    {
        ExclusiveReader::from_slot(self.slot::<Slot<T>>(requestor))
    }

    fn writer<T>(self, requestor: &'static str) -> Writer<'a, T>
    where
        T: Storable + 'static,
    {
        Writer::new(self.source().waiter(), self.slot::<Slot<T>>(requestor))
    }

    fn mpsc_writer<T, const N: usize>(self, requestor: &'static str) -> mpsc::Writer<'a, T, N>
    where
        T: Storable + 'static,
    {
        mpsc::Writer::new(
            self.source().waiter(),
            self.slot::<MpscSlot<T, N>>(requestor),
        )
    }

    fn mpsc_reader<T, const N: usize>(self, requestor: &'static str) -> mpsc::Reader<'a, T, N>
    where
        T: Storable + 'static,
    {
        mpsc::Reader::from_slot(self.slot::<MpscSlot<T, N>>(requestor))
    }
}
