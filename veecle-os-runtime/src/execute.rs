#![expect(
    private_bounds,
    private_interfaces,
    reason = "
        everything defined in here except the macro are internal helpers,
        they often mention private types
    "
)]

use crate::actor::{Actor, DatastoreExt, StoreRequest};
use crate::cons::{Cons, Nil, TupleConsToCons};
use crate::datastore::{
    ExclusiveReader, InitializedReader, Reader, Slot, Storable, Writer, generational,
};
use crate::find::{Find, NewDatastore, create_locals};
use core::any::TypeId;
use core::pin::Pin;

/// Internal helper to implement [`Datastore::slot`] recursively for a cons-list of slots.
trait Slots {
    /// See [`Datastore::slot`].
    fn slot<T>(self: Pin<&Self>) -> Pin<&Slot<T>>
    where
        T: Storable + 'static;

    /// Returns the [`TypeId`] and type names for all the slots stored in this type.
    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)>;
}

impl Slots for Nil {
    fn slot<T>(self: Pin<&Self>) -> Pin<&Slot<T>>
    where
        T: Storable + 'static,
    {
        panic!("no slot available for `{}`", core::any::type_name::<T>())
    }

    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)> {
        core::iter::empty()
    }
}

impl<U, R> Slots for Cons<Slot<U>, R>
where
    U: Storable + 'static,
    R: Slots,
{
    fn slot<T>(self: Pin<&Self>) -> Pin<&Slot<T>>
    where
        T: Storable + 'static,
    {
        let this = self.project_ref();
        if TypeId::of::<U>() == TypeId::of::<T>() {
            this.0.assert_is_type()
        } else {
            this.1.slot::<T>()
        }
    }

    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)> {
        R::all_slots().chain(core::iter::once((
            TypeId::of::<U>(),
            core::any::type_name::<U>(),
        )))
    }
}

/// Internal helper to take a cons-list of `Storable` types and return a cons-list of slots for them.
trait IntoSlots {
    /// A cons-list that contains a slot for every type in this cons-list.
    type Slots: Slots;

    /// Creates a new instance of the slot cons-list with all slots empty.
    fn make_slots() -> Self::Slots;
}

impl IntoSlots for Nil {
    type Slots = Nil;

    fn make_slots() -> Self::Slots {
        Nil
    }
}

impl<T, R> IntoSlots for Cons<T, R>
where
    T: Storable + 'static,
    R: IntoSlots,
{
    type Slots = Cons<Slot<T>, R::Slots>;

    fn make_slots() -> Self::Slots {
        Cons(Slot::<T>::new(), R::make_slots())
    }
}

#[allow(
    rustdoc::private_intra_doc_links,
    reason = "
        This lint is hit when documenting with `--document-private-items`.
        If we use `expect`, a warning is emitted when not using `--document-private-items`.
        If we remove the lint, a warning is emitted when using `--document-private-items`.
        To be able to deny warning, we need to allow the lint here.
        https://github.com/rust-lang/rust/issues/145449
    "
)]
// /// Given a slot cons-list, combines it with a [`generational::Source`] to implement [`Datastore`].
// impl<S: Slots> Datastore for Cons<generational::Source, S>
// where
//     S: Slots,
// {
//     fn source(self: Pin<&Self>) -> Pin<&generational::Source> {
//         let this = self.project_ref();
//         this.0
//     }
//
//     fn slot<T>(self: Pin<&Self>) -> Pin<&Slot<T>>
//     where
//         T: Storable + 'static,
//     {
//         let this = self.project_ref();
//         this.1.slot::<T>()
//     }
// }

// /// Given a cons-list of [`Storable`] types, returns a complete [`Datastore`] that contains a slot for each type.
// pub fn make_store<T>() -> impl Datastore
// where
//     T: IntoSlots,
// {
//     Cons(generational::Source::new(), T::make_slots())
// }

/// Internal helper to query how a [`StoreRequest`] type will use a specific type.
pub trait AccessKind {
    /// Returns whether this is a writer.
    fn writer(_type_id: TypeId) -> bool {
        false
    }

    /// Returns whether this is a reader (both exclusive and non-exclusive).
    fn reader(_type_id: TypeId) -> bool {
        false
    }

    /// Returns whether this is an exclusive reader.
    fn exclusive_reader(_type_id: TypeId) -> bool {
        false
    }
}

impl<T> AccessKind for Writer<'_, T>
where
    T: Storable + 'static,
{
    fn writer(type_id: TypeId) -> bool {
        type_id == TypeId::of::<T>()
    }
}

impl<T> AccessKind for Reader<'_, T>
where
    T: Storable + 'static,
{
    fn reader(type_id: TypeId) -> bool {
        type_id == TypeId::of::<T>()
    }
}

impl<T> AccessKind for InitializedReader<'_, T>
where
    T: Storable + 'static,
{
    fn reader(type_id: TypeId) -> bool {
        type_id == TypeId::of::<T>()
    }
}

impl<T> AccessKind for ExclusiveReader<'_, T>
where
    T: Storable + 'static,
{
    fn reader(type_id: TypeId) -> bool {
        type_id == TypeId::of::<T>()
    }

    fn exclusive_reader(type_id: TypeId) -> bool {
        type_id == TypeId::of::<T>()
    }
}

/// Internal helper to query how a cons-lists of [`StoreRequest`] types will use a specific type.
pub trait AccessCount {
    /// Returns how many writers for the given type exist in this list.
    fn writers(type_id: TypeId) -> usize;

    /// Returns how many readers for the given type exist in this list (both exclusive and non-exclusive).
    fn readers(type_id: TypeId) -> usize;

    /// Returns how many exclusive readers for the given type exist in this list.
    fn exclusive_readers(type_id: TypeId) -> usize;
}

impl AccessCount for Nil {
    fn writers(_type_id: TypeId) -> usize {
        0
    }

    fn readers(_type_id: TypeId) -> usize {
        0
    }

    fn exclusive_readers(_type_id: TypeId) -> usize {
        0
    }
}

impl<T, U> AccessCount for Cons<T, U>
where
    T: AccessKind,
    U: AccessCount,
{
    fn writers(type_id: TypeId) -> usize {
        (if T::writer(type_id) { 1 } else { 0 }) + U::writers(type_id)
    }

    fn readers(type_id: TypeId) -> usize {
        (if T::reader(type_id) { 1 } else { 0 }) + U::readers(type_id)
    }

    fn exclusive_readers(type_id: TypeId) -> usize {
        (if T::exclusive_reader(type_id) { 1 } else { 0 }) + U::exclusive_readers(type_id)
    }
}

/// Internal helper to query how a cons-list of cons-lists of [`StoreRequest`] types will use a specific type.
pub trait NestedAccessCount {
    /// Returns how many writers for the given type exist in this list of lists.
    fn writers(type_id: TypeId) -> usize;

    /// Returns how many readers for the given type exist in this list of lists (both exclusive and
    /// non-exclusive).
    fn readers(type_id: TypeId) -> usize;

    /// Returns how many exclusive readers for the given type exist in this list of lists.
    fn exclusive_readers(type_id: TypeId) -> usize;
}

impl NestedAccessCount for Nil {
    fn writers(_type_id: TypeId) -> usize {
        0
    }

    fn readers(_type_id: TypeId) -> usize {
        0
    }

    fn exclusive_readers(_type_id: TypeId) -> usize {
        0
    }
}

impl<T, U> NestedAccessCount for Cons<T, U>
where
    T: AccessCount,
    U: NestedAccessCount,
{
    fn writers(type_id: TypeId) -> usize {
        T::writers(type_id) + U::writers(type_id)
    }

    fn readers(type_id: TypeId) -> usize {
        T::readers(type_id) + U::readers(type_id)
    }

    fn exclusive_readers(type_id: TypeId) -> usize {
        T::exclusive_readers(type_id) + U::exclusive_readers(type_id)
    }
}

/// Internal helper to access details about a cons-list of actors so they can be validated against a store.
pub trait ActorList<'a> {
    /// A cons-list-of-cons-list-of-store-requests for this cons-list (essentially `self.map(|actor| actor.store_request)`
    /// where each actor has a cons-list of store-requests).
    type StoreRequests: NestedAccessCount;

    /// A cons-list of init-contexts for this cons-list (essentially `self.map(|actor| actor.init_context)`).
    type InitContexts;
}

impl ActorList<'_> for Nil {
    type StoreRequests = Nil;
    type InitContexts = Nil;
}

impl<'a, T, U> ActorList<'a> for Cons<T, U>
where
    T: Actor<'a, StoreRequest: TupleConsToCons>,
    U: ActorList<'a>,
    <<T as Actor<'a>>::StoreRequest as TupleConsToCons>::Cons: AccessCount,
{
    /// `Actor::StoreRequest` for the `#[actor]` generated types is a tuple-cons-list, for each actor in this list
    /// convert its store requests into our nominal cons-list.
    ///
    /// This doesn't work with manual `Actor` implementations that have non-tuple-cons-list `StoreRequest`s.
    type StoreRequests = Cons<
        <<T as Actor<'a>>::StoreRequest as TupleConsToCons>::Cons,
        <U as ActorList<'a>>::StoreRequests,
    >;

    /// For `Actor::InitContext` we just need to map directly to the associated type.
    type InitContexts = Cons<<T as Actor<'a>>::InitContext, <U as ActorList<'a>>::InitContexts>;
}

/// Internal helper that for given sets of actors and slots validates the guarantees around slot access that we want to
/// always uphold.
///
/// `init_contexts` is a cons-list of the init-context values for the actors in `A`. It is required to be passed here to
/// drive type-inference for `A` but then just returned.
///
/// `_store` is the reference to the store the actors will use. A copy is passed in here as the lifetime of this
/// reference may be required for the init-contexts inference.
pub fn validate_actors<'a, A, S, I>(init_contexts: I) -> I
where
    A: ActorList<'a, InitContexts = I>,
    S: IntoSlots,
{
    for (type_id, type_name) in S::Slots::all_slots() {
        assert!(
            A::StoreRequests::writers(type_id) > 0,
            "missing writer for `{type_name}`",
        );
        assert!(
            A::StoreRequests::readers(type_id) > 0,
            "missing reader for `{type_name}`",
        );
        assert!(
            A::StoreRequests::writers(type_id) == 1,
            "multiple writers for `{type_name}`",
        );
        if A::StoreRequests::exclusive_readers(type_id) > 0 {
            assert!(
                A::StoreRequests::readers(type_id) == 1,
                "conflict with exclusive reader for `{type_name}`",
            );
        }
    }

    init_contexts
}

/// Internal helper to get a full future that initializes and executes an [`Actor`] given a [`Datastore`]
pub async fn execute_actor<'a, A>(
    store: &'a (impl NewDatastore + DatastoreExt<'a>),
    init_context: A::InitContext,
) -> core::convert::Infallible
where
    A: Actor<'a>,
{
    async move {
        match A::new(A::StoreRequest::request(store).await, init_context)
            .run()
            .await
        {
            Err(error) => panic!("{error}"),
        }
    }
    .await
}

/// Execute a given set of actors without heap allocation.
///
/// ```rust
/// use core::convert::Infallible;
/// use core::fmt::Debug;
///
/// use veecle_os_runtime::{Reader, Storable, Writer};
///
/// #[derive(Debug, Clone, PartialEq, Eq, Default, Storable)]
/// pub struct Ping {
///     value: u32,
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Default, Storable)]
/// pub struct Pong {
///     value: u32,
/// }
///
/// #[veecle_os_runtime::actor]
/// async fn ping_actor(mut ping: Writer<'_, Ping>, pong: Reader<'_, Pong>) -> Infallible {
///     let mut value = 0;
///     ping.write(Ping { value }).await;
///
///     let mut pong = pong.wait_init().await;
///     loop {
///         ping.write(Ping { value }).await;
///         value += 1;
///
///         pong.wait_for_update().await.read(|pong| {
///             println!("Pong: {}", pong.value);
///         });
/// #       // Exit the application to allow doc-tests to complete.
/// #       std::process::exit(0);
///     }
/// }
///
/// #[veecle_os_runtime::actor]
/// async fn pong_actor(mut pong: Writer<'_, Pong>, ping: Reader<'_, Ping>) -> Infallible {
///     let mut ping = ping.wait_init().await;
///     loop {
///         let ping = ping.wait_for_update().await.read_cloned();
///         println!("Ping: {}", ping.value);
///
///         let data = Pong { value: ping.value };
///         pong.write(data).await;
///     }
/// }
///
/// futures::executor::block_on(
///    veecle_os_runtime::execute! {
///        store: [Ping, Pong],
///        actors: [PingActor, PongActor],
///    }
/// )
#[macro_export]
macro_rules! execute {
    (
        store: [
            $($data_type:ty),* $(,)?
        ],
        actors: [
            $($actor_type:ty $(: $init_context:expr )? ),* $(,)?
        ] $(,)?
    ) => {{
        async {

            async fn handler_fn<'a>(store: &'a (impl $crate::find::NewDatastore + $crate::__exports::DatastoreExt<'a>)) -> core::convert::Infallible{
                let init_contexts = $crate::__exports::validate_actors::<
                    $crate::__make_cons!(@type $($actor_type,)*),
                    $crate::__make_cons!(@type $($data_type,)*),
                    _,
                >($crate::__make_cons!(@value $(
                    // Wrapper block is used to provide a `()` if no expression is passed.
                    { $($init_context)? },
                )*));

                // To count how many actors there are, we create an array of `()` with the appropriate length.
                const LEN: usize = [$($crate::discard_to_unit!($actor_type),)*].len();

                let futures: [core::pin::Pin<&mut dyn core::future::Future<Output = core::convert::Infallible>>; LEN] =
                    $crate::make_futures! {
                        init_contexts: init_contexts,
                        store: store,
                        actors: [$($actor_type,)*],
                    };

                static SHARED: $crate::__exports::ExecutorShared<LEN>
                    = $crate::__exports::ExecutorShared::new(&SHARED);

                let executor = $crate::__exports::Executor::new(
                    &SHARED,
                    $crate::find::NewDatastore::source(store),
                    futures,
                );

                executor.run().await
            }
            $crate::find::create_locals!(handler_fn, $($data_type),*);
        }
    }};
}

/// Internal helper to construct an array of pinned futures for given actors + init-contexts + store.
///
/// Returns essentially `[Pin<&mut dyn Future<Output = Infallible>; actors.len()]`, but likely needs annotation at the
/// use site to force the unsize coercion.
#[doc(hidden)]
#[macro_export]
macro_rules! make_futures {
    (
        // A cons-list of init-contexts for the passed actors.
        init_contexts: $init_contexts:expr,
        store: $store:expr,
        actors: [
            $($types:ty,)*
        ],
    ) => {
        $crate::make_futures! {
            init_contexts: $init_contexts,
            store: $store,
            done: [],
            todo: [$($types,)*],
            futures: [],
        }
    };

    // When there are no more actors, just return the futures as an array.
    (
        init_contexts: $init_contexts:expr,
        store: $store:expr,
        done: [$($done:ty,)*],
        todo: [],
        futures: [
            $($futures:expr,)*
        ],
    ) => {
        [$($futures,)*]
    };

    // For each actor, add an element to the futures array, using the already done actors as the depth to read from the
    // init-contexts cons-list. Then push this actor onto the done list so that the next actor will read deeper from the
    // init-contexts.
    (
        init_contexts: $init_contexts:expr,
        store: $store:expr,
        done: [$($done:ty,)*],
        todo: [$current:ty, $($todo:ty,)*],
        futures: [
            $($futures:expr,)*
        ],
    ) => {
        $crate::make_futures! {
            init_contexts: $init_contexts,
            store: $store,
            done: [$($done,)* $current,],
            todo: [$($todo,)*],
            futures: [
                $($futures,)*
                core::pin::pin!(
                    $crate::__exports::execute_actor::<$current>(
                        $store,
                        $crate::__read_cons! {
                            from: $init_contexts,
                            depth: [$($done)*],
                        },
                    )
                ),
            ],
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! discard_to_unit {
    ($_:tt) => {
        ()
    };
}
