//! Smallest unit of work within a runtime instance.

pub use crate::datastore::StoreRequest;
use crate::{Never, Sealed};
#[doc(inline)]
pub use veecle_os_runtime_macros::actor;

/// Actor interface.
///
/// The [`Actor`] trait allows writing actors that communicate within a runtime.
/// It allows to define an initial context, which will be available for the whole life of the actor;
/// a constructor method, with all the [`StoreRequest`] types it needs to communicate with other actors;
/// and also the [`Actor::run`] method.
///
/// # Usage
///
/// Add the `Actor` implementing types to the actor list in [`veecle_os::runtime::execute!`](crate::execute!) when
/// constructing a runtime instance.
///
/// The [`Actor::run`] method implements the actor's event loop.
/// To yield back to the executor, every event loop must contain at least one `await`.
/// Otherwise, the endless loop of the actor will block the executor and other actors.
///
/// ## Macros
///
/// The [`actor`][macro@crate::actor::actor] attribute macro can be used to implement actors.
/// The function the macro is applied to is converted into the event loop.
/// See its documentation for more details.
///
/// ### Example
///
/// ```rust
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::single_writer::{Reader, Writer};
/// # use veecle_os_runtime::{Never, Storable};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Bar;
/// #
/// # pub struct Ctx;
///
/// #[veecle_os_runtime::actor]
/// async fn my_actor(
///     reader: Reader<'_, Foo>,
///     writer: Writer<'_, Bar>,
///     #[init_context] ctx: Ctx,
/// ) -> Never {
///     loop {
///         // Do something here.
///     }
/// }
/// ```
///
/// This will create a new struct called `MyActor` which implements [`Actor`], letting you register it into a runtime.
///
/// ## Manual
///
/// For cases where the macro is not sufficient, the [`Actor`] trait can also be implemented manually:
///
/// ```rust
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::single_writer::{Reader, Writer};
/// # use veecle_os_runtime::{Never, Storable, Actor};
/// # use veecle_os_runtime::__exports::{AppendCons, DefinesSlot};
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Foo;
/// #
/// # #[derive(Debug, Default, Storable)]
/// # pub struct Bar;
/// #
/// # pub struct Ctx;
///
/// struct MyActor<'a> {
///     reader: Reader<'a, Foo>,
///     writer: Writer<'a, Bar>,
///     context: Ctx,
/// }
///
/// impl<'a> Actor<'a> for MyActor<'a> {
///     type StoreRequest = (Reader<'a, Foo>, Writer<'a, Bar>);
///     type InitContext = Ctx;
///     type Error = Never;
///     type Slots = <<Reader<'a, Foo> as DefinesSlot>::Slot as AppendCons<<Writer<'a, Bar> as DefinesSlot>::Slot>>::Result;
///
///     fn new((reader, writer): Self::StoreRequest, context: Self::InitContext) -> Self {
///         Self {
///             reader,
///             writer,
///             context,
///         }
///     }
///
///     async fn run(mut self) -> Result<Never, Self::Error> {
///         loop {
///             // Do something here.
///         }
///     }
/// }
/// ```
pub trait Actor<'a> {
    /// Readers and writers this actor requires.
    ///
    /// See [`single_writer`][crate::single_writer] for a slot implementation with one writer and
    /// multiple readers.
    type StoreRequest: StoreRequest<'a>;

    /// Context that needs to be passed to the actor at initialisation.
    type InitContext;

    /// Cons list of slots required by this actor.
    ///
    /// This is a type-level cons list of `Slot<T>` types.
    type Slots;

    /// Error that this actor might return while running.
    ///
    /// This error is treated as fatal, if any actor returns an error the whole runtime will shutdown.
    type Error: core::error::Error;

    /// Creates a new instance of the struct implementing [`Actor`].
    ///
    /// See the [crate documentation][crate] for examples.
    fn new(input: Self::StoreRequest, init_context: Self::InitContext) -> Self;

    /// Runs the [`Actor`] event loop.
    ///
    /// See the [crate documentation][crate] for examples.
    fn run(self) -> impl core::future::Future<Output = Result<Never, Self::Error>>;
}

/// Macro helper to allow actors to return either a [`Result`] type or [`Never`] (and eventually [`!`]).
#[diagnostic::on_unimplemented(
    message = "#[veecle_os_runtime::actor] functions should return either a `Result<Never, _>` or `Never`",
    label = "not a valid actor return type"
)]
#[expect(private_bounds, reason = "Sealed trait")]
pub trait IsActorResult: Sealed {
    /// The error type this result converts into.
    type Error;

    /// Convert the result into an actual [`Result`] value.
    fn into_result(self) -> Result<Never, Self::Error>;
}

impl<E> Sealed for Result<Never, E> {}

impl<E> IsActorResult for Result<Never, E> {
    type Error = E;

    fn into_result(self) -> Result<Never, E> {
        self
    }
}

impl Sealed for Never {}

impl IsActorResult for Never {
    type Error = Never;

    fn into_result(self) -> Result<Never, Self::Error> {
        match self {}
    }
}
