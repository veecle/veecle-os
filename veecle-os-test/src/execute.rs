/// Computes the `Slots` type for a list of Reader/Writer types.
#[doc(hidden)]
#[macro_export]
macro_rules! __validator_slots {
    () => {
        $crate::__exports::veecle_os_runtime::__exports::Nil
    };
    ($first:ty $(,)?) => {
        <<$first as $crate::__exports::veecle_os_runtime::__exports::DefinesSlot>::Slot as $crate::__exports::veecle_os_runtime::__exports::AppendCons<$crate::__exports::veecle_os_runtime::__exports::Nil>>::Result
    };
    ($first:ty, $($rest:ty),+ $(,)?) => {
        <<$first as $crate::__exports::veecle_os_runtime::__exports::DefinesSlot>::Slot as $crate::__exports::veecle_os_runtime::__exports::AppendCons<$crate::__validator_slots!($($rest),+)>>::Result
    };
}

/// Creates a cons-list type from a list of types.
#[doc(hidden)]
#[macro_export]
macro_rules! __make_cons_ty {
    () => {
        ()
    };
    ($first:ty $(,)?) => {
        ($first, ())
    };
    ($first:ty, $($rest:ty),+ $(,)?) => {
        ($first, $crate::__make_cons_ty!($($rest),+))
    };
}

/// Creates a cons-list pattern from a list of patterns.
#[doc(hidden)]
#[macro_export]
macro_rules! __make_cons_pat {
    () => {
        ()
    };
    ($first:tt $(,)?) => {
        ($first, ())
    };
    ($first:tt, $($rest:tt),+ $(,)?) => {
        ($first, $crate::__make_cons_pat!($($rest),+))
    };
}

/// Execute a test case with a set of actors.
///
/// This macro's syntax mirrors that of `veecle_os::runtime::execute!` with an extra `validation` argument.
/// The argument should be an async closure that runs any needed validations on the actors behaviors.
///
/// Any store lifetimes in the `validation` argument should use `'a` as a placeholder.
///
/// ```rust
/// use veecle_os::runtime::{Never, Reader, Writer, Storable};
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, Storable)]
/// pub struct Data(u32);
///
/// #[derive(Debug, Storable)]
/// pub struct Trigger;
///
/// #[veecle_os::runtime::actor]
/// async fn incrementor(mut writer: Writer<'_, Data>, mut trigger: Reader<'_, Trigger>) -> Never {
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
        actors: [
            $($actor_type:ty $(: $init_context:expr )? ),* $(,)?
        ],

        validation: async |$(mut $arg_pat:ident : $arg_ty:ty),* $(,)?| $validation_body:block $(,)?
    ) => {{
        struct __Validator<'a> {
            $(
                $arg_pat: $arg_ty,
            )*
            complete: $crate::__exports::futures::channel::oneshot::Sender<()>,
            // In case the validation body doesn't use readers/writers, we need to use the lifetime
            // to avoid the compiler complaining about unused lifetimes.
            _phantom: core::marker::PhantomData<&'a ()>,
        }

        impl<'a> $crate::__exports::veecle_os_runtime::Actor<'a> for __Validator<'a> {
            type StoreRequest = $crate::__make_cons_ty!($($arg_ty),*);
            type InitContext = $crate::__exports::futures::channel::oneshot::Sender<()>;
            type Error = $crate::__exports::Never;
            type Slots = $crate::__validator_slots!($($arg_ty),*);

            fn new($crate::__make_cons_pat!($($arg_pat),*): Self::StoreRequest, complete: Self::InitContext) -> Self {
                Self {
                    $($arg_pat,)*
                    complete,
                    _phantom: core::marker::PhantomData,
                }
            }

            async fn run(self) -> Result<$crate::__exports::Never, Self::Error> {
                $(let mut $arg_pat = self.$arg_pat;)*
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
                    actors: [
                        $($actor_type $(: $init_context)? ,)*
                        __Validator: complete_tx,
                    ],
                }
            );

            $crate::__exports::futures::future::select(executor, complete_rx).await;
        }
    }};

    // The previous arm doesn't support `validation: async ||` (no space between first `|´ and second ´|´) for some reason.
    // To avoid forcing users to add whitespace between `||`, we add this arm.
    (
        actors: [
            $($actor_type:ty $(: $init_context:expr )? ),* $(,)?
        ],

        validation: async || $validation_body:block $(,)?
    ) => {{
        $crate::execute!(
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
    ) -> veecle_os_runtime::Never {
        std::future::pending().await
    }

    #[test]
    fn local_context() {
        let local = vec![1];
        futures::executor::block_on(crate::execute! {
            actors: [
                ContextualActor<&Vec<i32>>: &local,
            ],
            validation: async || {}
        });
        dbg!(&local);
    }
}
