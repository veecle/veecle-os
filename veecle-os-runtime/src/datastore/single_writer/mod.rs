//! Single-writer slot implementation.
//!
//! This module provides a slot implementation where exactly one writer
//! can write to a slot, and multiple readers can read from it.

mod exclusive_reader;
mod reader;
mod slot;
mod waiter;
mod writer;

pub use self::exclusive_reader::ExclusiveReader;
pub use self::reader::Reader;
pub(crate) use self::slot::Slot;
pub use self::writer::Writer;
