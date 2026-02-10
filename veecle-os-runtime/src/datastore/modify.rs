use core::fmt::{Debug, Formatter};
use core::ops::{Deref, DerefMut};

/// A wrapper around `&mut Option<T>` that allows inspecting or modifying the value.
///
/// Modifying the value marks it as modified and notifies readers.
pub struct Modify<'a, T> {
    pub(crate) inner: &'a mut Option<T>,
    pub(crate) modified: &'a mut bool,
}

impl<'a, T> Modify<'a, T> {
    /// Creates a new instance.
    pub(crate) fn new(inner: &'a mut Option<T>, modified: &'a mut bool) -> Self {
        Self { inner, modified }
    }
}

impl<'a, T> Debug for Modify<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Modify")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'a, T> Deref for Modify<'a, T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> DerefMut for Modify<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.modified = true;
        self.inner
    }
}
