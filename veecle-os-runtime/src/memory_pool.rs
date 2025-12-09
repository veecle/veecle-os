//! An interrupt/thread-safe memory pool.
//!
//! The memory pool allows using static, stack or heap memory to store `SIZE` instances of `T`.
//! [`MemoryPool::chunk`] provides [`Chunk`]s to interact with instances of `T`.
//! [`Chunk`] is a pointer type, which means it is cheap to move.
//! This makes the memory pool well suited for moving data between actors without copying.
//! The memory pool is especially useful for large chunks of data or data that is expensive to move.
//!
//! [`Chunk`]s are automatically made available for re-use on drop.
//!
//! [`Chunk`]s can be created by:
//! - [`MemoryPool::reserve`] and [`MemoryPoolToken::init`], which uses the provided value of `T` to initialize the
//!   chunk. [`MemoryPool::chunk`] combines both into a single method call.
//! - [`MemoryPool::reserve`] and [`MemoryPoolToken::init_in_place`] to initialize `T` in place.
//!
//! # Example
//!
//! ```
//! use veecle_os_runtime::{ExclusiveReader, Writer};
//! use veecle_os_runtime::memory_pool::{Chunk, MemoryPool};
//! use core::convert::Infallible;
//! use veecle_os_runtime::Storable;
//!
//! #[derive(Debug, Storable)]
//! #[storable(data_type = "Chunk<'static, u8>")]
//! pub struct Data;
//!
//! #[veecle_os_runtime::actor]
//! async fn exclusive_read_actor(mut reader: ExclusiveReader<'_, Data>) -> Infallible {
//!     loop {
//!         if let Some(chunk) = reader.take() {
//!             println!("Chunk received: {:?}", chunk);
//!             println!("Chunk content: {:?}", *chunk);
//!         } else {
//!             reader.wait_for_update().await;
//!         }
//!     }
//! }
//!
//! #[veecle_os_runtime::actor]
//! async fn write_actor(
//!     mut writer: Writer<'_, Data>,
//!     #[init_context] pool: &'static MemoryPool<u8, 5>,
//! ) -> Infallible {
//!     for index in 0..10 {
//!         writer.write(pool.chunk(index).unwrap()).await;
//!     }
//! #       // Exit the application to allow doc-tests to complete.
//! #       std::process::exit(0);
//! }
//!
//! static POOL: MemoryPool<u8, 5> = MemoryPool::new();
//!
//! # futures::executor::block_on(
//! #
//! veecle_os_runtime::execute! {
//!    store: [Data],
//!    actors: [
//!        ExclusiveReadActor,
//!        WriteActor: &POOL,
//!    ]
//! }
//! # );
//!  ```

use core::cell::UnsafeCell;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// Interrupt- and thread-safe memory pool.
///
/// See [module-level documentation][self] for more information.
#[derive(Debug)]
pub struct MemoryPool<T, const SIZE: usize> {
    chunks: [MemoryPoolInner<T>; SIZE],
}

impl<T, const SIZE: usize> Default for MemoryPool<T, SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SIZE: usize> MemoryPool<T, SIZE> {
    /// Creates a new [`MemoryPool`].
    ///
    /// `SIZE` is required to be larger than 0.
    pub const fn new() -> Self {
        const {
            assert!(SIZE > 0, "empty ObjectPool");
        }

        Self {
            chunks: [const { MemoryPoolInner::new() }; SIZE],
        }
    }

    /// Reserves an element in the [`MemoryPool`].
    ///
    /// Returns `None` if no element is available.
    ///
    /// The returned token has to be initialized via [`MemoryPoolToken::init`] before use.
    /// See [`MemoryPool::chunk`] for a convenience wrapper combining reserving and initializing a [`Chunk`].
    pub fn reserve(&self) -> Option<MemoryPoolToken<'_, T>> {
        self.chunks.iter().find_map(|chunk| chunk.reserve())
    }

    /// Retrieves a [`Chunk`] from the [`MemoryPool`] and initializes it with `init_value`.
    ///
    /// Returns `Err(init_value)` if no more [`Chunk`]s are available.
    ///
    /// Convenience wrapper combining [`MemoryPool::reserve`] and [`MemoryPoolToken::init].
    pub fn chunk(&self, init_value: T) -> Result<Chunk<'_, T>, T> {
        // We need to split reserving and initializing of the `Chunk` because we cannot copy the `init_value` into
        // every `reserve` call.
        let token = self.reserve();

        if let Some(token) = token {
            Ok(token.init(init_value))
        } else {
            Err(init_value)
        }
    }

    /// Calculates the amount of chunks currently available.
    ///
    /// Due to accesses from interrupts and/or other threads, this value might not be correct.
    /// Only intended for metrics.
    pub fn chunks_available(&self) -> usize {
        self.chunks
            .iter()
            .map(|chunk| usize::from(chunk.is_available()))
            .sum()
    }
}

// SAFETY: All accesses to the `MemoryPool` are done through the `MemoryPool::chunk` method which is synchronized by
// atomics.
unsafe impl<T, const N: usize> Sync for MemoryPool<T, N> {}

/// Container for the `T` instance and synchronization atomic for the [`MemoryPool`].
#[derive(Debug)]
struct MemoryPoolInner<T> {
    data: UnsafeCell<MaybeUninit<T>>,
    available: AtomicBool,
}

impl<T> MemoryPoolInner<T> {
    /// Creates a new `MemoryPoolInner`.
    ///
    /// Marked available and uninitialized.
    const fn new() -> Self {
        Self {
            data: UnsafeCell::new(MaybeUninit::uninit()),
            available: AtomicBool::new(true),
        }
    }

    /// Reserves this [`MemoryPoolInner`].
    fn reserve(&self) -> Option<MemoryPoolToken<'_, T>> {
        if self.available.swap(false, Ordering::AcqRel) {
            Some(MemoryPoolToken { inner: Some(self) })
        } else {
            None
        }
    }

    /// Returns `true` if the [`MemoryPoolInner`] is currently available.
    fn is_available(&self) -> bool {
        self.available.load(Ordering::Acquire)
    }
}

/// A token reserving an element in a [`MemoryPool`] which can be initialized to create a [`Chunk`].
#[derive(Debug)]
pub struct MemoryPoolToken<'a, T> {
    inner: Option<&'a MemoryPoolInner<T>>,
}

impl<'a, T> MemoryPoolToken<'a, T> {
    /// Consumes the [`MemoryPoolToken.inner`][field@MemoryPoolToken::inner] to prevent [`MemoryPoolToken`]'s drop
    /// implementation from making the element available.
    fn consume(&mut self) -> (&'a mut MaybeUninit<T>, &'a AtomicBool) {
        let Some(inner) = self.inner.take() else {
            unreachable!("`MemoryPoolToken` should only be consumed once");
        };

        let inner_data = {
            let inner_data_ptr = inner.data.get();
            // SAFETY:
            // - `UnsafeCell` has the same layout as its content, thus the `chunk_ptr` points to an aligned and valid
            //   value of `MaybeUninit<T>`.
            // - We ensure via the `ChunkMetadata` that only this single mutable reference to the content of the
            //   `UnsafeCell` exists.
            unsafe { inner_data_ptr.as_mut() }
                .expect("pointer to the contents of an `UnsafeCell` should not be null")
        };

        (inner_data, &inner.available)
    }

    /// Consumes and turns the [`MemoryPoolToken`] into an initialized [`Chunk`].
    pub fn init(mut self, init_value: T) -> Chunk<'a, T> {
        let (inner_data, available) = self.consume();

        inner_data.write(init_value);

        // SAFETY:
        // `inner_data` has be initialized by writing the `init_value`.
        unsafe { Chunk::new(inner_data, available) }
    }

    /// Initializes a [`Chunk`] in place via `init_function`.
    ///
    /// # Safety
    ///
    /// `init_function` must initialize the passed parameter to a valid `T` before the function returns.
    pub unsafe fn init_in_place(
        mut self,
        init_function: impl FnOnce(&mut MaybeUninit<T>),
    ) -> Chunk<'a, T> {
        let (inner_data, available) = self.consume();

        init_function(inner_data);

        // SAFETY:
        // `inner_data` has be initialized by `init_function`.
        unsafe { Chunk::new(inner_data, available) }
    }
}

impl<T> Drop for MemoryPoolToken<'_, T> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            inner.available.store(true, Ordering::Release);
        }
    }
}

/// A pointer type pointing to an instance of `T` in a [`MemoryPool`].
///
/// See [module-level documentation][self] for more information.
pub struct Chunk<'a, T> {
    // We're using `&mut MaybeUninit<T>` instead of `&mut T` to be able to drop `T` without going through a pointer
    // while only having a reference.
    // We cannot drop the contents of a reference without creating a dangling reference in the `Drop` implementation.
    inner: &'a mut MaybeUninit<T>,
    // Only held to ensure the chunk is made available on drop.
    token: &'a AtomicBool,
}

// Required so `Chunk` can be used in `yoke::Yoke` as the cart.
// SAFETY: While `Chunk` has a reference to its assigned memory location in the `MemoryPool`,
// the address of that memory cannot change as a reference to the `MemoryPool` instance is held.
// With that, the address returned by the `Deref` and `DerefMut` implementations
// are stable for the duration of the lifetime of `Chunk`.
unsafe impl<'a, T> stable_deref_trait::StableDeref for Chunk<'a, T> {}

impl<T> Debug for Chunk<'_, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<'a, T> Chunk<'a, T> {
    /// Creates a new [`Chunk`].
    ///
    /// # Safety
    ///
    /// The `chunk` must be initialized.
    unsafe fn new(chunk: &'a mut MaybeUninit<T>, token: &'a AtomicBool) -> Self {
        Self {
            inner: chunk,
            token,
        }
    }
}

impl<T> Deref for Chunk<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The `Self::new` safety documentation requires the chunk to be initialized.
        // It is only dropped in the drop implementation and cannot be un-initialized by any `Chunk` method, thus it is
        // initialized here.
        unsafe { self.inner.assume_init_ref() }
    }
}

impl<T> DerefMut for Chunk<'_, T> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        // SAFETY: The `Self::new` safety documentation requires the chunk to be initialized.
        // It is only dropped in the drop implementation and cannot be un-initialized by any `Chunk` method, thus it is
        // initialized here.
        unsafe { self.inner.assume_init_mut() }
    }
}

impl<T> Drop for Chunk<'_, T> {
    fn drop(&mut self) {
        // SAFETY: The `Self::new` safety documentation requires the chunk to be initialized.
        // It is only dropped in the drop implementation and cannot be un-initialized by any `Chunk` method, thus it is
        // initialized here.
        unsafe { self.inner.assume_init_drop() };
        debug_assert!(
            !self.token.swap(true, Ordering::AcqRel),
            "chunk was made available a second time"
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use std::format;
    use std::sync::atomic::AtomicUsize;

    use super::*;

    #[test]
    fn pool() {
        static POOL: MemoryPool<[u8; 10], 2> = MemoryPool::new();

        let mut chunk = POOL.chunk([0; 10]).unwrap();
        let chunk1 = POOL.chunk([0; 10]).unwrap();
        assert!(POOL.chunk([0; 10]).is_err());
        assert_eq!(chunk[0], 0);
        chunk[0] += 1;
        assert_eq!(chunk[0], 1);
        assert_eq!(chunk1[0], 0);
    }

    #[test]
    fn drop_test() {
        #[derive(Debug)]
        pub struct Dropper {}
        impl Drop for Dropper {
            fn drop(&mut self) {
                COUNTER.fetch_add(1, Ordering::Relaxed);
            }
        }

        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        {
            let pool: MemoryPool<Dropper, 2> = MemoryPool::new();

            let _ = pool.chunk(Dropper {});
            assert_eq!(COUNTER.load(Ordering::Relaxed), 1);

            {
                let _dropper1 = pool.chunk(Dropper {}).unwrap();
                let _dropper2 = pool.chunk(Dropper {}).unwrap();
                assert!(pool.chunk(Dropper {}).is_err());
            }
            assert_eq!(COUNTER.load(Ordering::Relaxed), 4);
            let _ = pool.chunk(Dropper {});
            assert_eq!(COUNTER.load(Ordering::Relaxed), 5);
        }

        // After dropping `pool`, there were no additional drops of the contained type.
        assert_eq!(COUNTER.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn drop_memory_pool_token() {
        let pool = MemoryPool::<usize, 1>::new();
        assert_eq!(pool.chunks_available(), 1);
        {
            let _token = pool.reserve().unwrap();
            assert_eq!(pool.chunks_available(), 0);
        }
        assert_eq!(pool.chunks_available(), 1);
    }

    #[test]
    fn chunks_available() {
        let pool = MemoryPool::<usize, 2>::new();
        assert_eq!(pool.chunks_available(), 2);
        {
            let _chunk = pool.chunk(0);
            assert_eq!(pool.chunks_available(), 1);
            let _chunk = pool.chunk(0);
            assert_eq!(pool.chunks_available(), 0);
        }
        assert_eq!(pool.chunks_available(), 2);
    }

    #[test]
    fn reserve_init() {
        let pool = MemoryPool::<usize, 2>::new();
        let token = pool.reserve().unwrap();
        let chunk = token.init(2);
        assert_eq!(*chunk, 2);
    }

    #[test]
    fn reserve_init_in_place() {
        let pool = MemoryPool::<usize, 2>::new();
        let token = pool.reserve().unwrap();
        // SAFETY: The passed closure initializes the chunk correctly.
        let chunk = unsafe {
            token.init_in_place(|m| {
                m.write(2);
            })
        };
        assert_eq!(*chunk, 2);
    }

    #[test]
    #[should_panic(expected = "`MemoryPoolToken` should only be consumed once")]
    fn consume_none() {
        let pool = MemoryPool::<usize, 2>::new();
        let mut token = pool.reserve().unwrap();
        let _ = token.consume();
        let _ = token.consume();
    }

    /// Ensures the `MemoryPool` and `Chunk` don't lose their `Send` & `Sync` auto trait implementations when
    /// refactoring.
    #[test]
    fn send_sync() {
        fn send<T>()
        where
            T: Send,
        {
        }
        fn sync<T>()
        where
            T: Sync,
        {
        }
        send::<MemoryPool<[u8; 10], 2>>();
        sync::<MemoryPool<[u8; 10], 2>>();

        send::<Chunk<[u8; 10]>>();
        sync::<Chunk<[u8; 10]>>();
    }

    #[test]
    fn debug_chunk() {
        let pool = MemoryPool::<usize, 2>::new();
        let chunk = pool.chunk(0).unwrap();
        assert_eq!(format!("{chunk:?}"), "0");
    }

    #[test]
    fn default_memory_pool() {
        let pool: MemoryPool<usize, 2> = MemoryPool::default();
        assert_eq!(pool.chunks_available(), 2);
    }
}
