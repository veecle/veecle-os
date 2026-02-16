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

use core::pin::Pin;

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
