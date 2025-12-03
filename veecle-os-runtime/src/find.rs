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

#[pin_project::pin_project]
pub struct Wrapper<'a, X>
where
    X: Find,
{
    pub source: Pin<&'a generational::Source>,
    pub inner: X,
}

pub fn make_source() -> Source {
    Source::new()
}

pub fn make_slot<T>() -> Slot<T>
where
    T: Storable,
{
    Slot::new()
}

impl<'a, X> NewDatastore for Wrapper<'a, X>
where
    X: Find,
{
    fn source(&self) -> Pin<&Source> {
        self.source
    }

    fn slot<T>(&self) -> Pin<&Slot<T>>
    where
        T: Storable + 'static,
    {
        self.inner.find().unwrap()
    }
}

trait NewDatastore {
    fn source(&self) -> Pin<&generational::Source>;

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
        std::println!("{}", core::any::type_name_of_val(&tuple));

        $handler(tuple)
    };
}

pub use create_locals;

#[cfg(test)]
mod tests {
    use crate::Storable;
    use crate::find::Find;
    use crate::find::Wrapper;

    #[test]
    fn foo() {
        pub struct Foo {
            _x: u32,
        };
        impl Storable for Foo {
            type DataType = u32;
        }

        fn x(x: impl Find) {
            let foo = core::pin::pin!(crate::datastore::generational::Source::new());
            let wrapper = Wrapper {
                source: foo.as_ref(),
                inner: x,
            };
        }
        create_locals!(x, Foo, Foo, Foo);
    }
}
