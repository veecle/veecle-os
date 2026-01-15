mod slot;
mod storable;
mod waiter;

#[doc(inline)]
pub use veecle_os_runtime_macros::Storable;

pub(crate) use self::slot::{Slot, SlotTrait};
pub use self::storable::Storable;
pub(crate) use self::waiter::Waiter;
