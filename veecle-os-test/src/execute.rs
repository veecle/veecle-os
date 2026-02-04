/// Execute a test case with a set of actors.
///
/// This macro's syntax mirrors that of `veecle_os::runtime::execute!` with an extra `validation` argument.
/// The argument should be an async closure that runs any needed validations on the actors behaviors.
///
/// Any store lifetimes in the `validation` argument should use `'_` as a placeholder.
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
///         validation: async |mut reader: Reader<'_, Data>, mut trigger: Writer<'_, Trigger>| {
///             trigger.write(Trigger).await;
///             assert_eq!(reader.read_updated_cloned().await, Data(0));
///             trigger.write(Trigger).await;
///             assert_eq!(reader.read_updated_cloned().await, Data(1));
///             trigger.write(Trigger).await;
///             assert_eq!(reader.read_updated_cloned().await, Data(2));
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
        #[$crate::__exports::veecle_os_runtime::actor(crate = $crate::__exports::veecle_os_runtime)]
        async fn veecle_os_test_validator_generated_actor(
            $(mut $arg_pat : $arg_ty,)*
            #[init_context] __complete: $crate::__exports::futures::channel::oneshot::Sender<()>,
        ) -> $crate::__exports::veecle_os_runtime::Never {
            $validation_body;
            __complete.send(()).unwrap();
            core::future::pending().await
        }

        async {
            let (complete_tx, complete_rx) =
                $crate::__exports::futures::channel::oneshot::channel::<()>();

            let executor = core::pin::pin!(
                $crate::__exports::veecle_os_runtime::execute! {
                    actors: [
                        $($actor_type $(: $init_context)? ,)*
                        VeecleOsTestValidatorGeneratedActor: complete_tx,
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
