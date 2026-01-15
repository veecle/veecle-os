mod slot;
mod storable;
mod waiter;

#[doc(inline)]
pub use veecle_os_runtime_macros::Storable;

pub use self::slot::Slot;
pub use self::storable::Storable;
pub(crate) use self::waiter::Waiter;
