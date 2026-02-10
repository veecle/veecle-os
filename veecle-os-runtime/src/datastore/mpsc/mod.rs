//! MPSC (multiple-producer, single-consumer) slot implementation.

mod reader;
mod slot;
mod writer;

pub use self::reader::Reader;
pub(crate) use self::slot::Slot;
pub use self::writer::Writer;
