// SPDX-License-Identifier: GPL-2.0

//! A kernel read-write lock (rw_lock).
//!
//! This module allows Rust code to use the kernel's `rwlock_t`.

/// Creates a [`RwLock`] initializer with the given name and a newly-created lock class.
///
/// It uses the name if one is given, otherwise, it generates one based on the file name and line
/// number.
#[macro_export]
macro_rules! new_rwlock {
    ($inner:expr $(, $name:literal)? $(,)?) => {
        $crate::sync::RwLock::new(
            $inner, $crate::optional_name!($($name)?), $crate::static_lock_class!())
    };
}
pub use new_rwlock;

/// A read-write lock.
///
/// Exposes the kernel's [`rwlock_t`]. This allows multiple readers to hold the lock simultaneously, 
/// but only one writer can hold it at a time, and only when no readers are holding the lock.
///
/// Since it may block, [`RwLock`] needs to be used with care in atomic contexts.
///
/// Instances of [`RwLock`] need a lock class and to be pinned. The recommended way to create such 
/// instances is with the [`pin_init`](crate::pin_init) and [`new_rwlock`] macros.
///
/// # Examples
///
/// The following example shows how to declare, allocate, and initialize a struct (`Example`) that
/// contains an inner struct (`Inner`) that is protected by a read-write lock.
///
/// ```rust
/// use kernel::sync::{new_rwlock, RwLock};
///
/// struct Inner {
///     a: u32,
///     b: u32,
/// }
///
/// #[pin_data]
/// struct Example {
///     c: u32,
///     #[pin]
///     d: RwLock<Inner>,
/// }
///
/// impl Example {
///     fn new() -> impl PinInit<Self> {
///         pin_init!(Self {
///             c: 10,
///             d <- new_rwlock!(Inner { a: 20, b: 30 }),
///         })
///     }
/// }
///
/// // Allocate a boxed `Example`.
/// let e = Box::pin_init(Example::new(), GFP_KERNEL)?;
/// assert_eq!(e.c, 10);
/// assert_eq!(e.d.read_lock().a, 20);
/// assert_eq!(e.d.read_lock().b, 30);
/// # Ok::<(), Error>(())
/// ```
///
/// The following example shows how to modify the contents of a struct protected by an `RwLock`:
///
/// ```rust
/// use kernel::sync::RwLock;
///
/// struct Example {
///     a: u32,
///     b: u32,
/// }
///
/// fn modify_example(m: &RwLock<Example>) {
///     {
///         let mut guard = m.write_lock();
///         guard.a += 10;
///         guard.b += 20;
///     }
///     {
///         let guard = m.read_lock();
///         println!("a: {}, b: {}", guard.a, guard.b);
///     }
/// }
/// ```
///
/// [`rwlock_t`]: srctree/include/linux/rwlock.h
pub type RwLock<T> = super::Lock<T, RwBackend>;

/// A kernel `rwlock_t` lock backend.
pub struct RwBackend;

/// Used to track if the lock is a read/or write lock
pub struct LockType {
    is_read: bool,  // Track if the lock is a read lock
}

impl LockType {
    /// Returns true if the lock is a read lock, false if it's a write lock.
    pub fn is_read_lock(&self) -> bool {
        self.is_read
    }
}



// SAFETY: The underlying kernel `rwlock_t` object ensures mutual exclusion for writers
// and allows multiple readers.
unsafe impl super::Backend for RwBackend {
    type State = bindings::rwlock_t;
    type GuardState = ();

    unsafe fn init(
        ptr: *mut Self::State,
        name: *const core::ffi::c_char,
        key: *mut bindings::lock_class_key,
    ) {
        // SAFETY: The safety requirements ensure that `ptr` is valid for writes, and `name` and
        // `key` are valid for read indefinitely.
        unsafe { bindings::rwlock_init(ptr, name, key) }
    }

    unsafe fn lock(ptr: *mut Self::State) -> Self::GuardState {
        // SAFETY: The safety requirements of this function ensure that `ptr` points to valid
        // memory, and that it has been initialized before.
        unsafe { bindings::write_lock(ptr) };
        GuardState { is_read: false }
    }

    unsafe fn unlock(ptr: *mut Self::State, _guard_state: &Self::GuardState) {
        // SAFETY: The safety requirements of this function ensure that `ptr` is valid and that
        // the caller holds a write lock.
        unsafe { bindings::write_unlock(ptr) }
    }
}

unsafe impl super::RWLockBackend for RwBackend {
    unsafe fn read_lock(ptr: *mut Self::State) -> Self::GuardState {
        bindings::read_lock(ptr);
        GuardState { is_read: true }
    }

    unsafe fn read_unlock(ptr: *mut Self::State, _guard_state: &Self::GuardState) {
        bindings::read_unlock(ptr);
    }
}
