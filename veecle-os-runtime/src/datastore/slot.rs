//! Traits and helpers for slot implementations.
use core::any::TypeId;

/// Marker trait for all slot types.
///
/// This trait must be implemented by any type that can be used as a slot in the datastore.
pub(crate) trait SlotTrait: Sized + 'static + core::any::Any {
    /// Creates a new empty slot.
    fn new() -> Self;

    /// Returns the TypeId of the data type stored in this slot.
    fn data_type_id() -> TypeId;

    /// Returns the type name of the data type stored in this slot.
    fn data_type_name() -> &'static str;

    /// Validates that this slot type meets its requirements given the access patterns.
    ///
    /// The defining reader/writer amount cannot be zero because a slot is only created for types
    /// that are used by at least one defining reader/writer (see [`DefinesSlot`]).
    ///
    /// # Panics
    ///
    /// Panics with an appropriate error message if validation fails.
    fn validate_access_pattern(
        writers: (usize, impl Iterator<Item = &'static str>),
        exclusive_readers: (usize, impl Iterator<Item = &'static str>),
        non_exclusive_readers: (usize, impl Iterator<Item = &'static str>),
    );
}

/// Determines whether a reader/writer defines a slot.
///
/// A slot is defined by one side of a reader/writer pair.
/// For example, with [`single_writer::Writer`] and [`single_writer::Reader`], the writer defines
/// the slot since it is the unique side.
/// As a consequence, every reader/writer combination must have a side that defines the slot.
///
/// [`single_writer::Reader`]: crate::single_writer::Reader
/// [`single_writer::Writer`]: crate::single_writer::Writer
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "invalid actor parameter type",
    label = "the function signature contains parameters that are neither init_context nor reader/writers",
    note = "only the init_context and readers/writers provided by the Veecle OS runtime may be used as actor parameters",
    note = "parameters passed as initialization context need to be marked with `#[init_context]`"
)]
pub trait DefinesSlot {
    /// The slot cons list for this store request type.
    ///
    /// Returns [`Nil`] for readers or `Cons<Slot<T>, Nil>` for writers.
    ///
    /// [`Nil`]: crate::cons::Nil
    type Slot;
}

/// Returns a type that will write the given list of types out comma separated with backtick
/// quoting, or `nothing` if it is empty.
///
/// Useful in [`SlotTrait::validate_access_pattern`].
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
