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

/// Returns a type that will write the given list of types out comma separated with backtick
/// quoting, or `nothing` if it is empty.
///
/// ```text
/// [] => "nothing"
/// ["A"] => "`A`"
/// ["A", "B"] => "`A`, `B`"
/// ["A", "B", "C"] => "`A`, `B`, `C`"
/// ```
pub(crate) fn format_types(
    types: impl IntoIterator<Item = &'static str>,
) -> impl core::fmt::Display {
    struct Helper<T>(core::cell::RefCell<T>);

    impl<T> core::fmt::Display for Helper<T>
    where
        T: Iterator<Item = &'static str>,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let mut iter = self.0.borrow_mut();
            if let Some(first) = iter.next() {
                f.write_str("`")?;
                f.write_str(first)?;
                f.write_str("`")?;
                for next in &mut *iter {
                    f.write_str(", `")?;
                    f.write_str(next)?;
                    f.write_str("`")?;
                }
            } else {
                f.write_str("nothing")?;
            }
            Ok(())
        }
    }

    Helper(core::cell::RefCell::new(types.into_iter()))
}
