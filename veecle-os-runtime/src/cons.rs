//! Helper traits to work with type-level [cons-lists](https://en.wikipedia.org/wiki/Cons#Lists).
//!
//! These are useful when working with macros and traits that need to support arbitrary length type
//! lists, without having to macro generate hundreds of trait implementations for different length
//! tuples.
//!
//! Rather than using raw tuples `type Nil = (); type Cons<T, U> = (T, U);` this includes new
//! nominal types so that we can have safe pin-projection.

/// The terminal element of a cons-list.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Nil;

/// Prepends an element to the cons-list, somewhat equivalent to the array `[T, ...U]`.
#[pin_project::pin_project]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Cons<T, U>(#[pin] pub T, #[pin] pub U);

/// Internal helper that asserts post-normalization types are the same, see usage in below doc-tests.
#[doc(hidden)]
#[macro_export]
macro_rules! __assert_same_type {
    (
        for<$($generic:ident),*>
        $type1:ty,
        $type2:ty $(,)?
    ) => {
        const _: () = {
            fn equivalent<$($generic,)*>(value: $type1) -> $type2 { value }
        };
    };
}

/// Converts a tuple-based cons-list into one using our nominal types.
pub(crate) trait TupleConsToCons {
    /// The [`Cons`]-based cons-list
    type Cons;
}

impl TupleConsToCons for () {
    type Cons = Nil;
}

impl<T, U> TupleConsToCons for (T, U)
where
    U: TupleConsToCons,
{
    type Cons = Cons<T, <U as TupleConsToCons>::Cons>;
}

/// Given a list of types or values, generate a cons-list for those types or values.
///
/// ```rust
/// use veecle_os_runtime::{__assert_same_type, __make_cons};
/// use veecle_os_runtime::__exports::{Cons, Nil};
///
/// __assert_same_type! {
///     for<>
///     __make_cons!(@type),
///     Nil,
/// }
///
/// __assert_same_type! {
///     for<A, B, C>
///     __make_cons!(@type A, B, C),
///     Cons<A, Cons<B, Cons<C, Nil>>>,
/// }
///
/// assert_eq! {
///     __make_cons!(@value),
///     Nil,
/// }
///
/// assert_eq! {
///     __make_cons!(@value 1u32, "hello ferris", 3.141594f64),
///     Cons(1, Cons("hello ferris", Cons(3.141594, Nil))),
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __make_cons {
    (@type) => {
        $crate::__exports::Nil
    };

    (@type $first:ty $(, $rest:ty)* $(,)? ) => {
        $crate::__exports::Cons<$first, $crate::__make_cons!(@type $($rest,)*)>
    };

    (@value) => {
        $crate::__exports::Nil
    };

    (@value $first:expr $(, $rest:expr)* $(,)? ) => {
        $crate::__exports::Cons($first, $crate::__make_cons!(@value $($rest,)*))
    };
}

/// Given a cons-list value, and a depth denoted by a series of any kind of token-tree, read the value at that depth
/// from the list.
///
/// ```rust
/// use veecle_os_runtime::{__make_cons, __read_cons};
///
/// let cons = __make_cons!(@value 1u32, "hello ferris", 3.141594f64);
///
/// assert_eq! {
///     __read_cons! {
///         from: cons,
///         depth: [],
///     },
///     1u32,
/// }
///
/// assert_eq! {
///     __read_cons! {
///         from: cons,
///         depth: [()],
///     },
///     "hello ferris",
/// }
///
/// assert_eq! {
///     __read_cons! {
///         from: cons,
///         depth: [() ()],
///     },
///     3.141594f64,
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! __read_cons {
    (
        from: $from:expr,
        depth: [$($depth:tt)*],
    ) => {
        $crate::__read_cons! {
            depth: [$($depth)*],
            result: [$from],
        }
    };

    // Once at the final depth, read the value stored in the first field of the current element.
    (
        depth: [],
        result: [$($result:tt)*],
    ) => {
        $($result)* . 0
    };

    // Discard one token from the head of the depth, and index into the second field of the current element.
    (
        depth: [$_:tt $($rest:tt)*],
        result: [$($result:tt)*],
    ) => {
        $crate::__read_cons! {
            depth: [$($rest)*],
            result: [$($result)* . 1],
        }
    };
}

/// Internal helper to append two cons-lists.
#[doc(hidden)]
pub trait AppendCons<Other> {
    /// The result of appending `Other` to self.
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
