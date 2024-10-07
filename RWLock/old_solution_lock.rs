// SPDX-License-Identifier: GPL-2.0

//! Generic kernel lock and guard.
//!
//! It contains a generic Rust lock and guard that allow for different backends (e.g., mutexes,
//! spinlocks, raw spinlocks) to be provided with minimal effort.

use super::LockClassKey;
use crate::{init::PinInit, pin_init, str::CStr, types::Opaque, types::ScopeGuard};
use core::{cell::UnsafeCell, marker::PhantomData, marker::PhantomPinned};
use macros::pin_data;

pub mod mutex;
pub mod spinlock;
pub mod rwlock;
/// The "backend" of a lock.
///
/// It is the actual implementation of the lock, without the need to repeat patterns used in all
/// locks.
///
/// # Safety
///
/// - Implementers must ensure that only one thread/CPU may access the protected data once the lock
///   is owned, that is, between calls to [`lock`] and [`unlock`].
/// - Implementers must also ensure that [`relock`] uses the same locking method as the original
///   lock operation.
///
/// [`lock`]: Backend::lock
/// [`unlock`]: Backend::unlock
/// [`relock`]: Backend::relock
pub unsafe trait Backend {
    /// The state required by the lock.
    type State;

    /// The state required to be kept between [`lock`] and [`unlock`].
    ///
    /// [`lock`]: Backend::lock
    /// [`unlock`]: Backend::unlock
    type GuardState;

    /// Initialises the lock.
    ///
    /// # Safety
    ///
    /// `ptr` must be valid for write for the duration of the call, while `name` and `key` must
    /// remain valid for read indefinitely.
    unsafe fn init(
        ptr: *mut Self::State,
        name: *const core::ffi::c_char,
        key: *mut bindings::lock_class_key,
    );

    /// Acquires the lock, making the caller its owner.
    ///
    /// # Safety
    ///
    /// Callers must ensure that [`Backend::init`] has been previously called.
    #[must_use]
    unsafe fn lock(ptr: *mut Self::State) -> Self::GuardState;

    /// Releases the lock, giving up its ownership.
    ///
    /// # Safety
    ///
    /// It must only be called by the current owner of the lock.
    unsafe fn unlock(ptr: *mut Self::State, guard_state: &Self::GuardState);

    /// Reacquires the lock, making the caller its owner.
    ///
    /// # Safety
    ///
    /// Callers must ensure that `guard_state` comes from a previous call to [`Backend::lock`] (or
    /// variant) that has been unlocked with [`Backend::unlock`] and will be relocked now.
    unsafe fn relock(ptr: *mut Self::State, guard_state: &mut Self::GuardState) {
        // SAFETY: The safety requirements ensure that the lock is initialised.
        *guard_state = unsafe { Self::lock(ptr) };
    }
}

// ---RWLock PATCH-START---
/// A backend that supports both read and write locks.
///
/// # Safety
///
/// Implementers must ensure that:
/// - A read lock allows multiple readers but no writers.
/// - A write lock ensures exclusive access to the lock.
pub unsafe trait RWLockBackend: Backend {
    type LockType;
    /// Acquires the lock for reading.
    ///
    /// # Safety
    ///
    /// Callers must ensure that [`Backend::init`] has been previously called.
    #[must_use]
    unsafe fn read_lock(ptr: *mut Self::State) -> Self::GuardState;

    /// Releases the read lock.
    ///
    /// # Safety
    ///
    /// It must only be called by the current owner of the read lock.
    unsafe fn read_unlock(ptr: *mut Self::State, guard_state: &Self::GuardState);
}


/// A marker trait to differentiate between RWLock and Regular Locks.
pub trait IsRWLock {
    fn is_rwlock() -> bool;
}

/// Marker for read-write locks.
pub struct RWLockMarker;

/// Marker for rust-kernele-native locks (spinlock and mutex).
pub struct RegularLockMarker;

impl IsRWLock for RWLockMarker {
    fn is_rwlock() -> bool { true }
}

impl IsRWLock for RegularLockMarker {
    fn is_rwlock() -> bool { false }
}
// ---RWLock PATCH-END----


/// A mutual exclusion primitive.
///
/// Exposes one of the kernel locking primitives. Which one is exposed depends on the lock
/// [`Backend`] specified as the generic parameter `B`.
#[pin_data]
pub struct Lock<T: ?Sized, B: Backend, M: IsRWLock> {
    /// The kernel lock object.
    #[pin]
    state: Opaque<B::State>,

    /// Some locks are known to be self-referential (e.g., mutexes), while others are architecture
    /// or config defined (e.g., spinlocks). So we conservatively require them to be pinned in case
    /// some architecture uses self-references now or in the future.
    #[pin]
    _pin: PhantomPinned,

    /// The data protected by the lock.
    pub(crate) data: UnsafeCell<T>,

    /// Marker added to track lock type
    _marker: PhantomData<M>,    

}

// SAFETY: `Lock` can be transferred across thread boundaries iff the data it protects can.
unsafe impl<T: ?Sized + Send, B: Backend> Send for Lock<T, B> {}

// SAFETY: `Lock` serialises the interior mutability it provides, so it is `Sync` as long as the
// data it protects is `Send`.
unsafe impl<T: ?Sized + Send, B: Backend> Sync for Lock<T, B> {}

impl<T, B: Backend> Lock<T, B> {
    /// Constructs a new lock initialiser.
    pub fn new(t: T, name: &'static CStr, key: &'static LockClassKey) -> impl PinInit<Self> {
        pin_init!(Self {
            data: UnsafeCell::new(t),
            _pin: PhantomPinned,
            // SAFETY: `slot` is valid while the closure is called and both `name` and `key` have
            // static lifetimes so they live indefinitely.
            state <- Opaque::ffi_init(|slot| unsafe {
                B::init(slot, name.as_char_ptr(), key.as_ptr())
            }),
        })
    }
}

impl<T: ?Sized, B: Backend> Lock<T, B> {
    /// Acquires the lock and gives the caller access to the data protected by it.
    pub fn lock(&self) -> Guard<'_, T, B> {
        // SAFETY: The constructor of the type calls `init`, so the existence of the object proves
        // that `init` was called.
        let state = unsafe { B::lock(self.state.get()) };
        // SAFETY: The lock was just acquired.
        unsafe { Guard::new(self, state) }
    }
}

// ---RWLock PATCH-START---
impl<T: ?Sized, B: RWLockBackend> Lock<T, B> {
    /// Acquires the lock for reading.
    pub fn read_lock(&self) -> Guard<'_, T, B> {
        // SAFETY: The constructor of the type calls `init`, so the existence of the object proves
        // that `init` was called.
        let state = unsafe { B::read_lock(self.state.get()) };
        // SAFETY: The lock was just acquired.
        unsafe { Guard::new(self, state) }
    }

    /// Acquires the lock for writing.
    pub fn write_lock(&self) -> Guard<'_, T, B> {
        // SAFETY: The constructor of the type calls `init`, so the existence of the object proves
        // that `init` was called.
        let state = unsafe { B::lock(self.state.get()) };
        // SAFETY: The lock was just acquired.
        unsafe { Guard::new(self, state) }
    }
}
// ---RWLock PATCH-END----


/// A lock guard.
///
/// Allows mutual exclusion primitives that implement the [`Backend`] trait to automatically unlock
/// when a guard goes out of scope. It also provides a safe and convenient way to access the data
/// protected by the lock.
#[must_use = "the lock unlocks immediately when the guard is unused"]
pub struct Guard<'a, T: ?Sized, B: Backend, M: IsRWLock> {
    pub(crate) lock: &'a Lock<T, B>,  // Reference to the lock
    pub(crate) state: B::GuardState,
    _not_send: PhantomData<*mut ()>,
}

// SAFETY: `Guard` is sync when the data protected by the lock is also sync.
unsafe impl<T: Sync + ?Sized, B: Backend> Sync for Guard<'_, T, B> {}

impl<T: ?Sized, B: Backend> Guard<'_, T, B> {
    pub(crate) fn do_unlocked<U>(&mut self, cb: impl FnOnce() -> U) -> U {
        // SAFETY: The caller owns the lock, so it is safe to unlock it.
        unsafe { B::unlock(self.lock.state.get(), &self.state) };

        // SAFETY: The lock was just unlocked above and is being relocked now.
        let _relock =
            ScopeGuard::new(|| unsafe { B::relock(self.lock.state.get(), &mut self.state) });

        cb()
    }
}

impl<T: ?Sized, B: Backend> core::ops::Deref for Guard<'_, T, B> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The caller owns the lock, so it is safe to deref the protected data.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized, B: Backend> core::ops::DerefMut for Guard<'_, T, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The caller owns the lock, so it is safe to deref the protected data.
        unsafe { &mut *self.lock.data.get() }
    }
}

/// Drop implementation for native locks.
impl<T: ?Sized, B: Backend> Drop for Guard<'_, T, B, RegularLockMarker> {
    fn drop(&mut self) {
        // SAFETY: The caller owns the lock, so it is safe to unlock it.
        unsafe { B::unlock(self.lock.state.get(), &self.state) };
    }
}

impl<'a, T: ?Sized, B: Backend> Guard<'a, T, B> {
    /// Constructs a new immutable lock guard.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it owns the lock.
    pub(crate) unsafe fn new(lock: &'a Lock<T, B>, state: B::GuardState) -> Self {
        Self {
            lock,
            state,
            _not_send: PhantomData,
        }
    }
}


// ---RWLock PATCH-START---
impl<T: ?Sized, B: RWLockBackend> Drop for Guard<'_, T, B, RWLockMarker> {
    fn drop(&mut self) {
        if self.state.is_read_lock() {
            // SAFETY: The caller owns the read lock, so it is safe to unlock it.
            unsafe { B::read_unlock(self.lock.state.get(), &self.state) };
        } else {
            // SAFETY: The caller owns the write lock, so it is safe to unlock it.
            unsafe { B::unlock(self.lock.state.get(), &self.state) };
        }
    }
}

impl<'a, T: ?Sized, B: RWLockBackend> Guard<'a, T, B> {
    /// Constructs a new immutable read lock guard.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it owns the read lock.
    pub(crate) unsafe fn new_read(lock: &'a Lock<T, B>, state: B::GuardState) -> Self {
        Self {
            lock,
            state,
            _not_send: PhantomData,
        }
    }

    /// Constructs a new immutable write lock guard.
    ///
    /// # Safety
    ///
    /// The caller must ensure that it owns the write lock.
    pub(crate) unsafe fn new_write(lock: &'a Lock<T, B>, state: B::GuardState) -> Self {
        Self {
            lock,
            state,
            _not_send: PhantomData,
        }
    }
}
// ---RWLock PATCH-END----

