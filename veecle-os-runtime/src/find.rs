use crate::Storable;
use crate::datastore::generational::Source;
use crate::datastore::{Slot, generational};
use core::any::TypeId;
use core::mem::transmute;
use core::pin::Pin;

pub trait Find<'a> {
    fn find<T>(&self) -> Option<Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static;
    fn find_ref<T>(&self) -> Option<&Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static;
}

impl<'a> Find<'a> for () {
    #[inline(always)]
    fn find<T>(&self) -> Option<Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        None
    }
    #[inline(always)]
    fn find_ref<T>(&self) -> Option<&Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        None
    }
}

impl<'a, X, Y> Find<'a> for (&Pin<&'a Slot<X>>, Y)
where
    X: Storable,
    Y: Find<'a>,
{
    #[inline(always)]
    fn find<T>(&self) -> Option<Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        if TypeId::of::<X>() == TypeId::of::<T>() {
            Some(self.0.assert_is_type())
        } else {
            Find::find(&self.1)
        }
    }

    #[inline(always)]
    fn find_ref<T>(&self) -> Option<&Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        if TypeId::of::<X>() == TypeId::of::<T>() {
            Some(unsafe { transmute(self.0) })
        } else {
            Find::find_ref(&self.1)
        }
    }
}

impl<'a, Y> Find<'a> for ((), Y)
where
    Y: Find<'a>,
{
    #[inline(always)]
    fn find<T>(&self) -> Option<Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        Find::find(&self.1)
    }

    #[inline(always)]
    fn find_ref<T>(&self) -> Option<&Pin<&'a Slot<T>>>
    where
        T: Storable + Sized + 'static,
    {
        Find::find_ref(&self.1)
    }
}

impl<'a, X> NewDatastore<'a> for (Pin<&'a generational::Source>, X)
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
            let a = core::pin::pin!($crate::find::make_slot::<$t>());
            let mut tuple = (&a.as_ref(), tuple);
            if let Some(a) = $crate::find::Find::find_ref::<$t>(&tuple.1){
                let x = tuple.0;
                tuple.0 = a;
            }
        )*


        let source = core::pin::pin!($crate::__exports::Source::new());
        let wrapper = (
             source.as_ref(),
             tuple,
        );

        $handler(wrapper).await
    };
}

pub use create_locals;
