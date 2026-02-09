mod fake_veecle_os_runtime {
    pub enum Never {}

    pub trait StoreRequest<'a> {}

    pub trait Actor<'a> {
        type StoreRequest: StoreRequest<'a>;
        type InitContext: core::any::Any + 'a;
        type Error;
        type Slots;
        fn new(request: Self::StoreRequest, init_context: Self::InitContext) -> Self;
        fn run(
            self,
        ) -> impl core::future::Future<Output = Result<Never, Self::Error>>;
    }

    impl<'a> StoreRequest<'a> for () {}
    impl<'a, T, U> StoreRequest<'a> for (T, U)
    where
        T: StoreRequest<'a>,
        U: StoreRequest<'a>,
    {
    }

    pub mod __exports {
        pub trait IsActorResult {
            type Error;
            fn into_result(self) -> Result<super::Never, Self::Error>;
        }

        impl IsActorResult for super::Never {
            type Error = super::Never;
            fn into_result(self) -> Result<super::Never, Self::Error> {
                match self {}
            }
        }
        pub trait AppendCons<Other> {
            type Result;
        }

        impl<Other> AppendCons<Other> for Nil {
            type Result = Other;
        }

        impl<T, R, Other> AppendCons<Other> for Cons<T, R>
        where
            R: AppendCons<Other>,
        {
            type Result = Cons<T, R::Result>;
        }

        pub struct Nil;
        pub struct Slot<T>(core::marker::PhantomData<T>);
        pub trait DefinesSlot {
            type Slot;
        }

        pub struct Cons<T, U>(pub T, pub U);
    }

    #[derive(Debug)]
    pub struct Reader<'a, T>(core::marker::PhantomData<(&'a (), fn(&T))>);

    #[derive(Debug)]
    pub struct ExclusiveReader<'a, T>(core::marker::PhantomData<(&'a (), fn(&T))>);

    #[derive(Debug)]
    pub struct Writer<'a, T>(core::marker::PhantomData<(&'a (), fn(T))>);

    impl<'a, T> StoreRequest<'a> for Reader<'a, T> {}
    impl<'a, T> StoreRequest<'a> for ExclusiveReader<'a, T> {}
    impl<'a, T> StoreRequest<'a> for Writer<'a, T> {}

    impl<'a, T> __exports::DefinesSlot for Writer<'a, T> {
        type Slot = __exports::Cons<__exports::Slot<T>, __exports::Nil>;
    }

    impl<'a, T> __exports::DefinesSlot for Reader<'a, T> {
        type Slot = __exports::Nil;
    }

    impl<'a, T> __exports::DefinesSlot for ExclusiveReader<'a, T> {
        type Slot = __exports::Nil;
    }

    pub mod single_writer {
        pub use super::{ExclusiveReader, Reader, Writer};
    }

    pub fn assert_right_actor_trait<'a, T>()
    where
        T: self::Actor<'a>,
    {
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor(crate = self::fake_veecle_os_runtime)]
async fn macro_test_actor(
    _sensor_reader: fake_veecle_os_runtime::single_writer::Reader<'_, Sensor>,
    _sensor_reader_excl: fake_veecle_os_runtime::single_writer::ExclusiveReader<'_, Sensor>,
    _sensor_writer: fake_veecle_os_runtime::single_writer::Writer<'_, Sensor>,
) -> fake_veecle_os_runtime::Never {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    fake_veecle_os_runtime::assert_right_actor_trait::<MacroTestActor>();
}
