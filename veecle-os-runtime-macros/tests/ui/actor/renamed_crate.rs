mod fake_veecle_os_runtime {
    pub trait StoreRequest<'a> {}

    pub trait Actor<'a> {
        type StoreRequest: StoreRequest<'a>;
        type InitContext: core::any::Any + 'a;
        type Error;
        fn new(request: Self::StoreRequest, init_context: Self::InitContext) -> Self;
        fn run(self) -> impl core::future::Future<Output = Result<core::convert::Infallible, Self::Error>>;
    }

    impl<'a> StoreRequest<'a> for () {}
    impl<'a, T, U> StoreRequest<'a> for (T, U)
    where
        T: StoreRequest<'a>,
        U: StoreRequest<'a>,
    {}

    pub mod __exports {
        pub trait IsActorResult {
            type Error;
            fn into_result(self) -> Result<core::convert::Infallible, Self::Error>;
        }

        impl IsActorResult for core::convert::Infallible {
            type Error = core::convert::Infallible;
            fn into_result(self) -> Result<core::convert::Infallible, Self::Error> {
                match self {}
            }
        }
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

    pub fn assert_right_actor_trait<'a, T>()
    where
        T: self::Actor<'a>,
    {}
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Sensor(pub u8);

#[veecle_os_runtime_macros::actor(crate = self::fake_veecle_os_runtime)]
async fn macro_test_actor(
    _sensor_reader: fake_veecle_os_runtime::Reader<'_, Sensor>,
    _sensor_reader_excl: fake_veecle_os_runtime::ExclusiveReader<'_, Sensor>,
    _sensor_writer: fake_veecle_os_runtime::Writer<'_, Sensor>,
) -> std::convert::Infallible {
    unreachable!("We only care about the code compiling.");
}

fn main() {
    fake_veecle_os_runtime::assert_right_actor_trait::<MacroTestActor>();
}
