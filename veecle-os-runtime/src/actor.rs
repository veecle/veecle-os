//! Smallest unit of work within a runtime instance.
use core::convert::Infallible;
use core::pin::Pin;

#[doc(inline)]
pub use veecle_os_runtime_macros::actor;

use crate::datastore::{ExclusiveReader, InitializedReader, Reader, Storable, Writer};
use crate::datastore::{Slot, generational};
use crate::find::NewDatastore;

mod sealed {
    pub trait Sealed {}
}

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
/// # use std::convert::Infallible;
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, Reader, Writer};
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
/// ) -> Infallible {
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
/// # use std::convert::Infallible;
/// # use std::fmt::Debug;
/// #
/// # use veecle_os_runtime::{Storable, Reader, Writer, Actor};
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
///     type Error = Infallible;
///
///     fn new((reader, writer): Self::StoreRequest, context: Self::InitContext) -> Self {
///         Self {
///             reader,
///             writer,
///             context,
///         }
///     }
///
///     async fn run(mut self) -> Result<Infallible, Self::Error> {
///         loop {
///             // Do something here.
///         }
///     }
/// }
/// ```
pub trait Actor<'a> {
    /// [`Reader`]s and [`Writer`]s this actor requires.
    type StoreRequest: StoreRequest<'a>;

    /// Context that needs to be passed to the actor at initialisation.
    type InitContext;

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
    fn run(
        self,
    ) -> impl core::future::Future<Output = Result<core::convert::Infallible, Self::Error>>;
}

/// Allows requesting a (nearly) arbitrary amount of [`Reader`]s and [`Writer`]s in an [`Actor`].
///
/// This trait is not intended for direct usage by users.
// Developer notes: This works by using type inference via `Datastore::reader` etc. to request `Reader`s etc. from the
// `Datastore`.
pub trait StoreRequest<'a>: sealed::Sealed {
    /// Requests an instance of `Self` from the [`Datastore`].
    #[doc(hidden)]
    #[allow(async_fn_in_trait, reason = "it's actually private so it's fine")]
    async fn request(datastore: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self;
}

impl sealed::Sealed for () {}

// /// Internal trait to abstract out type-erased and concrete data stores.
// pub trait Datastore {
//     /// Returns a generational source tracking the global datastore generation.
//     ///
//     /// This is used to ensure that every reader has had (or will have) a chance to read a value before a writer may
//     /// overwrite it.
//     fn source(self: Pin<&Self>) -> Pin<&generational::Source>;
//
//     #[expect(
//         rustdoc::private_intra_doc_links,
//         reason = "`rustdoc` is buggy with links from `pub` but unreachable types"
//     )]
//     /// Returns a reference to the slot for a specific type.
//     ///
//     /// # Panics
//     ///
//     /// * If there is no [`Slot`] for `T` in the [`Datastore`].
//     #[expect(private_interfaces, reason = "the methods are internal")]
//     fn slot<T>(self: Pin<&Self>) -> Pin<&Slot<T>>
//     where
//         T: Storable + 'static;
// }
//
// impl<S> Datastore for Pin<&S>
// where
//     S: Datastore,
// {
//     fn source(self: Pin<&Self>) -> Pin<&generational::Source> {
//         Pin::into_inner(self).source()
//     }
//
//     #[expect(private_interfaces, reason = "the methods are internal")]
//     fn slot<T>(self: Pin<&Self>) -> Pin<&Slot<T>>
//     where
//         T: Storable + 'static,
//     {
//         Pin::into_inner(self).slot()
//     }
// }

pub trait DatastoreExt<'a> {
    #[cfg(test)]
    /// Increments the global datastore generation.
    ///
    /// Asserts that every reader has had (or will have) a chance to read a value before a writer may overwrite it.
    fn increment_generation(&'a self);

    /// Returns the [`Reader`] for a specific slot.
    ///
    /// # Panics
    ///
    /// * If there is no [`Slot`] for `T` in the [`Datastore`].
    fn reader<T>(&'a self) -> Reader<'a, T>
    where
        T: Storable + 'static;

    /// Returns the [`ExclusiveReader`] for a specific slot.
    ///
    /// Exclusivity of the reader is not guaranteed by this method and must be ensured via other means (e.g.
    /// [`crate::execute::validate_actors`]).
    ///
    /// # Panics
    ///
    /// * If there is no [`Slot`] for `T` in the [`Datastore`].
    fn exclusive_reader<T>(&'a self) -> ExclusiveReader<'a, T>
    where
        T: Storable + 'static;

    /// Returns the [`Writer`] for a specific slot.
    ///
    /// # Panics
    ///
    /// * If the [`Writer`] for this slot has already been acquired.
    ///
    /// * If there is no [`Slot`] for `T` in the [`Datastore`].
    fn writer<T>(&'a self) -> Writer<'a, T>
    where
        T: Storable + 'static;
}

// impl<'a, S> DatastoreExt<'a> for Pin<&'a S>
// where
//     S: NewDatastore,
// {
//     #[cfg(test)]
//     #[cfg_attr(coverage_nightly, coverage(off))]
//     fn increment_generation(&'a self) {
//         self.source().increment_generation()
//     }
//
//     fn reader<T>(&'a self) -> Reader<'a, T>
//     where
//         T: Storable + 'static,
//     {
//         Reader::from_slot(self.slot::<T>())
//     }
//
//     fn exclusive_reader<T>(&'a self) -> ExclusiveReader<'a, T>
//     where
//         T: Storable + 'static,
//     {
//         ExclusiveReader::from_slot(self.slot::<T>())
//     }
//
//     fn writer<T>(&'a self) -> Writer<'a, T>
//     where
//         T: Storable + 'static,
//     {
//         Writer::new(self.source().waiter(), self.slot::<T>())
//     }
// }

// impl<'a, S> DatastoreExt<'a> for &'a S
// where
//     S: NewDatastore,
// {
//     #[cfg(test)]
//     #[cfg_attr(coverage_nightly, coverage(off))]
//     fn increment_generation(&'a self) {
//         self.source().increment_generation()
//     }
//
//     fn reader<T>(&'a self) -> Reader<'a, T>
//     where
//         T: Storable + 'static,
//     {
//         Reader::from_slot(self.slot::<T>())
//     }
//
//     fn exclusive_reader<T>(&'a self) -> ExclusiveReader<'a, T>
//     where
//         T: Storable + 'static,
//     {
//         ExclusiveReader::from_slot(self.slot::<T>())
//     }
//
//     fn writer<T>(&'a self) -> Writer<'a, T>
//     where
//         T: Storable + 'static,
//     {
//         Writer::new(self.source().waiter(), self.slot::<T>())
//     }
// }

impl<'a, S> DatastoreExt<'a> for S
where
    S: NewDatastore,
{
    #[cfg(test)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn increment_generation(&'a self) {
        self.source().increment_generation()
    }

    fn reader<T>(&'a self) -> Reader<'a, T>
    where
        T: Storable + 'static,
    {
        Reader::from_slot(self.slot::<T>())
    }

    fn exclusive_reader<T>(&'a self) -> ExclusiveReader<'a, T>
    where
        T: Storable + 'static,
    {
        ExclusiveReader::from_slot(self.slot::<T>())
    }

    fn writer<T>(&'a self) -> Writer<'a, T>
    where
        T: Storable + 'static,
    {
        Writer::new(self.source().waiter(), self.slot::<T>())
    }
}

/// Implements a no-op for Actors that do not read or write any values.
impl<'a> StoreRequest<'a> for () {
    async fn request(_store: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self {}
}

impl<T> sealed::Sealed for Reader<'_, T> where T: Storable + 'static {}

impl<'a, T> StoreRequest<'a> for Reader<'a, T>
where
    T: Storable + 'static,
{
    async fn request(datastore: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self {
        datastore.reader()
    }
}

impl<T> sealed::Sealed for ExclusiveReader<'_, T> where T: Storable + 'static {}

impl<'a, T> StoreRequest<'a> for ExclusiveReader<'a, T>
where
    T: Storable + 'static,
{
    async fn request(datastore: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self {
        datastore.exclusive_reader()
    }
}

impl<T> sealed::Sealed for InitializedReader<'_, T> where T: Storable + 'static {}

impl<'a, T> StoreRequest<'a> for InitializedReader<'a, T>
where
    T: Storable + 'static,
{
    async fn request(datastore: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self {
        Reader::from_slot(datastore.slot()).wait_init().await
    }
}

impl<T> sealed::Sealed for Writer<'_, T> where T: Storable + 'static {}

impl<'a, T> StoreRequest<'a> for Writer<'a, T>
where
    T: Storable + 'static,
{
    async fn request(datastore: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self {
        datastore.writer()
    }
}

/// Implements [`StoreRequest`] for provided types.
macro_rules! impl_request_helper {
    ($t:ident) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        /// This trait is implemented for tuples up to seven items long.
        impl<'a, $t> sealed::Sealed for ($t,) { }

        #[cfg_attr(docsrs, doc(fake_variadic))]
        /// This trait is implemented for tuples up to seven items long.
        impl<'a, $t> StoreRequest<'a> for ($t,)
        where
            $t: StoreRequest<'a>,
        {
            async fn request(datastore: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self {
                (<$t as StoreRequest>::request(datastore).await,)
            }
        }
    };

    (@impl $($t:ident)*) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<'a, $($t),*> sealed::Sealed for ( $( $t, )* )
        where
            $($t: sealed::Sealed),*
        { }

        #[cfg_attr(docsrs, doc(hidden))]
        impl<'a, $($t),*> StoreRequest<'a> for ( $( $t, )* )
        where
            $($t: StoreRequest<'a>),*
        {
            async fn request(datastore: &'a (impl NewDatastore + DatastoreExt<'a>)) -> Self {
                // join! is necessary here to avoid argument-order-dependence with the #[actor] macro.
                // This ensures that any `InitializedReaders` in self correctly track the generation at which they were
                // first ready, so that the first `wait_for_update` sees the value that caused them to become
                // initialized.
                // See `multi_request_order_independence` for the verification of this.
                futures::join!($( <$t as StoreRequest>::request(datastore), )*)
            }
        }
    };

    ($head:ident $($rest:ident)*) => {
        impl_request_helper!(@impl $head $($rest)*);
        impl_request_helper!($($rest)*);
    };
}

impl_request_helper!(Z Y X W V U T);

/// Macro helper to allow actors to return either a [`Result`] type or [`Infallible`] (and eventually [`!`]).
#[diagnostic::on_unimplemented(
    message = "#[veecle_os_runtime::actor] functions should return either a `Result<Infallible, _>` or `Infallible`",
    label = "not a valid actor return type"
)]
pub trait IsActorResult: sealed::Sealed {
    /// The error type this result converts into.
    type Error;

    /// Convert the result into an actual [`Result`] value.
    fn into_result(self) -> Result<Infallible, Self::Error>;
}

impl<E> sealed::Sealed for Result<Infallible, E> {}

impl<E> IsActorResult for Result<Infallible, E> {
    type Error = E;

    fn into_result(self) -> Result<Infallible, E> {
        self
    }
}

impl sealed::Sealed for Infallible {}

impl IsActorResult for Infallible {
    type Error = Infallible;

    fn into_result(self) -> Result<Infallible, Self::Error> {
        match self {}
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::future::Future;
    use core::pin::pin;
    use core::task::{Context, Poll};

    use futures::future::FutureExt;

    use crate::actor::{DatastoreExt, StoreRequest};
    use crate::cons::{Cons, Nil};
    use crate::datastore::{InitializedReader, Storable};

    #[test]
    fn multi_request_order_independence() {
        #[derive(Debug, Storable)]
        #[storable(crate = crate)]
        struct A;

        #[derive(Debug, Storable)]
        #[storable(crate = crate)]
        struct B;

        let datastore = pin!(crate::execute::make_store::<Cons<A, Cons<B, Nil>>>());

        let mut a_writer = datastore.as_ref().writer::<A>();
        let mut b_writer = datastore.as_ref().writer::<B>();

        // No matter the order these two request the readers, they should both resolve during the generation where the
        // later of the two is first written.
        let mut request_1 = pin!(<(InitializedReader<A>, InitializedReader<B>)>::request(
            datastore.as_ref()
        ));
        let mut request_2 = pin!(<(InitializedReader<B>, InitializedReader<A>)>::request(
            datastore.as_ref()
        ));

        let (request_1_waker, request_1_wake_count) = futures_test::task::new_count_waker();
        let (request_2_waker, request_2_wake_count) = futures_test::task::new_count_waker();

        let mut request_1_context = Context::from_waker(&request_1_waker);
        let mut request_2_context = Context::from_waker(&request_2_waker);

        assert!(matches!(
            request_1.as_mut().poll(&mut request_1_context),
            Poll::Pending
        ));
        assert!(matches!(
            request_2.as_mut().poll(&mut request_2_context),
            Poll::Pending
        ));

        let old_request_1_wake_count = request_1_wake_count.get();
        let old_request_2_wake_count = request_2_wake_count.get();

        datastore.as_ref().increment_generation();

        a_writer.write(A).now_or_never().unwrap();

        // When the first value is written, each future may or may not wake up, but if they do we need to poll them.
        if request_1_wake_count.get() > old_request_1_wake_count {
            assert!(matches!(
                request_1.as_mut().poll(&mut request_1_context),
                Poll::Pending
            ));
        }
        if request_2_wake_count.get() > old_request_2_wake_count {
            assert!(matches!(
                request_2.as_mut().poll(&mut request_2_context),
                Poll::Pending
            ));
        }

        let old_request_1_wake_count = request_1_wake_count.get();
        let old_request_2_wake_count = request_2_wake_count.get();

        datastore.as_ref().increment_generation();

        b_writer.write(B).now_or_never().unwrap();

        // When the second value is written, both futures _must_ wake up and complete.
        assert!(request_1_wake_count.get() > old_request_1_wake_count);
        assert!(request_2_wake_count.get() > old_request_2_wake_count);

        let Poll::Ready((mut request_1_a, mut request_1_b)) =
            request_1.as_mut().poll(&mut request_1_context)
        else {
            panic!("request 1 was not ready")
        };

        let Poll::Ready((mut request_2_a, mut request_2_b)) =
            request_2.as_mut().poll(&mut request_2_context)
        else {
            panic!("request 2 was not ready")
        };

        // All readers should see an update, since they've just been initialized but not `wait_for_update`d.
        assert!(request_1_a.wait_for_update().now_or_never().is_some());
        assert!(request_1_b.wait_for_update().now_or_never().is_some());

        assert!(request_2_a.wait_for_update().now_or_never().is_some());
        assert!(request_2_b.wait_for_update().now_or_never().is_some());
    }
}
