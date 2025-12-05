use crate::Storable;
use crate::datastore::generational::Source;
use crate::datastore::{Slot, generational};
use core::any::TypeId;
use core::mem::transmute;
use core::pin::Pin;

pub trait Find<'a> {
    fn find<T>(&self) -> Option<&'a Slot<T>>
    where
        T: Storable + Sized + 'static;
}

impl<'a> Find<'a> for () {
    fn find<T>(&self) -> Option<&'a Slot<T>>
    where
        T: Storable + Sized + 'static,
    {
        None
    }
}

impl<'a, X, Y> Find<'a> for (&'a Slot<X>, Y)
where
    X: Storable,
    Y: Find<'a>,
{
    fn find<T>(&self) -> Option<&'a Slot<T>>
    where
        T: Storable + Sized + 'static,
    {
        if TypeId::of::<X>() == TypeId::of::<T>() {
            Some(unsafe { transmute::<&Slot<X>, &Slot<T>>(self.0) })
        } else {
            Find::find(&self.1)
        }
    }
}

impl<'a, X> NewDatastore<'a> for (Pin<&'a generational::Source>, &X)
where
    X: Find<'a>,
{
    #[inline(always)]
    fn source(self) -> Pin<&'a Source> {
        self.0
    }

    #[inline(always)]
    fn slot<T>(self) -> Pin<&'a Slot<T>>
    where
        T: Storable + 'static,
    {
        unsafe { Pin::new_unchecked(self.1.find().unwrap()) }
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

pub trait NewDatastore<'a> {
    fn source(self) -> Pin<&'a generational::Source>;

    fn slot<T>(self) -> Pin<&'a Slot<T>>
    where
        T: Storable + 'static;
}

#[macro_export]
macro_rules! create_locals {
    ($handler:tt, $($t:ty),*) => {
        let tuple = ();
        $(
            let a = $crate::find::make_slot::<$t>();
            let mut tuple = (&a, tuple);
            if let Some(a) = $crate::find::Find::find::<$t>(&tuple.1){
                tuple.0 = a;
            }
        )*
        let source = core::pin::pin!($crate::__exports::Source::new());

        $handler(source.as_ref(),&tuple).await
    };
}

pub use create_locals;
