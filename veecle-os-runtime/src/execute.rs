#![expect(
    private_bounds,
    reason = "
        everything defined in here except the macro are internal helpers,
        they often mention private types
    "
)]

use crate::Never;
use crate::actor::{Actor, Datastore, StoreRequest};
use crate::cons::{Cons, Nil, TupleConsToCons};
use crate::datastore::{
    ExclusiveReader, InitializedReader, Reader, SlotTrait, Storable, Writer, generational,
};
use core::any::TypeId;
use core::pin::Pin;

/// Internal helper to implement [`Datastore::slot`] recursively for a cons-list of slots.
trait Slots {
    /// Attempts to find a slot of the given type.
    /// Returns None if no such slot exists.
    fn try_slot<S>(self: Pin<&Self>) -> Option<Pin<&S>>
    where
        S: SlotTrait;

    /// Returns the [`TypeId`] and type names for all the slots stored in this type.
    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)>;
}

impl Slots for Nil {
    fn try_slot<S>(self: Pin<&Self>) -> Option<Pin<&S>>
    where
        S: SlotTrait,
    {
        None
    }

    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)> {
        core::iter::empty()
    }
}

impl<T> Slots for T
where
    T: SlotTrait + core::any::Any,
{
    fn try_slot<S>(self: Pin<&Self>) -> Option<Pin<&S>>
    where
        S: SlotTrait + core::any::Any,
    {
        if TypeId::of::<S>() == TypeId::of::<T>() {
            // SAFETY:
            // `Pin::map_unchecked`: We're only transforming the type, so it retains its pinned-ness.
            // `cast` + `as_ref`: We verified above that the types of `S` and `T` are the same.
            Some(unsafe {
                Pin::map_unchecked(self, |this| {
                    core::ptr::NonNull::from_ref(this).cast::<S>().as_ref()
                })
            })
        } else {
            None
        }
    }

    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)> {
        core::iter::once((T::data_type_id(), T::data_type_name()))
    }
}

impl<U, R> Slots for Cons<U, R>
where
    U: Slots,
    R: Slots,
{
    fn try_slot<S>(self: Pin<&Self>) -> Option<Pin<&S>>
    where
        S: SlotTrait,
    {
        let this = self.project_ref();

        this.0.try_slot::<S>().or_else(|| this.1.try_slot::<S>())
    }

    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)> {
        U::all_slots().chain(R::all_slots())
    }
}

/// Internal helper to construct runtime slot instances from a type-level cons list of slots.
trait IntoSlots {
    /// The same cons-list type, used to construct slot instances.
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

impl<S> IntoSlots for S
where
    S: SlotTrait + 'static,
{
    type Slots = S;

    fn make_slots() -> Self::Slots {
        S::new()
    }
}

impl<S, R> IntoSlots for Cons<S, R>
where
    S: IntoSlots,
    R: IntoSlots,
{
    type Slots = Cons<S::Slots, R::Slots>;

    fn make_slots() -> Self::Slots {
        Cons(S::make_slots(), R::make_slots())
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
/// Given a slot cons-list, combines it with a [`generational::Source`] to implement [`Datastore`].
impl<S: Slots> Datastore for Cons<generational::Source, S>
where
    S: Slots,
{
    fn source(self: Pin<&Self>) -> Pin<&generational::Source> {
        let this = self.project_ref();
        this.0
    }

    fn slot<T>(self: Pin<&Self>, requestor: &'static str) -> Pin<&T>
    where
        T: SlotTrait,
    {
        let this = self.project_ref();
        this.1.try_slot::<T>().unwrap_or_else(|| {
            panic!(
                "no slot available for `{}`, required by `{requestor}`",
                T::data_type_name()
            )
        })
    }
}

/// Given a cons-list of slot types, returns a complete [`Datastore`] that contains those slots.
pub(crate) fn make_store<T>() -> impl Datastore
where
    T: IntoSlots,
{
    Cons(generational::Source::new(), T::make_slots())
}

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

/// Internal helper to access details about a cons-list of actors so they can be validated against a store.
pub(crate) trait ActorList<'a>
where
    Self: 'a,
{
    /// A cons-list of init-contexts for this cons-list (essentially `self.map(|actor| actor.init_context)`).
    type InitContexts;

    /// A cons-list of slot cons-lists for this actor list (nested structure).
    type AllSlots;

    /// Returns an iterator over all slots required by actors in this list as `(TypeId, type_name)` pairs.
    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)>;

    /// Returns the type names of the actors in this list that write to the given type.
    ///
    /// If an actor has multiple writers for the same type it will be in the list multiple times.
    fn writers(type_id: TypeId) -> impl Iterator<Item = &'static str>;

    /// Returns the type names of the actors in this list that read from the given type, both
    /// exclusive and non-exclusive.
    ///
    /// If an actor has multiple readers for the same type it will be in the list multiple times.
    fn readers(type_id: TypeId) -> impl Iterator<Item = &'static str>;

    /// Returns the type names of the actors in this list that read from the given type with an
    /// exclusive reader.
    ///
    /// If an actor has multiple exclusive readers for the same type it will be in the list multiple
    /// times.
    fn exclusive_readers(type_id: TypeId) -> impl Iterator<Item = &'static str>;

    /// Returns the type names of the actors in this list that read from the given type with a
    /// non-exclusive reader.
    ///
    /// If an actor has multiple non-exclusive readers for the same type it will be in the list
    /// multiple times.
    fn other_readers(type_id: TypeId) -> impl Iterator<Item = &'static str>;
}

impl ActorList<'_> for Nil {
    type InitContexts = Nil;
    type AllSlots = Nil;

    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)> {
        core::iter::empty()
    }

    fn writers(_type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::empty()
    }

    fn readers(_type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::empty()
    }

    fn exclusive_readers(_type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::empty()
    }

    fn other_readers(_type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::empty()
    }
}

impl<'a, T, U> ActorList<'a> for Cons<T, U>
where
    T: Actor<'a, StoreRequest: TupleConsToCons> + 'a,
    <T as Actor<'a>>::Slots: IntoSlots,
    <<T as Actor<'a>>::Slots as IntoSlots>::Slots: Slots,
    U: ActorList<'a> + 'a,
    <<T as Actor<'a>>::StoreRequest as TupleConsToCons>::Cons: AccessCount,
{
    /// For `Actor::InitContext` we just need to map directly to the associated type.
    type InitContexts = Cons<<T as Actor<'a>>::InitContext, <U as ActorList<'a>>::InitContexts>;

    /// For `AllSlots` we create a cons list of each actor's slots (nested structure).
    type AllSlots = Cons<<T as Actor<'a>>::Slots, <U as ActorList<'a>>::AllSlots>;

    fn all_slots() -> impl Iterator<Item = (TypeId, &'static str)> {
        U::all_slots().chain(<<<T as Actor<'a>>::Slots as IntoSlots>::Slots as Slots>::all_slots())
    }

    fn writers(type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::repeat_n(
            core::any::type_name::<T>(),
            <T::StoreRequest as TupleConsToCons>::Cons::writers(type_id),
        )
        .chain(U::writers(type_id))
    }

    fn readers(type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::repeat_n(
            core::any::type_name::<T>(),
            <T::StoreRequest as TupleConsToCons>::Cons::readers(type_id),
        )
        .chain(U::readers(type_id))
    }

    fn exclusive_readers(type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::repeat_n(
            core::any::type_name::<T>(),
            <T::StoreRequest as TupleConsToCons>::Cons::exclusive_readers(type_id),
        )
        .chain(U::exclusive_readers(type_id))
    }

    fn other_readers(type_id: TypeId) -> impl Iterator<Item = &'static str> {
        core::iter::repeat_n(
            core::any::type_name::<T>(),
            <T::StoreRequest as TupleConsToCons>::Cons::readers(type_id)
                - <T::StoreRequest as TupleConsToCons>::Cons::exclusive_readers(type_id),
        )
        .chain(U::other_readers(type_id))
    }
}

/// Returns a type that will write the given list of types out comma separated with backtick
/// quoting, or `nothing` if it is empty.
///
/// ```text
/// [] => "nothing"
/// ["A"] => "`A`"
/// ["A", "B"] => "`A`, `B`"
/// ["A", "B", "C"] => "`A`, `B`, `C`"
/// ```
fn format_types(types: impl IntoIterator<Item = &'static str>) -> impl core::fmt::Display {
    struct Helper<T>(core::cell::RefCell<T>);

    impl<T> core::fmt::Display for Helper<T>
    where
        T: Iterator<Item = &'static str>,
    {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let mut iter = self.0.borrow_mut();
            if let Some(first) = iter.next() {
                f.write_str("`")?;
                f.write_str(first)?;
                f.write_str("`")?;
                for next in &mut *iter {
                    f.write_str(", `")?;
                    f.write_str(next)?;
                    f.write_str("`")?;
                }
            } else {
                f.write_str("nothing")?;
            }
            Ok(())
        }
    }

    Helper(core::cell::RefCell::new(types.into_iter()))
}

/// Creates a store and validates actors in a single call to enable type inference.
///
/// This function combines store creation and validation so that the actor list type parameter appears only once,
/// allowing Rust's type inference to work across both operations.
///
/// The slots are computed from the actor list's associated type.
pub fn make_store_and_validate<'a, A, I>(init_contexts: I) -> (impl Datastore + 'a, I)
where
    A: ActorList<'a, InitContexts = I>,
    A::AllSlots: IntoSlots,
{
    let store = make_store::<A::AllSlots>();

    for (type_id, type_name) in A::all_slots() {
        let writers = A::writers(type_id).count();
        let readers = A::readers(type_id).count();
        let exclusive_readers = A::exclusive_readers(type_id).count();
        assert!(
            writers > 0,
            "missing writer for `{type_name}`, read by: {}",
            format_types(A::readers(type_id)),
        );
        assert!(
            readers > 0,
            "missing reader for `{type_name}`, written by: {}",
            format_types(A::writers(type_id)),
        );
        assert!(
            writers == 1,
            "multiple writers for `{type_name}`: {}",
            format_types(A::writers(type_id)),
        );
        if exclusive_readers > 0 {
            assert!(
                readers == 1,
                "conflict with exclusive reader for `{type_name}`:\nexclusive readers: {}\n    other readers: {}",
                format_types(A::exclusive_readers(type_id)),
                format_types(A::other_readers(type_id)),
            );
        }
    }

    (store, init_contexts)
}

/// Internal helper to get a full future that initializes and executes an [`Actor`] given a [`Datastore`]
pub async fn execute_actor<'a, A>(
    store: Pin<&'a impl Datastore>,
    init_context: A::InitContext,
) -> Never
where
    A: Actor<'a>,
{
    let requestor = core::any::type_name::<A>();
    veecle_telemetry::future::FutureExt::with_span(
        async move {
            match A::new(
                A::StoreRequest::request(store, requestor).await,
                init_context,
            )
            .run()
            .await
            {
                Err(error) => panic!("{error}"),
            }
        },
        veecle_telemetry::span!("actor", actor = core::any::type_name::<A>()),
    )
    .await
}

/// Execute a given set of actors without heap allocation.
///
/// ```rust
/// use core::fmt::Debug;
///
/// use veecle_os_runtime::{Never, Reader, Storable, Writer};
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
/// async fn ping_actor(mut ping: Writer<'_, Ping>, pong: Reader<'_, Pong>) -> Never {
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
/// async fn pong_actor(mut pong: Writer<'_, Pong>, ping: Reader<'_, Ping>) -> Never {
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
///        actors: [PingActor, PongActor],
///    }
/// )
#[macro_export]
macro_rules! execute {
    (
        actors: [
            $($actor_type:ty $(: $init_context:expr )? ),* $(,)?
        ] $(,)?
    ) => {{
        async {
            let (store, init_contexts) = {
                let (store, init_contexts) = $crate::__exports::make_store_and_validate::<
                    $crate::__make_cons!(@type $($actor_type,)*),
                    _,
                >($crate::__make_cons!(@value $(
                    // Wrapper block is used to provide a `()` if no expression is passed.
                    { $($init_context)? },
                )*));
                (core::pin::pin!(store), init_contexts)
            };

            let store = store.as_ref();

            // To count how many actors there are, we create an array of `()` with the appropriate length.
            const LEN: usize = [$($crate::discard_to_unit!($actor_type),)*].len();

            let futures: [core::pin::Pin<&mut dyn core::future::Future<Output = $crate::Never>>; LEN] =
                $crate::make_futures! {
                    init_contexts: init_contexts,
                    store: store,
                    actors: [$($actor_type,)*],
                };

            static SHARED: $crate::__exports::ExecutorShared<LEN>
                = $crate::__exports::ExecutorShared::new(&SHARED);

            let executor = $crate::__exports::Executor::new(
                &SHARED,
                $crate::__exports::Datastore::source(store),
                futures,
            );

            executor.run().await
        }
    }};
}

/// Internal helper to construct an array of pinned futures for given actors + init-contexts + store.
///
/// Returns essentially `[Pin<&mut dyn Future<Output = Never>; actors.len()]`, but likely needs annotation at the
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::marker::PhantomData;
    use core::pin::pin;

    use super::Slots;
    use crate::actor::Datastore;
    use crate::cons::Cons;
    use crate::cons::Nil;
    use crate::datastore::Slot;
    use crate::execute::generational::Source;

    #[test]
    fn nil_all_slots_returns_empty() {
        let slots = Nil::all_slots();
        assert_eq!(slots.count(), 0);
    }

    #[test]
    fn cons_nil_all_slots_returns_empty() {
        let slots = <Cons<Nil, Nil>>::all_slots();
        assert_eq!(slots.count(), 0);
    }

    #[test]
    fn cons_cons_nil_all_slots_returns_empty() {
        let slots = <Cons<Nil, Cons<Nil, Nil>>>::all_slots();
        assert_eq!(slots.count(), 0);
    }

    #[test]
    #[should_panic(
        expected = "no slot available for `veecle_os_runtime::execute::tests::nil_slot_panics_with_correct_message::TestType`, required by `test_requestor`"
    )]
    fn nil_slot_panics_with_correct_message() {
        #[derive(Debug, crate::datastore::Storable)]
        #[storable(crate = crate)]
        struct TestType;

        let nil = pin!(Cons(Source::new(), Nil));
        let _slot: core::pin::Pin<&Slot<TestType>> =
            Datastore::slot(nil.as_ref(), "test_requestor");
    }

    #[test]
    #[should_panic(expected = "type inference works")]
    fn type_inference_for_generic_actors() {
        use crate::{Actor, Never};

        struct GenericActor<T> {
            _phantom: PhantomData<T>,
        }

        impl<'a, T> Actor<'a> for GenericActor<T>
        where
            T: core::fmt::Debug + 'static,
        {
            type StoreRequest = ();
            type InitContext = T;
            type Slots = Nil;
            type Error = Never;

            fn new((): Self::StoreRequest, _context: Self::InitContext) -> Self {
                Self {
                    _phantom: PhantomData,
                }
            }

            async fn run(self) -> Result<Never, Self::Error> {
                panic!("type inference works");
            }
        }

        futures::executor::block_on(crate::execute! {
            actors: [
                GenericActor<_>: 42_i32,
            ],
        });
    }
}
