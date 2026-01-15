//! Central communication hub for [`Actor`]s.
//!
//! Actors access data through [`Reader`]s and [`Writer`]s.
//!
//! [`Actor`]:crate::actor::Actor

mod combined_readers;
mod exclusive_reader;
pub(crate) mod generational;
mod initialized_reader;
mod reader;
mod slot;
mod writer;

pub use self::combined_readers::{CombinableReader, CombineReaders};
pub use self::exclusive_reader::ExclusiveReader;
pub use self::initialized_reader::InitializedReader;
pub use self::reader::Reader;
pub use self::slot::Storable;
pub(crate) use self::slot::{Slot, SlotTrait};
pub use self::writer::Writer;
