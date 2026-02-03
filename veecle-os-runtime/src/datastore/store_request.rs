//! Trait for requesting readers and writers from a datastore.

use super::Datastore;
use crate::Sealed;
use core::pin::Pin;

/// Allows requesting a (nearly) arbitrary amount of readers and writers in an [`Actor`][crate::Actor].
///
/// This trait is not intended for direct usage by users.
// Developer notes: This works by using type inference via `Datastore::reader` etc. to request `Reader`s etc. from the
// `Datastore`.
#[diagnostic::on_unimplemented(
    message = "invalid actor parameter type",
    label = "the function signature contains parameters that are neither init_context nor reader/writers",
    note = "only the init_context and readers/writers provided by the Veecle OS runtime may be used as actor parameters",
    note = "parameters passed as initialization context need to be marked with `#[init_context]`"
)]
#[expect(private_bounds, reason = "Sealed trait")]
pub trait StoreRequest<'a>: Sealed {
    /// Requests an instance of `Self` from the [`Datastore`].
    ///
    /// # Panics
    ///
    /// * If there is no slot for one of the types in `Self` in the [`Datastore`].
    ///
    /// `requestor` will be included in the panic message for context.
    #[doc(hidden)]
    #[allow(async_fn_in_trait, reason = "it's actually private so it's fine")]
    async fn request(datastore: Pin<&'a impl Datastore>, requestor: &'static str) -> Self;
}

impl Sealed for () {}

/// Implements a no-op for Actors that do not read or write any values.
#[diagnostic::do_not_recommend]
impl<'a> StoreRequest<'a> for () {
    async fn request(_store: Pin<&'a impl Datastore>, _requestor: &'static str) -> Self {}
}

/// Implements [`StoreRequest`] for provided types.
macro_rules! impl_request_helper {
    ($t:ident) => {
        #[cfg_attr(docsrs, doc(fake_variadic))]
        /// This trait is implemented for tuples up to seven items long.
        impl<'a, $t> Sealed for ($t,) { }

        #[cfg_attr(docsrs, doc(fake_variadic))]
        /// This trait is implemented for tuples up to seven items long.
        #[diagnostic::do_not_recommend]
        impl<'a, $t> StoreRequest<'a> for ($t,)
        where
            $t: StoreRequest<'a>,
        {
            async fn request(datastore: Pin<&'a impl Datastore>, requestor: &'static str) -> Self {
                (<$t as StoreRequest>::request(datastore, requestor).await,)
            }
        }
    };

    (@impl $($t:ident)*) => {
        #[cfg_attr(docsrs, doc(hidden))]
        impl<'a, $($t),*> Sealed for ( $( $t, )* )
        where
            $($t: Sealed),*
        { }

        #[cfg_attr(docsrs, doc(hidden))]
        #[diagnostic::do_not_recommend]
        impl<'a, $($t),*> StoreRequest<'a> for ( $( $t, )* )
        where
            $($t: StoreRequest<'a>),*
        {
            async fn request(datastore: Pin<&'a impl Datastore>, requestor: &'static str) -> Self {
                futures::join!($( <$t as StoreRequest>::request(datastore, requestor), )*)
            }
        }
    };

    ($head:ident $($rest:ident)*) => {
        impl_request_helper!(@impl $head $($rest)*);
        impl_request_helper!($($rest)*);
    };
}

impl_request_helper!(Z Y X W V U T);
