/// Makes a [cons-lists](https://en.wikipedia.org/wiki/Cons#Lists) of types/identifiers to allow the `validation`
/// "function" below to support an arbitrary number of input arguments.
#[doc(hidden)]
#[macro_export]
macro_rules! __make_tuple_cons {
    () => {
        ()
    };

    (mut $first:ident, $($rest:tt)* ) => {
        (mut $first, $crate::__make_tuple_cons!($($rest)*))
    };

    ($first:ty, $($rest:tt)* ) => {
        ($first, $crate::__make_tuple_cons!($($rest)*))
    };
}

/// Execute a test case with a set of actors.
///
/// This macro's syntax mirrors that of `veecle_os::runtime::execute!` with an extra `validation` argument.
/// The argument should be an async closure that runs any needed validations on the actors behaviors.
///
/// Any store lifetimes in the `validation` argument should be replaced with `'a`.
///
/// ```rust
/// use core::convert::Infallible;
/// use veecle_os::runtime::{Reader, Writer, Storable};
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, Storable)]
/// pub struct Data(u32);
///
/// #[derive(Debug, Storable)]
/// pub struct Trigger;
///
/// #[veecle_os::runtime::actor]
/// async fn incrementor(mut writer: Writer<'_, Data>, mut trigger: Reader<'_, Trigger>) -> Infallible {
///     loop {
///         trigger.wait_for_update().await;
///         writer.modify(|data| {
///             *data = Some(data.map_or(Data(0), |data| Data(data.0 + 1)));
///         }).await;
///     }
/// }
///
/// veecle_os_test::block_on_future(
///     veecle_os_test::execute! {
///         store: [Data, Trigger],
///
///         actors: [Incrementor],
///
///         validation: async |mut reader: Reader<'a, Data>, mut trigger: Writer<'a, Trigger>| {
///             trigger.write(Trigger).await;
///             assert_eq!(reader.wait_for_update().await.read_cloned(), Some(Data(0)));
///             trigger.write(Trigger).await;
///             assert_eq!(reader.wait_for_update().await.read_cloned(), Some(Data(1)));
///             trigger.write(Trigger).await;
///             assert_eq!(reader.wait_for_update().await.read_cloned(), Some(Data(2)));
///         },
///     }
/// );
/// ```
#[macro_export]
macro_rules! execute {
    (
        store: [
            $($data_type:ty),* $(,)?
        ],

        actors: [
            $($actor_type:ty $(: $init_context:expr )? ),* $(,)?
        ],

        validation: async |$(mut $arg_pat:ident : $arg_ty:ty),* $(,)?| $validation_body:block $(,)?
    ) => {{
        struct Validator<'a> {
            store_request: $crate::__make_tuple_cons!($($arg_ty,)*),
            complete: $crate::__exports::futures::channel::oneshot::Sender<()>,
            // In case the validation body doesn't use readers/writers, we need to use the lifetime
            // to avoid the compiler complaining about unused lifetimes.
            _phantom: core::marker::PhantomData<&'a ()>,
        }

        impl<'a> $crate::__exports::veecle_os_runtime::Actor<'a> for Validator<'a> {
            type StoreRequest = $crate::__make_tuple_cons!($($arg_ty,)*);
            type InitContext = $crate::__exports::futures::channel::oneshot::Sender<()>;
            type Error = core::convert::Infallible;

            fn new(store_request: Self::StoreRequest, complete: Self::InitContext) -> Self {
                Self {
                    store_request,
                    complete,
                    _phantom: core::marker::PhantomData
                }
            }

            async fn run(self) -> Result<core::convert::Infallible, Self::Error> {
                let $crate::__make_tuple_cons!($(mut $arg_pat,)*) = self.store_request;
                $validation_body;
                self.complete.send(()).unwrap();
                core::future::pending().await
            }
        }

        async {
            let (complete_tx, complete_rx) =
                $crate::__exports::futures::channel::oneshot::channel::<()>();

            let executor = core::pin::pin!(
                $crate::__exports::veecle_os_runtime::execute! {
                    store: [ $($data_type,)* ],

                    actors: [
                        $($actor_type $(: $init_context)? ,)*

                        Validator: complete_tx,
                    ],
                }
            );

            $crate::__exports::futures::future::select(executor, complete_rx).await;
        }
    }};

    // The previous arm doesn't support `validation: async ||` (no space between first `|´ and second ´|´) for some reason.
    // To avoid forcing users to add whitespace between `||`, we add this arm.
    (
        store: [
            $($data_type:ty),* $(,)?
        ],

        actors: [
            $($actor_type:ty $(: $init_context:expr )? ),* $(,)?
        ],

        validation: async || $validation_body:block $(,)?
    ) => {{
        $crate::execute!(
            store: [
            $($data_type),*
        ],

        actors: [
            $($actor_type $(: $init_context)? ),*
        ],

        validation: async | | $validation_body
        )
    }};
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #[veecle_os_runtime::actor]
    async fn contextual_actor<T: core::fmt::Debug>(
        #[init_context] _context: T,
    ) -> core::convert::Infallible {
        std::future::pending().await
    }

    #[test]
    fn local_context() {
        let local = vec![1];
        futures::executor::block_on(crate::execute! {
            store: [],

            actors: [
                ContextualActor<_>: &local,
            ],
                validation: async || {}
        });
        dbg!(&local);
    }
}
