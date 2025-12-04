use crate::Storable;
use crate::datastore::generational::Source;
use crate::datastore::{Slot, generational};
use core::any::TypeId;
use core::pin::Pin;

pub trait Find {
    fn find<T>(&self) -> Option<Pin<&Slot<T>>>
    where
        T: Storable + Sized + 'static;
}

impl Find for () {
    #[inline(always)]
    fn find<T>(&self) -> Option<Pin<&Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        None
    }
}

impl<X, Y> Find for (&Pin<&Slot<X>>, Y)
where
    X: Storable,
    Y: Find,
{
    #[inline(always)]
    fn find<T>(&self) -> Option<Pin<&Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        if TypeId::of::<X>() == TypeId::of::<T>() {
            Some(self.0.assert_is_type())
        } else {
            Find::find(&self.1)
        }
    }
}

impl<'a, X> NewDatastore for (Pin<&'a generational::Source>, X)
where
    X: Find,
{
    #[inline(always)]
    fn source(&self) -> Pin<&Source> {
        self.0
    }

    #[inline(always)]
    fn slot<T>(&self) -> Pin<&Slot<T>>
    where
        T: Storable + 'static,
    {
        self.1.find().unwrap()
    }
}

#[inline(always)]
pub fn make_source() -> Source {
    Source::new()
}

#[inline(always)]
pub fn make_slot<T>() -> Slot<T>
where
    T: Storable,
{
    Slot::new()
}

pub trait NewDatastore {
    #[inline(always)]
    fn source(&self) -> Pin<&generational::Source>;

    #[inline(always)]
    fn slot<T>(&self) -> Pin<&Slot<T>>
    where
        T: Storable + 'static;
}

#[macro_export]
macro_rules! create_locals {
    ($handler:tt, $($t:ty),*) => {
        let tuple = ();
        $(
            let a = core::pin::pin!($crate::find::make_slot::<$t>());
            let tuple = (&a.as_ref(), tuple);
        )*

        let source = core::pin::pin!($crate::__exports::Source::new());
        let wrapper = (
             source.as_ref(),
             tuple,
        );

        $handler(&wrapper).await
    };
}

pub use create_locals;

#[cfg(test)]
mod tests {
    use crate::Storable;
    use crate::find::{Find, NewDatastore};
    use std::dbg;

    #[test]
    fn foo() {
        pub struct Foo {
            _x: u32,
        };
        impl Storable for Foo {
            type DataType = u32;
        }

        fn x(x: impl NewDatastore) {
            dbg!(&x.source());
        }
        //create_locals!(x, Foo, Foo, Foo);
    }
}
